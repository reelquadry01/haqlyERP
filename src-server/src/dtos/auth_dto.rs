// Author: Quadri Atharu
use serde::{Deserialize, Serialize};
use validator::Validate;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct LoginRequest {
    #[validate(email(message = "invalid email format"))]
    pub email: String,
    #[validate(length(min = 6, message = "password must be at least 6 characters"))]
    pub password: String,
    pub company_id: Option<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct RegisterRequest {
    #[validate(email(message = "invalid email format"))]
    pub email: String,
    #[validate(length(min = 6, message = "password must be at least 6 characters"))]
    pub password: String,
    #[validate(length(min = 2, message = "full name must be at least 2 characters"))]
    pub full_name: String,
    pub phone: Option<String>,
    pub company_id: Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthResponse {
    pub user_id: Uuid,
    pub email: String,
    pub full_name: String,
    pub access_token: String,
    pub refresh_token: String,
    pub expires_in: u64,
    pub company_id: Uuid,
    pub mfa_enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct RefreshRequest {
    #[validate(length(min = 1, message = "refresh token is required"))]
    pub refresh_token: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MfaSetupResponse {
    pub secret: String,
    pub qr_code_url: String,
    pub recovery_codes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct MfaVerifyRequest {
    pub user_id: Uuid,
    #[validate(length(min = 6, max = 6, message = "MFA code must be 6 digits"))]
    pub code: String,
}
