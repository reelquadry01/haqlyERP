// Author: Quadri Atharu
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use validator::Validate;
use crate::dtos::license_dto::{ValidateLicenseRequest, LicenseStatusResponse};
use crate::middleware::rbac::AuthenticatedUser;
use crate::services::license_service::LicenseService;

pub async fn validate_license(
    Json(req): Json<ValidateLicenseRequest>,
) -> Response {
    match req.validate() {
        Ok(_) => {
            let validation = LicenseService::validate_license(&req.license_key);
            let response = LicenseStatusResponse {
                is_licensed: validation.valid,
                tier: Some(validation.tier.to_string()),
                features: validation.features,
                max_users: None,
                max_companies: None,
                days_remaining: Some(validation.days_remaining),
                warnings: validation.warnings,
                grace_period_remaining_days: None,
            };
            (StatusCode::OK, Json(serde_json::json!({
                "success": true,
                "data": response
            }))).into_response()
        }
        Err(e) => (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({
                "success": false,
                "error": { "code": 400, "message": e.to_string() }
            })),
        )
            .into_response(),
    }
}

pub async fn get_license_status(
    user: AuthenticatedUser,
) -> Response {
    let status = LicenseService::get_license_status();
    let response = LicenseStatusResponse {
        is_licensed: status.is_licensed,
        tier: status.validation.as_ref().map(|v| v.tier.to_string()),
        features: status.validation.as_ref().map(|v| v.features.clone()).unwrap_or_default(),
        max_users: status.license.as_ref().map(|l| l.max_users),
        max_companies: status.license.as_ref().map(|l| l.max_companies),
        days_remaining: status.validation.as_ref().map(|v| v.days_remaining),
        warnings: status.validation.as_ref().map(|v| v.warnings.clone()).unwrap_or_default(),
        grace_period_remaining_days: status.grace_period_remaining_days,
    };
    (StatusCode::OK, Json(serde_json::json!({
        "success": true,
        "data": response
    }))).into_response()
}
