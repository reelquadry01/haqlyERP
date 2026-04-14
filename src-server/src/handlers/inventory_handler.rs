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
pub struct ListProductsQuery {
    pub page: Option<i64>,
    pub per_page: Option<i64>,
    pub search: Option<String>,
    pub category: Option<String>,
    pub is_active: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct ListWarehousesQuery {
    pub page: Option<i64>,
    pub per_page: Option<i64>,
    pub company_id: Option<Uuid>,
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
}

#[derive(Debug, Deserialize)]
pub struct StockLevelsQuery {
    pub product_id: Option<Uuid>,
    pub warehouse_id: Option<Uuid>,
    pub company_id: Option<Uuid>,
}

pub fn router() -> Router<PgPool> {
    Router::new()
        .route("/products", post(create_product).get(list_products))
        .route("/products/:id", patch(update_product))
        .route("/warehouses", post(create_warehouse).get(list_warehouses))
        .route("/stock-movements", post(create_stock_movement).get(list_stock_movements))
        .route("/stock-levels", get(get_stock_levels))
}

async fn create_product(
    State(_pool): State<PgPool>,
    Json(_body): Json<Value>,
) -> Json<Value> {
    Json(json!({"message": "inventory/products POST - not implemented"}))
}

async fn list_products(
    State(_pool): State<PgPool>,
    Query(_params): Query<ListProductsQuery>,
) -> Json<Value> {
    Json(json!({"message": "inventory/products GET - not implemented"}))
}

async fn update_product(
    State(_pool): State<PgPool>,
    Path(_id): Path<Uuid>,
    Json(_body): Json<Value>,
) -> Json<Value> {
    Json(json!({"message": "inventory/products/:id PATCH - not implemented"}))
}

async fn create_warehouse(
    State(_pool): State<PgPool>,
    Json(_body): Json<Value>,
) -> Json<Value> {
    Json(json!({"message": "inventory/warehouses POST - not implemented"}))
}

async fn list_warehouses(
    State(_pool): State<PgPool>,
    Query(_params): Query<ListWarehousesQuery>,
) -> Json<Value> {
    Json(json!({"message": "inventory/warehouses GET - not implemented"}))
}

async fn create_stock_movement(
    State(_pool): State<PgPool>,
    Json(_body): Json<Value>,
) -> Json<Value> {
    Json(json!({"message": "inventory/stock-movements POST - not implemented"}))
}

async fn list_stock_movements(
    State(_pool): State<PgPool>,
    Query(_params): Query<ListStockMovementsQuery>,
) -> Json<Value> {
    Json(json!({"message": "inventory/stock-movements GET - not implemented"}))
}

async fn get_stock_levels(
    State(_pool): State<PgPool>,
    Query(_params): Query<StockLevelsQuery>,
) -> Json<Value> {
    Json(json!({"message": "inventory/stock-levels GET - not implemented"}))
}
