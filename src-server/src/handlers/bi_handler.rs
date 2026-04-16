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
pub struct ListDashboardsQuery {
    pub page: Option<i64>,
    pub per_page: Option<i64>,
    pub company_id: Option<Uuid>,
}

#[derive(Debug, Deserialize)]
pub struct KpisQuery {
    pub company_id: Option<Uuid>,
    pub period: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct FinancialSummaryQuery {
    pub company_id: Option<Uuid>,
    pub period_id: Option<Uuid>,
    pub as_of_date: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ListDatasetsQuery {
    pub page: Option<i64>,
    pub per_page: Option<i64>,
    pub company_id: Option<Uuid>,
}

pub fn router() -> Router<PgPool> {
    Router::new()
        .route("/dashboards", get(list_dashboards).post(create_dashboard))
        .route("/dashboards/:id", get(get_dashboard))
        .route("/dashboards/:id/widgets", post(add_widget))
        .route("/kpis", get(get_kpis))
        .route("/financial-summary", get(get_financial_summary))
        .route("/datasets", post(create_dataset).get(list_datasets))
        .route("/query", post(execute_query))
}

async fn list_dashboards(
    State(_pool): State<PgPool>,
    Query(_params): Query<ListDashboardsQuery>,
) -> Json<Value> {
    Json(json!({"message": "bi/dashboards GET - not implemented"}))
}

async fn create_dashboard(
    State(_pool): State<PgPool>,
    Json(_body): Json<Value>,
) -> Json<Value> {
    Json(json!({"message": "bi/dashboards POST - not implemented"}))
}

async fn get_dashboard(
    State(_pool): State<PgPool>,
    Path(_id): Path<Uuid>,
) -> Json<Value> {
    Json(json!({"message": "bi/dashboards/:id GET - not implemented"}))
}

async fn add_widget(
    State(_pool): State<PgPool>,
    Path(_id): Path<Uuid>,
    Json(_body): Json<Value>,
) -> Json<Value> {
    Json(json!({"message": "bi/dashboards/:id/widgets POST - not implemented"}))
}

async fn get_kpis(
    State(_pool): State<PgPool>,
    Query(_params): Query<KpisQuery>,
) -> Json<Value> {
    Json(json!({"message": "bi/kpis GET - not implemented"}))
}

async fn get_financial_summary(
    State(_pool): State<PgPool>,
    Query(_params): Query<FinancialSummaryQuery>,
) -> Json<Value> {
    Json(json!({"message": "bi/financial-summary GET - not implemented"}))
}

async fn create_dataset(
    State(_pool): State<PgPool>,
    Json(_body): Json<Value>,
) -> Json<Value> {
    Json(json!({"message": "bi/datasets POST - not implemented"}))
}

async fn list_datasets(
    State(_pool): State<PgPool>,
    Query(_params): Query<ListDatasetsQuery>,
) -> Json<Value> {
    Json(json!({"message": "bi/datasets GET - not implemented"}))
}

async fn execute_query(
    State(_pool): State<PgPool>,
    Json(_body): Json<Value>,
) -> Json<Value> {
    Json(json!({"message": "bi/query POST - not implemented"}))
}
