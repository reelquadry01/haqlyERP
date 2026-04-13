// Author: Quadri Atharu
use bigdecimal::BigDecimal;
use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type, PartialEq)]
#[sqlx(type_name = "journal_status", rename_all = "snake_case")]
pub enum JournalStatus {
    Draft,
    Validated,
    Submitted,
    Approved,
    Posted,
    Reversed,
    Cancelled,
}

impl std::fmt::Display for JournalStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            JournalStatus::Draft => write!(f, "draft"),
            JournalStatus::Validated => write!(f, "validated"),
            JournalStatus::Submitted => write!(f, "submitted"),
            JournalStatus::Approved => write!(f, "approved"),
            JournalStatus::Posted => write!(f, "posted"),
            JournalStatus::Reversed => write!(f, "reversed"),
            JournalStatus::Cancelled => write!(f, "cancelled"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct JournalHeader {
    pub id: Uuid,
    pub company_id: Uuid,
    pub branch_id: Option<Uuid>,
    pub fiscal_year_id: Uuid,
    pub period_id: Uuid,
    pub entry_number: String,
    pub reference: Option<String>,
    pub narration: String,
    pub status: JournalStatus,
    pub journal_type: Option<String>,
    pub source_module: Option<String>,
    pub source_document_id: Option<Uuid>,
    pub source_document_number: Option<String>,
    pub reversal_of: Option<Uuid>,
    pub total_debit: BigDecimal,
    pub total_credit: BigDecimal,
    pub currency_code: String,
    pub posted_at: Option<NaiveDateTime>,
    pub posted_by: Option<Uuid>,
    pub created_by: Uuid,
    pub approved_by: Option<Uuid>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct JournalLine {
    pub id: Uuid,
    pub journal_header_id: Uuid,
    pub account_id: Uuid,
    pub line_number: i32,
    pub narration: Option<String>,
    pub debit: BigDecimal,
    pub credit: BigDecimal,
    pub currency_code: String,
    pub exchange_rate: Option<BigDecimal>,
    pub cost_center_id: Option<Uuid>,
    pub project_id: Option<Uuid>,
    pub department_id: Option<Uuid>,
    pub created_at: NaiveDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JournalHeaderWithLines {
    pub header: JournalHeader,
    pub lines: Vec<JournalLine>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct JournalTemplate {
    pub id: Uuid,
    pub company_id: Uuid,
    pub name: String,
    pub narration_template: String,
    pub journal_type: Option<String>,
    pub recurrence: Option<String>,
    pub is_active: bool,
    pub created_by: Uuid,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct JournalTemplateLine {
    pub id: Uuid,
    pub template_id: Uuid,
    pub account_id: Uuid,
    pub line_number: i32,
    pub narration_template: Option<String>,
    pub debit_expression: Option<String>,
    pub credit_expression: Option<String>,
    pub cost_center_id: Option<Uuid>,
    pub project_id: Option<Uuid>,
}
