// Author: Quadri Atharu
// Crate: haqly-erp-server

use axum::{
    Json,
    http::StatusCode,
    routing::post,
    Router,
};
use axum::extract::State;
use axum::response::IntoResponse;
use serde::Deserialize;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use uuid::Uuid;
use validator::Validate;

use crate::dtos::auth_dto::{ForgotPasswordRequest, ResetPasswordRequest};
use crate::models::user::User;
use crate::routes::AppState;
use crate::services::auth_service::AuthService;
use crate::services::email_service::EmailService;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/login", post(login))
        .route("/register", post(register))
        .route("/refresh", post(refresh_token))
        .route("/logout", post(logout))
        .route("/mfa/status", post(mfa_status))
        .route("/mfa/setup", post(mfa_setup))
        .route("/mfa/activate", post(mfa_activate))
        .route("/mfa/disable", post(mfa_disable))
        .route("/mfa/verify", post(mfa_verify))
        .route("/mfa/verify-login", post(mfa_verify))
        .route("/mfa/recover", post(mfa_recover))
        .route("/forgot-password", post(forgot_password))
        .route("/reset-password", post(reset_password))
}

// ── helpers ─────────────────────────────────────────────────────────────────

fn auth_svc(state: &AppState) -> AuthService {
    AuthService::new(
        state.pool.clone(),
        state.rsa_keypair.clone(),
        state.jwt_expiration,
    )
}

async fn company_name(pool: &sqlx::PgPool, company_id: Uuid) -> String {
    sqlx::query_scalar::<_, String>("SELECT name FROM companies WHERE id = $1")
        .bind(company_id)
        .fetch_optional(pool)
        .await
        .ok()
        .flatten()
        .unwrap_or_default()
}

fn ok(body: Value) -> impl IntoResponse {
    (StatusCode::OK, Json(body))
}

fn err(code: StatusCode, msg: &str) -> impl IntoResponse {
    (code, Json(json!({ "message": msg })))
}

fn login_payload(token: &str, user: &User, co_name: &str) -> Value {
    json!({
        "token": token,
        "user": {
            "id":    user.id,
            "email": user.email,
            "name":  user.full_name,
        },
        "companies": [{ "id": user.company_id, "name": co_name }]
    })
}

// ── login ────────────────────────────────────────────────────────────────────

#[derive(Deserialize)]
struct LoginBody {
    email: String,
    password: String,
    #[serde(default)]
    remember_me: bool,
    #[serde(default)]
    company_id: Option<Uuid>,
}

async fn login(
    State(state): State<AppState>,
    Json(body): Json<LoginBody>,
) -> impl IntoResponse {
    let svc = auth_svc(&state);
    let req = crate::dtos::auth_dto::LoginRequest {
        email: body.email,
        password: body.password,
        company_id: body.company_id,
    };

    match svc.login(req).await {
        Err(e) => {
            tracing::warn!("Login failed: {}", e);
            err(StatusCode::UNAUTHORIZED, "Invalid email or password").into_response()
        }
        Ok(res) if res.mfa_enabled => {
            ok(json!({
                "mfaRequired": true,
                "mfaToken": res.user_id.to_string()
            })).into_response()
        }
        Ok(res) => {
            let co = company_name(&state.pool, res.company_id).await;
            let user = crate::models::user::User {
                id: res.user_id,
                company_id: Some(res.company_id),
                email: res.email,
                full_name: res.full_name,
                password_hash: String::new(),
                phone: None,
                avatar_url: None,
                is_active: true,
                mfa_enabled: false,
                mfa_secret: None,
                mfa_recovery_codes: None,
                last_login_at: None,
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
            };
            ok(login_payload(&res.access_token, &user, &co)).into_response()
        }
    }
}

// ── register ─────────────────────────────────────────────────────────────────

#[derive(Deserialize)]
struct RegisterBody {
    email: String,
    password: String,
    full_name: String,
    #[serde(default)]
    company_name: Option<String>,
    #[serde(default)]
    phone: Option<String>,
}

