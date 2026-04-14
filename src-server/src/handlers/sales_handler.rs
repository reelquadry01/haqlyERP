// Author: Quadri Atharu
// Crate: haqly-erp-server

use axum::{
    Json, Path, Query, State,
    routing::{get, post, patch},
    Router,
};
use bigdecimal::BigDecimal;
use sqlx::PgPool;
use serde::Deserialize;
use serde_json::{json, Value};
use uuid::Uuid;

use crate::dtos::sales_dto::{CreateCreditNoteRequest, CreateCustomerRequest, CreateInvoiceRequest, CreateReceiptRequest, RecordPaymentRequest};
use crate::models::sales::PaymentMethod;
use crate::services::sales_service::SalesService;

#[derive(Debug, Deserialize)]
pub struct ListCustomersQuery {
    pub page: Option<i64>,
    pub per_page: Option<i64>,
    pub search: Option<String>,
    pub is_active: Option<bool>,
    pub company_id: Uuid,
}

#[derive(Debug, Deserialize)]
pub struct ListInvoicesQuery {
    pub page: Option<i64>,
    pub per_page: Option<i64>,
    pub status: Option<String>,
    pub customer_id: Option<Uuid>,
    pub date_from: Option<String>,
    pub date_to: Option<String>,
    pub company_id: Uuid,
}

#[derive(Debug, Deserialize)]
pub struct ListReceiptsQuery {
    pub page: Option<i64>,
    pub per_page: Option<i64>,
    pub status: Option<String>,
    pub customer_id: Option<Uuid>,
    pub company_id: Uuid,
}

#[derive(Debug, Deserialize)]
pub struct ListProformaQuery {
    pub page: Option<i64>,
    pub per_page: Option<i64>,
    pub customer_id: Option<Uuid>,
    pub company_id: Uuid,
}

#[derive(Debug, Deserialize)]
pub struct InvoiceAgingQuery {
    pub company_id: Uuid,
}

#[derive(Debug, Deserialize)]
pub struct RevenueSummaryQuery {
    pub company_id: Uuid,
    pub period_start: String,
    pub period_end: String,
}

#[derive(Debug, Deserialize)]
pub struct PostReceiptBody {
    pub posted_by: Uuid,
}

pub fn router() -> Router<PgPool> {
    Router::new()
        .route("/customers", post(create_customer).get(list_customers))
        .route("/customers/:id", patch(update_customer))
        .route("/invoices", post(create_invoice).get(list_invoices))
        .route("/invoices/:id", get(get_invoice))
        .route("/invoices/:id/send", post(send_invoice))
        .route("/invoices/:id/record-payment", post(record_payment))
        .route("/credit-notes", post(create_credit_note))
        .route("/aging", get(get_invoice_aging))
        .route("/revenue-summary", get(get_revenue_summary))
        .route("/receipts/metadata/options/:legal_entity_id", post(get_receipts_metadata))
        .route("/receipts", post(create_receipt).get(list_receipts))
        .route("/receipts/:id", get(get_receipt))
        .route("/receipts/:id/post", post(post_receipt))
        .route("/proforma", post(create_proforma).get(list_proforma))
}

async fn create_customer(
    State(pool): State<PgPool>,
    Json(body): Json<CreateCustomerRequest>,
) -> Json<Value> {
    let service = SalesService::new(pool);
    match service.create_customer(body).await {
        Ok(customer) => Json(json!(customer)),
        Err(e) => Json(json!({"error": e.to_string()})),
    }
}

async fn list_customers(
    State(pool): State<PgPool>,
    Query(params): Query<ListCustomersQuery>,
) -> Json<Value> {
    let service = SalesService::new(pool);
    match service.list_customers(params.company_id).await {
        Ok(customers) => Json(json!(customers)),
        Err(e) => Json(json!({"error": e.to_string()})),
    }
}

async fn update_customer(
    State(_pool): State<PgPool>,
    Path(_id): Path<Uuid>,
    Json(_body): Json<Value>,
) -> Json<Value> {
    Json(json!({"message": "sales/customers/:id PATCH - not implemented"}))
}

async fn create_invoice(
    State(pool): State<PgPool>,
    Json(body): Json<Value>,
) -> Json<Value> {
    let service = SalesService::new(pool);
    let req: CreateInvoiceRequest = match serde_json::from_value(body.clone()) {
        Ok(r) => r,
        Err(e) => return Json(json!({"error": e.to_string()})),
    };
    let created_by = Uuid::now_v7();
    match service.create_invoice(req, created_by).await {
        Ok(invoice) => Json(json!(invoice)),
        Err(e) => Json(json!({"error": e.to_string()})),
    }
}

