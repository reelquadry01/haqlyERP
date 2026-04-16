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

use crate::dtos::purchase_dto::{CreateBillRequest, CreateDebitNoteRequest, CreatePaymentRequest, CreateSupplierRequest, RecordBillPaymentRequest};
use crate::services::purchases_service::PurchasesService;

#[derive(Debug, Deserialize)]
pub struct ListSuppliersQuery {
    pub page: Option<i64>,
    pub per_page: Option<i64>,
    pub search: Option<String>,
    pub is_active: Option<bool>,
    pub company_id: Uuid,
}

#[derive(Debug, Deserialize)]
pub struct ListBillsQuery {
    pub page: Option<i64>,
    pub per_page: Option<i64>,
    pub status: Option<String>,
    pub supplier_id: Option<Uuid>,
    pub date_from: Option<String>,
    pub date_to: Option<String>,
    pub company_id: Uuid,
}

#[derive(Debug, Deserialize)]
pub struct ListPaymentsQuery {
    pub page: Option<i64>,
    pub per_page: Option<i64>,
    pub supplier_id: Option<Uuid>,
    pub status: Option<String>,
    pub company_id: Uuid,
}

#[derive(Debug, Deserialize)]
pub struct WhtSummaryQuery {
    pub company_id: Uuid,
    pub period_start: String,
    pub period_end: String,
}

#[derive(Debug, Deserialize)]
pub struct ExpenseSummaryQuery {
    pub company_id: Uuid,
    pub period_start: String,
    pub period_end: String,
}

pub fn router() -> Router<PgPool> {
    Router::new()
        .route("/suppliers", post(create_supplier).get(list_suppliers))
        .route("/suppliers/:id", patch(update_supplier))
        .route("/bills", post(create_bill).get(list_bills))
        .route("/bills/:id", get(get_bill))
        .route("/bills/:id/approve", post(approve_bill))
        .route("/bills/:id/record-payment", post(record_bill_payment))
        .route("/debit-notes", post(create_debit_note))
        .route("/wht-summary", get(get_wht_summary))
        .route("/expense-summary", get(get_expense_summary))
        .route("/payments", post(create_payment).get(list_payments))
}

async fn create_supplier(
    State(pool): State<PgPool>,
    Json(body): Json<CreateSupplierRequest>,
) -> Json<Value> {
    let service = PurchasesService::new(pool);
    match service.create_supplier(body).await {
        Ok(supplier) => Json(json!(supplier)),
        Err(e) => Json(json!({"error": e.to_string()})),
    }
}

async fn list_suppliers(
    State(pool): State<PgPool>,
    Query(params): Query<ListSuppliersQuery>,
) -> Json<Value> {
    let service = PurchasesService::new(pool);
    match service.list_suppliers(params.company_id).await {
        Ok(suppliers) => Json(json!(suppliers)),
        Err(e) => Json(json!({"error": e.to_string()})),
    }
}

async fn update_supplier(
    State(_pool): State<PgPool>,
    Path(_id): Path<Uuid>,
    Json(_body): Json<Value>,
) -> Json<Value> {
    Json(json!({"message": "purchases/suppliers/:id PATCH - not implemented"}))
}

async fn create_bill(
    State(pool): State<PgPool>,
    Json(body): Json<Value>,
) -> Json<Value> {
    let service = PurchasesService::new(pool);
    let req: CreateBillRequest = match serde_json::from_value(body.clone()) {
        Ok(r) => r,
        Err(e) => return Json(json!({"error": e.to_string()})),
    };
    let created_by = Uuid::now_v7();
    match service.create_bill(req, created_by).await {
        Ok(bill) => Json(json!(bill)),
        Err(e) => Json(json!({"error": e.to_string()})),
    }
}

async fn list_bills(
    State(pool): State<PgPool>,
    Query(params): Query<ListBillsQuery>,
) -> Json<Value> {
    let service = PurchasesService::new(pool);
    match service.list_bills(params.company_id).await {
        Ok(bills) => Json(json!(bills)),
        Err(e) => Json(json!({"error": e.to_string()})),
    }
}