async fn register(
    State(state): State<AppState>,
    Json(body): Json<RegisterBody>,
) -> impl IntoResponse {
    // Create a new company for the registering user
    let co_name = body.company_name.unwrap_or_else(|| format!("{}'s Company", body.full_name));
    let company_id = Uuid::now_v7();

    if let Err(e) = sqlx::query(
        "INSERT INTO companies (id, code, name, is_active, created_at, updated_at) \
         VALUES ($1, $2, $3, true, NOW(), NOW())",
    )
    .bind(company_id)
    .bind(format!("CO-{}", &company_id.to_string()[..8].to_uppercase()))
    .bind(&co_name)
    .execute(&state.pool)
    .await
    {
        tracing::error!("Failed to create company: {}", e);
        return err(StatusCode::INTERNAL_SERVER_ERROR, "Registration failed").into_response();
    }

    let svc = auth_svc(&state);
    let req = crate::dtos::auth_dto::RegisterRequest {
        email: body.email,
        password: body.password,
        full_name: body.full_name,
        phone: body.phone,
        company_id,
    };

    match svc.register(req).await {
        Err(e) => {
            tracing::warn!("Registration failed: {}", e);
            // Roll back the company we just created
            let _ = sqlx::query("DELETE FROM companies WHERE id = $1")
                .bind(company_id)
                .execute(&state.pool)
                .await;
            err(StatusCode::BAD_REQUEST, &e.to_string()).into_response()
        }
        Ok(res) => {
            let user = crate::models::user::User {
                id: res.user_id,
                company_id: Some(res.company_id),
                email: res.email,
                full_name: res.full_name,
                password_hash: String::new(),
                phone: None,
                avatar_url: None,
                is_active: true,
                mfa_enabled: false,
                mfa_secret: None,
                mfa_recovery_codes: None,
                last_login_at: None,
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
            };
            ok(login_payload(&res.access_token, &user, &co_name)).into_response()
        }
    }
}

// ── refresh token ─────────────────────────────────────────────────────────────

#[derive(Deserialize)]
struct RefreshBody {
    refresh_token: String,
}

async fn refresh_token(
    State(state): State<AppState>,
    Json(body): Json<RefreshBody>,
) -> impl IntoResponse {
    let svc = auth_svc(&state);
    match svc.refresh_token(&body.refresh_token).await {
        Err(_) => err(StatusCode::UNAUTHORIZED, "Invalid refresh token").into_response(),
        Ok(res) => {
            let co = company_name(&state.pool, res.company_id).await;
            ok(json!({
                "token": res.access_token,
                "refresh_token": res.refresh_token,
                "expires_in": res.expires_in,
                "company_id": res.company_id,
                "company_name": co,
            })).into_response()
        }
    }
}

// ── logout ───────────────────────────────────────────────────────────────────

#[derive(Deserialize)]
struct LogoutBody {
    #[serde(default)]
    refresh_token: Option<String>,
}

async fn logout(
    State(state): State<AppState>,
    Json(body): Json<LogoutBody>,
) -> impl IntoResponse {
    if let Some(rt) = body.refresh_token {
        let _ = sqlx::query("DELETE FROM sessions WHERE refresh_token = $1")
            .bind(&rt)
            .execute(&state.pool)
            .await;
    }
    ok(json!({ "success": true }))
}

// ── MFA status ────────────────────────────────────────────────────────────────

#[derive(Deserialize)]
struct UserIdBody {
    user_id: Uuid,
}

async fn mfa_status(
    State(state): State<AppState>,
    Json(body): Json<UserIdBody>,
) -> impl IntoResponse {
    let row = sqlx::query_scalar::<_, bool>("SELECT mfa_enabled FROM users WHERE id = $1")
        .bind(body.user_id)
        .fetch_optional(&state.pool)
        .await
        .ok()
        .flatten()
        .unwrap_or(false);
    ok(json!({ "mfa_enabled": row }))
}

// ── MFA setup ─────────────────────────────────────────────────────────────────

