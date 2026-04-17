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
pub struct ListCompaniesQuery {
    pub page: Option<i64>,
    pub per_page: Option<i64>,
    pub search: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ListBranchesQuery {
    pub page: Option<i64>,
    pub per_page: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub struct ListDepartmentsQuery {
    pub page: Option<i64>,
    pub per_page: Option<i64>,
}

pub fn router() -> Router<crate::routes::AppState> {
    Router::new()
        .route("/companies", post(create_company).get(list_companies))
        .route("/companies/:id", patch(update_company))
        .route("/companies/:id/branches", post(create_branch).get(list_branches))
        .route("/companies/:id/departments", post(create_department).get(list_departments))
        .route("/companies/:id/cost-centers", post(create_cost_center))
        .route("/companies/:id/projects", post(create_project))
        .route("/companies/:id/settings", patch(update_company_settings))
}

async fn create_company(
    State(_pool): State<PgPool>,
    Json(_body): Json<Value>,
) -> Json<Value> {
    Json(json!({"message": "org/companies POST - not implemented"}))
}

async fn list_companies(
    State(_pool): State<PgPool>,
    Query(_params): Query<ListCompaniesQuery>,
) -> Json<Value> {
    Json(json!({"message": "org/companies GET - not implemented"}))
}

async fn update_company(
    State(_pool): State<PgPool>,
    Path(_id): Path<Uuid>,
    Json(_body): Json<Value>,
) -> Json<Value> {
    Json(json!({"message": "org/companies/:id PATCH - not implemented"}))
}

async fn create_branch(
    State(_pool): State<PgPool>,
    Path(_id): Path<Uuid>,
    Json(_body): Json<Value>,
) -> Json<Value> {
    Json(json!({"message": "org/companies/:id/branches POST - not implemented"}))
}

async fn list_branches(
    State(_pool): State<PgPool>,
    Path(_id): Path<Uuid>,
    Query(_params): Query<ListBranchesQuery>,
) -> Json<Value> {
    Json(json!({"message": "org/companies/:id/branches GET - not implemented"}))
}

async fn create_department(
    State(_pool): State<PgPool>,
    Path(_id): Path<Uuid>,
    Json(_body): Json<Value>,
) -> Json<Value> {
    Json(json!({"message": "org/companies/:id/departments POST - not implemented"}))
}

async fn list_departments(
    State(_pool): State<PgPool>,
    Path(_id): Path<Uuid>,
    Query(_params): Query<ListDepartmentsQuery>,
) -> Json<Value> {
    Json(json!({"message": "org/companies/:id/departments GET - not implemented"}))
}

async fn create_cost_center(
    State(_pool): State<PgPool>,
    Path(_id): Path<Uuid>,
    Json(_body): Json<Value>,
) -> Json<Value> {
    Json(json!({"message": "org/companies/:id/cost-centers POST - not implemented"}))
}

async fn create_project(
    State(_pool): State<PgPool>,
    Path(_id): Path<Uuid>,
    Json(_body): Json<Value>,
) -> Json<Value> {
    Json(json!({"message": "org/companies/:id/projects POST - not implemented"}))
}

async fn update_company_settings(
    State(_pool): State<PgPool>,
    Path(_id): Path<Uuid>,
    Json(_body): Json<Value>,
) -> Json<Value> {
    Json(json!({"message": "org/companies/:id/settings PATCH - not implemented"}))
}
