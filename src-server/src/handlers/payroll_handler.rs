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
pub struct ListEmployeesQuery {
    pub page: Option<i64>,
    pub per_page: Option<i64>,
    pub search: Option<String>,
    pub department_id: Option<Uuid>,
    pub status: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ListRunsQuery {
    pub page: Option<i64>,
    pub per_page: Option<i64>,
    pub status: Option<String>,
    pub company_id: Option<Uuid>,
    pub period: Option<String>,
}

pub fn router() -> Router<PgPool> {
    Router::new()
        .route("/employees", post(create_employee).get(list_employees))
        .route("/employees/:id", patch(update_employee))
        .route("/runs", post(create_run).get(list_runs))
        .route("/runs/:id/process", post(process_run))
        .route("/runs/:id/payslips", get(get_run_payslips))
        .route("/payslip/:id", get(get_payslip))
        .route("/runs/:id/post", post(post_run))
}

async fn create_employee(
    State(_pool): State<PgPool>,
    Json(_body): Json<Value>,
) -> Json<Value> {
    Json(json!({"message": "payroll/employees POST - not implemented"}))
}

async fn list_employees(
    State(_pool): State<PgPool>,
    Query(_params): Query<ListEmployeesQuery>,
) -> Json<Value> {
    Json(json!({"message": "payroll/employees GET - not implemented"}))
}

async fn update_employee(
    State(_pool): State<PgPool>,
    Path(_id): Path<Uuid>,
    Json(_body): Json<Value>,
) -> Json<Value> {
    Json(json!({"message": "payroll/employees/:id PATCH - not implemented"}))
}

async fn create_run(
    State(_pool): State<PgPool>,
    Json(_body): Json<Value>,
) -> Json<Value> {
    Json(json!({"message": "payroll/runs POST - not implemented"}))
}

async fn list_runs(
    State(_pool): State<PgPool>,
    Query(_params): Query<ListRunsQuery>,
) -> Json<Value> {
    Json(json!({"message": "payroll/runs GET - not implemented"}))
}

async fn process_run(
    State(_pool): State<PgPool>,
    Path(_id): Path<Uuid>,
    Json(_body): Json<Value>,
) -> Json<Value> {
    Json(json!({"message": "payroll/runs/:id/process - not implemented"}))
}

async fn get_run_payslips(
    State(_pool): State<PgPool>,
    Path(_id): Path<Uuid>,
) -> Json<Value> {
    Json(json!({"message": "payroll/runs/:id/payslips - not implemented"}))
}

async fn get_payslip(
    State(_pool): State<PgPool>,
    Path(_id): Path<Uuid>,
) -> Json<Value> {
    Json(json!({"message": "payroll/payslip/:id - not implemented"}))
}

async fn post_run(
    State(_pool): State<PgPool>,
    Path(_id): Path<Uuid>,
    Json(_body): Json<Value>,
) -> Json<Value> {
    Json(json!({"message": "payroll/runs/:id/post - not implemented"}))
}