async fn mfa_setup(
    State(state): State<AppState>,
    Json(body): Json<UserIdBody>,
) -> impl IntoResponse {
    let svc = auth_svc(&state);
    match svc.setup_mfa(body.user_id).await {
        Err(e) => err(StatusCode::BAD_REQUEST, &e.to_string()).into_response(),
        Ok(res) => ok(json!({
            "secret": res.secret,
            "qr_code_url": res.qr_code_url,
            "recovery_codes": res.recovery_codes,
        })).into_response(),
    }
}

// ── MFA activate ──────────────────────────────────────────────────────────────

#[derive(Deserialize)]
struct MfaActivateBody {
    user_id: Uuid,
    code: String,
}

async fn mfa_activate(
    State(state): State<AppState>,
    Json(body): Json<MfaActivateBody>,
) -> impl IntoResponse {
    let svc = auth_svc(&state);
    match svc.verify_mfa(body.user_id, &body.code).await {
        Ok(true) => ok(json!({ "success": true })).into_response(),
        Ok(false) => err(StatusCode::BAD_REQUEST, "Invalid MFA code").into_response(),
        Err(e) => err(StatusCode::BAD_REQUEST, &e.to_string()).into_response(),
    }
}

// ── MFA disable ───────────────────────────────────────────────────────────────

async fn mfa_disable(
    State(state): State<AppState>,
    Json(body): Json<UserIdBody>,
) -> impl IntoResponse {
    match sqlx::query(
        "UPDATE users SET mfa_enabled = false, mfa_secret = NULL, mfa_recovery_codes = NULL WHERE id = $1"
    )
    .bind(body.user_id)
    .execute(&state.pool)
    .await
    {
        Ok(_) => ok(json!({ "success": true })).into_response(),
        Err(e) => {
            tracing::error!("Failed to disable MFA: {}", e);
            err(StatusCode::INTERNAL_SERVER_ERROR, "Failed to disable MFA").into_response()
        }
    }
}

// ── MFA verify (post-login) ───────────────────────────────────────────────────

#[derive(Deserialize)]
struct MfaVerifyBody {
    #[serde(rename = "mfaToken")]
    mfa_token: String,
    code: String,
}

async fn mfa_verify(
    State(state): State<AppState>,
    Json(body): Json<MfaVerifyBody>,
) -> impl IntoResponse {
    let user_id = match Uuid::parse_str(&body.mfa_token) {
        Ok(id) => id,
        Err(_) => return err(StatusCode::BAD_REQUEST, "Invalid MFA token").into_response(),
    };

    let svc = auth_svc(&state);
    match svc.verify_mfa(user_id, &body.code).await {
        Ok(true) => {}
        Ok(false) => return err(StatusCode::UNAUTHORIZED, "Invalid MFA code").into_response(),
        Err(e) => return err(StatusCode::UNAUTHORIZED, &e.to_string()).into_response(),
    }

    // Issue a real JWT now that MFA passed
    let user = match sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = $1")
        .bind(user_id)
        .fetch_optional(&state.pool)
        .await
    {
        Ok(Some(u)) => u,
        _ => return err(StatusCode::UNAUTHORIZED, "User not found").into_response(),
    };

    let role = sqlx::query_scalar::<_, String>(
        "SELECT r.name FROM user_roles ur JOIN roles r ON ur.role_id = r.id WHERE ur.user_id = $1 LIMIT 1"
    )
    .bind(user_id)
    .fetch_optional(&state.pool)
    .await
    .ok()
    .flatten()
    .unwrap_or_else(|| "Viewer".to_string());

    let token = match svc.generate_jwt(&user, &role, state.jwt_expiration) {
        Ok(t) => t,
        Err(_) => return err(StatusCode::INTERNAL_SERVER_ERROR, "Failed to generate token").into_response(),
    };

    // Update last_login_at and create session
    let _ = sqlx::query("UPDATE users SET last_login_at = NOW() WHERE id = $1")
        .bind(user_id)
        .execute(&state.pool)
        .await;

    let session_id = Uuid::now_v7();
    let refresh_token = Uuid::now_v7().to_string();
    let expires_at = chrono::Utc::now()
        + chrono::Duration::seconds(state.jwt_expiration as i64);
    let _ = sqlx::query(
        "INSERT INTO sessions (id, user_id, token, refresh_token, expires_at, created_at) VALUES ($1,$2,$3,$4,$5,NOW())"
    )
    .bind(session_id).bind(user_id).bind(&token).bind(&refresh_token).bind(expires_at)
    .execute(&state.pool)
    .await;

    let co = company_name(&state.pool, user.company_id.unwrap_or_default()).await;
    ok(login_payload(&token, &user, &co)).into_response()
}

