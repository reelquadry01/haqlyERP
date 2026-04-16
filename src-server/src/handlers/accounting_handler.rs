// Author: Quadri Atharu
// Crate: haqly-erp-server

use axum::{
    Json,
    routing::{get, post},
    Router,
};
use axum::extract::{Path, Query, State};
use sqlx::PgPool;
use serde::Deserialize;
use serde_json::{json, Value};
use uuid::Uuid;

#[derive(Debug, Deserialize)]
pub struct ListAccountsQuery {
    pub page: Option<i64>,
    pub per_page: Option<i64>,
    pub account_type: Option<String>,
    pub parent_id: Option<Uuid>,
    pub is_active: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct ListFiscalYearsQuery {
    pub page: Option<i64>,
    pub per_page: Option<i64>,
    pub company_id: Option<Uuid>,
}

#[derive(Debug, Deserialize)]
pub struct ListPeriodsQuery {
    pub fiscal_year_id: Option<Uuid>,
    pub page: Option<i64>,
    pub per_page: Option<i64>,
}

pub fn router() -> Router<PgPool> {
    Router::new()
        .route("/accounts", post(create_account).get(list_accounts))
        .route("/accounts/:id", get(get_account))
        .route("/fiscal-years", post(create_fiscal_year).get(list_fiscal_years))
        .route("/fiscal-years/:id/periods", post(create_period))
        .route("/periods", get(list_periods))
}

async fn create_account(
    State(_pool): State<PgPool>,
    Json(_body): Json<Value>,
) -> Json<Value> {
    Json(json!({"message": "accounting/accounts POST - not implemented"}))
}

async fn list_accounts(
    State(_pool): State<PgPool>,
    Query(_params): Query<ListAccountsQuery>,
) -> Json<Value> {
    Json(json!({"message": "accounting/accounts GET - not implemented"}))
}

async fn get_account(
    State(_pool): State<PgPool>,
    Path(_id): Path<Uuid>,
) -> Json<Value> {
    Json(json!({"message": "accounting/accounts/:id GET - not implemented"}))
}

async fn create_fiscal_year(
    State(_pool): State<PgPool>,
    Json(_body): Json<Value>,
) -> Json<Value> {
    Json(json!({"message": "accounting/fiscal-years POST - not implemented"}))
}

async fn list_fiscal_years(
    State(_pool): State<PgPool>,
    Query(_params): Query<ListFiscalYearsQuery>,
) -> Json<Value> {
    Json(json!({"message": "accounting/fiscal-years GET - not implemented"}))
}

async fn create_period(
    State(_pool): State<PgPool>,
    Path(_id): Path<Uuid>,
    Json(_body): Json<Value>,
) -> Json<Value> {
    Json(json!({"message": "accounting/fiscal-years/:id/periods POST - not implemented"}))
}

async fn list_periods(
    State(_pool): State<PgPool>,
    Query(_params): Query<ListPeriodsQuery>,
) -> Json<Value> {
    Json(json!({"message": "accounting/periods GET - not implemented"}))
}
