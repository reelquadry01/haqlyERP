// Author: Quadri Atharu
use axum::body::Body;
use axum::http::Request;
use axum::response::Response;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine;
use sqlx::PgPool;
use std::convert::Infallible;
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};
use tower::Layer;
use tower::Service;
use uuid::Uuid;

fn extract_user_id_from_token(auth_header: &str) -> Option<Uuid> {
    let token = auth_header.strip_prefix("Bearer ")?;
    let parts: Vec<&str> = token.split('.').collect();
    if parts.len() != 3 {
        return None;
    }
    let payload = parts[1];
    let decoded = URL_SAFE_NO_PAD.decode(payload).ok()?;
    let value: serde_json::Value = serde_json::from_slice(&decoded).ok()?;
    let sub = value.get("sub")?.as_str()?;
    Uuid::parse_str(sub).ok()
}

#[derive(Clone)]
pub struct AuditLayer {
    pool: PgPool,
}

impl AuditLayer {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

impl<S> Layer<S> for AuditLayer
where
    S: Service<Request<Body>, Response = Response, Error = Infallible> + Clone + Send + 'static,
    S::Future: Send + 'static,
{
    type Service = AuditService<S>;

    fn layer(&self, inner: S) -> Self::Service {
        AuditService {
            inner,
            pool: self.pool.clone(),
        }
    }
}

#[derive(Clone)]
pub struct AuditService<S> {
    inner: S,
    pool: PgPool,
}

impl<S> Service<Request<Body>> for AuditService<S>
where
    S: Service<Request<Body>, Response = Response, Error = Infallible> + Clone + Send + 'static,
    S::Future: Send + 'static,
{
    type Response = Response;
    type Error = Infallible;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send + 'static>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: Request<Body>) -> Self::Future {
        let inner = self.inner.clone();
        let pool = self.pool.clone();

        let method = req.method().clone().to_string();
        let path = req.uri().path().to_string();
        let ip_address = req
            .headers()
            .get("x-forwarded-for")
            .or_else(|| req.headers().get("x-real-ip"))
            .and_then(|v| v.to_str().ok())
            .map(|s| {
                let ips = s.split(',');
                ips.next().unwrap_or(s).trim().to_string()
            });
        let auth_header = req
            .headers()
            .get(axum::http::header::AUTHORIZATION)
            .and_then(|v| v.to_str().ok())
            .map(|s| s.to_string());

        let entity = path
            .split('/')
            .filter(|s| !s.is_empty())
            .nth(3)
            .unwrap_or("system")
            .to_string();

        Box::pin(async move {
            let response = inner.call(req).await?;

            let user_id = auth_header
                .as_deref()
                .and_then(extract_user_id_from_token);

            let action = format!("{} {}", method, path);

            let pool_bg = pool.clone();
            let ip_bg = ip_address.clone();
            let entity_bg = entity.clone();
            tokio::spawn(async move {
                let result = sqlx::query(
                    "INSERT INTO audit_logs (user_id, action, entity, ip_address) VALUES ($1, $2, $3, $4)",
                )
                .bind(user_id)
                .bind(&action)
                .bind(&entity_bg)
                .bind(&ip_bg)
                .execute(&pool_bg)
                .await;

                if let Err(e) = result {
                    tracing::error!("Failed to write audit log: {}", e);
                }
            });

            Ok(response)
        })
    }
}
