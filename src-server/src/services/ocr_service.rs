// Author: Quadri Atharu
use anyhow::{anyhow, Result};
use bigdecimal::{BigDecimal, FromPrimitive};
use sqlx::PgPool;
use uuid::Uuid;

use crate::dtos::ocr_dto::{DocumentReviewRequest, FieldOverride};
use crate::models::ocr_document::{OcrDocument, OcrDocumentStatus, OcrExtractionField, DocumentType};

#[derive(Clone)]
pub struct OcrService {
    pub pool: PgPool,
}

impl OcrService {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn upload_document(
        &self,
        company_id: Uuid,
        branch_id: Option<Uuid>,
        document_type: DocumentType,
        file_name: String,
        file_path: String,
        file_size: i64,
        content_type: String,
        uploaded_by: Uuid,
    ) -> Result<OcrDocument> {
        let id = Uuid::now_v7();
        sqlx::query(
            r#"INSERT INTO ocr_documents (id, company_id, branch_id, document_type, status, file_name, file_path, file_size, content_type, uploaded_by, created_at, updated_at)
               VALUES ($1, $2, $3, $4, 'uploaded', $5, $6, $7, $8, $9, NOW(), NOW())"#,
        )
        .bind(id)
        .bind(company_id)
        .bind(branch_id)
        .bind(&document_type)
        .bind(&file_name)
        .bind(&file_path)
        .bind(file_size)
        .bind(&content_type)
        .bind(uploaded_by)
        .execute(&self.pool)
        .await?;

        sqlx::query(
            "UPDATE ocr_documents SET status = 'processing', updated_at = NOW() WHERE id = $1",
        )
        .bind(id)
        .execute(&self.pool)
        .await?;

        let simulated_text = self.simulate_ocr_extraction(&document_type);

        sqlx::query(
            "UPDATE ocr_documents SET ocr_raw_text = $1, ocr_confidence = $2, status = 'extracted', updated_at = NOW() WHERE id = $3",
        )
        .bind(&simulated_text.raw_text)
        .bind(simulated_text.confidence)
        .bind(id)
        .execute(&self.pool)
        .await?;

        for field in &simulated_text.fields {
            let field_id = Uuid::now_v7();
            sqlx::query(
                r#"INSERT INTO ocr_extraction_fields (id, document_id, field_name, field_value, confidence, is_edited, created_at, updated_at)
                   VALUES ($1, $2, $3, $4, $5, false, NOW(), NOW())"#,
            )
            .bind(field_id)
            .bind(id)
            .bind(&field.field_name)
            .bind(&field.field_value)
            .bind(field.confidence)
            .execute(&self.pool)
            .await?;
        }

        sqlx::query(
            "UPDATE ocr_documents SET status = 'review_pending', updated_at = NOW() WHERE id = $1",
        )
        .bind(id)
        .execute(&self.pool)
        .await?;

        self.get_document(id).await
    }

    pub async fn get_document(&self, document_id: Uuid) -> Result<OcrDocument> {
        sqlx::query_as::<_, OcrDocument>("SELECT * FROM ocr_documents WHERE id = $1")
            .bind(document_id)
            .fetch_optional(&self.pool)
            .await?
            .ok_or_else(|| anyhow!("Document not found"))
    }

    pub async fn list_documents(
        &self,
        company_id: Uuid,
        page: i64,
        limit: i64,
        status: Option<String>,
    ) -> Result<Vec<OcrDocument>> {
        let offset = (page - 1) * limit;
        match status {
            Some(s) => sqlx::query_as::<_, OcrDocument>(
                "SELECT * FROM ocr_documents WHERE company_id = $1 AND status = $2 ORDER BY created_at DESC LIMIT $3 OFFSET $4",
            )
            .bind(company_id)
            .bind(&s)
            .bind(limit)
            .bind(offset)
            .fetch_all(&self.pool)
            .await,
            None => sqlx::query_as::<_, OcrDocument>(
                "SELECT * FROM ocr_documents WHERE company_id = $1 ORDER BY created_at DESC LIMIT $2 OFFSET $3",
            )
            .bind(company_id)
            .bind(limit)
            .bind(offset)
            .fetch_all(&self.pool)
            .await,
        }
        .map_err(|e| anyhow!("Failed to list documents: {}", e))
    }

