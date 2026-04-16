use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, State};
use uuid::Uuid;
use chrono::{DateTime, Utc};

use crate::ocr::pipeline::{DocumentPipeline, PipelineResult};
use crate::sidecar::manager::SidecarManager;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct OcrStatus {
    pub ready: bool,
    pub ollama_connected: bool,
    pub tesseract_available: bool,
    pub documents_processed: u64,
    pub last_error: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DocumentRecord {
    pub id: Uuid,
    pub file_name: String,
    pub file_type: String,
    pub document_type: String,
    pub confidence_score: f64,
    pub processed_at: DateTime<Utc>,
    pub status: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ReviewSubmission {
    pub document_id: Uuid,
    pub reviewer_notes: String,
    pub corrections: serde_json::Value,
    pub approved: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProcessDocumentRequest {
    pub file_path: String,
    pub force_ocr: Option<bool>,
    pub preferred_language: Option<String>,
}

#[tauri::command]
pub async fn process_document(
    app: AppHandle,
    request: ProcessDocumentRequest,
) -> Result<PipelineResult, String> {
    let pipeline = DocumentPipeline::new(app.clone());

    let force_ocr = request.force_ocr.unwrap_or(false);
    let language = request.preferred_language.unwrap_or_else(|| "eng".to_string());

    let result = pipeline
        .process(&request.file_path, force_ocr, &language)
        .await
        .map_err(|e| format!("Document processing failed: {e}"))?;

    let _ = app.emit("ocr:document-processed", &result);
    Ok(result)
}

#[tauri::command]
pub async fn get_ocr_status(
    _app: AppHandle,
    _sidecar_manager: State<'_, SidecarManager>,
) -> Result<OcrStatus, String> {
    let ollama_connected = reqwest::Client::new()
        .get("http://localhost:11434/api/tags")
        .timeout(std::time::Duration::from_secs(3))
        .send()
        .await
        .is_ok();

    let tesseract_available = which::which("tesseract").is_ok();

    let ready = ollama_connected && tesseract_available;

    Ok(OcrStatus {
        ready,
        ollama_connected,
        tesseract_available,
        documents_processed: 0,
        last_error: if ready {
            None
        } else {
            Some(format!(
                "Missing: {}{}",
                if !ollama_connected { "Ollama " } else { "" },
                if !tesseract_available { "Tesseract" } else { "" }
            ))
        },
    })
}

#[tauri::command]
pub async fn get_document_history(
    _app: AppHandle,
    limit: Option<u64>,
) -> Result<Vec<DocumentRecord>, String> {
    let limit = limit.unwrap_or(50);
    let client = reqwest::Client::new();
    let url = format!("http://localhost:8100/api/v1/documents?limit={limit}");

    let response = client
        .get(&url)
        .timeout(std::time::Duration::from_secs(10))
        .send()
        .await
        .map_err(|e| format!("Backend unreachable: {e}"))?;

    if !response.status().is_success() {
        return Err(format!("Backend returned status: {}", response.status()));
    }

    let records: Vec<DocumentRecord> = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse document history: {e}"))?;

    Ok(records)
}

#[tauri::command]
pub async fn submit_for_review(
    app: AppHandle,
    submission: ReviewSubmission,
) -> Result<(), String> {
    let client = reqwest::Client::new();
    let url = format!("http://localhost:8100/api/v1/documents/{}/review", submission.document_id);

    let response = client
        .post(&url)
        .json(&submission)
        .timeout(std::time::Duration::from_secs(10))
        .send()
        .await
        .map_err(|e| format!("Failed to submit review: {e}"))?;

    if !response.status().is_success() {
        let body = response.text().await.unwrap_or_default();
        return Err(format!("Review submission failed: {body}"));
    }

    let _ = app.emit("ocr:review-submitted", &submission.document_id);
    Ok(())
}
