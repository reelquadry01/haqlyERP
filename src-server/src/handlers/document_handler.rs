// Author: Quadri Atharu
// Crate: haqly-erp-server

use axum::{
    Json,
    extract::Multipart,
    routing::{get, post, delete},
    Router,
    body::Body,
    http::{StatusCode, header},
    response::{IntoResponse, Response},
};
use axum::extract::{Path, Query, State};
use sqlx::PgPool;
use serde::Deserialize;
use serde_json::{json, Value};
use uuid::Uuid;
use tokio::fs;

use crate::services::document_service::DocumentService;

#[derive(Debug, Deserialize)]
pub struct ListDocumentsQuery {
    pub company_id: Option<Uuid>,
    pub entity_type: Option<String>,
    pub entity_id: Option<Uuid>,
}

pub fn router() -> Router<PgPool> {
    Router::new()
        .route("/upload", post(upload_document))
        .route("/:entity_type/:entity_id", get(list_documents))
        .route("/:id", get(get_document))
        .route("/:id/download", get(download_document))
        .route("/:id", delete(delete_document))
}

async fn upload_document(
    State(pool): State<PgPool>,
    mut multipart: Multipart,
) -> Result<impl IntoResponse, (StatusCode, Json<Value>)> {
    let service = DocumentService::new(pool);
    let mut company_id: Option<Uuid> = None;
    let mut entity_type: Option<String> = None;
    let mut entity_id: Option<Uuid> = None;
    let mut description: Option<String> = None;
    let mut uploaded_by: Option<Uuid> = None;
    let mut file_name = String::new();
    let mut file_data = Vec::new();
    let mut mime_type = "application/octet-stream".to_string();

    while let Some(field) = multipart.next_field().await.map_err(|e| {
        (StatusCode::BAD_REQUEST, Json(json!({"error": format!("Multipart error: {}", e)})))
    })? {
        let name = field.name().unwrap_or("").to_string();
        match name.as_str() {
            "company_id" => {
                let text = field.text().await.map_err(|e| {
                    (StatusCode::BAD_REQUEST, Json(json!({"error": format!("Field error: {}", e)})))
                })?;
                company_id = text.parse().ok();
            }
            "entity_type" => {
                let text = field.text().await.map_err(|e| {
                    (StatusCode::BAD_REQUEST, Json(json!({"error": format!("Field error: {}", e)})))
                })?;
                entity_type = Some(text);
            }
            "entity_id" => {
                let text = field.text().await.map_err(|e| {
                    (StatusCode::BAD_REQUEST, Json(json!({"error": format!("Field error: {}", e)})))
                })?;
                entity_id = text.parse().ok();
            }
            "description" => {
                let text = field.text().await.map_err(|e| {
                    (StatusCode::BAD_REQUEST, Json(json!({"error": format!("Field error: {}", e)})))
                })?;
                description = Some(text);
            }
            "uploaded_by" => {
                let text = field.text().await.map_err(|e| {
                    (StatusCode::BAD_REQUEST, Json(json!({"error": format!("Field error: {}", e)})))
                })?;
                uploaded_by = text.parse().ok();
            }
            "file" => {
                file_name = field.file_name().unwrap_or("upload").to_string();
                mime_type = field.content_type().unwrap_or("application/octet-stream").to_string();
                file_data = field.bytes().await.map_err(|e| {
                    (StatusCode::BAD_REQUEST, Json(json!({"error": format!("File read error: {}", e)})))
                })?.to_vec();
            }
            _ => {}
        }
    }

    let company_id = company_id.ok_or((
        StatusCode::BAD_REQUEST,
        Json(json!({"error": "company_id is required"})),
    ))?;
    let entity_type = entity_type.ok_or((
        StatusCode::BAD_REQUEST,
        Json(json!({"error": "entity_type is required"})),
    ))?;
    let entity_id = entity_id.ok_or((
        StatusCode::BAD_REQUEST,
        Json(json!({"error": "entity_id is required"})),
    ))?;

    if file_data.is_empty() {
        return Err((StatusCode::BAD_REQUEST, Json(json!({"error": "No file provided"}))));
    }

    let doc = service
        .upload_document(company_id, entity_type, entity_id, file_name, file_data, mime_type, description, uploaded_by)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))))?;

    Ok(Json(json!({"data": doc})))
}

async fn list_documents(
    State(pool): State<PgPool>,
    Path((entity_type, entity_id)): Path<(String, Uuid)>,
    Query(params): Query<ListDocumentsQuery>,
) -> Result<impl IntoResponse, (StatusCode, Json<Value>)> {
    let service = DocumentService::new(pool);
    let company_id = params.company_id.ok_or((
        StatusCode::BAD_REQUEST,
        Json(json!({"error": "company_id query param is required"})),
    ))?;

    let docs = service
        .list_documents(company_id, entity_type, entity_id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))))?;

    Ok(Json(json!({"data": docs})))
}

async fn get_document(
    State(pool): State<PgPool>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, (StatusCode, Json<Value>)> {
    let service = DocumentService::new(pool);
    let doc = service
        .get_document(id)
        .await
        .map_err(|e| (StatusCode::NOT_FOUND, Json(json!({"error": e.to_string()}))))?;

    Ok(Json(json!({"data": doc})))
}

async fn download_document(
    State(pool): State<PgPool>,
    Path(id): Path<Uuid>,
) -> Result<Response, (StatusCode, Json<Value>)> {
    let service = DocumentService::new(pool);
    let doc = service
        .get_document(id)
        .await
        .map_err(|e| (StatusCode::NOT_FOUND, Json(json!({"error": e.to_string()}))))?;

    let file_path = service
        .get_file_path(id)
        .await
        .map_err(|e| (StatusCode::NOT_FOUND, Json(json!({"error": e.to_string()}))))?;

    let data = fs::read(&file_path).await.map_err(|e| {
        (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": format!("File read error: {}", e)})))
    })?;

    let body = Body::from(data);
    Ok(Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, &doc.mime_type)
        .header(header::CONTENT_DISPOSITION, format!("attachment; filename=\"{}\"", doc.file_name))
        .body(body)
        .unwrap())
}

async fn delete_document(
    State(pool): State<PgPool>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, (StatusCode, Json<Value>)> {
    let service = DocumentService::new(pool);
    service
        .delete_document(id)
        .await
        .map_err(|e| (StatusCode::NOT_FOUND, Json(json!({"error": e.to_string()}))))?;

    Ok(Json(json!({"message": "Document deleted"})))
}