    pub async fn review_document(
        &self,
        document_id: Uuid,
        reviewer_id: Uuid,
        field_overrides: Vec<FieldOverride>,
    ) -> Result<OcrDocument> {
        let doc = self.get_document(document_id).await?;

        if doc.status != OcrDocumentStatus::ReviewPending && doc.status != OcrDocumentStatus::Extracted {
            return Err(anyhow!("Document must be in review_pending or extracted status"));
        }

        for override_field in field_overrides {
            sqlx::query(
                r#"UPDATE ocr_extraction_fields SET field_value = $1, is_edited = true, updated_at = NOW()
                   WHERE document_id = $2 AND field_name = $3"#,
            )
            .bind(&override_field.field_value)
            .bind(document_id)
            .bind(&override_field.field_name)
            .execute(&self.pool)
            .await?;

            let exists: bool = sqlx::query_scalar(
                "SELECT EXISTS(SELECT 1 FROM ocr_extraction_fields WHERE document_id = $1 AND field_name = $2)",
            )
            .bind(document_id)
            .bind(&override_field.field_name)
            .fetch_one(&self.pool)
            .await?;

            if !exists {
                let field_id = Uuid::now_v7();
                sqlx::query(
                    r#"INSERT INTO ocr_extraction_fields (id, document_id, field_name, field_value, confidence, is_edited, created_at, updated_at)
                       VALUES ($1, $2, $3, $4, 1.0, true, NOW(), NOW())"#,
                )
                .bind(field_id)
                .bind(document_id)
                .bind(&override_field.field_name)
                .bind(&override_field.field_value)
                .execute(&self.pool)
                .await?;
            }
        }

        sqlx::query(
            "UPDATE ocr_documents SET reviewed_by = $1, reviewed_at = NOW(), updated_at = NOW() WHERE id = $2",
        )
        .bind(reviewer_id)
        .bind(document_id)
        .execute(&self.pool)
        .await?;

        self.get_document(document_id).await
    }

    pub async fn approve_document(
        &self,
        document_id: Uuid,
        approver_id: Uuid,
    ) -> Result<OcrDocument> {
        let doc = self.get_document(document_id).await?;

        if doc.status != OcrDocumentStatus::ReviewPending {
            return Err(anyhow!("Document must be in review_pending status to approve"));
        }

        sqlx::query(
            "UPDATE ocr_documents SET status = 'approved', reviewed_by = $1, reviewed_at = NOW(), updated_at = NOW() WHERE id = $2",
        )
        .bind(approver_id)
        .bind(document_id)
        .execute(&self.pool)
        .await?;

        self.get_document(document_id).await
    }

    pub async fn reject_document(
        &self,
        document_id: Uuid,
        reviewer_id: Uuid,
        reason: Option<String>,
    ) -> Result<OcrDocument> {
        let doc = self.get_document(document_id).await?;

        if doc.status != OcrDocumentStatus::ReviewPending {
            return Err(anyhow!("Document must be in review_pending status to reject"));
        }

        sqlx::query(
            r#"UPDATE ocr_documents SET status = 'rejected', reviewed_by = $1, reviewed_at = NOW(), ocr_raw_text = COALESCE($2, ocr_raw_text), updated_at = NOW() WHERE id = $3"#,
        )
        .bind(reviewer_id)
        .bind(reason)
        .bind(document_id)
        .execute(&self.pool)
        .await?;

        self.get_document(document_id).await
    }