// ── MFA recovery code ─────────────────────────────────────────────────────────

#[derive(Deserialize)]
struct MfaRecoverBody {
    #[serde(rename = "mfaToken")]
    mfa_token: String,
    #[serde(rename = "recoveryCode")]
    recovery_code: String,
}

async fn mfa_recover(
    State(state): State<AppState>,
    Json(body): Json<MfaRecoverBody>,
) -> impl IntoResponse {
    let user_id = match Uuid::parse_str(&body.mfa_token) {
        Ok(id) => id,
        Err(_) => return err(StatusCode::BAD_REQUEST, "Invalid MFA token").into_response(),
    };

    let user = match sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = $1")
        .bind(user_id)
        .fetch_optional(&state.pool)
        .await
    {
        Ok(Some(u)) => u,
        _ => return err(StatusCode::UNAUTHORIZED, "User not found").into_response(),
    };

    // Verify recovery code against stored hashes
    let codes: Vec<String> = user
        .mfa_recovery_codes
        .as_ref()
        .and_then(|v| serde_json::from_value(v.clone()).ok())
        .unwrap_or_default();

    use argon2::{Argon2, PasswordHash, PasswordVerifier};
    let svc = auth_svc(&state);
    let matched_hash = codes.iter().find(|hash| {
        PasswordHash::new(hash)
            .ok()
            .map(|h| Argon2::default().verify_password(body.recovery_code.as_bytes(), &h).is_ok())
            .unwrap_or(false)
    });

    if matched_hash.is_none() {
        return err(StatusCode::UNAUTHORIZED, "Invalid recovery code").into_response();
    }

    // Remove used code
    let remaining: Vec<&String> = codes.iter().filter(|h| *h != matched_hash.unwrap()).collect();
    let _ = sqlx::query("UPDATE users SET mfa_recovery_codes = $1 WHERE id = $2")
        .bind(serde_json::to_value(&remaining).unwrap_or(serde_json::Value::Null))
        .bind(user_id)
        .execute(&state.pool)
        .await;

    let role = sqlx::query_scalar::<_, String>(
        "SELECT r.name FROM user_roles ur JOIN roles r ON ur.role_id = r.id WHERE ur.user_id = $1 LIMIT 1"
    )
    .bind(user_id)
    .fetch_optional(&state.pool)
    .await
    .ok()
    .flatten()
    .unwrap_or_else(|| "Viewer".to_string());

    let token = match svc.generate_jwt(&user, &role, state.jwt_expiration) {
        Ok(t) => t,
        Err(_) => return err(StatusCode::INTERNAL_SERVER_ERROR, "Failed to generate token").into_response(),
    };

    let session_id = Uuid::now_v7();
    let refresh = Uuid::now_v7().to_string();
    let expires_at = chrono::Utc::now()
        + chrono::Duration::seconds(state.jwt_expiration as i64);
    let _ = sqlx::query(
        "INSERT INTO sessions (id, user_id, token, refresh_token, expires_at, created_at) VALUES ($1,$2,$3,$4,$5,NOW())"
    )
    .bind(session_id).bind(user_id).bind(&token).bind(&refresh).bind(expires_at)
    .execute(&state.pool)
    .await;

    let co = company_name(&state.pool, user.company_id.unwrap_or_default()).await;
    ok(login_payload(&token, &user, &co)).into_response()
}

// ── forgot password ───────────────────────────────────────────────────────────