async fn get_bill(
    State(_pool): State<PgPool>,
    Path(_id): Path<Uuid>,
) -> Json<Value> {
    Json(json!({"message": "purchases/bills/:id GET - not implemented"}))
}

async fn approve_bill(
    State(pool): State<PgPool>,
    Path(id): Path<Uuid>,
) -> Json<Value> {
    let service = PurchasesService::new(pool);
    match service.approve_bill(id).await {
        Ok(bill) => Json(json!(bill)),
        Err(e) => Json(json!({"error": e.to_string()})),
    }
}

async fn record_bill_payment(
    State(pool): State<PgPool>,
    Path(id): Path<Uuid>,
    Json(body): Json<RecordBillPaymentRequest>,
) -> Json<Value> {
    let service = PurchasesService::new(pool);
    let created_by = Uuid::now_v7();
    let req = RecordBillPaymentRequest {
        bill_id: id,
        amount: body.amount,
        payment_method: body.payment_method,
        reference: body.reference,
        payment_date: body.payment_date,
        bank_account_id: body.bank_account_id,
    };
    match service.record_payment(req, created_by).await {
        Ok(payment) => Json(json!(payment)),
        Err(e) => Json(json!({"error": e.to_string()})),
    }
}

async fn create_debit_note(
    State(pool): State<PgPool>,
    Json(body): Json<CreateDebitNoteRequest>,
) -> Json<Value> {
    let service = PurchasesService::new(pool);
    let created_by = Uuid::now_v7();
    match service.create_debit_note(body, created_by).await {
        Ok(dn) => Json(json!(dn)),
        Err(e) => Json(json!({"error": e.to_string()})),
    }
}

async fn get_wht_summary(
    State(pool): State<PgPool>,
    Query(params): Query<WhtSummaryQuery>,
) -> Json<Value> {
    let service = PurchasesService::new(pool);
    let period_start = match chrono::NaiveDate::parse_from_str(&params.period_start, "%Y-%m-%d") {
        Ok(d) => d,
        Err(e) => return Json(json!({"error": format!("Invalid period_start: {}", e)})),
    };
    let period_end = match chrono::NaiveDate::parse_from_str(&params.period_end, "%Y-%m-%d") {
        Ok(d) => d,
        Err(e) => return Json(json!({"error": format!("Invalid period_end: {}", e)})),
    };
    match service.get_wht_summary(params.company_id, period_start, period_end).await {
        Ok(summary) => Json(json!(summary)),
        Err(e) => Json(json!({"error": e.to_string()})),
    }
}

async fn get_expense_summary(
    State(pool): State<PgPool>,
    Query(params): Query<ExpenseSummaryQuery>,
) -> Json<Value> {
    let service = PurchasesService::new(pool);
    let period_start = match chrono::NaiveDate::parse_from_str(&params.period_start, "%Y-%m-%d") {
        Ok(d) => d,
        Err(e) => return Json(json!({"error": format!("Invalid period_start: {}", e)})),
    };
    let period_end = match chrono::NaiveDate::parse_from_str(&params.period_end, "%Y-%m-%d") {
        Ok(d) => d,
        Err(e) => return Json(json!({"error": format!("Invalid period_end: {}", e)})),
    };
    match service.get_expense_summary(params.company_id, period_start, period_end).await {
        Ok(summary) => Json(json!(summary)),
        Err(e) => Json(json!({"error": e.to_string()})),
    }
}

async fn create_payment(
    State(pool): State<PgPool>,
    Json(body): Json<CreatePaymentRequest>,
) -> Json<Value> {
    let service = PurchasesService::new(pool);
    let created_by = Uuid::now_v7();
    match service.create_payment(body, created_by).await {
        Ok(payment) => Json(json!(payment)),
        Err(e) => Json(json!({"error": e.to_string()})),
    }
}

async fn list_payments(
    State(_pool): State<PgPool>,
    Query(_params): Query<ListPaymentsQuery>,
) -> Json<Value> {
    Json(json!({"message": "purchases/payments GET - not implemented"}))
}
