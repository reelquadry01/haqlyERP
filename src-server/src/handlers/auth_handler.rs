// Author: Quadri Atharu
// Crate: haqly-erp-server

use axum::{
    Json, State,
    routing::post,
    Router,
};
use sqlx::PgPool;
use serde_json::{json, Value};

pub fn router() -> Router<PgPool> {
    Router::new()
        .route("/login", post(login))
        .route("/register", post(register))
        .route("/refresh", post(refresh_token))
        .route("/logout", post(logout))
        .route("/mfa/status", post(mfa_status))
        .route("/mfa/setup", post(mfa_setup))
        .route("/mfa/activate", post(mfa_activate))
        .route("/mfa/disable", post(mfa_disable))
        .route("/mfa/verify-login", post(mfa_verify_login))
}

async fn login(
    State(_pool): State<PgPool>,
    Json(_body): Json<Value>,
) -> Json<Value> {
    Json(json!({"message": "auth/login - not implemented"}))
}

async fn register(
    State(_pool): State<PgPool>,
    Json(_body): Json<Value>,
) -> Json<Value> {
    Json(json!({"message": "auth/register - not implemented"}))
}

async fn refresh_token(
    State(_pool): State<PgPool>,
    Json(_body): Json<Value>,
) -> Json<Value> {
    Json(json!({"message": "auth/refresh - not implemented"}))
}

async fn logout(
    State(_pool): State<PgPool>,
    Json(_body): Json<Value>,
) -> Json<Value> {
    Json(json!({"message": "auth/logout - not implemented"}))
}

async fn mfa_status(
    State(_pool): State<PgPool>,
    Json(_body): Json<Value>,
) -> Json<Value> {
    Json(json!({"message": "auth/mfa/status - not implemented"}))
}

async fn mfa_setup(
    State(_pool): State<PgPool>,
    Json(_body): Json<Value>,
) -> Json<Value> {
    Json(json!({"message": "auth/mfa/setup - not implemented"}))
}

async fn mfa_activate(
    State(_pool): State<PgPool>,
    Json(_body): Json<Value>,
) -> Json<Value> {
    Json(json!({"message": "auth/mfa/activate - not implemented"}))
}

async fn mfa_disable(
    State(_pool): State<PgPool>,
    Json(_body): Json<Value>,
) -> Json<Value> {
    Json(json!({"message": "auth/mfa/disable - not implemented"}))
}

async fn mfa_verify_login(
    State(_pool): State<PgPool>,
    Json(_body): Json<Value>,
) -> Json<Value> {
    Json(json!({"message": "auth/mfa/verify-login - not implemented"}))
}
