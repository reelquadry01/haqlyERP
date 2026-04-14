// Author: Quadri Atharu
// Crate: haqly-erp-server

use axum::{
    Json, Path, Query, State,
    extract::Multipart,
    routing::{get, post},
    Router,
};
use sqlx::PgPool;
use serde::Deserialize;
use serde_json::{json, Value};
use uuid::Uuid;

#[derive(Debug, Deserialize)]
pub struct ListDocumentsQuery {
    pub page: Option<i64>,
    pub per_page: Option<i64>,
    pub status: Option<String>,
    pub document_type: Option<String>,
    pub company_id: Option<Uuid>,
}

pub fn router() -> Router<PgPool> {
    Router::new()
        .route("/upload", post(upload_document))
        .route("/", get(list_documents))
        .route("/:id", get(get_document))
        .route("/:id/review", post(review_document))
        .route("/:id/approve", post(approve_document))
        .route("/:id/reject", post(reject_document))
        .route("/:id/create-journal", post(create_journal_from_document))
}

async fn upload_document(
    State(_pool): State<PgPool>,
    mut _multipart: Multipart,
) -> Json<Value> {
    Json(json!({"message": "documents/upload - not implemented"}))
}

async fn list_documents(
    State(_pool): State<PgPool>,
    Query(_params): Query<ListDocumentsQuery>,
) -> Json<Value> {
    Json(json!({"message": "documents/list - not implemented"}))
}

async fn get_document(
    State(_pool): State<PgPool>,
    Path(_id): Path<Uuid>,
) -> Json<Value> {
    Json(json!({"message": "documents/:id GET - not implemented"}))
}

async fn review_document(
    State(_pool): State<PgPool>,
    Path(_id): Path<Uuid>,
    Json(_body): Json<Value>,
) -> Json<Value> {
    Json(json!({"message": "documents/:id/review - not implemented"}))
}

async fn approve_document(
    State(_pool): State<PgPool>,
    Path(_id): Path<Uuid>,
    Json(_body): Json<Value>,
) -> Json<Value> {
    Json(json!({"message": "documents/:id/approve - not implemented"}))
}

async fn reject_document(
    State(_pool): State<PgPool>,
    Path(_id): Path<Uuid>,
    Json(_body): Json<Value>,
) -> Json<Value> {
    Json(json!({"message": "documents/:id/reject - not implemented"}))
}

async fn create_journal_from_document(
    State(_pool): State<PgPool>,
    Path(_id): Path<Uuid>,
    Json(_body): Json<Value>,
) -> Json<Value> {
    Json(json!({"message": "documents/:id/create-journal - not implemented"}))
}
