// Author: Quadri Atharu
use bigdecimal::BigDecimal;
use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type, PartialEq)]
#[sqlx(type_name = "account_type", rename_all = "snake_case")]
pub enum AccountType {
    Asset,
    Liability,
    Equity,
    Revenue,
    Expense,
}

impl std::fmt::Display for AccountType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AccountType::Asset => write!(f, "asset"),
            AccountType::Liability => write!(f, "liability"),
            AccountType::Equity => write!(f, "equity"),
            AccountType::Revenue => write!(f, "revenue"),
            AccountType::Expense => write!(f, "expense"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Account {
    pub id: Uuid,
    pub company_id: Uuid,
    pub code: String,
    pub name: String,
    pub account_type: AccountType,
    pub sub_type: Option<String>,
    pub parent_id: Option<Uuid>,
    pub is_control_account: bool,
    pub is_active: bool,
    pub allowed_posting: bool,
    pub currency_code: String,
    pub balance: BigDecimal,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct FiscalYear {
    pub id: Uuid,
    pub company_id: Uuid,
    pub name: String,
    pub start_date: chrono::NaiveDate,
    pub end_date: chrono::NaiveDate,
    pub is_closed: bool,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct AccountingPeriod {
    pub id: Uuid,
    pub fiscal_year_id: Uuid,
    pub company_id: Uuid,
    pub name: String,
    pub period_number: i32,
    pub start_date: chrono::NaiveDate,
    pub end_date: chrono::NaiveDate,
    pub status: PeriodStatus,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type, PartialEq)]
#[sqlx(type_name = "period_status", rename_all = "snake_case")]
pub enum PeriodStatus {
    Open,
    Closed,
    Locked,
}

impl std::fmt::Display for PeriodStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PeriodStatus::Open => write!(f, "open"),
            PeriodStatus::Closed => write!(f, "closed"),
            PeriodStatus::Locked => write!(f, "locked"),
        }
    }
}
