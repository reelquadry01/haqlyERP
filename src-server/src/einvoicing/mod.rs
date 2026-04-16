// Author: Quadri Atharu
pub mod firs_client;
pub mod payload_builder;
pub mod rules_engine;

use bigdecimal::BigDecimal;
use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use std::str::FromStr;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum InvoiceCategory {
    #[serde(rename = "B2B")]
    B2B,
    #[serde(rename = "B2C")]
    B2C,
    #[serde(rename = "SIMPLIFIED")]
    Simplified,
}

impl InvoiceCategory {
    pub fn as_str(&self) -> &'static str {
        match self {
            InvoiceCategory::B2B => "B2B",
            InvoiceCategory::B2C => "B2C",
            InvoiceCategory::Simplified => "SIMPLIFIED",
        }
    }
}

impl FromStr for InvoiceCategory {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_uppercase().as_str() {
            "B2B" => Ok(InvoiceCategory::B2B),
            "B2C" => Ok(InvoiceCategory::B2C),
            "SIMPLIFIED" => Ok(InvoiceCategory::Simplified),
            other => Err(format!("unknown invoice category: {}", other)),
        }
    }
}

impl std::fmt::Display for InvoiceCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EInvoiceStatus {
    #[serde(rename = "LOCAL_ONLY")]
    LocalOnly,
    #[serde(rename = "PENDING_VALIDATION")]
    PendingValidation,
    #[serde(rename = "VALIDATED")]
    Validated,
    #[serde(rename = "PENDING_SIGNING")]
    PendingSigning,
    #[serde(rename = "SIGNED")]
    Signed,
    #[serde(rename = "PENDING_CONFIRMATION")]
    PendingConfirmation,
    #[serde(rename = "CONFIRMED")]
    Confirmed,
    #[serde(rename = "DOWNLOADED")]
    Downloaded,
    #[serde(rename = "UPDATED")]
    Updated,
    #[serde(rename = "REJECTED")]
    Rejected,
    #[serde(rename = "ERROR")]
    Error,
    #[serde(rename = "EXCHANGE_SENT")]
    ExchangeSent,
    #[serde(rename = "EXCHANGE_ACKNOWLEDGED")]
    ExchangeAcknowledged,
}

impl EInvoiceStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            EInvoiceStatus::LocalOnly => "LOCAL_ONLY",
            EInvoiceStatus::PendingValidation => "PENDING_VALIDATION",
            EInvoiceStatus::Validated => "VALIDATED",
            EInvoiceStatus::PendingSigning => "PENDING_SIGNING",
            EInvoiceStatus::Signed => "SIGNED",
            EInvoiceStatus::PendingConfirmation => "PENDING_CONFIRMATION",
            EInvoiceStatus::Confirmed => "CONFIRMED",
            EInvoiceStatus::Downloaded => "DOWNLOADED",
            EInvoiceStatus::Updated => "UPDATED",
            EInvoiceStatus::Rejected => "REJECTED",
            EInvoiceStatus::Error => "ERROR",
            EInvoiceStatus::ExchangeSent => "EXCHANGE_SENT",
            EInvoiceStatus::ExchangeAcknowledged => "EXCHANGE_ACKNOWLEDGED",
        }
    }
}

impl FromStr for EInvoiceStatus {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "LOCAL_ONLY" => Ok(EInvoiceStatus::LocalOnly),
            "PENDING_VALIDATION" => Ok(EInvoiceStatus::PendingValidation),
            "VALIDATED" => Ok(EInvoiceStatus::Validated),
            "PENDING_SIGNING" => Ok(EInvoiceStatus::PendingSigning),
            "SIGNED" => Ok(EInvoiceStatus::Signed),
            "PENDING_CONFIRMATION" => Ok(EInvoiceStatus::PendingConfirmation),
            "CONFIRMED" => Ok(EInvoiceStatus::Confirmed),
            "DOWNLOADED" => Ok(EInvoiceStatus::Downloaded),
            "UPDATED" => Ok(EInvoiceStatus::Updated),
            "REJECTED" => Ok(EInvoiceStatus::Rejected),
            "ERROR" => Ok(EInvoiceStatus::Error),
            "EXCHANGE_SENT" => Ok(EInvoiceStatus::ExchangeSent),
            "EXCHANGE_ACKNOWLEDGED" => Ok(EInvoiceStatus::ExchangeAcknowledged),
            other => Err(format!("unknown e-invoice status: {}", other)),
        }
    }
}

