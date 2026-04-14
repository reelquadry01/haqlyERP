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

use crate::models::inventory::ProductType;
use crate::services::inventory_service::InventoryService;

#[derive(Debug, Deserialize)]
pub struct ListProductsQuery {
    pub page: Option<i64>,
    pub per_page: Option<i64>,
    pub search: Option<String>,
    pub category: Option<String>,
    pub is_active: Option<bool>,
    pub company_id: Uuid,
}

#[derive(Debug, Deserialize)]
pub struct ListWarehousesQuery {
    pub page: Option<i64>,
    pub per_page: Option<i64>,
    pub company_id: Uuid,
}

#[derive(Debug, Deserialize)]
pub struct ListStockMovementsQuery {
    pub page: Option<i64>,
    pub per_page: Option<i64>,
    pub product_id: Option<Uuid>,
    pub warehouse_id: Option<Uuid>,
    pub movement_type: Option<String>,
    pub date_from: Option<String>,
    pub date_to: Option<String>,
    pub company_id: Uuid,
}

#[derive(Debug, Deserialize)]
pub struct StockLevelsQuery {
    pub product_id: Option<Uuid>,
    pub warehouse_id: Option<Uuid>,
    pub company_id: Uuid,
}

#[derive(Debug, Deserialize)]
pub struct StockValuationQuery {
    pub company_id: Uuid,
}

#[derive(Debug, Deserialize)]
pub struct LowStockAlertsQuery {
    pub company_id: Uuid,
}

#[derive(Debug, Deserialize)]
pub struct AdjustStockRequest {
    pub product_id: Uuid,
    pub adjustment_type: String,
    pub quantity: BigDecimal,
    pub reason: String,
    pub warehouse_id: Uuid,
    pub created_by: Uuid,
}

pub fn router() -> Router<PgPool> {
    Router::new()
        .route("/products", post(create_product).get(list_products))
        .route("/products/:id", patch(update_product))
        .route("/warehouses", post(create_warehouse).get(list_warehouses))
        .route("/stock-movements", post(create_stock_movement).get(list_stock_movements))
        .route("/stock-levels", get(get_stock_levels))
        .route("/stock-valuation", get(get_stock_valuation))
        .route("/low-stock-alerts", get(get_low_stock_alerts))
        .route("/adjust-stock", post(adjust_stock))
}

async fn create_product(
    State(pool): State<PgPool>,
    Json(body): Json<Value>,
) -> Json<Value> {
    let service = InventoryService::new(pool);
    let code = body.get("code").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let name = body.get("name").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let company_id: Uuid = match body.get("company_id").and_then(|v| serde_json::from_value(v.clone()).ok()) {
        Some(id) => id,
        None => return Json(json!({"error": "company_id is required"})),
    };
    let product_type: ProductType = match body.get("product_type").and_then(|v| v.as_str()) {
        Some("service") => ProductType::Service,
        _ => ProductType::Goods,
    };
    let category = body.get("category").and_then(|v| v.as_str()).map(|s| s.to_string());
    let unit_of_measure = body.get("unit_of_measure").and_then(|v| v.as_str()).map(|s| s.to_string());
    let sales_price = body.get("sales_price").and_then(|v| serde_json::from_value(v.clone()).ok());
    let purchase_price = body.get("purchase_price").and_then(|v| serde_json::from_value(v.clone()).ok());
    let cost_price = body.get("cost_price").and_then(|v| serde_json::from_value(v.clone()).ok());
    let tax_rate = body.get("tax_rate").and_then(|v| serde_json::from_value(v.clone()).ok());
    let is_taxable = body.get("is_taxable").and_then(|v| v.as_bool()).unwrap_or(false);
    let revenue_account_id = body.get("revenue_account_id").and_then(|v| serde_json::from_value(v.clone()).ok());
    let inventory_account_id = body.get("inventory_account_id").and_then(|v| serde_json::from_value(v.clone()).ok());
    let cogs_account_id = body.get("cogs_account_id").and_then(|v| serde_json::from_value(v.clone()).ok());
    let reorder_level = body.get("reorder_level").and_then(|v| serde_json::from_value(v.clone()).ok());
    let reorder_quantity = body.get("reorder_quantity").and_then(|v| serde_json::from_value(v.clone()).ok());

    match service.create_product(
        company_id, code, name, product_type, category, unit_of_measure,
        sales_price, purchase_price, cost_price, tax_rate, is_taxable,
        revenue_account_id, inventory_account_id, cogs_account_id,
        reorder_level, reorder_quantity,
    ).await {
        Ok(product) => Json(json!(product)),
        Err(e) => Json(json!({"error": e.to_string()})),
    }
}

