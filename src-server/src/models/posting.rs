// Author: Quadri Atharu
use bigdecimal::BigDecimal;
use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct PostingRule {
    pub id: Uuid,
    pub company_id: Uuid,
    pub module: String,
    pub transaction_type: String,
    pub transaction_subtype: Option<String>,
    pub debit_account_id: Uuid,
    pub credit_account_id: Uuid,
    pub tax_account_id: Option<Uuid>,
    pub discount_account_id: Option<Uuid>,
    pub round_off_account_id: Option<Uuid>,
    pub branch_id: Option<Uuid>,
    pub department_id: Option<Uuid>,
    pub cost_center_id: Option<Uuid>,
    pub project_id: Option<Uuid>,
    pub effective_from: chrono::NaiveDate,
    pub effective_to: Option<chrono::NaiveDate>,
    pub is_active: bool,
    pub priority: i32,
    pub requires_explicit_rule: bool,
    pub narration_template: Option<String>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct PostingAudit {
    pub id: Uuid,
    pub company_id: Uuid,
    pub journal_header_id: Uuid,
    pub posting_rule_id: Uuid,
    pub source_module: String,
    pub source_document_id: Option<Uuid>,
    pub source_document_number: Option<String>,
    pub correlation_id: Option<String>,
    pub idempotency_key: Option<String>,
    pub status: String,
    pub error_message: Option<String>,
    pub posted_by: Option<Uuid>,
    pub posted_at: NaiveDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PostingContext {
    pub company_id: Uuid,
    pub source_module: String,
    pub source_document_id: Option<Uuid>,
    pub source_document_number: Option<String>,
    pub reference_id: Option<String>,
    pub customer_or_vendor: Option<String>,
    pub branch: Option<Uuid>,
    pub department: Option<Uuid>,
    pub cost_center: Option<Uuid>,
    pub project: Option<Uuid>,
    pub tax_code: Option<String>,
    pub currency: String,
    pub amount: BigDecimal,
    pub tax_amount: Option<BigDecimal>,
    pub discount_amount: Option<BigDecimal>,
    pub narration: Option<String>,
    pub correlation_id: Option<String>,
    pub idempotency_key: Option<String>,
    pub transaction_type: String,
    pub transaction_subtype: Option<String>,
    pub posted_by: Option<Uuid>,
    pub posting_date: chrono::NaiveDate,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewJournalLine {
    pub account_id: Uuid,
    pub narration: Option<String>,
    pub debit: BigDecimal,
    pub credit: BigDecimal,
    pub currency_code: String,
    pub exchange_rate: Option<BigDecimal>,
    pub cost_center_id: Option<Uuid>,
    pub project_id: Option<Uuid>,
    pub department_id: Option<Uuid>,
}
