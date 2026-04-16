// Author: Quadri Atharu
// Crate: haqly-erp-server

use axum::{
    Json,
    routing::post,
    Router,
};
use axum::extract::State;
use sqlx::PgPool;
use serde::Deserialize;
use serde_json::{json, Value};
use validator::Validate;

use crate::dtos::auth_dto::{ForgotPasswordRequest, ResetPasswordRequest};
use crate::services::email_service::EmailService;

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
        .route("/forgot-password", post(forgot_password))
        .route("/reset-password", post(reset_password))
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

async fn forgot_password(
    State(pool): State<PgPool>,
    Json(body): Json<ForgotPasswordRequest>,
) -> Json<Value> {
    if let Err(e) = body.validate() {
        return Json(json!({"success": false, "error": {"code": 400, "message": e.to_string()}}));
    }

    let email_service = EmailService::from_env();

    let reset_url = std::env::var("PASSWORD_RESET_URL")
        .unwrap_or_else(|_| "https://app.haqly-erp.com/reset-password".to_string());

    match sqlx::query_as::<_, crate::models::user::User>(
        "SELECT * FROM users WHERE email = $1 AND is_active = true",
    )
    .bind(&body.email)
    .fetch_optional(&pool)
    .await
    {
        Ok(Some(_user)) => {
            let user_id = _user.id;
            let token_raw = match generate_password_reset_token(&pool, user_id).await {
                Ok(t) => t,
                Err(e) => {
                    tracing::error!("Failed to create reset token: {}", e);
                    return Json(json!({"success": true, "message": "If an account exists with this email, a reset link has been sent."}));
                }
            };

            if let Err(e) = email_service
                .send_password_reset_email(&body.email, &token_raw, &reset_url)
                .await
            {
                tracing::warn!("Failed to send password reset email: {}", e);
            }

            Json(json!({"success": true, "message": "If an account exists with this email, a reset link has been sent."}))
        }
        Ok(None) => {
            tracing::info!("Password reset requested for non-existent email: {}", body.email);
            Json(json!({"success": true, "message": "If an account exists with this email, a reset link has been sent."}))
        }
        Err(e) => {
            tracing::error!("Database error during password reset: {}", e);
            Json(json!({"success": true, "message": "If an account exists with this email, a reset link has been sent."}))
        }
    }
}

async fn reset_password(
    State(pool): State<PgPool>,
    Json(body): Json<ResetPasswordRequest>,
) -> Json<Value> {
    if let Err(e) = body.validate() {
        return Json(json!({"success": false, "error": {"code": 400, "message": e.to_string()}}));
    }

    use argon2::password_hash::SaltString;
    use argon2::{Argon2, PasswordHasher};
    use sha2::{Digest, Sha256};

    let token_hash = hex::encode(Sha256::digest(body.token.as_bytes()));

    let reset_token = match sqlx::query_as::<_, PasswordResetTokenRow>(
        "SELECT * FROM password_reset_tokens WHERE token_hash = $1",
    )
    .bind(&token_hash)
    .fetch_optional(&pool)
    .await
    {
        Ok(Some(t)) => t,
        Ok(None) => {
            return Json(json!({"success": false, "error": {"code": 400, "message": "Invalid or expired reset token"}}));
        }
        Err(e) => {
            tracing::error!("Database error during password reset: {}", e);
            return Json(json!({"success": false, "error": {"code": 500, "message": "Internal server error"}}));
        }
    };

    if reset_token.used_at.is_some() {
        return Json(json!({"success": false, "error": {"code": 400, "message": "Token has already been used"}}));
    }

    let now = chrono::Utc::now().naive_utc();
    if reset_token.expires_at < now {
        return Json(json!({"success": false, "error": {"code": 400, "message": "Token has expired"}}));
    }

    let salt = SaltString::generate(&mut rand::rngs::OsRng);
    let argon2 = Argon2::default();
    let new_hash = match argon2.hash_password(body.new_password.as_bytes(), &salt) {
        Ok(h) => h.to_string(),
        Err(_) => {
            return Json(json!({"success": false, "error": {"code": 500, "message": "Failed to hash password"}}));
        }
    };

    match sqlx::query("UPDATE users SET password_hash = $1, updated_at = NOW() WHERE id = $2")
        .bind(&new_hash)
        .bind(reset_token.user_id)
        .execute(&pool)
        .await
    {
        Ok(_) => {}
        Err(e) => {
            tracing::error!("Failed to update password: {}", e);
            return Json(json!({"success": false, "error": {"code": 500, "message": "Failed to update password"}}));
        }
    }

    match sqlx::query("UPDATE password_reset_tokens SET used_at = NOW() WHERE id = $1")
        .bind(reset_token.id)
        .execute(&pool)
        .await
    {
        Ok(_) => {}
        Err(e) => {
            tracing::warn!("Failed to mark reset token as used: {}", e);
        }
    }

    tracing::info!("Password reset successful for user: {}", reset_token.user_id);
    Json(json!({"success": true, "message": "Password has been reset successfully"}))
}

#[derive(sqlx::FromRow)]
struct PasswordResetTokenRow {
    id: uuid::Uuid,
    user_id: uuid::Uuid,
    token_hash: String,
    expires_at: chrono::NaiveDateTime,
    used_at: Option<chrono::NaiveDateTime>,
    created_at: chrono::NaiveDateTime,
}

async fn generate_password_reset_token(
    pool: &PgPool,
    user_id: uuid::Uuid,
) -> anyhow::Result<String> {
    use sha2::{Digest, Sha256};

    let token_raw = uuid::Uuid::now_v7().to_string()
        + &uuid::Uuid::now_v7().to_string().replace("-", "");
    let token_hash = hex::encode(Sha256::digest(token_raw.as_bytes()));
    let expires_at = chrono::Utc::now().naive_utc() + chrono::Duration::hours(1);

    let id = uuid::Uuid::now_v7();
    sqlx::query(
        r#"INSERT INTO password_reset_tokens (id, user_id, token_hash, expires_at, created_at)
           VALUES ($1, $2, $3, $4, NOW())"#,
    )
    .bind(id)
    .bind(user_id)
    .bind(&token_hash)
    .bind(expires_at)
    .execute(pool)
    .await?;

    Ok(token_raw)
}
