// Author: Quadri Atharu
use axum::body::Body;
use axum::http::{Request, StatusCode};
use axum::response::{IntoResponse, Response};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use std::convert::Infallible;
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};
use tower::Layer;
use tower::Service;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    pub sub: Uuid,
    pub email: String,
    pub role: String,
    pub exp: usize,
    pub iat: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenResponse {
    pub token: String,
    pub token_type: String,
    pub expires_in: u64,
}

pub fn validate_token(token: &str, secret: &str) -> Result<Claims, jsonwebtoken::errors::Error> {
    let validation = Validation::default();
    let token_data = decode::<Claims>(token, &DecodingKey::from_secret(secret), &validation)?;
    Ok(token_data.claims)
}

pub fn generate_token(claims: Claims, secret: &str) -> Result<String, jsonwebtoken::errors::Error> {
    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret),
    )
}

fn is_public_route(path: &str) -> bool {
    let path = path.split('?').next().unwrap_or(path);
    matches!(
        path,
        "/api/v1/auth/login"
            | "/api/v1/auth/register"
            | "/api/v1/health"
            | "/health"
            | "/"
    )
}

#[derive(Clone)]
pub struct AuthLayer {
    jwt_secret: String,
    jwt_expiration: u64,
}

impl AuthLayer {
    pub fn new(jwt_secret: String, jwt_expiration: u64) -> Self {
        Self {
            jwt_secret,
            jwt_expiration,
        }
    }
}

impl<S> Layer<S> for AuthLayer
where
    S: Service<Request<Body>, Response = Response, Error = Infallible> + Clone + Send + 'static,
    S::Future: Send + 'static,
{
    type Service = AuthService<S>;

    fn layer(&self, inner: S) -> Self::Service {
        AuthService {
            inner,
            jwt_secret: self.jwt_secret.clone(),
            jwt_expiration: self.jwt_expiration,
        }
    }
}

#[derive(Clone)]
pub struct AuthService<S> {
    inner: S,
    jwt_secret: String,
    jwt_expiration: u64,
}

impl<S> Service<Request<Body>> for AuthService<S>
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
        let jwt_secret = self.jwt_secret.clone();
        let _jwt_expiration = self.jwt_expiration;

        Box::pin(async move {
            let path = req.uri().path().to_string();

            if is_public_route(&path) {
                return inner.call(req).await;
            }

            let auth_header = req
                .headers()
                .get(axum::http::header::AUTHORIZATION)
                .and_then(|v| v.to_str().ok());

            match auth_header {
                Some(header) if header.starts_with("Bearer ") => {
                    let token = &header[7..];
                    match validate_token(token, &jwt_secret) {
                        Ok(claims) => {
                            let (mut parts, body) = req.into_parts();
                            parts.extensions.insert(claims);
                            let req = Request::from_parts(parts, body);
                            inner.call(req).await
                        }
                        Err(e) => {
                            tracing::warn!("JWT validation failed: {}", e);
                            let body = serde_json::json!({
                                "success": false,
                                "error": {
                                    "code": 401,
                                    "message": "Invalid or expired authentication token"
                                }
                            });
                            Ok((
                                StatusCode::UNAUTHORIZED,
                                [(
                                    axum::http::header::CONTENT_TYPE,
                                    "application/json",
                                )],
                                serde_json::to_string(&body).unwrap_or_default(),
                            )
                                .into_response())
                        }
                    }
                }
                _ => {
                    let body = serde_json::json!({
                        "success": false,
                        "error": {
                            "code": 401,
                            "message": "Missing authentication token"
                        }
                    });
                    Ok((
                        StatusCode::UNAUTHORIZED,
                        [(
                            axum::http::header::CONTENT_TYPE,
                            "application/json",
                        )],
                        serde_json::to_string(&body).unwrap_or_default(),
                    )
                        .into_response())
                }
            }
        })
    }
}
