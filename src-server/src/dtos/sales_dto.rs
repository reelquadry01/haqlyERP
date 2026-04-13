// Author: Quadri Atharu
use bigdecimal::BigDecimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct CreateCustomerRequest {
    pub company_id: Uuid,
    #[validate(length(min = 1, message = "customer name is required"))]
    pub name: String,
    pub code: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub tax_identification_number: Option<String>,
    pub rc_number: Option<String>,
    pub contact_person: Option<String>,
    pub credit_limit: Option<BigDecimal>,
    pub currency_code: Option<String>,
    pub branch_id: Option<Uuid>,
    pub address: Option<CustomerAddressDto>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomerAddressDto {
    pub address_type: String,
    pub line1: String,
    pub line2: Option<String>,
    pub city: String,
    pub state: Option<String>,
    pub country: String,
    pub postal_code: Option<String>,
    pub is_default: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct CreateInvoiceRequest {
    pub company_id: Uuid,
    pub branch_id: Option<Uuid>,
    pub customer_id: Uuid,
    pub invoice_date: String,
    pub due_date: String,
    pub currency_code: Option<String>,
    pub narration: Option<String>,
    #[validate(length(min = 1, message = "invoice must have at least 1 line item"))]
    pub items: Vec<InvoiceItemDto>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InvoiceItemDto {
    pub product_id: Option<Uuid>,
    pub description: String,
    pub quantity: BigDecimal,
    pub unit_price: BigDecimal,
    pub discount_percent: Option<BigDecimal>,
    pub tax_rate: Option<BigDecimal>,
    pub cost_center_id: Option<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct CreateReceiptRequest {
    pub company_id: Uuid,
    pub branch_id: Option<Uuid>,
    pub customer_id: Uuid,
    pub invoice_id: Option<Uuid>,
    pub receipt_date: String,
    pub amount: BigDecimal,
    pub currency_code: Option<String>,
    pub payment_method: String,
    pub reference: Option<String>,
    pub bank_account_id: Option<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct CreateProformaRequest {
    pub company_id: Uuid,
    pub customer_id: Uuid,
    pub proforma_date: String,
    pub valid_until: String,
    pub currency_code: Option<String>,
    pub narration: Option<String>,
    #[validate(length(min = 1, message = "proforma must have at least 1 line item"))]
    pub items: Vec<InvoiceItemDto>,
}
