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
pub struct ListVouchersQuery {
    pub page: Option<i64>,
    pub per_page: Option<i64>,
    pub status: Option<String>,
    pub vendor_id: Option<Uuid>,
    pub company_id: Option<Uuid>,
}

#[derive(Debug, Deserialize)]
pub struct QueueQuery {
    pub assigned_to: Option<Uuid>,
    pub status: Option<String>,
}

pub fn router() -> Router<PgPool> {
    Router::new()
        .route("/", get(list_vouchers).post(create_voucher))
        .route("/queue", get(get_queue))
        .route("/metadata/options", get(get_metadata_options))
        .route("/:id", get(get_voucher).patch(update_voucher))
        .route("/:id/validate", post(validate_voucher))
        .route("/:id/submit", post(submit_voucher))
        .route("/:id/recall", post(recall_voucher))
        .route("/:id/approve", post(approve_voucher))
        .route("/:id/reject", post(reject_voucher))
        .route("/:id/return", post(return_voucher))
        .route("/:id/post", post(post_voucher))
        .route("/:id/initiate-payment", post(initiate_payment))
        .route("/:id/mark-paid", post(mark_paid))
        .route("/:id/cancel", post(cancel_voucher))
        .route("/:id/comments", post(add_comment))
        .route("/:id/preview", get(preview_voucher))
        .route("/:id/attachments/upload", post(upload_attachment))
}

async fn list_vouchers(
    State(_pool): State<PgPool>,
    Query(_params): Query<ListVouchersQuery>,
) -> Json<Value> {
    Json(json!({"message": "payment-vouchers/list - not implemented"}))
}

async fn get_queue(
    State(_pool): State<PgPool>,
    Query(_params): Query<QueueQuery>,
) -> Json<Value> {
    Json(json!({"message": "payment-vouchers/queue - not implemented"}))
}

async fn get_metadata_options(
    State(_pool): State<PgPool>,
) -> Json<Value> {
    Json(json!({"message": "payment-vouchers/metadata/options - not implemented"}))
}

async fn create_voucher(
    State(_pool): State<PgPool>,
    Json(_body): Json<Value>,
) -> Json<Value> {
    Json(json!({"message": "payment-vouchers/create - not implemented"}))
}

async fn update_voucher(
    State(_pool): State<PgPool>,
    Path(_id): Path<Uuid>,
    Json(_body): Json<Value>,
) -> Json<Value> {
    Json(json!({"message": "payment-vouchers/:id PATCH - not implemented"}))
}

async fn get_voucher(
    State(_pool): State<PgPool>,
    Path(_id): Path<Uuid>,
) -> Json<Value> {
    Json(json!({"message": "payment-vouchers/:id GET - not implemented"}))
}

async fn validate_voucher(
    State(_pool): State<PgPool>,
    Path(_id): Path<Uuid>,
    Json(_body): Json<Value>,
) -> Json<Value> {
    Json(json!({"message": "payment-vouchers/:id/validate - not implemented"}))
}

async fn submit_voucher(
    State(_pool): State<PgPool>,
    Path(_id): Path<Uuid>,
    Json(_body): Json<Value>,
) -> Json<Value> {
    Json(json!({"message": "payment-vouchers/:id/submit - not implemented"}))
}

async fn recall_voucher(
    State(_pool): State<PgPool>,
    Path(_id): Path<Uuid>,
    Json(_body): Json<Value>,
) -> Json<Value> {
    Json(json!({"message": "payment-vouchers/:id/recall - not implemented"}))
}

async fn approve_voucher(
    State(_pool): State<PgPool>,
    Path(_id): Path<Uuid>,
    Json(_body): Json<Value>,
) -> Json<Value> {
    Json(json!({"message": "payment-vouchers/:id/approve - not implemented"}))
}

async fn reject_voucher(
    State(_pool): State<PgPool>,
    Path(_id): Path<Uuid>,
    Json(_body): Json<Value>,
) -> Json<Value> {
    Json(json!({"message": "payment-vouchers/:id/reject - not implemented"}))
}

async fn return_voucher(
    State(_pool): State<PgPool>,
    Path(_id): Path<Uuid>,
    Json(_body): Json<Value>,
) -> Json<Value> {
    Json(json!({"message": "payment-vouchers/:id/return - not implemented"}))
}

async fn post_voucher(
    State(_pool): State<PgPool>,
    Path(_id): Path<Uuid>,
    Json(_body): Json<Value>,
) -> Json<Value> {
    Json(json!({"message": "payment-vouchers/:id/post - not implemented"}))
}

async fn initiate_payment(
    State(_pool): State<PgPool>,
    Path(_id): Path<Uuid>,
    Json(_body): Json<Value>,
) -> Json<Value> {
    Json(json!({"message": "payment-vouchers/:id/initiate-payment - not implemented"}))
}

async fn mark_paid(
    State(_pool): State<PgPool>,
    Path(_id): Path<Uuid>,
    Json(_body): Json<Value>,
) -> Json<Value> {
    Json(json!({"message": "payment-vouchers/:id/mark-paid - not implemented"}))
}

async fn cancel_voucher(
    State(_pool): State<PgPool>,
    Path(_id): Path<Uuid>,
    Json(_body): Json<Value>,
) -> Json<Value> {
    Json(json!({"message": "payment-vouchers/:id/cancel - not implemented"}))
}

async fn add_comment(
    State(_pool): State<PgPool>,
    Path(_id): Path<Uuid>,
    Json(_body): Json<Value>,
) -> Json<Value> {
    Json(json!({"message": "payment-vouchers/:id/comments - not implemented"}))
}

async fn preview_voucher(
    State(_pool): State<PgPool>,
    Path(_id): Path<Uuid>,
) -> Json<Value> {
    Json(json!({"message": "payment-vouchers/:id/preview - not implemented"}))
}

async fn upload_attachment(
    State(_pool): State<PgPool>,
    Path(_id): Path<Uuid>,
    Json(_body): Json<Value>,
) -> Json<Value> {
    Json(json!({"message": "payment-vouchers/:id/attachments/upload - not implemented"}))
}
