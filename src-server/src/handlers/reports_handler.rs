// Author: Quadri Atharu
// Crate: haqly-erp-server

use axum::{
    Json, Path, Query, State,
    routing::get,
    Router,
};
use sqlx::PgPool;
use serde::Deserialize;
use serde_json::{json, Value};

#[derive(Debug, Deserialize)]
pub struct TrialBalanceQuery {
    pub company_id: Option<uuid::Uuid>,
    pub as_of_date: Option<String>,
    pub period_id: Option<uuid::Uuid>,
}

#[derive(Debug, Deserialize)]
pub struct AccountStatementSummaryQuery {
    pub company_id: Option<uuid::Uuid>,
    pub as_of_date: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct AccountStatementQuery {
    pub company_id: Option<uuid::Uuid>,
    pub account_id: Option<uuid::Uuid>,
    pub date_from: Option<String>,
    pub date_to: Option<String>,
    pub page: Option<i64>,
    pub per_page: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub struct RatioAnalysisQuery {
    pub company_id: Option<uuid::Uuid>,
    pub period_id: Option<uuid::Uuid>,
    pub as_of_date: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ExportQuery {
    pub company_id: Option<uuid::Uuid>,
    pub report_type: Option<String>,
    pub date_from: Option<String>,
    pub date_to: Option<String>,
}

pub fn router() -> Router<PgPool> {
    Router::new()
        .route("/trial-balance", get(trial_balance))
        .route("/account-statement-summary", get(account_statement_summary))
        .route("/account-statement", get(account_statement))
        .route("/financial-statements/:statement", get(financial_statements))
        .route("/ratio-analysis", get(ratio_analysis))
        .route("/export/:format", get(export_report))
}

async fn trial_balance(
    State(_pool): State<PgPool>,
    Query(_params): Query<TrialBalanceQuery>,
) -> Json<Value> {
    Json(json!({"message": "reports/trial-balance - not implemented"}))
}

async fn account_statement_summary(
    State(_pool): State<PgPool>,
    Query(_params): Query<AccountStatementSummaryQuery>,
) -> Json<Value> {
    Json(json!({"message": "reports/account-statement-summary - not implemented"}))
}

async fn account_statement(
    State(_pool): State<PgPool>,
    Query(_params): Query<AccountStatementQuery>,
) -> Json<Value> {
    Json(json!({"message": "reports/account-statement - not implemented"}))
}

async fn financial_statements(
    State(_pool): State<PgPool>,
    Path(statement): Path<String>,
) -> Json<Value> {
    Json(json!({"message": format!("reports/financial-statements/{} - not implemented", statement)}))
}

async fn ratio_analysis(
    State(_pool): State<PgPool>,
    Query(_params): Query<RatioAnalysisQuery>,
) -> Json<Value> {
    Json(json!({"message": "reports/ratio-analysis - not implemented"}))
}

async fn export_report(
    State(_pool): State<PgPool>,
    Path(_format): Path<String>,
    Query(_params): Query<ExportQuery>,
) -> Json<Value> {
    Json(json!({"message": "reports/export/:format - not implemented"}))
}
