// Author: Quadri Atharu
// Crate: haqly-erp-server

use axum::{
    Json, Path, Query, State,
    routing::{get, post, patch},
    Router,
};
use sqlx::PgPool;
use serde::Deserialize;
use serde_json::{json, Value};
use uuid::Uuid;

use crate::security::audit_chain;

#[derive(Debug, Deserialize)]
pub struct ListFiscalYearsQuery {
    pub page: Option<i64>,
    pub per_page: Option<i64>,
    pub company_id: Option<Uuid>,
}

#[derive(Debug, Deserialize)]
pub struct ApprovalRulesQuery {
    pub company_id: Option<Uuid>,
    pub entity_type: Option<String>,
}

pub fn router() -> Router<PgPool> {
    Router::new()
        .route("/permissions", post(create_permission))
        .route("/roles", post(create_role))
        .route("/users/:user_id/roles", post(assign_user_roles))
        .route("/fiscal-years", get(list_fiscal_years).post(create_fiscal_year))
        .route("/fiscal-years/:fiscal_year_id/close", post(close_fiscal_year))
        .route("/fiscal-years/:fiscal_year_id/lock", post(lock_fiscal_year))
        .route("/accounting-periods/:period_id", patch(update_accounting_period))
        .route("/settings/:company_id", get(get_settings).patch(update_settings))
        .route("/approval-rules", get(list_approval_rules).post(create_approval_rule))
        .route("/audit-chain/verify", post(verify_audit_chain))
        .route("/audit-chain/status", get(audit_chain_status))
}

async fn create_permission(
    State(_pool): State<PgPool>,
    Json(_body): Json<Value>,
) -> Json<Value> {
    Json(json!({"message": "admin/permissions POST - not implemented"}))
}

async fn create_role(
    State(_pool): State<PgPool>,
    Json(_body): Json<Value>,
) -> Json<Value> {
    Json(json!({"message": "admin/roles POST - not implemented"}))
}

async fn assign_user_roles(
    State(_pool): State<PgPool>,
    Path(_user_id): Path<Uuid>,
    Json(_body): Json<Value>,
) -> Json<Value> {
    Json(json!({"message": "admin/users/:userId/roles POST - not implemented"}))
}

async fn list_fiscal_years(
    State(_pool): State<PgPool>,
    Query(_params): Query<ListFiscalYearsQuery>,
) -> Json<Value> {
    Json(json!({"message": "admin/fiscal-years GET - not implemented"}))
}

async fn create_fiscal_year(
    State(_pool): State<PgPool>,
    Json(_body): Json<Value>,
) -> Json<Value> {
    Json(json!({"message": "admin/fiscal-years POST - not implemented"}))
}

async fn close_fiscal_year(
    State(_pool): State<PgPool>,
    Path(_fiscal_year_id): Path<Uuid>,
    Json(_body): Json<Value>,
) -> Json<Value> {
    Json(json!({"message": "admin/fiscal-years/:fiscalYearId/close - not implemented"}))
}

async fn lock_fiscal_year(
    State(_pool): State<PgPool>,
    Path(_fiscal_year_id): Path<Uuid>,
    Json(_body): Json<Value>,
) -> Json<Value> {
    Json(json!({"message": "admin/fiscal-years/:fiscalYearId/lock - not implemented"}))
}

async fn update_accounting_period(
    State(_pool): State<PgPool>,
    Path(_period_id): Path<Uuid>,
    Json(_body): Json<Value>,
) -> Json<Value> {
    Json(json!({"message": "admin/accounting-periods/:periodId PATCH - not implemented"}))
}

async fn get_settings(
    State(_pool): State<PgPool>,
    Path(_company_id): Path<Uuid>,
) -> Json<Value> {
    Json(json!({"message": "admin/settings/:companyId GET - not implemented"}))
}

async fn update_settings(
    State(_pool): State<PgPool>,
    Path(_company_id): Path<Uuid>,
    Json(_body): Json<Value>,
) -> Json<Value> {
    Json(json!({"message": "admin/settings/:companyId PATCH - not implemented"}))
}

async fn list_approval_rules(
    State(_pool): State<PgPool>,
    Query(_params): Query<ApprovalRulesQuery>,
) -> Json<Value> {
    Json(json!({"message": "admin/approval-rules GET - not implemented"}))
}

async fn create_approval_rule(
    State(_pool): State<PgPool>,
    Json(_body): Json<Value>,
) -> Json<Value> {
    Json(json!({"message": "admin/approval-rules POST - not implemented"}))
}

async fn verify_audit_chain(
    State(pool): State<PgPool>,
) -> Json<Value> {
    match audit_chain::verify_chain(&pool).await {
        Ok(result) => Json(json!({
            "is_intact": result.is_intact,
            "total_entries": result.total_entries,
            "first_broken_entry_id": result.first_broken_entry_id,
            "verified_at": result.verified_at.to_rfc3339(),
        })),
        Err(e) => Json(json!({"error": e})),
    }
}

async fn audit_chain_status(
    State(pool): State<PgPool>,
) -> Json<Value> {
    match audit_chain::get_last_verification(&pool).await {
        Ok(Some(result)) => Json(json!({
            "is_intact": result.is_intact,
            "total_entries": result.total_entries,
            "first_broken_entry_id": result.first_broken_entry_id,
            "verified_at": result.verified_at.to_rfc3339(),
        })),
        Ok(None) => Json(json!({
            "is_intact": null,
            "total_entries": 0,
            "first_broken_entry_id": null,
            "verified_at": null,
            "message": "No verification has been performed yet",
        })),
        Err(e) => Json(json!({"error": e})),
    }
}
