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
pub struct ListCategoriesQuery {
    pub page: Option<i64>,
    pub per_page: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub struct ListAssetsQuery {
    pub page: Option<i64>,
    pub per_page: Option<i64>,
    pub category_id: Option<Uuid>,
    pub status: Option<String>,
    pub company_id: Option<Uuid>,
}

pub fn router() -> Router<PgPool> {
    Router::new()
        .route("/categories", post(create_category).get(list_categories))
        .route("/assets", post(create_asset).get(list_assets))
        .route("/assets/:id", get(get_asset).patch(update_asset))
}

async fn create_category(
    State(_pool): State<PgPool>,
    Json(_body): Json<Value>,
) -> Json<Value> {
    Json(json!({"message": "fixed-assets/categories POST - not implemented"}))
}

async fn list_categories(
    State(_pool): State<PgPool>,
    Query(_params): Query<ListCategoriesQuery>,
) -> Json<Value> {
    Json(json!({"message": "fixed-assets/categories GET - not implemented"}))
}

async fn create_asset(
    State(_pool): State<PgPool>,
    Json(_body): Json<Value>,
) -> Json<Value> {
    Json(json!({"message": "fixed-assets/assets POST - not implemented"}))
}

async fn list_assets(
    State(_pool): State<PgPool>,
    Query(_params): Query<ListAssetsQuery>,
) -> Json<Value> {
    Json(json!({"message": "fixed-assets/assets GET - not implemented"}))
}

async fn get_asset(
    State(_pool): State<PgPool>,
    Path(_id): Path<Uuid>,
) -> Json<Value> {
    Json(json!({"message": "fixed-assets/assets/:id GET - not implemented"}))
}

async fn update_asset(
    State(_pool): State<PgPool>,
    Path(_id): Path<Uuid>,
    Json(_body): Json<Value>,
) -> Json<Value> {
    Json(json!({"message": "fixed-assets/assets/:id PATCH - not implemented"}))
}
