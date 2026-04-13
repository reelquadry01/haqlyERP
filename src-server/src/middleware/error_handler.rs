// Author: Quadri Atharu
use axum::body::Body;
use axum::http::{Request, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::{Deserialize, Serialize};
use std::convert::Infallible;
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};
use tower::Layer;
use tower::Service;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorBody {
    pub success: bool,
    pub error: ErrorDetail,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorDetail {
    pub code: u16,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<serde_json::Value>,
}

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("Resource not found: {0}")]
    NotFound(String),

    #[error("Unauthorized: {0}")]
    Unauthorized(String),

    #[error("Forbidden: {0}")]
    Forbidden(String),

    #[error("Bad request: {0}")]
    BadRequest(String),

    #[error("Internal server error: {0}")]
    Internal(String),

    #[error("Validation error: {0}")]
    Validation(String),
}

impl AppError {
    pub fn status_code(&self) -> StatusCode {
        match self {
            AppError::NotFound(_) => StatusCode::NOT_FOUND,
            AppError::Unauthorized(_) => StatusCode::UNAUTHORIZED,
            AppError::Forbidden(_) => StatusCode::FORBIDDEN,
            AppError::BadRequest(_) => StatusCode::BAD_REQUEST,
            AppError::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::Validation(_) => StatusCode::UNPROCESSABLE_ENTITY,
        }
    }

    pub fn error_code(&self) -> u16 {
        self.status_code().as_u16()
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let body = ErrorBody {
            success: false,
            error: ErrorDetail {
                code: self.error_code(),
                message: self.to_string(),
                details: None,
            },
        };
        (self.status_code(), Json(body)).into_response()
    }
}

impl From<sqlx::Error> for AppError {
    fn from(err: sqlx::Error) -> Self {
        match err {
            sqlx::Error::RowNotFound => AppError::NotFound("Resource not found".to_string()),
            sqlx::Error::Database(ref e) => {
                if e.code().map_or(false, |c| c == "23505") {
                    AppError::BadRequest("Duplicate entry - resource already exists".to_string())
                } else if e.code().map_or(false, |c| c == "23503") {
                    AppError::BadRequest("Referenced resource not found".to_string())
                } else {
                    AppError::Internal(format!("Database error: {}", err))
                }
            }
            _ => AppError::Internal(format!("Database error: {}", err)),
        }
    }
}

impl From<validator::ValidationErrors> for AppError {
    fn from(err: validator::ValidationErrors) -> Self {
        let details = err
            .field_errors()
            .into_iter()
            .map(|(field, errors)| {
                let messages: Vec<String> = errors
                    .iter()
                    .map(|e| {
                        e.message
                            .clone()
                            .unwrap_or_else(|| format!("{} validation failed", e.code))
                    })
                    .collect();
                (field.to_string(), messages)
            })
            .collect::<serde_json::Map<_, _>>();

        AppError::Validation(serde_json::to_string(&details).unwrap_or_else(|_| "Validation failed".to_string()))
    }
}

#[derive(Clone)]
pub struct ErrorLayer;

impl ErrorLayer {
    pub fn new() -> Self {
        Self
    }
}

impl Default for ErrorLayer {
    fn default() -> Self {
        Self::new()
    }
}

impl<S> Layer<S> for ErrorLayer
where
    S: Service<Request<Body>, Response = Response, Error = Infallible> + Clone + Send + 'static,
    S::Future: Send + 'static,
{
    type Service = ErrorService<S>;

    fn layer(&self, inner: S) -> Self::Service {
        ErrorService { inner }
    }
}

#[derive(Clone)]
pub struct ErrorService<S> {
    inner: S,
}

impl<S> Service<Request<Body>> for ErrorService<S>
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

        Box::pin(async move {
            let response = inner.call(req).await?;

            let status = response.status();
            if status.is_client_error() || status.is_server_error() {
                let (parts, body) = response.into_parts();
                let is_json = parts
                    .headers
                    .get(axum::http::header::CONTENT_TYPE)
                    .and_then(|v| v.to_str().ok())
                    .map_or(false, |ct| ct.contains("application/json"));

                if is_json {
                    return Ok(Response::from_parts(parts, body));
                }

                let body_bytes = axum::body::to_bytes(body, 1024 * 1024)
                    .await
                    .unwrap_or_default();
                let original_body = String::from_utf8_lossy(&body_bytes);

                let error_body = ErrorBody {
                    success: false,
                    error: ErrorDetail {
                        code: status.as_u16(),
                        message: original_body.to_string(),
                        details: None,
                    },
                };

                return Ok((
                    status,
                    [(axum::http::header::CONTENT_TYPE, "application/json")],
                    serde_json::to_string(&error_body).unwrap_or_default(),
                )
                    .into_response());
            }

            Ok(response)
        })
    }
}
