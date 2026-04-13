// Author: Quadri Atharu
use bigdecimal::BigDecimal;
use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type, PartialEq)]
#[sqlx(type_name = "invoice_type", rename_all = "snake_case")]
pub enum InvoiceType {
    Standard,
    CreditNote,
    DebitNote,
    Proforma,
}

impl std::fmt::Display for InvoiceType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            InvoiceType::Standard => write!(f, "standard"),
            InvoiceType::CreditNote => write!(f, "credit_note"),
            InvoiceType::DebitNote => write!(f, "debit_note"),
            InvoiceType::Proforma => write!(f, "proforma"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type, PartialEq)]
#[sqlx(type_name = "payment_method", rename_all = "snake_case")]
pub enum PaymentMethod {
    Cash,
    BankTransfer,
    Cheque,
    Card,
    MobileMoney,
    Ussd,
}

impl std::fmt::Display for PaymentMethod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PaymentMethod::Cash => write!(f, "cash"),
            PaymentMethod::BankTransfer => write!(f, "bank_transfer"),
            PaymentMethod::Cheque => write!(f, "cheque"),
            PaymentMethod::Card => write!(f, "card"),
            PaymentMethod::MobileMoney => write!(f, "mobile_money"),
            PaymentMethod::Ussd => write!(f, "ussd"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Customer {
    pub id: Uuid,
    pub company_id: Uuid,
    pub code: String,
    pub name: String,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub tax_identification_number: Option<String>,
    pub rc_number: Option<String>,
    pub contact_person: Option<String>,
    pub credit_limit: Option<BigDecimal>,
    pub outstanding_balance: BigDecimal,
    pub currency_code: String,
    pub is_active: bool,
    pub branch_id: Option<Uuid>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct CustomerAddress {
    pub id: Uuid,
    pub customer_id: Uuid,
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
pub struct SalesInvoice {
    pub id: Uuid,
    pub company_id: Uuid,
    pub branch_id: Option<Uuid>,
    pub customer_id: Uuid,
    pub invoice_number: String,
    pub invoice_type: InvoiceType,
    pub invoice_date: chrono::NaiveDate,
    pub due_date: chrono::NaiveDate,
    pub subtotal: BigDecimal,
    pub tax_amount: BigDecimal,
    pub discount_amount: BigDecimal,
    pub total_amount: BigDecimal,
    pub amount_paid: BigDecimal,
    pub currency_code: String,
    pub status: String,
    pub narration: Option<String>,
    pub einvoice_irn: Option<String>,
    pub einvoice_status: Option<String>,
    pub journal_header_id: Option<Uuid>,
    pub created_by: Uuid,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct SalesInvoiceItem {
    pub id: Uuid,
    pub invoice_id: Uuid,
    pub product_id: Option<Uuid>,
    pub line_number: i32,
    pub description: String,
    pub quantity: BigDecimal,
    pub unit_price: BigDecimal,
    pub discount_percent: Option<BigDecimal>,
    pub tax_rate: Option<BigDecimal>,
    pub tax_amount: BigDecimal,
    pub line_total: BigDecimal,
    pub cost_center_id: Option<Uuid>,
    pub created_at: NaiveDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct CustomerReceipt {
    pub id: Uuid,
    pub company_id: Uuid,
    pub branch_id: Option<Uuid>,
    pub customer_id: Uuid,
    pub receipt_number: String,
    pub receipt_date: chrono::NaiveDate,
    pub amount: BigDecimal,
    pub currency_code: String,
    pub payment_method: PaymentMethod,
    pub reference: Option<String>,
    pub bank_account_id: Option<Uuid>,
    pub invoice_id: Option<Uuid>,
    pub journal_header_id: Option<Uuid>,
    pub created_by: Uuid,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ProformaInvoice {
    pub id: Uuid,
    pub company_id: Uuid,
    pub customer_id: Uuid,
    pub proforma_number: String,
    pub proforma_date: chrono::NaiveDate,
    pub valid_until: chrono::NaiveDate,
    pub subtotal: BigDecimal,
    pub tax_amount: BigDecimal,
    pub total_amount: BigDecimal,
    pub currency_code: String,
    pub status: String,
    pub narration: Option<String>,
    pub converted_invoice_id: Option<Uuid>,
    pub created_by: Uuid,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}
