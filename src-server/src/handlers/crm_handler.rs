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
pub struct ListContactsQuery {
    pub page: Option<i64>,
    pub per_page: Option<i64>,
    pub search: Option<String>,
    pub contact_type: Option<String>,
    pub company_id: Option<Uuid>,
}

#[derive(Debug, Deserialize)]
pub struct ListDealsQuery {
    pub page: Option<i64>,
    pub per_page: Option<i64>,
    pub stage: Option<String>,
    pub contact_id: Option<Uuid>,
    pub company_id: Option<Uuid>,
}

#[derive(Debug, Deserialize)]
pub struct ListActivitiesQuery {
    pub page: Option<i64>,
    pub per_page: Option<i64>,
    pub contact_id: Option<Uuid>,
    pub deal_id: Option<Uuid>,
    pub activity_type: Option<String>,
}

pub fn router() -> Router<crate::routes::AppState> {
    Router::new()
        .route("/contacts", post(create_contact).get(list_contacts))
        .route("/contacts/:id", patch(update_contact))
        .route("/deals", post(create_deal).get(list_deals))
        .route("/deals/:id/stage", patch(update_deal_stage))
        .route("/pipeline", get(get_pipeline))
        .route("/activities", post(create_activity).get(list_activities))
}

async fn create_contact(
    State(_pool): State<PgPool>,
    Json(_body): Json<Value>,
) -> Json<Value> {
    Json(json!({"message": "crm/contacts POST - not implemented"}))
}

async fn list_contacts(
    State(_pool): State<PgPool>,
    Query(_params): Query<ListContactsQuery>,
) -> Json<Value> {
    Json(json!({"message": "crm/contacts GET - not implemented"}))
}

async fn update_contact(
    State(_pool): State<PgPool>,
    Path(_id): Path<Uuid>,
    Json(_body): Json<Value>,
) -> Json<Value> {
    Json(json!({"message": "crm/contacts/:id PATCH - not implemented"}))
}

async fn create_deal(
    State(_pool): State<PgPool>,
    Json(_body): Json<Value>,
) -> Json<Value> {
    Json(json!({"message": "crm/deals POST - not implemented"}))
}

async fn list_deals(
    State(_pool): State<PgPool>,
    Query(_params): Query<ListDealsQuery>,
) -> Json<Value> {
    Json(json!({"message": "crm/deals GET - not implemented"}))
}

async fn update_deal_stage(
    State(_pool): State<PgPool>,
    Path(_id): Path<Uuid>,
    Json(_body): Json<Value>,
) -> Json<Value> {
    Json(json!({"message": "crm/deals/:id/stage PATCH - not implemented"}))
}

async fn get_pipeline(
    State(_pool): State<PgPool>,
) -> Json<Value> {
    Json(json!({"message": "crm/pipeline GET - not implemented"}))
}

async fn create_activity(
    State(_pool): State<PgPool>,
    Json(_body): Json<Value>,
) -> Json<Value> {
    Json(json!({"message": "crm/activities POST - not implemented"}))
}

async fn list_activities(
    State(_pool): State<PgPool>,
    Query(_params): Query<ListActivitiesQuery>,
) -> Json<Value> {
    Json(json!({"message": "crm/activities GET - not implemented"}))
}