    pub async fn create_journal_from_document(&self, document_id: Uuid, created_by: Uuid) -> Result<Uuid> {
        let doc = self.get_document(document_id).await?;

        if doc.status != OcrDocumentStatus::Approved {
            return Err(anyhow!("Document must be approved before creating journal"));
        }

        let fields = sqlx::query_as::<_, OcrExtractionField>(
            "SELECT * FROM ocr_extraction_fields WHERE document_id = $1",
        )
        .bind(document_id)
        .fetch_all(&self.pool)
        .await?;

        let mut account_code = String::new();
        let mut amount = bigdecimal::BigDecimal::from(0);
        let mut description = String::new();
        let mut date = String::new();

        for field in &fields {
            match field.field_name.as_str() {
                "account_code" => account_code = field.field_value.clone().unwrap_or_default(),
                "amount" => {
                    amount = field.field_value.as_ref()
                        .and_then(|v| v.parse::<f64>().ok())
                        .map(BigDecimal::from_f64)
                        .flatten()
                        .unwrap_or(BigDecimal::from(0));
                }
                "description" => description = field.field_value.clone().unwrap_or_default(),
                "date" => date = field.field_value.clone().unwrap_or_default(),
                _ => {}
            }
        }

        let journal_id = Uuid::now_v7();
        let entry_number = format!("OCR-{}", document_id);
        let narration = if description.is_empty() {
            format!("From OCR document: {}", doc.file_name)
        } else {
            description
        };

        let account_id: Option<Uuid> = if !account_code.is_empty() {
            sqlx::query_scalar(
                "SELECT id FROM accounts WHERE company_id = $1 AND code = $2 AND is_active = true",
            )
            .bind(doc.company_id)
            .bind(&account_code)
            .fetch_optional(&self.pool)
            .await?
        } else {
            None
        };

        let debit_account_id = account_id.unwrap_or(Uuid::nil());
        let credit_account_id: Uuid = sqlx::query_scalar(
            "SELECT id FROM accounts WHERE company_id = $1 AND code = '2101' AND is_active = true LIMIT 1",
        )
        .bind(doc.company_id)
        .fetch_optional(&self.pool)
        .await?
        .unwrap_or(Uuid::nil());

        sqlx::query(
            r#"INSERT INTO journal_headers (id, company_id, branch_id, entry_number, narration, status, journal_type, source_module, source_document_id, total_debit, total_credit, currency_code, posted_at, posted_by, created_by, created_at, updated_at)
               VALUES ($1, $2, $3, $4, $5, 'posted', 'auto', 'ocr', $6, $7, $7, 'NGN', NOW(), $8, $8, NOW(), NOW())"#,
        )
        .bind(journal_id)
        .bind(doc.company_id)
        .bind(doc.branch_id)
        .bind(&entry_number)
        .bind(&narration)
        .bind(document_id)
        .bind(&amount)
        .bind(created_by)
        .execute(&self.pool)
        .await?;

        sqlx::query(
            r#"INSERT INTO journal_lines (id, journal_header_id, account_id, line_number, narration, debit, credit, currency_code, created_at)
               VALUES ($1, $2, $3, 1, $4, $5, 0, 'NGN', NOW())"#,
        )
        .bind(Uuid::now_v7())
        .bind(journal_id)
        .bind(debit_account_id)
        .bind(&narration)
        .bind(&amount)
        .execute(&self.pool)
        .await?;

        sqlx::query(
            r#"INSERT INTO journal_lines (id, journal_header_id, account_id, line_number, narration, debit, credit, currency_code, created_at)
               VALUES ($1, $2, $3, 2, $4, 0, $5, 'NGN', NOW())"#,
        )
        .bind(Uuid::now_v7())
        .bind(journal_id)
        .bind(credit_account_id)
        .bind(&narration)
        .bind(&amount)
        .execute(&self.pool)
        .await?;

        sqlx::query(
            "UPDATE ocr_documents SET status = 'journal_created', journal_header_id = $1, updated_at = NOW() WHERE id = $2",
        )
        .bind(journal_id)
        .bind(document_id)
        .execute(&self.pool)
        .await?;

        Ok(journal_id)
    }

    fn simulate_ocr_extraction(&self, document_type: &DocumentType) -> SimulatedOcrResult {
        match document_type {
            DocumentType::Invoice => SimulatedOcrResult {
                raw_text: "INVOICE\nAmount: 0\nDate: 2024-01-01\nVendor: Unknown".to_string(),
                confidence: 0.85,
                fields: vec![
                    SimulatedField { field_name: "account_code".to_string(), field_value: Some("5101".to_string()), confidence: 0.75 },
                    SimulatedField { field_name: "amount".to_string(), field_value: Some("0".to_string()), confidence: 0.90 },
                    SimulatedField { field_name: "description".to_string(), field_value: Some("Invoice from OCR".to_string()), confidence: 0.80 },
                    SimulatedField { field_name: "date".to_string(), field_value: Some("2024-01-01".to_string()), confidence: 0.85 },
                ],
            },
            DocumentType::Receipt => SimulatedOcrResult {
                raw_text: "RECEIPT\nAmount: 0\nDate: 2024-01-01".to_string(),
                confidence: 0.80,
                fields: vec![
                    SimulatedField { field_name: "amount".to_string(), field_value: Some("0".to_string()), confidence: 0.85 },
                    SimulatedField { field_name: "date".to_string(), field_value: Some("2024-01-01".to_string()), confidence: 0.80 },
                ],
            },
            DocumentType::BankStatement => SimulatedOcrResult {
                raw_text: "BANK STATEMENT\nBalance: 0".to_string(),
                confidence: 0.70,
                fields: vec![
                    SimulatedField { field_name: "balance".to_string(), field_value: Some("0".to_string()), confidence: 0.70 },
                ],
            },
            _ => SimulatedOcrResult {
                raw_text: "DOCUMENT\nNo structured data".to_string(),
                confidence: 0.50,
                fields: vec![],
            },
        }
    }
}

struct SimulatedOcrResult {
    raw_text: String,
    confidence: f64,
    fields: Vec<SimulatedField>,
}

struct SimulatedField {
    field_name: String,
    field_value: Option<String>,
    confidence: f64,
}
