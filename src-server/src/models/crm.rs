// Author: Quadri Atharu
use bigdecimal::BigDecimal;
use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type, PartialEq)]
#[sqlx(type_name = "contact_status", rename_all = "snake_case")]
pub enum ContactStatus {
    Lead,
    Qualified,
    Opportunity,
    Customer,
    Churned,
}

impl std::fmt::Display for ContactStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ContactStatus::Lead => write!(f, "lead"),
            ContactStatus::Qualified => write!(f, "qualified"),
            ContactStatus::Opportunity => write!(f, "opportunity"),
            ContactStatus::Customer => write!(f, "customer"),
            ContactStatus::Churned => write!(f, "churned"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type, PartialEq)]
#[sqlx(type_name = "deal_stage", rename_all = "snake_case")]
pub enum DealStage {
    Prospecting,
    Qualification,
    Proposal,
    Negotiation,
    ClosedWon,
    ClosedLost,
}

impl std::fmt::Display for DealStage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DealStage::Prospecting => write!(f, "prospecting"),
            DealStage::Qualification => write!(f, "qualification"),
            DealStage::Proposal => write!(f, "proposal"),
            DealStage::Negotiation => write!(f, "negotiation"),
            DealStage::ClosedWon => write!(f, "closed_won"),
            DealStage::ClosedLost => write!(f, "closed_lost"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type, PartialEq)]
#[sqlx(type_name = "activity_type", rename_all = "snake_case")]
pub enum ActivityType {
    Call,
    Email,
    Meeting,
    Task,
    Note,
}

impl std::fmt::Display for ActivityType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ActivityType::Call => write!(f, "call"),
            ActivityType::Email => write!(f, "email"),
            ActivityType::Meeting => write!(f, "meeting"),
            ActivityType::Task => write!(f, "task"),
            ActivityType::Note => write!(f, "note"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type, PartialEq)]
#[sqlx(type_name = "activity_status", rename_all = "snake_case")]
pub enum ActivityStatus {
    Pending,
    Completed,
    Cancelled,
}

impl std::fmt::Display for ActivityStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ActivityStatus::Pending => write!(f, "pending"),
            ActivityStatus::Completed => write!(f, "completed"),
            ActivityStatus::Cancelled => write!(f, "cancelled"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Contact {
    pub id: Uuid,
    pub company_id: Uuid,
    pub first_name: String,
    pub last_name: String,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub organization: Option<String>,
    pub title: Option<String>,
    pub lead_source: Option<String>,
    pub status: ContactStatus,
    pub assigned_to: Option<Uuid>,
    pub notes: Option<String>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Deal {
    pub id: Uuid,
    pub company_id: Uuid,
    pub contact_id: Uuid,
    pub title: String,
    pub value: BigDecimal,
    pub stage: DealStage,
    pub probability: i32,
    pub expected_close_date: Option<chrono::NaiveDate>,
    pub assigned_to: Option<Uuid>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Activity {
    pub id: Uuid,
    pub company_id: Uuid,
    pub contact_id: Option<Uuid>,
    pub deal_id: Option<Uuid>,
    pub activity_type: ActivityType,
    pub subject: String,
    pub description: Option<String>,
    pub due_date: Option<chrono::NaiveDate>,
    pub status: ActivityStatus,
    pub assigned_to: Option<Uuid>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}
