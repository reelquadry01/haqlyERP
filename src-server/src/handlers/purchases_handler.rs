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

#[derive(Debug, Deserialize)]
pub struct ListSuppliersQuery {
    pub page: Option<i64>,
    pub per_page: Option<i64>,
    pub search: Option<String>,
    pub is_active: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct ListBillsQuery {
    pub page: Option<i64>,
    pub per_page: Option<i64>,
    pub status: Option<String>,
    pub supplier_id: Option<Uuid>,
    pub date_from: Option<String>,
    pub date_to: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ListPaymentsQuery {
    pub page: Option<i64>,
    pub per_page: Option<i64>,
    pub supplier_id: Option<Uuid>,
    pub status: Option<String>,
}

pub fn router() -> Router<PgPool> {
    Router::new()
        .route("/suppliers", post(create_supplier).get(list_suppliers))
        .route("/suppliers/:id", patch(update_supplier))
        .route("/bills", post(create_bill).get(list_bills))
        .route("/bills/:id", get(get_bill))
        .route("/payments", post(create_payment).get(list_payments))
}

async fn create_supplier(
    State(_pool): State<PgPool>,
    Json(_body): Json<Value>,
) -> Json<Value> {
    Json(json!({"message": "purchases/suppliers POST - not implemented"}))
}

async fn list_suppliers(
    State(_pool): State<PgPool>,
    Query(_params): Query<ListSuppliersQuery>,
) -> Json<Value> {
    Json(json!({"message": "purchases/suppliers GET - not implemented"}))
}

async fn update_supplier(
    State(_pool): State<PgPool>,
    Path(_id): Path<Uuid>,
    Json(_body): Json<Value>,
) -> Json<Value> {
    Json(json!({"message": "purchases/suppliers/:id PATCH - not implemented"}))
}

async fn create_bill(
    State(_pool): State<PgPool>,
    Json(_body): Json<Value>,
) -> Json<Value> {
    Json(json!({"message": "purchases/bills POST - not implemented"}))
}

async fn list_bills(
    State(_pool): State<PgPool>,
    Query(_params): Query<ListBillsQuery>,
) -> Json<Value> {
    Json(json!({"message": "purchases/bills GET - not implemented"}))
}

async fn get_bill(
    State(_pool): State<PgPool>,
    Path(_id): Path<Uuid>,
) -> Json<Value> {
    Json(json!({"message": "purchases/bills/:id GET - not implemented"}))
}

async fn create_payment(
    State(_pool): State<PgPool>,
    Json(_body): Json<Value>,
) -> Json<Value> {
    Json(json!({"message": "purchases/payments POST - not implemented"}))
}

async fn list_payments(
    State(_pool): State<PgPool>,
    Query(_params): Query<ListPaymentsQuery>,
) -> Json<Value> {
    Json(json!({"message": "purchases/payments GET - not implemented"}))
}