impl std::fmt::Display for EInvoiceStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct EInvoiceProfileRow {
    pub id: Uuid,
    pub company_id: Uuid,
    pub tin: String,
    pub legal_name: String,
    pub trade_name: Option<String>,
    pub business_email: Option<String>,
    pub business_phone: Option<String>,
    pub country_code: String,
    pub state: Option<String>,
    pub city: Option<String>,
    pub address_line1: String,
    pub address_line2: Option<String>,
    pub postal_code: Option<String>,
    pub access_point_provider_name: Option<String>,
    pub access_point_provider_code: Option<String>,
    pub default_currency_code: String,
    pub is_complete: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct EInvoiceCredentialRow {
    pub id: Uuid,
    pub company_id: Uuid,
    pub api_key: String,
    pub api_secret: String,
    pub crypto_key: Option<String>,
    pub base_url: String,
    pub environment: String,
    pub is_active: bool,
    pub last_tested_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct EInvoiceDocumentRow {
    pub id: Uuid,
    pub company_id: Uuid,
    pub sales_invoice_id: Uuid,
    pub irn: Option<String>,
    pub status: String,
    pub invoice_category: Option<String>,
    pub validation_result: Option<serde_json::Value>,
    pub signing_result: Option<serde_json::Value>,
    pub confirmation_result: Option<serde_json::Value>,
    pub download_data: Option<serde_json::Value>,
    pub firs_submitted_at: Option<DateTime<Utc>>,
    pub firs_confirmed_at: Option<DateTime<Utc>>,
    pub error_message: Option<String>,
    pub retry_count: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct EInvoiceWebhookEventRow {
    pub id: Uuid,
    pub company_id: Uuid,
    pub event_type: String,
    pub irn: Option<String>,
    pub payload: serde_json::Value,
    pub processed: bool,
    pub processed_at: Option<DateTime<Utc>>,
    pub error_message: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct EInvoiceAuditTrailRow {
    pub id: Uuid,
    pub company_id: Uuid,
    pub einvoice_document_id: Uuid,
    pub action: String,
    pub endpoint: Option<String>,
    pub request_payload: Option<serde_json::Value>,
    pub response_payload: Option<serde_json::Value>,
    pub status_code: Option<i32>,
    pub user_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct SalesInvoiceRow {
    pub id: Uuid,
    pub company_id: Uuid,
    pub branch_id: Option<Uuid>,
    pub customer_id: Uuid,
    pub number: String,
    pub date: NaiveDate,
    pub due_date: Option<NaiveDate>,
    pub invoice_type: String,
    pub status: String,
    pub currency_code: String,
    pub exchange_rate: BigDecimal,
    pub taxable_amount: BigDecimal,
    pub tax_amount: BigDecimal,
    pub total_amount: BigDecimal,
    pub amount_paid: BigDecimal,
    pub narration: Option<String>,
    pub is_einvoice_eligible: bool,
    pub einvoice_irn: Option<String>,
    pub created_by: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct SalesInvoiceItemRow {
    pub id: Uuid,
    pub sales_invoice_id: Uuid,
    pub line_number: i32,
    pub product_id: Option<Uuid>,
    pub sku: Option<String>,
    pub description: String,
    pub quantity: BigDecimal,
    pub unit_price: BigDecimal,
    pub discount_percent: BigDecimal,
    pub tax_rate: Option<BigDecimal>,
    pub taxable_amount: BigDecimal,
    pub tax_amount: BigDecimal,
    pub line_amount: BigDecimal,
    pub cost_center_id: Option<Uuid>,
    pub project_id: Option<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct CustomerRow {
    pub id: Uuid,
    pub company_id: Uuid,
    pub code: String,
    pub name: String,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub tax_id: Option<String>,
    pub customer_type: String,
    pub credit_limit: BigDecimal,
    pub payment_terms: i32,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct CustomerAddressRow {
    pub id: Uuid,
    pub customer_id: Uuid,
    pub line1: String,
    pub line2: Option<String>,
    pub city: Option<String>,
    pub state: Option<String>,
    pub country_code: String,
    pub postal_code: Option<String>,
    pub is_default: bool,
}

pub fn bd_to_f64(bd: &BigDecimal) -> f64 {
    use std::str::FromStr;
    f64::from_str(&bd.to_string()).unwrap_or(0.0)
}

pub fn opt_bd_to_f64(bd: &Option<BigDecimal>) -> f64 {
    bd.as_ref().map_or(0.0, bd_to_f64)
}

pub fn round_money(value: f64) -> f64 {
    (value * 100.0).round() / 100.0
}
