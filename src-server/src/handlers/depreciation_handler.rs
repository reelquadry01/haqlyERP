// Author: Quadri Atharu
// Crate: haqly-erp-server

use axum::{
    Json, Path, Query, State,
    routing::{get, post},
    Router,
};
use sqlx::PgPool;
use serde::Deserialize;
use serde_json::{json, Value};
use uuid::Uuid;

#[derive(Debug, Deserialize)]
pub struct ListRunsQuery {
    pub page: Option<i64>,
    pub per_page: Option<i64>,
    pub status: Option<String>,
    pub company_id: Option<Uuid>,
}

#[derive(Debug, Deserialize)]
pub struct ListSchedulesQuery {
    pub asset_id: Option<Uuid>,
    pub run_id: Option<Uuid>,
    pub page: Option<i64>,
    pub per_page: Option<i64>,
}

pub fn router() -> Router<PgPool> {
    Router::new()
        .route("/runs", post(create_run).get(list_runs))
        .route("/runs/:id", get(get_run).post(post_run))
        .route("/schedules", get(list_schedules))
}

async fn create_run(
    State(_pool): State<PgPool>,
    Json(_body): Json<Value>,
) -> Json<Value> {
    Json(json!({"message": "depreciation/runs POST - not implemented"}))
}

async fn list_runs(
    State(_pool): State<PgPool>,
    Query(_params): Query<ListRunsQuery>,
) -> Json<Value> {
    Json(json!({"message": "depreciation/runs GET - not implemented"}))
}

async fn list_schedules(
    State(_pool): State<PgPool>,
    Query(_params): Query<ListSchedulesQuery>,
) -> Json<Value> {
    Json(json!({"message": "depreciation/schedules GET - not implemented"}))
}

async fn get_run(
    State(_pool): State<PgPool>,
    Path(_id): Path<Uuid>,
) -> Json<Value> {
    Json(json!({"message": "depreciation/runs/:id GET - not implemented"}))
}

async fn post_run(
    State(_pool): State<PgPool>,
    Path(_id): Path<Uuid>,
    Json(_body): Json<Value>,
) -> Json<Value> {
    Json(json!({"message": "depreciation/runs/:id/post - not implemented"}))
}
