// Author: Quadri Atharu
use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type, PartialEq)]
#[sqlx(type_name = "license_tier", rename_all = "snake_case")]
pub enum LicenseTier {
    Starter,
    Professional,
    Enterprise,
    Government,
}

impl std::fmt::Display for LicenseTier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LicenseTier::Starter => write!(f, "starter"),
            LicenseTier::Professional => write!(f, "professional"),
            LicenseTier::Enterprise => write!(f, "enterprise"),
            LicenseTier::Government => write!(f, "government"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct LicenseKey {
    pub id: Uuid,
    pub key: String,
    pub tier: LicenseTier,
    pub max_users: i32,
    pub max_companies: i32,
    pub features: serde_json::Value,
    pub issued_at: NaiveDateTime,
    pub expires_at: NaiveDateTime,
    pub is_active: bool,
    pub issued_to: String,
    pub signature: String,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LicenseValidation {
    pub valid: bool,
    pub tier: LicenseTier,
    pub features: Vec<String>,
    pub expires_at: NaiveDateTime,
    pub days_remaining: i32,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LicenseStatus {
    pub license: Option<LicenseKey>,
    pub validation: Option<LicenseValidation>,
    pub is_licensed: bool,
    pub grace_period_remaining_days: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct FeatureFlag {
    pub id: Uuid,
    pub key: String,
    pub display_name: String,
    pub description: Option<String>,
    pub tier_required: LicenseTier,
    pub is_active: bool,
    pub created_at: NaiveDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct SubscriptionRecord {
    pub id: Uuid,
    pub company_id: Uuid,
    pub license_key_id: Uuid,
    pub tier: LicenseTier,
    pub status: String,
    pub started_at: NaiveDateTime,
    pub current_period_start: NaiveDateTime,
    pub current_period_end: NaiveDateTime,
    pub cancel_at_period_end: bool,
    pub amount: serde_json::Value,
    pub payment_method: Option<String>,
    pub last_payment_at: Option<NaiveDateTime>,
    pub next_payment_at: Option<NaiveDateTime>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}