async fn forgot_password(
    State(state): State<AppState>,
    Json(body): Json<ForgotPasswordRequest>,
) -> impl IntoResponse {
    if let Err(e) = body.validate() {
        return err(StatusCode::BAD_REQUEST, &e.to_string()).into_response();
    }

    let email_service = EmailService::from_env();
    let reset_url = std::env::var("PASSWORD_RESET_URL")
        .unwrap_or_else(|_| "https://app.haqly-erp.com/reset-password".to_string());

    if let Ok(Some(user)) = sqlx::query_as::<_, User>(
        "SELECT * FROM users WHERE email = $1 AND is_active = true",
    )
    .bind(&body.email)
    .fetch_optional(&state.pool)
    .await
    {
        if let Ok(token) = generate_reset_token(&state.pool, user.id).await {
            let _ = email_service
                .send_password_reset_email(&body.email, &token, &reset_url)
                .await;
        }
    }

    ok(json!({ "success": true, "message": "If an account exists with this email, a reset link has been sent." }))
        .into_response()
}

// ── reset password ────────────────────────────────────────────────────────────

async fn reset_password(
    State(state): State<AppState>,
    Json(body): Json<ResetPasswordRequest>,
) -> impl IntoResponse {
    if let Err(e) = body.validate() {
        return err(StatusCode::BAD_REQUEST, &e.to_string()).into_response();
    }

    use argon2::password_hash::SaltString;
    use argon2::{Argon2, PasswordHasher};

    let token_hash = hex::encode(Sha256::digest(body.token.as_bytes()));

    let reset_token = match sqlx::query_as::<_, PasswordResetTokenRow>(
        "SELECT * FROM password_reset_tokens WHERE token_hash = $1",
    )
    .bind(&token_hash)
    .fetch_optional(&state.pool)
    .await
    {
        Ok(Some(t)) => t,
        _ => return err(StatusCode::BAD_REQUEST, "Invalid or expired reset token").into_response(),
    };

    if reset_token.used_at.is_some() {
        return err(StatusCode::BAD_REQUEST, "Token has already been used").into_response();
    }
    if reset_token.expires_at < chrono::Utc::now() {
        return err(StatusCode::BAD_REQUEST, "Token has expired").into_response();
    }

    let salt = SaltString::generate(&mut rand::rngs::OsRng);
    let new_hash = match Argon2::default().hash_password(body.new_password.as_bytes(), &salt) {
        Ok(h) => h.to_string(),
        Err(_) => return err(StatusCode::INTERNAL_SERVER_ERROR, "Failed to hash password").into_response(),
    };

    let _ = sqlx::query("UPDATE users SET password_hash = $1, updated_at = NOW() WHERE id = $2")
        .bind(&new_hash)
        .bind(reset_token.user_id)
        .execute(&state.pool)
        .await;

    let _ = sqlx::query("UPDATE password_reset_tokens SET used_at = NOW() WHERE id = $1")
        .bind(reset_token.id)
        .execute(&state.pool)
        .await;

    ok(json!({ "success": true, "message": "Password has been reset successfully" }))
        .into_response()
}

// ── shared helpers ────────────────────────────────────────────────────────────

#[derive(sqlx::FromRow)]
struct PasswordResetTokenRow {
    id: Uuid,
    user_id: Uuid,
    #[allow(dead_code)]
    token_hash: String,
    expires_at: chrono::DateTime<chrono::Utc>,
    used_at: Option<chrono::DateTime<chrono::Utc>>,
    #[allow(dead_code)]
    created_at: chrono::DateTime<chrono::Utc>,
}

async fn generate_reset_token(pool: &sqlx::PgPool, user_id: Uuid) -> anyhow::Result<String> {
    let token_raw = Uuid::now_v7().to_string() + &Uuid::now_v7().to_string().replace('-', "");
    let token_hash = hex::encode(Sha256::digest(token_raw.as_bytes()));
    let expires_at = chrono::Utc::now() + chrono::Duration::hours(1);
    sqlx::query(
        "INSERT INTO password_reset_tokens (id, user_id, token_hash, expires_at, created_at) VALUES ($1,$2,$3,$4,NOW())",
    )
    .bind(Uuid::now_v7())
    .bind(user_id)
    .bind(&token_hash)
    .bind(expires_at)
    .execute(pool)
    .await?;
    Ok(token_raw)
}
