// Author: Quadri Atharu
use bigdecimal::BigDecimal;
use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type, PartialEq)]
#[sqlx(type_name = "employment_type", rename_all = "snake_case")]
pub enum EmploymentType {
    FullTime,
    PartTime,
    Contract,
    Probation,
}

impl std::fmt::Display for EmploymentType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EmploymentType::FullTime => write!(f, "full_time"),
            EmploymentType::PartTime => write!(f, "part_time"),
            EmploymentType::Contract => write!(f, "contract"),
            EmploymentType::Probation => write!(f, "probation"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type, PartialEq)]
#[sqlx(type_name = "payroll_run_status", rename_all = "snake_case")]
pub enum PayrollRunStatus {
    Draft,
    Processed,
    Posted,
    Reversed,
}

impl std::fmt::Display for PayrollRunStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PayrollRunStatus::Draft => write!(f, "draft"),
            PayrollRunStatus::Processed => write!(f, "processed"),
            PayrollRunStatus::Posted => write!(f, "posted"),
            PayrollRunStatus::Reversed => write!(f, "reversed"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Employee {
    pub id: Uuid,
    pub company_id: Uuid,
    pub branch_id: Option<Uuid>,
    pub department_id: Option<Uuid>,
    pub employee_number: String,
    pub first_name: String,
    pub last_name: String,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub hire_date: chrono::NaiveDate,
    pub termination_date: Option<chrono::NaiveDate>,
    pub employment_type: EmploymentType,
    pub designation: Option<String>,
    pub grade_level: Option<String>,
    pub salary_amount: BigDecimal,
    pub currency_code: String,
    pub tax_id: Option<String>,
    pub pension_provider: Option<String>,
    pub bank_name: Option<String>,
    pub bank_account_number: Option<String>,
    pub is_active: bool,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct PayrollRun {
    pub id: Uuid,
    pub company_id: Uuid,
    pub fiscal_year_id: Uuid,
    pub period_id: Uuid,
    pub run_date: chrono::NaiveDate,
    pub status: PayrollRunStatus,
    pub total_gross: BigDecimal,
    pub total_deductions: BigDecimal,
    pub total_net: BigDecimal,
    pub total_employer_contributions: BigDecimal,
    pub processed_by: Option<Uuid>,
    pub processed_at: Option<NaiveDateTime>,
    pub created_at: NaiveDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Payslip {
    pub id: Uuid,
    pub payroll_run_id: Uuid,
    pub employee_id: Uuid,
    pub basic_salary: BigDecimal,
    pub housing_allowance: BigDecimal,
    pub transport_allowance: BigDecimal,
    pub meal_allowance: BigDecimal,
    pub utility_allowance: BigDecimal,
    pub other_allowances: BigDecimal,
    pub total_gross: BigDecimal,
    pub paye: BigDecimal,
    pub pension_employee: BigDecimal,
    pub pension_employer: BigDecimal,
    pub nhf: BigDecimal,
    pub nsitf: BigDecimal,
    pub itf_levy: BigDecimal,
    pub other_deductions: BigDecimal,
    pub total_deductions: BigDecimal,
    pub net_pay: BigDecimal,
    pub currency_code: String,
    pub created_at: NaiveDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PayrollItem {
    Earning {
        name: String,
        amount: BigDecimal,
        is_taxable: bool,
    },
    Deduction {
        name: String,
        amount: BigDecimal,
        is_statutory: bool,
    },
}
