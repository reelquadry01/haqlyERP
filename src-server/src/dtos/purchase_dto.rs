// Author: Quadri Atharu
use bigdecimal::BigDecimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct CreateSupplierRequest {
    pub company_id: Uuid,
    #[validate(length(min = 1, message = "supplier name is required"))]
    pub name: String,
    pub code: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub tax_identification_number: Option<String>,
    pub rc_number: Option<String>,
    pub contact_person: Option<String>,
    pub payment_terms: Option<String>,
    pub bank_name: Option<String>,
    pub bank_account_number: Option<String>,
    pub bank_sort_code: Option<String>,
    pub currency_code: Option<String>,
    pub branch_id: Option<Uuid>,
    pub address: Option<SupplierAddressDto>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SupplierAddressDto {
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
pub struct CreateBillRequest {
    pub company_id: Uuid,
    pub branch_id: Option<Uuid>,
    pub supplier_id: Uuid,
    pub bill_date: String,
    pub due_date: String,
    pub currency_code: Option<String>,
    pub narration: Option<String>,
    #[validate(length(min = 1, message = "bill must have at least 1 line item"))]
    pub items: Vec<BillItemDto>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BillItemDto {
    pub product_id: Option<Uuid>,
    pub description: String,
    pub quantity: BigDecimal,
    pub unit_price: BigDecimal,
    pub discount_percent: Option<BigDecimal>,
    pub tax_rate: Option<BigDecimal>,
    pub wht_rate: Option<BigDecimal>,
    pub cost_center_id: Option<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct CreatePaymentRequest {
    pub company_id: Uuid,
    pub branch_id: Option<Uuid>,
    pub supplier_id: Uuid,
    pub bill_id: Option<Uuid>,
    pub payment_date: String,
    pub amount: BigDecimal,
    pub currency_code: Option<String>,
    pub payment_method: String,
    pub reference: Option<String>,
    pub bank_account_id: Option<Uuid>,
}
