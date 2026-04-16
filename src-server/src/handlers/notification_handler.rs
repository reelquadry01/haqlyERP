// Author: Quadri Atharu
// Crate: haqly-erp-server

use axum::{
    Json,
    routing::{get, post, patch},
    Router,
};
use axum::extract::{Query, State};
use sqlx::PgPool;
use serde::Deserialize;
use serde_json::{json, Value};
use uuid::Uuid;

#[derive(Debug, Deserialize)]
pub struct ListNotificationsQuery {
    pub page: Option<i64>,
    pub per_page: Option<i64>,
    pub unread_only: Option<bool>,
    pub notification_type: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct MarkReadBody {
    pub notification_ids: Vec<Uuid>,
}

#[derive(Debug, Deserialize)]
pub struct UpdatePreferencesBody {
    pub email_enabled: Option<bool>,
    pub push_enabled: Option<bool>,
    pub in_app_enabled: Option<bool>,
    pub notification_types: Option<Vec<String>>,
}

pub fn router() -> Router<PgPool> {
    Router::new()
        .route("/", get(list_notifications))
        .route("/mark-read", post(mark_read))
        .route("/mark-all-read", post(mark_all_read))
        .route("/preferences", get(get_preferences).patch(update_preferences))
}

async fn list_notifications(
    State(_pool): State<PgPool>,
    Query(_params): Query<ListNotificationsQuery>,
) -> Json<Value> {
    Json(json!({"message": "notifications/list - not implemented"}))
}

async fn mark_read(
    State(_pool): State<PgPool>,
    Json(_body): Json<MarkReadBody>,
) -> Json<Value> {
    Json(json!({"message": "notifications/mark-read - not implemented"}))
}

async fn mark_all_read(
    State(_pool): State<PgPool>,
) -> Json<Value> {
    Json(json!({"message": "notifications/mark-all-read - not implemented"}))
}

async fn get_preferences(
    State(_pool): State<PgPool>,
) -> Json<Value> {
    Json(json!({"message": "notifications/preferences GET - not implemented"}))
}

async fn update_preferences(
    State(_pool): State<PgPool>,
    Json(_body): Json<UpdatePreferencesBody>,
) -> Json<Value> {
    Json(json!({"message": "notifications/preferences PATCH - not implemented"}))
}
