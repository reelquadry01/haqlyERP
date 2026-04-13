// Author: Quadri Atharu
use bigdecimal::BigDecimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct CreateTaxConfigRequest {
    pub company_id: Uuid,
    pub tax_type: String,
    #[validate(length(min = 1, message = "tax name is required"))]
    pub name: String,
    pub rate: BigDecimal,
    pub effective_from: String,
    pub effective_to: Option<String>,
    pub account_id: Option<Uuid>,
    pub wht_category: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaxComputationRequest {
    pub company_id: Uuid,
    pub tax_type: String,
    pub base_amount: BigDecimal,
    pub currency: String,
    pub category: Option<String>,
    pub annual_revenue: Option<BigDecimal>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaxComputationResult {
    pub tax_type: String,
    pub base_amount: BigDecimal,
    pub rate: BigDecimal,
    pub tax_amount: BigDecimal,
    pub currency: String,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaxReturnRequest {
    pub company_id: Uuid,
    pub tax_type: String,
    pub period_id: Uuid,
    pub from_date: String,
    pub to_date: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WhtCategoryRequest {
    pub category: String,
    pub amount: BigDecimal,
    pub currency: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaxDashboard {
    pub vat_payable: BigDecimal,
    pub vat_receivable: BigDecimal,
    pub wht_deducted: BigDecimal,
    pub cit_estimate: BigDecimal,
    pub edu_tax_estimate: BigDecimal,
    pub pending_returns: Vec<String>,
    pub upcoming_deadlines: Vec<String>,
}
