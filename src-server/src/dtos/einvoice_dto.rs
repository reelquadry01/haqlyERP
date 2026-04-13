// Author: Quadri Atharu
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct SaveProfileRequest {
    pub company_id: Uuid,
    #[validate(length(min = 1, message = "business ID is required"))]
    pub business_id: String,
    #[validate(length(min = 1, message = "business name is required"))]
    pub business_name: String,
    #[validate(length(min = 1, message = "tax ID is required"))]
    pub tax_id: String,
    pub integration_type: String,
    pub api_base_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct SaveCredentialsRequest {
    pub profile_id: Uuid,
    pub company_id: Uuid,
    #[validate(length(min = 1, message = "client ID is required"))]
    pub client_id: String,
    #[validate(length(min = 1, message = "client secret is required"))]
    pub client_secret: String,
    pub certificate_path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct SubmitEInvoiceRequest {
    pub company_id: Uuid,
    pub invoice_id: Uuid,
    pub category: String,
    pub seller_name: String,
    pub seller_tax_id: Option<String>,
    pub seller_business_id: Option<String>,
    pub seller_address_line1: String,
    pub seller_city: String,
    pub seller_state: Option<String>,
    pub seller_country: String,
    pub buyer_name: String,
    pub buyer_tax_id: Option<String>,
    pub buyer_address_line1: String,
    pub buyer_city: String,
    pub buyer_state: Option<String>,
    pub buyer_country: String,
    pub invoice_number: String,
    pub invoice_date: String,
    pub currency: String,
    pub lines: Vec<EInvoiceLineDto>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EInvoiceLineDto {
    pub line_number: i32,
    pub description: String,
    pub quantity: bigdecimal::BigDecimal,
    pub unit_price: bigdecimal::BigDecimal,
    pub tax_rate: bigdecimal::BigDecimal,
    pub tax_amount: bigdecimal::BigDecimal,
    pub line_total: bigdecimal::BigDecimal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EInvoiceValidationResult {
    pub is_valid: bool,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EInvoiceReadinessCheck {
    pub profile_configured: bool,
    pub credentials_valid: bool,
    pub api_reachable: bool,
    pub ready: bool,
    pub issues: Vec<String>,
}
