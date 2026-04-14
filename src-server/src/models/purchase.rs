// Author: Quadri Atharu
use bigdecimal::BigDecimal;
use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type, PartialEq)]
#[sqlx(type_name = "bill_type", rename_all = "snake_case")]
pub enum BillType {
    Standard,
    DebitNote,
}

impl std::fmt::Display for BillType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BillType::Standard => write!(f, "standard"),
            BillType::DebitNote => write!(f, "debit_note"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Supplier {
    pub id: Uuid,
    pub company_id: Uuid,
    pub code: String,
    pub name: String,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub tax_identification_number: Option<String>,
    pub rc_number: Option<String>,
    pub contact_person: Option<String>,
    pub payment_terms: Option<String>,
    pub bank_name: Option<String>,
    pub bank_account_number: Option<String>,
    pub bank_sort_code: Option<String>,
    pub outstanding_balance: BigDecimal,
    pub currency_code: String,
    pub is_active: bool,
    pub branch_id: Option<Uuid>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct SupplierAddress {
    pub id: Uuid,
    pub supplier_id: Uuid,
    pub address_type: String,
    pub line1: String,
    pub line2: Option<String>,
    pub city: String,
    pub state: Option<String>,
    pub country: String,
    pub postal_code: Option<String>,
    pub is_default: bool,
    pub created_at: NaiveDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct PurchaseBill {
    pub id: Uuid,
    pub company_id: Uuid,
    pub branch_id: Option<Uuid>,
    pub supplier_id: Uuid,
    pub bill_number: String,
    pub bill_date: chrono::NaiveDate,
    pub due_date: chrono::NaiveDate,
    pub subtotal: BigDecimal,
    pub tax_amount: BigDecimal,
    pub withholding_amount: BigDecimal,
    pub total_amount: BigDecimal,
    pub amount_paid: BigDecimal,
    pub currency_code: String,
    pub status: String,
    pub bill_type: BillType,
    pub linked_bill_id: Option<Uuid>,
    pub narration: Option<String>,
    pub journal_header_id: Option<Uuid>,
    pub created_by: Uuid,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct PurchaseBillItem {
    pub id: Uuid,
    pub bill_id: Uuid,
    pub product_id: Option<Uuid>,
    pub line_number: i32,
    pub description: String,
    pub quantity: BigDecimal,
    pub unit_price: BigDecimal,
    pub discount_percent: Option<BigDecimal>,
    pub tax_rate: Option<BigDecimal>,
    pub tax_amount: BigDecimal,
    pub wht_rate: Option<BigDecimal>,
    pub wht_amount: BigDecimal,
    pub line_total: BigDecimal,
    pub cost_center_id: Option<Uuid>,
    pub created_at: NaiveDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct SupplierPayment {
    pub id: Uuid,
    pub company_id: Uuid,
    pub branch_id: Option<Uuid>,
    pub supplier_id: Uuid,
    pub payment_number: String,
    pub payment_date: chrono::NaiveDate,
    pub amount: BigDecimal,
    pub currency_code: String,
    pub payment_method: String,
    pub reference: Option<String>,
    pub bank_account_id: Option<Uuid>,
    pub bill_id: Option<Uuid>,
    pub voucher_id: Option<Uuid>,
    pub journal_header_id: Option<Uuid>,
    pub created_by: Uuid,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}
