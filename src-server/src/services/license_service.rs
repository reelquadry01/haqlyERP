// Author: Quadri Atharu
use crate::models::license::{LicenseKey, LicenseTier, LicenseValidation, LicenseStatus, FeatureFlag, SubscriptionRecord};
use chrono::{NaiveDateTime, Utc};
use uuid::Uuid;

pub struct LicenseService;

impl LicenseService {
    pub fn validate_license(key: &str) -> LicenseValidation {
        let tier = Self::extract_tier_from_key(key);
        let features = Self::tier_features(&tier);
        let now = Utc::now().naive_utc();
        let expires_at = now + chrono::Duration::days(365);

        let days_remaining = (expires_at - now).num_days() as i32;
        let mut warnings = Vec::new();

        if days_remaining <= 0 {
            warnings.push("License has expired".to_string());
        } else if days_remaining <= 30 {
            warnings.push(format!("License expires in {} days", days_remaining));
        }

        LicenseValidation {
            valid: days_remaining > 0,
            tier,
            features,
            expires_at,
            days_remaining,
            warnings,
        }
    }

    pub fn check_feature_allowed(key: &str, feature: &str) -> bool {
        let tier = Self::extract_tier_from_key(key);
        let features = Self::tier_features(&tier);
        features.contains(&feature.to_string())
    }

    pub fn check_user_limit(key: &str, current_users: i32) -> bool {
        let tier = Self::extract_tier_from_key(key);
        let max_users = Self::tier_max_users(&tier);
        if max_users == 0 {
            return true;
        }
        current_users <= max_users
    }

    pub fn generate_license(
        tier: LicenseTier,
        max_users: i32,
        max_companies: i32,
        features: Vec<String>,
        duration_days: i32,
    ) -> LicenseKey {
        let key = Self::generate_key_string(&tier);
        let now = Utc::now().naive_utc();
        let expires_at = now + chrono::Duration::days(duration_days as i64);

        LicenseKey {
            id: Uuid::new_v4(),
            key,
            tier,
            max_users,
            max_companies,
            features: serde_json::json!(features),
            issued_at: now,
            expires_at,
            is_active: true,
            issued_to: String::new(),
            signature: Self::sign_key(&key),
            created_at: now,
            updated_at: now,
        }
    }

    pub fn get_license_status() -> LicenseStatus {
        LicenseStatus {
            license: None,
            validation: None,
            is_licensed: false,
            grace_period_remaining_days: Some(7),
        }
    }

    fn extract_tier_from_key(key: &str) -> LicenseTier {
        if key.contains("starter") {
            LicenseTier::Starter
        } else if key.contains("professional") {
            LicenseTier::Professional
        } else if key.contains("enterprise") {
            LicenseTier::Enterprise
        } else if key.contains("government") {
            LicenseTier::Government
        } else {
            LicenseTier::Starter
        }
    }

    fn tier_features(tier: &LicenseTier) -> Vec<String> {
        let base = vec![
            "accounting".to_string(),
            "tax_vat".to_string(),
            "tax_paye".to_string(),
            "tax_wht".to_string(),
            "einvoicing_basic".to_string(),
            "reports_basic".to_string(),
        ];

        match tier {
            LicenseTier::Starter => base,
            LicenseTier::Professional => {
                let mut features = base;
                features.extend(vec![
                    "tax_all".to_string(),
                    "einvoicing_full".to_string(),
                    "payroll".to_string(),
                    "bi_basic".to_string(),
                    "crm_basic".to_string(),
                    "reports_advanced".to_string(),
                ]);
                features
            }
            LicenseTier::Enterprise => {
                let mut features = base;
                features.extend(vec![
                    "tax_all".to_string(),
                    "einvoicing_full".to_string(),
                    "payroll".to_string(),
                    "payroll_loans".to_string(),
                    "bi_full".to_string(),
                    "crm_full".to_string(),
                    "ai_agents".to_string(),
                    "ocr".to_string(),
                    "reports_advanced".to_string(),
                    "api_access".to_string(),
                    "custom_integrations".to_string(),
                    "multi_company".to_string(),
                ]);
                features
            }
            LicenseTier::Government => {
                let mut features = base;
                features.extend(vec![
                    "tax_all".to_string(),
                    "einvoicing_full".to_string(),
                    "payroll".to_string(),
                    "bi_full".to_string(),
                    "crm_full".to_string(),
                    "ai_agents".to_string(),
                    "ocr".to_string(),
                    "on_premise".to_string(),
                    "audit_trail_enhanced".to_string(),
                    "compliance_reports".to_string(),
                    "custom_dashboards".to_string(),
                    "api_access".to_string(),
                    "custom_integrations".to_string(),
                    "multi_company".to_string(),
                ]);
                features
            }
        }
    }

    fn tier_max_users(tier: &LicenseTier) -> i32 {
        match tier {
            LicenseTier::Starter => 5,
            LicenseTier::Professional => 20,
            LicenseTier::Enterprise => 0,
            LicenseTier::Government => 0,
        }
    }

    fn generate_key_string(tier: &LicenseTier) -> String {
        let tier_str = match tier {
            LicenseTier::Starter => "STA",
            LicenseTier::Professional => "PRO",
            LicenseTier::Enterprise => "ENT",
            LicenseTier::Government => "GOV",
        };
        let random_part: String = (0..24)
            .map(|_| {
                let chars = "ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";
                let idx = (rand::random::<f64>() * chars.len() as f64) as usize;
                chars.chars().nth(idx).unwrap()
            })
            .collect();
        format!("HAQLY-{}-{}", tier_str, random_part)
    }

    fn sign_key(key: &str) -> String {
        use sha2::{Sha256, Digest};
        let mut hasher = Sha256::new();
        hasher.update(key.as_bytes());
        hasher.update(b"HAQLY_SIGNING_KEY_v1");
        let result = hasher.finalize();
        hex::encode(result)
    }
}
