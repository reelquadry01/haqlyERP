// Author: Quadri Atharu
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct ValidateLicenseRequest {
    #[validate(length(min = 1, message = "license key is required"))]
    pub license_key: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LicenseStatusResponse {
    pub is_licensed: bool,
    pub tier: Option<String>,
    pub features: Vec<String>,
    pub max_users: Option<i32>,
    pub max_companies: Option<i32>,
    pub days_remaining: Option<i32>,
    pub warnings: Vec<String>,
    pub grace_period_remaining_days: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct GenerateLicenseRequest {
    pub tier: String,
    pub max_users: i32,
    pub max_companies: i32,
    pub features: Vec<String>,
    pub duration_days: i32,
    pub issued_to: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LicenseKeyResponse {
    pub key: String,
    pub tier: String,
    pub max_users: i32,
    pub max_companies: i32,
    pub features: Vec<String>,
    pub issued_at: String,
    pub expires_at: String,
    pub signature: String,
}
