// Author: Quadri Atharu
use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type, PartialEq)]
#[sqlx(type_name = "notification_type", rename_all = "snake_case")]
pub enum NotificationType {
    JournalApproval,
    VoucherApproval,
    TaxDeadline,
    ReportReady,
    PeriodClosing,
    InvoiceOverdue,
    Custom,
}

impl std::fmt::Display for NotificationType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NotificationType::JournalApproval => write!(f, "journal_approval"),
            NotificationType::VoucherApproval => write!(f, "voucher_approval"),
            NotificationType::TaxDeadline => write!(f, "tax_deadline"),
            NotificationType::ReportReady => write!(f, "report_ready"),
            NotificationType::PeriodClosing => write!(f, "period_closing"),
            NotificationType::InvoiceOverdue => write!(f, "invoice_overdue"),
            NotificationType::Custom => write!(f, "custom"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Notification {
    pub id: Uuid,
    pub company_id: Uuid,
    pub user_id: Uuid,
    pub notification_type: NotificationType,
    pub title: String,
    pub message: String,
    pub is_read: bool,
    pub action_url: Option<String>,
    pub related_entity: Option<String>,
    pub related_entity_id: Option<Uuid>,
    pub created_at: NaiveDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct NotificationPreference {
    pub id: Uuid,
    pub user_id: Uuid,
    pub notification_type: NotificationType,
    pub email_enabled: bool,
    pub in_app_enabled: bool,
    pub push_enabled: bool,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}
