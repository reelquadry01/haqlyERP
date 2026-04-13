// Author: Quadri Atharu
use bigdecimal::BigDecimal;
use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type, PartialEq)]
#[sqlx(type_name = "asset_status", rename_all = "snake_case")]
pub enum AssetStatus {
    Draft,
    Active,
    Disposed,
    WrittenOff,
}

impl std::fmt::Display for AssetStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AssetStatus::Draft => write!(f, "draft"),
            AssetStatus::Active => write!(f, "active"),
            AssetStatus::Disposed => write!(f, "disposed"),
            AssetStatus::WrittenOff => write!(f, "written_off"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type, PartialEq)]
#[sqlx(type_name = "depreciation_method", rename_all = "snake_case")]
pub enum DepreciationMethod {
    StraightLine,
    DecliningBalance,
    SumOfYearsDigits,
}

impl std::fmt::Display for DepreciationMethod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DepreciationMethod::StraightLine => write!(f, "straight_line"),
            DepreciationMethod::DecliningBalance => write!(f, "declining_balance"),
            DepreciationMethod::SumOfYearsDigits => write!(f, "sum_of_years_digits"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct AssetCategory {
    pub id: Uuid,
    pub company_id: Uuid,
    pub name: String,
    pub depreciation_method: DepreciationMethod,
    pub useful_life_years: i32,
    pub residual_value_percent: BigDecimal,
    pub depreciation_rate: BigDecimal,
    pub asset_account_id: Uuid,
    pub accumulated_dep_account_id: Uuid,
    pub depreciation_expense_account_id: Uuid,
    pub disposal_account_id: Option<Uuid>,
    pub capital_allowance_category: Option<String>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct FixedAsset {
    pub id: Uuid,
    pub company_id: Uuid,
    pub branch_id: Option<Uuid>,
    pub category_id: Uuid,
    pub asset_code: String,
    pub name: String,
    pub description: Option<String>,
    pub acquisition_date: chrono::NaiveDate,
    pub acquisition_cost: BigDecimal,
    pub residual_value: BigDecimal,
    pub useful_life_years: i32,
    pub depreciation_method: DepreciationMethod,
    pub depreciation_rate: BigDecimal,
    pub accumulated_depreciation: BigDecimal,
    pub net_book_value: BigDecimal,
    pub status: AssetStatus,
    pub location: Option<String>,
    pub custodian: Option<String>,
    pub serial_number: Option<String>,
    pub asset_account_id: Uuid,
    pub accumulated_dep_account_id: Uuid,
    pub depreciation_expense_account_id: Uuid,
    pub disposal_account_id: Option<Uuid>,
    pub journal_header_id: Option<Uuid>,
    pub created_by: Uuid,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct DepreciationSchedule {
    pub id: Uuid,
    pub asset_id: Uuid,
    pub period_id: Uuid,
    pub fiscal_year_id: Uuid,
    pub depreciation_date: chrono::NaiveDate,
    pub opening_book_value: BigDecimal,
    pub depreciation_amount: BigDecimal,
    pub closing_book_value: BigDecimal,
    pub is_posted: bool,
    pub journal_header_id: Option<Uuid>,
    pub created_at: NaiveDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct DepreciationRun {
    pub id: Uuid,
    pub company_id: Uuid,
    pub fiscal_year_id: Uuid,
    pub period_id: Uuid,
    pub run_date: chrono::NaiveDate,
    pub total_depreciation: BigDecimal,
    pub asset_count: i32,
    pub status: String,
    pub journal_header_id: Option<Uuid>,
    pub created_by: Uuid,
    pub created_at: NaiveDateTime,
}
