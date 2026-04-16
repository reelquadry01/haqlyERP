// Author: Quadri Atharu
// Crate: haqly-erp-server

use axum::{
    Json,
    routing::{get, post},
    Router,
};
use axum::extract::{Path, State};
use sqlx::PgPool;
use serde::Deserialize;
use serde_json::{json, Value};
use uuid::Uuid;

#[derive(Debug, Deserialize)]
pub struct StatusQuery {
    pub detail: Option<bool>,
}

pub fn router() -> Router<PgPool> {
    Router::new()
        .route("/status/:company_id", get(get_einvoicing_status))
        .route("/profile", post(create_profile))
        .route("/profile/:company_id", get(get_profile))
        .route("/credentials", post(store_credentials))
        .route("/credentials/:company_id", get(get_credentials))
        .route("/submit/:invoice_id", post(submit_invoice))
        .route("/document/:invoice_id", get(get_document))
        .route("/confirm/:irn", post(confirm_invoice))
        .route("/download/:irn", get(download_document))
        .route("/report-payment", post(report_payment))
}

async fn get_einvoicing_status(
    State(_pool): State<PgPool>,
    Path(_company_id): Path<Uuid>,
) -> Json<Value> {
    Json(json!({"message": "einvoicing/status/:companyId - not implemented"}))
}

async fn create_profile(
    State(_pool): State<PgPool>,
    Json(_body): Json<Value>,
) -> Json<Value> {
    Json(json!({"message": "einvoicing/profile POST - not implemented"}))
}

async fn get_profile(
    State(_pool): State<PgPool>,
    Path(_company_id): Path<Uuid>,
) -> Json<Value> {
    Json(json!({"message": "einvoicing/profile/:companyId - not implemented"}))
}

async fn store_credentials(
    State(_pool): State<PgPool>,
    Json(_body): Json<Value>,
) -> Json<Value> {
    Json(json!({"message": "einvoicing/credentials POST - not implemented"}))
}

async fn get_credentials(
    State(_pool): State<PgPool>,
    Path(_company_id): Path<Uuid>,
) -> Json<Value> {
    Json(json!({"message": "einvoicing/credentials/:companyId - not implemented"}))
}

async fn submit_invoice(
    State(_pool): State<PgPool>,
    Path(_invoice_id): Path<Uuid>,
    Json(_body): Json<Value>,
) -> Json<Value> {
    Json(json!({"message": "einvoicing/submit/:invoiceId - not implemented"}))
}

async fn get_document(
    State(_pool): State<PgPool>,
    Path(_invoice_id): Path<Uuid>,
) -> Json<Value> {
    Json(json!({"message": "einvoicing/document/:invoiceId - not implemented"}))
}

async fn confirm_invoice(
    State(_pool): State<PgPool>,
    Path(_irn): Path<String>,
    Json(_body): Json<Value>,
) -> Json<Value> {
    Json(json!({"message": "einvoicing/confirm/:irn - not implemented"}))
}

async fn download_document(
    State(_pool): State<PgPool>,
    Path(_irn): Path<String>,
) -> Json<Value> {
    Json(json!({"message": "einvoicing/download/:irn - not implemented"}))
}

async fn report_payment(
    State(_pool): State<PgPool>,
    Json(_body): Json<Value>,
) -> Json<Value> {
    Json(json!({"message": "einvoicing/report-payment - not implemented"}))
}
