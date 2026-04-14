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
pub struct ListCustomersQuery {
    pub page: Option<i64>,
    pub per_page: Option<i64>,
    pub search: Option<String>,
    pub is_active: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct ListInvoicesQuery {
    pub page: Option<i64>,
    pub per_page: Option<i64>,
    pub status: Option<String>,
    pub customer_id: Option<Uuid>,
    pub date_from: Option<String>,
    pub date_to: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ListReceiptsQuery {
    pub page: Option<i64>,
    pub per_page: Option<i64>,
    pub status: Option<String>,
    pub customer_id: Option<Uuid>,
}

#[derive(Debug, Deserialize)]
pub struct ListProformaQuery {
    pub page: Option<i64>,
    pub per_page: Option<i64>,
    pub customer_id: Option<Uuid>,
}

pub fn router() -> Router<PgPool> {
    Router::new()
        .route("/customers", post(create_customer).get(list_customers))
        .route("/customers/:id", patch(update_customer))
        .route("/invoices", post(create_invoice).get(list_invoices))
        .route("/invoices/:id", get(get_invoice))
        .route("/receipts/metadata/options/:legal_entity_id", post(get_receipts_metadata))
        .route("/receipts", post(create_receipt).get(list_receipts))
        .route("/receipts/:id", get(get_receipt))
        .route("/receipts/:id/post", post(post_receipt))
        .route("/proforma", post(create_proforma).get(list_proforma))
}

async fn create_customer(
    State(_pool): State<PgPool>,
    Json(_body): Json<Value>,
) -> Json<Value> {
    Json(json!({"message": "sales/customers POST - not implemented"}))
}

async fn list_customers(
    State(_pool): State<PgPool>,
    Query(_params): Query<ListCustomersQuery>,
) -> Json<Value> {
    Json(json!({"message": "sales/customers GET - not implemented"}))
}

async fn update_customer(
    State(_pool): State<PgPool>,
    Path(_id): Path<Uuid>,
    Json(_body): Json<Value>,
) -> Json<Value> {
    Json(json!({"message": "sales/customers/:id PATCH - not implemented"}))
}

async fn create_invoice(
    State(_pool): State<PgPool>,
    Json(_body): Json<Value>,
) -> Json<Value> {
    Json(json!({"message": "sales/invoices POST - not implemented"}))
}

async fn list_invoices(
    State(_pool): State<PgPool>,
    Query(_params): Query<ListInvoicesQuery>,
) -> Json<Value> {
    Json(json!({"message": "sales/invoices GET - not implemented"}))
}

async fn get_invoice(
    State(_pool): State<PgPool>,
    Path(_id): Path<Uuid>,
) -> Json<Value> {
    Json(json!({"message": "sales/invoices/:id GET - not implemented"}))
}

async fn get_receipts_metadata(
    State(_pool): State<PgPool>,
    Path(_legal_entity_id): Path<Uuid>,
) -> Json<Value> {
    Json(json!({"message": "sales/receipts/metadata/options/:legalEntityId - not implemented"}))
}

async fn create_receipt(
    State(_pool): State<PgPool>,
    Json(_body): Json<Value>,
) -> Json<Value> {
    Json(json!({"message": "sales/receipts POST - not implemented"}))
}

async fn list_receipts(
    State(_pool): State<PgPool>,
    Query(_params): Query<ListReceiptsQuery>,
) -> Json<Value> {
    Json(json!({"message": "sales/receipts GET - not implemented"}))
}

async fn get_receipt(
    State(_pool): State<PgPool>,
    Path(_id): Path<Uuid>,
) -> Json<Value> {
    Json(json!({"message": "sales/receipts/:id GET - not implemented"}))
}

async fn post_receipt(
    State(_pool): State<PgPool>,
    Path(_id): Path<Uuid>,
    Json(_body): Json<Value>,
) -> Json<Value> {
    Json(json!({"message": "sales/receipts/:id/post - not implemented"}))
}

async fn create_proforma(
    State(_pool): State<PgPool>,
    Json(_body): Json<Value>,
) -> Json<Value> {
    Json(json!({"message": "sales/proforma POST - not implemented"}))
}

async fn list_proforma(
    State(_pool): State<PgPool>,
    Query(_params): Query<ListProformaQuery>,
) -> Json<Value> {
    Json(json!({"message": "sales/proforma GET - not implemented"}))
}
