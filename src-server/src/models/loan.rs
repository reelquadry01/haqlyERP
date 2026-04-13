// Author: Quadri Atharu
use bigdecimal::BigDecimal;
use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type, PartialEq)]
#[sqlx(type_name = "loan_type", rename_all = "snake_case")]
pub enum LoanType {
    TermLoan,
    Overdraft,
    Mortgage,
    EquipmentLoan,
    WorkingCapital,
}

impl std::fmt::Display for LoanType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LoanType::TermLoan => write!(f, "term_loan"),
            LoanType::Overdraft => write!(f, "overdraft"),
            LoanType::Mortgage => write!(f, "mortgage"),
            LoanType::EquipmentLoan => write!(f, "equipment_loan"),
            LoanType::WorkingCapital => write!(f, "working_capital"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Loan {
    pub id: Uuid,
    pub company_id: Uuid,
    pub branch_id: Option<Uuid>,
    pub loan_type: LoanType,
    pub lender_name: String,
    pub principal_amount: BigDecimal,
    pub interest_rate: BigDecimal,
    pub tenure_months: i32,
    pub start_date: chrono::NaiveDate,
    pub maturity_date: chrono::NaiveDate,
    pub outstanding_principal: BigDecimal,
    pub outstanding_interest: BigDecimal,
    pub currency_code: String,
    pub status: String,
    pub loan_account_id: Uuid,
    pub interest_account_id: Uuid,
    pub bank_account_id: Option<Uuid>,
    pub narration: Option<String>,
    pub created_by: Uuid,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct LoanDisbursement {
    pub id: Uuid,
    pub loan_id: Uuid,
    pub amount: BigDecimal,
    pub disbursement_date: chrono::NaiveDate,
    pub reference: Option<String>,
    pub journal_header_id: Option<Uuid>,
    pub created_by: Uuid,
    pub created_at: NaiveDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct LoanRepayment {
    pub id: Uuid,
    pub loan_id: Uuid,
    pub principal_amount: BigDecimal,
    pub interest_amount: BigDecimal,
    pub repayment_date: chrono::NaiveDate,
    pub reference: Option<String>,
    pub journal_header_id: Option<Uuid>,
    pub created_by: Uuid,
    pub created_at: NaiveDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct AmortizationScheduleEntry {
    pub id: Uuid,
    pub loan_id: Uuid,
    pub period_number: i32,
    pub payment_date: chrono::NaiveDate,
    pub opening_balance: BigDecimal,
    pub principal_payment: BigDecimal,
    pub interest_payment: BigDecimal,
    pub total_payment: BigDecimal,
    pub closing_balance: BigDecimal,
    pub is_paid: bool,
    pub created_at: NaiveDateTime,
}
