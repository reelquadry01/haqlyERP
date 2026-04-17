// Author: Quadri Atharu
// Crate: haqly-erp-server

use axum::{
    Json,
    routing::{get, post, patch},
    Router,
};
use axum::extract::{Path, Query, State};
use sqlx::PgPool;
use serde::Deserialize;
use serde_json::{json, Value};
use uuid::Uuid;

#[derive(Debug, Deserialize)]
pub struct ListTaxConfigsQuery {
    pub page: Option<i64>,
    pub per_page: Option<i64>,
    pub company_id: Option<Uuid>,
    pub tax_type: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct TaxDashboardQuery {
    pub company_id: Option<Uuid>,
    pub period: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct TaxActivityQuery {
    pub page: Option<i64>,
    pub per_page: Option<i64>,
    pub company_id: Option<Uuid>,
    pub date_from: Option<String>,
    pub date_to: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ComputeTaxBody {
    pub company_id: Option<Uuid>,
    pub amount: Option<String>,
    pub tax_rate_id: Option<Uuid>,
    pub items: Option<Vec<Value>>,
}

pub fn router() -> Router<crate::routes::AppState> {
    Router::new()
        .route("/configs", get(list_tax_configs).post(create_tax_config))
        .route("/configs/:id", patch(update_tax_config))
        .route("/dashboard", get(get_tax_dashboard))
        .route("/activity", get(get_tax_activity))
        .route("/compute/vat", post(compute_vat))
        .route("/compute/wht", post(compute_wht))
        .route("/compute/cit", post(compute_cit))
        .route("/compute/all", post(compute_all))
}

async fn list_tax_configs(
    State(_pool): State<PgPool>,
    Query(_params): Query<ListTaxConfigsQuery>,
) -> Json<Value> {
    Json(json!({"message": "tax/configs GET - not implemented"}))
}

async fn create_tax_config(
    State(_pool): State<PgPool>,
    Json(_body): Json<Value>,
) -> Json<Value> {
    Json(json!({"message": "tax/configs POST - not implemented"}))
}

async fn update_tax_config(
    State(_pool): State<PgPool>,
    Path(_id): Path<Uuid>,
    Json(_body): Json<Value>,
) -> Json<Value> {
    Json(json!({"message": "tax/configs/:id PATCH - not implemented"}))
}

async fn get_tax_dashboard(
    State(_pool): State<PgPool>,
    Query(_params): Query<TaxDashboardQuery>,
) -> Json<Value> {
    Json(json!({"message": "tax/dashboard GET - not implemented"}))
}

async fn get_tax_activity(
    State(_pool): State<PgPool>,
    Query(_params): Query<TaxActivityQuery>,
) -> Json<Value> {
    Json(json!({"message": "tax/activity GET - not implemented"}))
}

async fn compute_vat(
    State(_pool): State<PgPool>,
    Json(_body): Json<ComputeTaxBody>,
) -> Json<Value> {
    Json(json!({"message": "tax/compute/vat - not implemented"}))
}

async fn compute_wht(
    State(_pool): State<PgPool>,
    Json(_body): Json<ComputeTaxBody>,
) -> Json<Value> {
    Json(json!({"message": "tax/compute/wht - not implemented"}))
}

async fn compute_cit(
    State(_pool): State<PgPool>,
    Json(_body): Json<ComputeTaxBody>,
) -> Json<Value> {
    Json(json!({"message": "tax/compute/cit - not implemented"}))
}

async fn compute_all(
    State(_pool): State<PgPool>,
    Json(_body): Json<ComputeTaxBody>,
) -> Json<Value> {
    Json(json!({"message": "tax/compute/all - not implemented"}))
}
