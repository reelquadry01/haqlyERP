// Author: Quadri Atharu
use bigdecimal::BigDecimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

use crate::models::crm::{ActivityType, ContactStatus, DealStage};

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct CreateContactRequest {
    pub company_id: Uuid,
    #[validate(length(min = 1, message = "first name is required"))]
    pub first_name: String,
    #[validate(length(min = 1, message = "last name is required"))]
    pub last_name: String,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub organization: Option<String>,
    pub title: Option<String>,
    pub lead_source: Option<String>,
    pub status: Option<ContactStatus>,
    pub assigned_to: Option<Uuid>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct CreateDealRequest {
    pub company_id: Uuid,
    pub contact_id: Uuid,
    #[validate(length(min = 1, message = "title is required"))]
    pub title: String,
    pub value: BigDecimal,
    pub stage: Option<DealStage>,
    pub probability: Option<i32>,
    pub expected_close_date: Option<String>,
    pub assigned_to: Option<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct UpdateDealStageRequest {
    pub deal_id: Uuid,
    pub stage: DealStage,
    pub probability: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct CreateActivityRequest {
    pub company_id: Uuid,
    pub contact_id: Option<Uuid>,
    pub deal_id: Option<Uuid>,
    pub activity_type: ActivityType,
    #[validate(length(min = 1, message = "subject is required"))]
    pub subject: String,
    pub description: Option<String>,
    pub due_date: Option<String>,
    pub assigned_to: Option<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineSummary {
    pub stage: DealStage,
    pub count: i64,
    pub total_value: BigDecimal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContactListParams {
    pub company_id: Option<Uuid>,
    pub status: Option<ContactStatus>,
    pub assigned_to: Option<Uuid>,
    pub page: Option<i64>,
    pub limit: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DealListParams {
    pub company_id: Option<Uuid>,
    pub stage: Option<DealStage>,
    pub assigned_to: Option<Uuid>,
    pub page: Option<i64>,
    pub limit: Option<i64>,
}
