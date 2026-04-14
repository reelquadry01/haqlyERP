// Author: Quadri Atharu
// Crate: haqly-erp-server

use axum::{
    Json, Path, Query, State,
    routing::{get, post},
    Router,
};
use sqlx::PgPool;
use serde::Deserialize;
use serde_json::{json, Value};
use uuid::Uuid;

#[derive(Debug, Deserialize)]
pub struct ListLoansQuery {
    pub page: Option<i64>,
    pub per_page: Option<i64>,
    pub status: Option<String>,
    pub borrower_id: Option<Uuid>,
    pub company_id: Option<Uuid>,
}

pub fn router() -> Router<PgPool> {
    Router::new()
        .route("/", post(create_loan).get(list_loans))
        .route("/:id", get(get_loan))
        .route("/:id/disburse", post(disburse_loan))
        .route("/:id/repay", post(repay_loan))
        .route("/:id/schedule", get(get_loan_schedule))
}

async fn create_loan(
    State(_pool): State<PgPool>,
    Json(_body): Json<Value>,
) -> Json<Value> {
    Json(json!({"message": "loans/create - not implemented"}))
}

async fn list_loans(
    State(_pool): State<PgPool>,
    Query(_params): Query<ListLoansQuery>,
) -> Json<Value> {
    Json(json!({"message": "loans/list - not implemented"}))
}

async fn get_loan(
    State(_pool): State<PgPool>,
    Path(_id): Path<Uuid>,
) -> Json<Value> {
    Json(json!({"message": "loans/:id GET - not implemented"}))
}

async fn disburse_loan(
    State(_pool): State<PgPool>,
    Path(_id): Path<Uuid>,
    Json(_body): Json<Value>,
) -> Json<Value> {
    Json(json!({"message": "loans/:id/disburse - not implemented"}))
}

async fn repay_loan(
    State(_pool): State<PgPool>,
    Path(_id): Path<Uuid>,
    Json(_body): Json<Value>,
) -> Json<Value> {
    Json(json!({"message": "loans/:id/repay - not implemented"}))
}

async fn get_loan_schedule(
    State(_pool): State<PgPool>,
    Path(_id): Path<Uuid>,
) -> Json<Value> {
    Json(json!({"message": "loans/:id/schedule - not implemented"}))
}
