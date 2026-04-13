// Author: Quadri Atharu
use bigdecimal::BigDecimal;
use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type, PartialEq)]
#[sqlx(type_name = "voucher_status", rename_all = "snake_case")]
pub enum VoucherStatus {
    Draft,
    Validated,
    Submitted,
    Approved,
    Posted,
    Paid,
    Cancelled,
}

impl std::fmt::Display for VoucherStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            VoucherStatus::Draft => write!(f, "draft"),
            VoucherStatus::Validated => write!(f, "validated"),
            VoucherStatus::Submitted => write!(f, "submitted"),
            VoucherStatus::Approved => write!(f, "approved"),
            VoucherStatus::Posted => write!(f, "posted"),
            VoucherStatus::Paid => write!(f, "paid"),
            VoucherStatus::Cancelled => write!(f, "cancelled"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct PaymentVoucher {
    pub id: Uuid,
    pub company_id: Uuid,
    pub branch_id: Option<Uuid>,
    pub voucher_number: String,
    pub payee_name: String,
    pub payee_account: Option<String>,
    pub payee_bank: Option<String>,
    pub payee_bank_code: Option<String>,
    pub amount: BigDecimal,
    pub currency_code: String,
    pub narration: Option<String>,
    pub reference: Option<String>,
    pub status: VoucherStatus,
    pub voucher_type: Option<String>,
    pub due_date: Option<chrono::NaiveDate>,
    pub payment_date: Option<chrono::NaiveDate>,
    pub supplier_id: Option<Uuid>,
    pub payment_method: Option<String>,
    pub payment_gateway_ref: Option<String>,
    pub journal_header_id: Option<Uuid>,
    pub created_by: Uuid,
    pub approved_by: Option<Uuid>,
    pub paid_by: Option<Uuid>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct VoucherLine {
    pub id: Uuid,
    pub voucher_id: Uuid,
    pub account_id: Uuid,
    pub line_number: i32,
    pub narration: Option<String>,
    pub amount: BigDecimal,
    pub cost_center_id: Option<Uuid>,
    pub project_id: Option<Uuid>,
    pub created_at: NaiveDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentGatewayRequest {
    pub voucher_id: Uuid,
    pub payee_account: String,
    pub payee_bank_code: String,
    pub amount: BigDecimal,
    pub currency: String,
    pub narration: String,
    pub reference: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct VoucherComment {
    pub id: Uuid,
    pub voucher_id: Uuid,
    pub user_id: Uuid,
    pub comment: String,
    pub created_at: NaiveDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct VoucherAttachment {
    pub id: Uuid,
    pub voucher_id: Uuid,
    pub file_name: String,
    pub file_path: String,
    pub file_size: i64,
    pub content_type: String,
    pub uploaded_by: Uuid,
    pub created_at: NaiveDateTime,
}
