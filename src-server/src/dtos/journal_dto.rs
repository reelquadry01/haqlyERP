// Author: Quadri Atharu
use bigdecimal::BigDecimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct CreateJournalRequest {
    pub company_id: Uuid,
    pub branch_id: Option<Uuid>,
    pub narration: String,
    pub reference: Option<String>,
    pub journal_type: Option<String>,
    pub currency_code: Option<String>,
    #[validate(length(min = 2, message = "journal must have at least 2 lines"))]
    pub lines: Vec<JournalLineDto>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct UpdateJournalRequest {
    pub narration: Option<String>,
    pub reference: Option<String>,
    pub lines: Option<Vec<JournalLineDto>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct JournalLineDto {
    pub account_id: Uuid,
    pub narration: Option<String>,
    pub debit: Option<BigDecimal>,
    pub credit: Option<BigDecimal>,
    pub cost_center_id: Option<Uuid>,
    pub project_id: Option<Uuid>,
    pub department_id: Option<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct SubmitJournalRequest {
    pub journal_id: Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct ApproveJournalRequest {
    pub journal_id: Uuid,
    pub comment: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct ReverseJournalRequest {
    pub journal_id: Uuid,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JournalFilterParams {
    pub company_id: Option<Uuid>,
    pub status: Option<String>,
    pub date_from: Option<String>,
    pub date_to: Option<String>,
    pub branch_id: Option<Uuid>,
    pub page: Option<i64>,
    pub limit: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct CreateTemplateRequest {
    pub company_id: Uuid,
    pub name: String,
    pub narration_template: String,
    pub journal_type: Option<String>,
    pub recurrence: Option<String>,
    pub lines: Vec<JournalLineDto>,
}
