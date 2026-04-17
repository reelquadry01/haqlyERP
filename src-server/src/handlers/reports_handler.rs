// Author: Quadri Atharu
// Crate: haqly-erp-server

use axum::{
    Json,
    routing::get,
    Router,
};
use axum::extract::{Path, Query, State};
use chrono::NaiveDate;
use sqlx::PgPool;
use serde::Deserialize;
use serde_json::{json, Value};

use crate::services::reports_service::ReportsService;

#[derive(Debug, Deserialize)]
pub struct TrialBalanceQuery {
    pub company_id: Option<uuid::Uuid>,
    pub as_of_date: Option<String>,
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
pub struct IncomeStatementQuery {
    pub company_id: Option<uuid::Uuid>,
    pub period_start: Option<String>,
    pub period_end: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct BalanceSheetQuery {
    pub company_id: Option<uuid::Uuid>,
    pub as_at_date: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CashFlowQuery {
    pub company_id: Option<uuid::Uuid>,
    pub period_start: Option<String>,
    pub period_end: Option<String>,
    pub method: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct RetainedEarningsQuery {
    pub company_id: Option<uuid::Uuid>,
    pub period_start: Option<String>,
    pub period_end: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct RatioAnalysisQuery {
    pub company_id: Option<uuid::Uuid>,
    pub period_id: Option<uuid::Uuid>,
    pub as_of_date: Option<String>,
    pub date_from: Option<String>,
    pub date_to: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ExportQuery {
    pub company_id: Option<uuid::Uuid>,
    pub report_type: Option<String>,
    pub date_from: Option<String>,
    pub date_to: Option<String>,
}

pub fn router() -> Router<crate::routes::AppState> {
    Router::new()
        .route("/trial-balance", get(trial_balance))
        .route("/account-statement-summary", get(account_statement_summary))
        .route("/account-statement", get(account_statement))
        .route("/income-statement", get(income_statement))
        .route("/balance-sheet", get(balance_sheet))
        .route("/cash-flow", get(cash_flow))
        .route("/retained-earnings", get(retained_earnings))
        .route("/ratio-analysis", get(ratio_analysis))
        .route("/export/{format}", get(export_report))
}

fn parse_date(s: &Option<String>, field: &str) -> Result<NaiveDate, Value> {
    match s {
        Some(d) => NaiveDate::parse_from_str(d, "%Y-%m-%d")
            .map_err(|_| json!({"error": format!("invalid {} format, use YYYY-MM-DD", field)})),
        None => Err(json!({"error": format!("{} is required", field)})),
    }
}

async fn trial_balance(
    State(pool): State<PgPool>,
    Query(params): Query<TrialBalanceQuery>,
) -> Json<Value> {
    let company_id = match params.company_id {
        Some(id) => id,
        None => return Json(json!({"error": "company_id is required"})),
    };
    let as_of_date = match &params.as_of_date {
        Some(d) => match NaiveDate::parse_from_str(d, "%Y-%m-%d") {
            Ok(date) => Some(date),
            Err(_) => return Json(json!({"error": "invalid as_of_date format, use YYYY-MM-DD"})),
        },
        None => None,
    };

    let service = ReportsService::new(pool);
    match service.trial_balance(company_id, as_of_date).await {
        Ok(report) => Json(json!(report)),
        Err(e) => Json(json!({"error": e.to_string()})),
    }
}

async fn account_statement_summary(
    State(pool): State<PgPool>,
    Query(params): Query<AccountStatementSummaryQuery>,
) -> Json<Value> {
    let company_id = match params.company_id {
        Some(id) => id,
        None => return Json(json!({"error": "company_id is required"})),
    };
    let as_of_date = match &params.as_of_date {
        Some(d) => match NaiveDate::parse_from_str(d, "%Y-%m-%d") {
            Ok(date) => Some(date),
            Err(_) => return Json(json!({"error": "invalid as_of_date format, use YYYY-MM-DD"})),
        },
        None => None,
    };

    let service = ReportsService::new(pool);
    match service.trial_balance(company_id, as_of_date).await {
        Ok(report) => Json(json!(report)),
        Err(e) => Json(json!({"error": e.to_string()})),
    }
}

async fn account_statement(
    State(pool): State<PgPool>,
    Query(params): Query<AccountStatementQuery>,
) -> Json<Value> {
    let account_id = match params.account_id {
        Some(id) => id,
        None => return Json(json!({"error": "account_id is required"})),
    };
    let from_date = match parse_date(&params.date_from, "date_from") {
        Ok(d) => d,
        Err(e) => return Json(e),
    };
    let to_date = match parse_date(&params.date_to, "date_to") {
        Ok(d) => d,
        Err(e) => return Json(e),
    };

    let service = ReportsService::new(pool);
    match service.account_statement(account_id, from_date, to_date).await {
        Ok(report) => Json(json!(report)),
        Err(e) => Json(json!({"error": e.to_string()})),
    }
}

async fn income_statement(
    State(pool): State<PgPool>,
    Query(params): Query<IncomeStatementQuery>,
) -> Json<Value> {
    let company_id = match params.company_id {
        Some(id) => id,
        None => return Json(json!({"error": "company_id is required"})),
    };
    let from_date = match parse_date(&params.period_start, "period_start") {
        Ok(d) => d,
        Err(e) => return Json(e),
    };
    let to_date = match parse_date(&params.period_end, "period_end") {
        Ok(d) => d,
        Err(e) => return Json(e),
    };

    let service = ReportsService::new(pool);
    match service.income_statement(company_id, from_date, to_date).await {
        Ok(report) => Json(json!(report)),
        Err(e) => Json(json!({"error": e.to_string()})),
    }
}

async fn balance_sheet(
    State(pool): State<PgPool>,
    Query(params): Query<BalanceSheetQuery>,
) -> Json<Value> {
    let company_id = match params.company_id {
        Some(id) => id,
        None => return Json(json!({"error": "company_id is required"})),
    };
    let as_at_date = match parse_date(&params.as_at_date, "as_at_date") {
        Ok(d) => d,
        Err(e) => return Json(e),
    };

    let service = ReportsService::new(pool);
    match service.balance_sheet(company_id, as_at_date).await {
        Ok(report) => Json(json!(report)),
        Err(e) => Json(json!({"error": e.to_string()})),
    }
}

async fn cash_flow(
    State(pool): State<PgPool>,
    Query(params): Query<CashFlowQuery>,
) -> Json<Value> {
    let company_id = match params.company_id {
        Some(id) => id,
        None => return Json(json!({"error": "company_id is required"})),
    };
    let from_date = match parse_date(&params.period_start, "period_start") {
        Ok(d) => d,
        Err(e) => return Json(e),
    };
    let to_date = match parse_date(&params.period_end, "period_end") {
        Ok(d) => d,
        Err(e) => return Json(e),
    };
    let method = params.method.unwrap_or_else(|| "indirect".to_string());

    let service = ReportsService::new(pool);
    match service.cash_flow_statement(company_id, from_date, to_date, &method).await {
        Ok(report) => Json(json!(report)),
        Err(e) => Json(json!({"error": e.to_string()})),
    }
}

async fn retained_earnings(
    State(pool): State<PgPool>,
    Query(params): Query<RetainedEarningsQuery>,
) -> Json<Value> {
    let company_id = match params.company_id {
        Some(id) => id,
        None => return Json(json!({"error": "company_id is required"})),
    };
    let from_date = match parse_date(&params.period_start, "period_start") {
        Ok(d) => d,
        Err(e) => return Json(e),
    };
    let to_date = match parse_date(&params.period_end, "period_end") {
        Ok(d) => d,
        Err(e) => return Json(e),
    };

    let service = ReportsService::new(pool);
    match service.retained_earnings(company_id, from_date, to_date).await {
        Ok(report) => Json(json!(report)),
        Err(e) => Json(json!({"error": e.to_string()})),
    }
}

async fn ratio_analysis(
    State(pool): State<PgPool>,
    Query(params): Query<RatioAnalysisQuery>,
) -> Json<Value> {
    let company_id = match params.company_id {
        Some(id) => id,
        None => return Json(json!({"error": "company_id is required"})),
    };
    let from_date = match parse_date(&params.date_from, "date_from") {
        Ok(d) => d,
        Err(e) => return Json(e),
    };
    let to_date = match parse_date(&params.date_to, "date_to") {
        Ok(d) => d,
        Err(e) => return Json(e),
    };

    let service = ReportsService::new(pool);
    match service.ratio_analysis(company_id, from_date, to_date).await {
        Ok(report) => Json(json!(report)),
        Err(e) => Json(json!({"error": e.to_string()})),
    }
}

async fn export_report(
    State(pool): State<PgPool>,
    Path(_format): Path<String>,
    Query(_params): Query<ExportQuery>,
) -> Json<Value> {
    Json(json!({"message": "reports/export - not implemented"}))
}