async fn list_invoices(
    State(pool): State<PgPool>,
    Query(params): Query<ListInvoicesQuery>,
) -> Json<Value> {
    let service = SalesService::new(pool);
    match service.list_invoices(params.company_id).await {
        Ok(invoices) => Json(json!(invoices)),
        Err(e) => Json(json!({"error": e.to_string()})),
    }
}

async fn get_invoice(
    State(pool): State<PgPool>,
    Path(id): Path<Uuid>,
) -> Json<Value> {
    let service = SalesService::new(pool);
    match service.list_invoices(Uuid::nil()).await {
        Ok(_) => Json(json!({"message": "Use list with company_id filter"})),
        Err(e) => Json(json!({"error": e.to_string()})),
    }
}

async fn send_invoice(
    State(pool): State<PgPool>,
    Path(id): Path<Uuid>,
) -> Json<Value> {
    let service = SalesService::new(pool);
    match service.send_invoice(id).await {
        Ok(invoice) => Json(json!(invoice)),
        Err(e) => Json(json!({"error": e.to_string()})),
    }
}

async fn record_payment(
    State(pool): State<PgPool>,
    Path(id): Path<Uuid>,
    Json(body): Json<RecordPaymentRequest>,
) -> Json<Value> {
    let service = SalesService::new(pool);
    let created_by = Uuid::now_v7();
    let req = RecordPaymentRequest {
        invoice_id: id,
        amount: body.amount,
        payment_method: body.payment_method,
        reference: body.reference,
        receipt_date: body.receipt_date,
        bank_account_id: body.bank_account_id,
    };
    match service.record_payment(req, created_by).await {
        Ok(receipt) => Json(json!(receipt)),
        Err(e) => Json(json!({"error": e.to_string()})),
    }
}

async fn create_credit_note(
    State(pool): State<PgPool>,
    Json(body): Json<CreateCreditNoteRequest>,
) -> Json<Value> {
    let service = SalesService::new(pool);
    let created_by = Uuid::now_v7();
    match service.create_credit_note(body, created_by).await {
        Ok(cn) => Json(json!(cn)),
        Err(e) => Json(json!({"error": e.to_string()})),
    }
}

async fn get_invoice_aging(
    State(pool): State<PgPool>,
    Query(params): Query<InvoiceAgingQuery>,
) -> Json<Value> {
    let service = SalesService::new(pool);
    match service.get_invoice_aging(params.company_id).await {
        Ok(aging) => Json(json!(aging)),
        Err(e) => Json(json!({"error": e.to_string()})),
    }
}

async fn get_revenue_summary(
    State(pool): State<PgPool>,
    Query(params): Query<RevenueSummaryQuery>,
) -> Json<Value> {
    let service = SalesService::new(pool);
    let period_start = match chrono::NaiveDate::parse_from_str(&params.period_start, "%Y-%m-%d") {
        Ok(d) => d,
        Err(e) => return Json(json!({"error": format!("Invalid period_start: {}", e)})),
    };
    let period_end = match chrono::NaiveDate::parse_from_str(&params.period_end, "%Y-%m-%d") {
        Ok(d) => d,
        Err(e) => return Json(json!({"error": format!("Invalid period_end: {}", e)})),
    };
    match service.get_revenue_summary(params.company_id, period_start, period_end).await {
        Ok(summary) => Json(json!(summary)),
        Err(e) => Json(json!({"error": e.to_string()})),
    }
}

async fn get_receipts_metadata(
    State(_pool): State<PgPool>,
    Path(_legal_entity_id): Path<Uuid>,
) -> Json<Value> {
    Json(json!({"message": "sales/receipts/metadata/options/:legalEntityId - not implemented"}))
}

async fn create_receipt(
    State(pool): State<PgPool>,
    Json(body): Json<CreateReceiptRequest>,
) -> Json<Value> {
    let service = SalesService::new(pool);
    let created_by = Uuid::now_v7();
    match service.create_receipt(body, created_by).await {
        Ok(receipt) => Json(json!(receipt)),
        Err(e) => Json(json!({"error": e.to_string()})),
    }
}

async fn list_receipts(
    State(pool): State<PgPool>,
    Query(params): Query<ListReceiptsQuery>,
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
    State(pool): State<PgPool>,
    Path(id): Path<Uuid>,
    Json(body): Json<PostReceiptBody>,
) -> Json<Value> {
    let service = SalesService::new(pool);
    match service.post_receipt_to_gl(id, body.posted_by).await {
        Ok(_) => Json(json!({"message": "Receipt posted to GL"})),
        Err(e) => Json(json!({"error": e.to_string()})),
    }
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
