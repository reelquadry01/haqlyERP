// Author: Quadri Atharu
use axum::body::Body;
use axum::http::{Request, StatusCode};
use axum::response::{IntoResponse, Response};
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use std::convert::Infallible;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};
use tower::Layer;
use tower::Service;
use uuid::Uuid;

use crate::config::rsa_keys::RsaKeypair;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    pub sub: Uuid,
    pub email: String,
    pub role: String,
    pub company_id: Uuid,
    pub exp: usize,
    pub iat: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenResponse {
    pub token: String,
    pub token_type: String,
    pub expires_in: u64,
}

pub fn validate_token(token: &str, public_key_pem: &[u8]) -> Result<Claims, jsonwebtoken::errors::Error> {
    let mut validation = Validation::new(Algorithm::RS256);
    validation.validate_exp = true;
    let decoding_key = DecodingKey::from_rsa_pem(public_key_pem)?;
    let token_data = decode::<Claims>(token, &decoding_key, &validation)?;
    Ok(token_data.claims)
}

pub fn generate_token(claims: Claims, private_key_pem: &[u8], kid: &str) -> Result<String, jsonwebtoken::errors::Error> {
    let mut header = Header::new(Algorithm::RS256);
    header.kid = Some(kid.to_string());
    let encoding_key = EncodingKey::from_rsa_pem(private_key_pem)?;
    encode(&header, &claims, &encoding_key)
}

fn is_public_route(path: &str) -> bool {
    let path = path.split('?').next().unwrap_or(path);
    matches!(
        path,
        "/api/v1/auth/login"
            | "/api/v1/auth/register"
            | "/api/v1/auth/forgot-password"
            | "/api/v1/auth/reset-password"
            | "/api/v1/health"
            | "/health"
            | "/"
    )
}

#[derive(Clone)]
pub struct AuthLayer {
    rsa_keypair: Arc<RsaKeypair>,
    jwt_expiration: u64,
}

impl AuthLayer {
    pub fn new(rsa_keypair: Arc<RsaKeypair>, jwt_expiration: u64) -> Self {
        Self {
            rsa_keypair,
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
            rsa_keypair: self.rsa_keypair.clone(),
            jwt_expiration: self.jwt_expiration,
        }
    }
}

#[derive(Clone)]
pub struct AuthService<S> {
    inner: S,
    rsa_keypair: Arc<RsaKeypair>,
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
        let mut inner = self.inner.clone();
        let rsa_keypair = self.rsa_keypair.clone();
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
                    match validate_token(token, &rsa_keypair.public_pem) {
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
