// Author: Quadri Atharu
use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type, PartialEq)]
#[sqlx(type_name = "document_type", rename_all = "snake_case")]
pub enum DocumentType {
    Invoice,
    Receipt,
    BankStatement,
    PurchaseOrder,
    DeliveryNote,
    Contract,
    Other,
}

impl std::fmt::Display for DocumentType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DocumentType::Invoice => write!(f, "invoice"),
            DocumentType::Receipt => write!(f, "receipt"),
            DocumentType::BankStatement => write!(f, "bank_statement"),
            DocumentType::PurchaseOrder => write!(f, "purchase_order"),
            DocumentType::DeliveryNote => write!(f, "delivery_note"),
            DocumentType::Contract => write!(f, "contract"),
            DocumentType::Other => write!(f, "other"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type, PartialEq)]
#[sqlx(type_name = "ocr_document_status", rename_all = "snake_case")]
pub enum OcrDocumentStatus {
    Uploaded,
    Processing,
    Extracted,
    ReviewPending,
    Approved,
    Rejected,
    JournalCreated,
}

impl std::fmt::Display for OcrDocumentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OcrDocumentStatus::Uploaded => write!(f, "uploaded"),
            OcrDocumentStatus::Processing => write!(f, "processing"),
            OcrDocumentStatus::Extracted => write!(f, "extracted"),
            OcrDocumentStatus::ReviewPending => write!(f, "review_pending"),
            OcrDocumentStatus::Approved => write!(f, "approved"),
            OcrDocumentStatus::Rejected => write!(f, "rejected"),
            OcrDocumentStatus::JournalCreated => write!(f, "journal_created"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct OcrDocument {
    pub id: Uuid,
    pub company_id: Uuid,
    pub branch_id: Option<Uuid>,
    pub document_type: DocumentType,
    pub status: OcrDocumentStatus,
    pub file_name: String,
    pub file_path: String,
    pub file_size: i64,
    pub content_type: String,
    pub ocr_raw_text: Option<String>,
    pub ocr_confidence: Option<f64>,
    pub reviewed_by: Option<Uuid>,
    pub reviewed_at: Option<NaiveDateTime>,
    pub journal_header_id: Option<Uuid>,
    pub uploaded_by: Uuid,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct OcrExtractionField {
    pub id: Uuid,
    pub document_id: Uuid,
    pub field_name: String,
    pub field_value: Option<String>,
    pub confidence: Option<f64>,
    pub is_edited: bool,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}
