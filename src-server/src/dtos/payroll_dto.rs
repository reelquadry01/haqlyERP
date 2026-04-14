// Author: Quadri Atharu
use bigdecimal::BigDecimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

use crate::models::payroll::EmploymentType;

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct CreateEmployeeRequest {
    pub company_id: Uuid,
    pub branch_id: Option<Uuid>,
    pub department_id: Option<Uuid>,
    #[validate(length(min = 1, message = "employee number is required"))]
    pub employee_number: String,
    #[validate(length(min = 1, message = "first name is required"))]
    pub first_name: String,
    #[validate(length(min = 1, message = "last name is required"))]
    pub last_name: String,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub hire_date: String,
    pub employment_type: EmploymentType,
    pub designation: Option<String>,
    pub grade_level: Option<String>,
    pub salary_amount: BigDecimal,
    pub currency_code: Option<String>,
    pub tax_id: Option<String>,
    pub pension_provider: Option<String>,
    pub bank_name: Option<String>,
    pub bank_account_number: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct UpdateEmployeeRequest {
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub employment_type: Option<EmploymentType>,
    pub designation: Option<String>,
    pub grade_level: Option<String>,
    pub salary_amount: Option<BigDecimal>,
    pub department_id: Option<Uuid>,
    pub branch_id: Option<Uuid>,
    pub tax_id: Option<String>,
    pub pension_provider: Option<String>,
    pub bank_name: Option<String>,
    pub bank_account_number: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct RunPayrollRequest {
    pub company_id: Uuid,
    pub fiscal_year_id: Uuid,
    pub period_id: Uuid,
    pub run_date: String,
    pub employee_ids: Option<Vec<Uuid>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct ProcessPayrollRequest {
    pub payroll_run_id: Uuid,
    pub processed_by: Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct TerminateEmployeeRequest {
    pub termination_date: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PayslipDetail {
    pub employee_id: Uuid,
    pub employee_name: String,
    pub employee_number: String,
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
}
