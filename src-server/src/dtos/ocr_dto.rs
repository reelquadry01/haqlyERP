// Author: Quadri Atharu
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct UploadDocumentRequest {
    pub company_id: Uuid,
    pub branch_id: Option<Uuid>,
    pub document_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentReviewRequest {
    pub document_id: Uuid,
    pub action: String,
    pub comment: Option<String>,
    pub field_overrides: Vec<FieldOverride>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldOverride {
    pub field_name: String,
    pub field_value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OcrExtractionResult {
    pub document_id: Uuid,
    pub document_type: String,
    pub confidence: f64,
    pub fields: Vec<ExtractedField>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractedField {
    pub field_name: String,
    pub field_value: Option<String>,
    pub confidence: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentListParams {
    pub company_id: Option<Uuid>,
    pub status: Option<String>,
    pub document_type: Option<String>,
    pub page: Option<i64>,
    pub limit: Option<i64>,
}
