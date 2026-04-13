// Author: Quadri Atharu
use bigdecimal::BigDecimal;
use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type, PartialEq)]
#[sqlx(type_name = "invoice_category", rename_all = "snake_case")]
pub enum InvoiceCategory {
    B2b,
    B2c,
    Simplified,
}

impl std::fmt::Display for InvoiceCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            InvoiceCategory::B2b => write!(f, "b2b"),
            InvoiceCategory::B2c => write!(f, "b2c"),
            InvoiceCategory::Simplified => write!(f, "simplified"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type, PartialEq)]
#[sqlx(type_name = "einvoice_status", rename_all = "snake_case")]
pub enum EInvoiceStatus {
    Pending,
    Submitted,
    Validated,
    Rejected,
    Cancelled,
    Expired,
}

impl std::fmt::Display for EInvoiceStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EInvoiceStatus::Pending => write!(f, "pending"),
            EInvoiceStatus::Submitted => write!(f, "submitted"),
            EInvoiceStatus::Validated => write!(f, "validated"),
            EInvoiceStatus::Rejected => write!(f, "rejected"),
            EInvoiceStatus::Cancelled => write!(f, "cancelled"),
            EInvoiceStatus::Expired => write!(f, "expired"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct EInvoiceProfile {
    pub id: Uuid,
    pub company_id: Uuid,
    pub business_id: String,
    pub business_name: String,
    pub tax_id: String,
    pub integration_type: String,
    pub api_base_url: Option<String>,
    pub is_active: bool,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct EInvoiceCredential {
    pub id: Uuid,
    pub profile_id: Uuid,
    pub company_id: Uuid,
    pub client_id: String,
    pub client_secret_encrypted: String,
    pub certificate_path: Option<String>,
    pub is_active: bool,
    pub expires_at: Option<NaiveDateTime>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct EInvoiceDocument {
    pub id: Uuid,
    pub company_id: Uuid,
    pub invoice_id: Option<Uuid>,
    pub irn: Option<String>,
    pub category: InvoiceCategory,
    pub status: EInvoiceStatus,
    pub payload_json: Option<serde_json::Value>,
    pub response_json: Option<serde_json::Value>,
    pub error_message: Option<String>,
    pub submitted_at: Option<NaiveDateTime>,
    pub validated_at: Option<NaiveDateTime>,
    pub qr_code: Option<String>,
    pub signed_invoice: Option<String>,
    pub created_by: Uuid,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EInvoicePayload {
    pub seller: PartyPayload,
    pub buyer: PartyPayload,
    pub document: DocumentPayload,
    pub totals: TotalsPayload,
    pub lines: Vec<LinePayload>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PartyPayload {
    pub name: String,
    pub tax_id: Option<String>,
    pub business_id: Option<String>,
    pub address: AddressPayload,
    pub email: Option<String>,
    pub phone: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddressPayload {
    pub line1: String,
    pub line2: Option<String>,
    pub city: String,
    pub state: Option<String>,
    pub country: String,
    pub postal_code: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentPayload {
    pub invoice_number: String,
    pub invoice_date: String,
    pub due_date: Option<String>,
    pub currency: String,
    pub category: InvoiceCategory,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TotalsPayload {
    pub subtotal: BigDecimal,
    pub tax_amount: BigDecimal,
    pub discount_amount: BigDecimal,
    pub total: BigDecimal,
    pub amount_due: BigDecimal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LinePayload {
    pub line_number: i32,
    pub description: String,
    pub quantity: BigDecimal,
    pub unit_price: BigDecimal,
    pub tax_rate: BigDecimal,
    pub tax_amount: BigDecimal,
    pub line_total: BigDecimal,
}
