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
pub struct AgentLogsQuery {
    pub page: Option<i64>,
    pub per_page: Option<i64>,
    pub agent_type: Option<String>,
    pub status: Option<String>,
}

pub fn router() -> Router<PgPool> {
    Router::new()
        .route("/analyze", post(analyze))
        .route("/compute-tax", post(compute_tax))
        .route("/generate-report", post(generate_report))
        .route("/status", get(get_status))
        .route("/agents/status", post(get_agents_status))
        .route("/agents/execute/:agent_type", post(execute_agent))
        .route("/agents/logs", get(get_agent_logs))
}

async fn analyze(
    State(_pool): State<PgPool>,
    Json(_body): Json<Value>,
) -> Json<Value> {
    Json(json!({"message": "ai/analyze - not implemented"}))
}

async fn compute_tax(
    State(_pool): State<PgPool>,
    Json(_body): Json<Value>,
) -> Json<Value> {
    Json(json!({"message": "ai/compute-tax - not implemented"}))
}

async fn generate_report(
    State(_pool): State<PgPool>,
    Json(_body): Json<Value>,
) -> Json<Value> {
    Json(json!({"message": "ai/generate-report - not implemented"}))
}

async fn get_status(
    State(_pool): State<PgPool>,
) -> Json<Value> {
    Json(json!({"message": "ai/status - not implemented"}))
}

async fn get_agents_status(
    State(_pool): State<PgPool>,
) -> Json<Value> {
    Json(json!({"message": "ai/agents/status - not implemented"}))
}

async fn execute_agent(
    State(_pool): State<PgPool>,
    Path(_agent_type): Path<String>,
    Json(_body): Json<Value>,
) -> Json<Value> {
    Json(json!({"message": "ai/agents/execute/:agentType - not implemented"}))
}

async fn get_agent_logs(
    State(_pool): State<PgPool>,
    Query(_params): Query<AgentLogsQuery>,
) -> Json<Value> {
    Json(json!({"message": "ai/agents/logs - not implemented"}))
}