async fn list_products(
    State(pool): State<PgPool>,
    Query(params): Query<ListProductsQuery>,
) -> Json<Value> {
    let service = InventoryService::new(pool);
    match service.list_products(params.company_id).await {
        Ok(products) => Json(json!(products)),
        Err(e) => Json(json!({"error": e.to_string()})),
    }
}

async fn update_product(
    State(_pool): State<PgPool>,
    Path(_id): Path<Uuid>,
    Json(_body): Json<Value>,
) -> Json<Value> {
    Json(json!({"message": "inventory/products/:id PATCH - not implemented"}))
}

async fn create_warehouse(
    State(pool): State<PgPool>,
    Json(body): Json<Value>,
) -> Json<Value> {
    let service = InventoryService::new(pool);
    let company_id: Uuid = match body.get("company_id").and_then(|v| serde_json::from_value(v.clone()).ok()) {
        Some(id) => id,
        None => return Json(json!({"error": "company_id is required"})),
    };
    let branch_id = body.get("branch_id").and_then(|v| serde_json::from_value(v.clone()).ok());
    let code = body.get("code").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let name = body.get("name").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let location = body.get("location").and_then(|v| v.as_str()).map(|s| s.to_string());

    match service.create_warehouse(company_id, branch_id, code, name, location).await {
        Ok(warehouse) => Json(json!(warehouse)),
        Err(e) => Json(json!({"error": e.to_string()})),
    }
}

async fn list_warehouses(
    State(_pool): State<PgPool>,
    Query(_params): Query<ListWarehousesQuery>,
) -> Json<Value> {
    Json(json!({"message": "inventory/warehouses GET - not implemented"}))
}

async fn create_stock_movement(
    State(pool): State<PgPool>,
    Json(body): Json<Value>,
) -> Json<Value> {
    let service = InventoryService::new(pool);
    Json(json!({"message": "inventory/stock-movements POST - use service directly"}))
}

async fn list_stock_movements(
    State(_pool): State<PgPool>,
    Query(_params): Query<ListStockMovementsQuery>,
) -> Json<Value> {
    Json(json!({"message": "inventory/stock-movements GET - not implemented"}))
}

async fn get_stock_levels(
    State(pool): State<PgPool>,
    Query(params): Query<StockLevelsQuery>,
) -> Json<Value> {
    let service = InventoryService::new(pool);
    match service.get_stock_levels(params.company_id).await {
        Ok(levels) => Json(json!(levels)),
        Err(e) => Json(json!({"error": e.to_string()})),
    }
}

async fn get_stock_valuation(
    State(pool): State<PgPool>,
    Query(params): Query<StockValuationQuery>,
) -> Json<Value> {
    let service = InventoryService::new(pool);
    match service.get_stock_valuation(params.company_id).await {
        Ok(valuation) => Json(json!(valuation)),
        Err(e) => Json(json!({"error": e.to_string()})),
    }
}

async fn get_low_stock_alerts(
    State(pool): State<PgPool>,
    Query(params): Query<LowStockAlertsQuery>,
) -> Json<Value> {
    let service = InventoryService::new(pool);
    match service.get_low_stock_alerts(params.company_id).await {
        Ok(alerts) => Json(json!(alerts)),
        Err(e) => Json(json!({"error": e.to_string()})),
    }
}

async fn adjust_stock(
    State(pool): State<PgPool>,
    Json(body): Json<AdjustStockRequest>,
) -> Json<Value> {
    let service = InventoryService::new(pool);
    match service.adjust_stock(
        body.product_id,
        body.adjustment_type,
        body.quantity,
        body.reason,
        body.warehouse_id,
        body.created_by,
    ).await {
        Ok(movement) => Json(json!(movement)),
        Err(e) => Json(json!({"error": e.to_string()})),
    }
}
