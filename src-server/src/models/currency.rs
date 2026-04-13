// Author: Quadri Atharu
use bigdecimal::BigDecimal;
use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Currency {
    pub id: Uuid,
    pub code: String,
    pub name: String,
    pub symbol: Option<String>,
    pub is_base: bool,
    pub decimal_places: i32,
    pub created_at: NaiveDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ExchangeRate {
    pub id: Uuid,
    pub company_id: Uuid,
    pub from_currency: String,
    pub to_currency: String,
    pub rate: BigDecimal,
    pub effective_date: chrono::NaiveDate,
    pub source: Option<String>,
    pub created_by: Uuid,
    pub created_at: NaiveDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct FxGainLossEntry {
    pub id: Uuid,
    pub company_id: Uuid,
    pub journal_header_id: Option<Uuid>,
    pub source_document_id: Option<Uuid>,
    pub source_document_type: Option<String>,
    pub original_amount: BigDecimal,
    pub revalued_amount: BigDecimal,
    pub gain_loss_amount: BigDecimal,
    pub currency_code: String,
    pub exchange_rate: BigDecimal,
    pub gain_loss_type: String,
    pub gain_loss_account_id: Uuid,
    pub period_id: Option<Uuid>,
    pub created_at: NaiveDateTime,
}
