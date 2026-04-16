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

#[derive(Debug, Deserialize)]
pub struct ListJournalsQuery {
    pub page: Option<i64>,
    pub per_page: Option<i64>,
    pub status: Option<String>,
    pub date_from: Option<String>,
    pub date_to: Option<String>,
    pub account_id: Option<Uuid>,
    pub company_id: Option<Uuid>,
}

pub fn router() -> Router<PgPool> {
    Router::new()
        .route("/", get(list_journals).post(create_draft))
        .route("/metadata/options", get(get_metadata_options))
        .route("/templates/all", get(list_templates))
        .route("/templates", post(create_template))
        .route("/templates/:id/generate", post(generate_from_template))
        .route("/:id", get(get_journal).patch(update_draft))
        .route("/:id/validate", post(validate_journal))
        .route("/:id/submit", post(submit_journal))
        .route("/:id/recall", post(recall_journal))
        .route("/:id/approve", post(approve_journal))
        .route("/:id/reject", post(reject_journal))
        .route("/:id/post", post(post_journal))
        .route("/:id/reverse", post(reverse_journal))
        .route("/:id/cancel", post(cancel_journal))
}

async fn list_journals(
    State(_pool): State<PgPool>,
    Query(_params): Query<ListJournalsQuery>,
) -> Json<Value> {
    Json(json!({"message": "journals/list - not implemented"}))
}

async fn get_metadata_options(
    State(_pool): State<PgPool>,
) -> Json<Value> {
    Json(json!({"message": "journals/metadata/options - not implemented"}))
}

async fn create_draft(
    State(_pool): State<PgPool>,
    Json(_body): Json<Value>,
) -> Json<Value> {
    Json(json!({"message": "journals/create - not implemented"}))
}

async fn update_draft(
    State(_pool): State<PgPool>,
    Path(_id): Path<Uuid>,
    Json(_body): Json<Value>,
) -> Json<Value> {
    Json(json!({"message": "journals/:id PATCH - not implemented"}))
}

async fn get_journal(
    State(_pool): State<PgPool>,
    Path(_id): Path<Uuid>,
) -> Json<Value> {
    Json(json!({"message": "journals/:id GET - not implemented"}))
}

async fn validate_journal(
    State(_pool): State<PgPool>,
    Path(_id): Path<Uuid>,
    Json(_body): Json<Value>,
) -> Json<Value> {
    Json(json!({"message": "journals/:id/validate - not implemented"}))
}

async fn submit_journal(
    State(_pool): State<PgPool>,
    Path(_id): Path<Uuid>,
    Json(_body): Json<Value>,
) -> Json<Value> {
    Json(json!({"message": "journals/:id/submit - not implemented"}))
}

async fn recall_journal(
    State(_pool): State<PgPool>,
    Path(_id): Path<Uuid>,
    Json(_body): Json<Value>,
) -> Json<Value> {
    Json(json!({"message": "journals/:id/recall - not implemented"}))
}

async fn approve_journal(
    State(_pool): State<PgPool>,
    Path(_id): Path<Uuid>,
    Json(_body): Json<Value>,
) -> Json<Value> {
    Json(json!({"message": "journals/:id/approve - not implemented"}))
}

async fn reject_journal(
    State(_pool): State<PgPool>,
    Path(_id): Path<Uuid>,
    Json(_body): Json<Value>,
) -> Json<Value> {
    Json(json!({"message": "journals/:id/reject - not implemented"}))
}

async fn post_journal(
    State(_pool): State<PgPool>,
    Path(_id): Path<Uuid>,
    Json(_body): Json<Value>,
) -> Json<Value> {
    Json(json!({"message": "journals/:id/post - not implemented"}))
}

async fn reverse_journal(
    State(_pool): State<PgPool>,
    Path(_id): Path<Uuid>,
    Json(_body): Json<Value>,
) -> Json<Value> {
    Json(json!({"message": "journals/:id/reverse - not implemented"}))
}

async fn cancel_journal(
    State(_pool): State<PgPool>,
    Path(_id): Path<Uuid>,
    Json(_body): Json<Value>,
) -> Json<Value> {
    Json(json!({"message": "journals/:id/cancel - not implemented"}))
}

async fn list_templates(
    State(_pool): State<PgPool>,
) -> Json<Value> {
    Json(json!({"message": "journals/templates/all - not implemented"}))
}

async fn create_template(
    State(_pool): State<PgPool>,
    Json(_body): Json<Value>,
) -> Json<Value> {
    Json(json!({"message": "journals/templates POST - not implemented"}))
}

async fn generate_from_template(
    State(_pool): State<PgPool>,
    Path(_id): Path<Uuid>,
    Json(_body): Json<Value>,
) -> Json<Value> {
    Json(json!({"message": "journals/templates/:id/generate - not implemented"}))
}
