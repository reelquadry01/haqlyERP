// Author: Quadri Atharu
use serde::{Deserialize, Serialize};
use tauri::AppHandle;

use crate::ocr::classifier::{classify, DocumentType};
use crate::ocr::extractor::extract;
use crate::ocr::ollama_reasoner::{fallback_extraction, reason, ExtractionResult};
use crate::ocr::preprocessor::preprocess;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineResult {
    pub document_type: String,
    pub raw_text: String,
    pub extraction: ExtractionResult,
    pub confidence_score: f64,
    pub processing_time_ms: u64,
}

pub struct DocumentPipeline {
    #[allow(dead_code)]
    app: AppHandle,
}

impl DocumentPipeline {
    pub fn new(app: AppHandle) -> Self {
        Self { app }
    }

    pub async fn process(
        &self,
        file_path: &str,
        force_ocr: bool,
        language: &str,
    ) -> anyhow::Result<PipelineResult> {
        let start = std::time::Instant::now();

        let path = std::path::Path::new(file_path);
        if !path.exists() {
            anyhow::bail!("File not found: {file_path}");
        }

        tracing::info!("Processing document: {file_path}");

        let processed = preprocess(path, force_ocr)?;

        let raw_text = extract(&processed, language)?;
        if raw_text.trim().is_empty() {
            tracing::warn!("No text extracted from document");
        }

        let doc_type = classify(&raw_text);
        tracing::info!("Classified document as: {doc_type}");

        let extraction = match reason(&raw_text, &doc_type).await {
            Ok(result) => {
                tracing::info!(
                    "Ollama extraction complete; confidence: {:.2}",
                    result.confidence_score
                );
                result
            }
            Err(e) => {
                tracing::warn!("Ollama reasoning failed: {e}; falling back to regex extraction");
                fallback_extraction(&raw_text, &doc_type)
            }
        };

        let confidence_score = extraction.confidence_score;
        let processing_time_ms = start.elapsed().as_millis() as u64;

        Ok(PipelineResult {
            document_type: doc_type.to_string(),
            raw_text,
            extraction,
            confidence_score,
            processing_time_ms,
        })
    }
}
