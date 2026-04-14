// Author: Quadri Atharu
// Crate: haqly-erp-server

use axum::{
    Json, State,
    routing::post,
    Router,
};
use sqlx::PgPool;
use serde_json::{json, Value};

pub fn router() -> Router<PgPool> {
    Router::new()
        .route("/chart-of-accounts", post(import_chart_of_accounts))
        .route("/customers", post(import_customers))
        .route("/suppliers", post(import_suppliers))
        .route("/products", post(import_products))
        .route("/tax-configs", post(import_tax_configs))
        .route("/branches", post(import_branches))
        .route("/departments", post(import_departments))
        .route("/warehouses", post(import_warehouses))
        .route("/bank-accounts", post(import_bank_accounts))
        .route("/asset-categories", post(import_asset_categories))
        .route("/gl-opening-balances", post(import_gl_opening_balances))
        .route("/ar-opening-balances", post(import_ar_opening_balances))
        .route("/ap-opening-balances", post(import_ap_opening_balances))
        .route("/customer-receipts", post(import_customer_receipts))
        .route("/supplier-payments", post(import_supplier_payments))
        .route("/fixed-assets", post(import_fixed_assets))
        .route("/stock-opening-balances", post(import_stock_opening_balances))
        .route("/employees", post(import_employees))
}

async fn import_chart_of_accounts(
    State(_pool): State<PgPool>,
    Json(_body): Json<Value>,
) -> Json<Value> {
    Json(json!({"message": "imports/chart-of-accounts - not implemented"}))
}

async fn import_customers(
    State(_pool): State<PgPool>,
    Json(_body): Json<Value>,
) -> Json<Value> {
    Json(json!({"message": "imports/customers - not implemented"}))
}

async fn import_suppliers(
    State(_pool): State<PgPool>,
    Json(_body): Json<Value>,
) -> Json<Value> {
    Json(json!({"message": "imports/suppliers - not implemented"}))
}

async fn import_products(
    State(_pool): State<PgPool>,
    Json(_body): Json<Value>,
) -> Json<Value> {
    Json(json!({"message": "imports/products - not implemented"}))
}

async fn import_tax_configs(
    State(_pool): State<PgPool>,
    Json(_body): Json<Value>,
) -> Json<Value> {
    Json(json!({"message": "imports/tax-configs - not implemented"}))
}

async fn import_branches(
    State(_pool): State<PgPool>,
    Json(_body): Json<Value>,
) -> Json<Value> {
    Json(json!({"message": "imports/branches - not implemented"}))
}

async fn import_departments(
    State(_pool): State<PgPool>,
    Json(_body): Json<Value>,
) -> Json<Value> {
    Json(json!({"message": "imports/departments - not implemented"}))
}

async fn import_warehouses(
    State(_pool): State<PgPool>,
    Json(_body): Json<Value>,
) -> Json<Value> {
    Json(json!({"message": "imports/warehouses - not implemented"}))
}

async fn import_bank_accounts(
    State(_pool): State<PgPool>,
    Json(_body): Json<Value>,
) -> Json<Value> {
    Json(json!({"message": "imports/bank-accounts - not implemented"}))
}

async fn import_asset_categories(
    State(_pool): State<PgPool>,
    Json(_body): Json<Value>,
) -> Json<Value> {
    Json(json!({"message": "imports/asset-categories - not implemented"}))
}

async fn import_gl_opening_balances(
    State(_pool): State<PgPool>,
    Json(_body): Json<Value>,
) -> Json<Value> {
    Json(json!({"message": "imports/gl-opening-balances - not implemented"}))
}

async fn import_ar_opening_balances(
    State(_pool): State<PgPool>,
    Json(_body): Json<Value>,
) -> Json<Value> {
    Json(json!({"message": "imports/ar-opening-balances - not implemented"}))
}

async fn import_ap_opening_balances(
    State(_pool): State<PgPool>,
    Json(_body): Json<Value>,
) -> Json<Value> {
    Json(json!({"message": "imports/ap-opening-balances - not implemented"}))
}

async fn import_customer_receipts(
    State(_pool): State<PgPool>,
    Json(_body): Json<Value>,
) -> Json<Value> {
    Json(json!({"message": "imports/customer-receipts - not implemented"}))
}

async fn import_supplier_payments(
    State(_pool): State<PgPool>,
    Json(_body): Json<Value>,
) -> Json<Value> {
    Json(json!({"message": "imports/supplier-payments - not implemented"}))
}

async fn import_fixed_assets(
    State(_pool): State<PgPool>,
    Json(_body): Json<Value>,
) -> Json<Value> {
    Json(json!({"message": "imports/fixed-assets - not implemented"}))
}

async fn import_stock_opening_balances(
    State(_pool): State<PgPool>,
    Json(_body): Json<Value>,
) -> Json<Value> {
    Json(json!({"message": "imports/stock-opening-balances - not implemented"}))
}

async fn import_employees(
    State(_pool): State<PgPool>,
    Json(_body): Json<Value>,
) -> Json<Value> {
    Json(json!({"message": "imports/employees - not implemented"}))
}
