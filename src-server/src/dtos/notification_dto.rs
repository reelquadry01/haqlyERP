// Author: Quadri Atharu
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

use crate::models::notification::NotificationType;

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct CreateNotificationRequest {
    pub company_id: Uuid,
    pub user_id: Uuid,
    pub notification_type: NotificationType,
    #[validate(length(min = 1, message = "title is required"))]
    pub title: String,
    #[validate(length(min = 1, message = "message is required"))]
    pub message: String,
    pub action_url: Option<String>,
    pub related_entity: Option<String>,
    pub related_entity_id: Option<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct MarkReadRequest {
    pub notification_ids: Vec<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct UpdatePreferencesRequest {
    pub user_id: Uuid,
    pub notification_type: NotificationType,
    pub email_enabled: Option<bool>,
    pub in_app_enabled: Option<bool>,
    pub push_enabled: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationListParams {
    pub user_id: Option<Uuid>,
    pub company_id: Option<Uuid>,
    pub is_read: Option<bool>,
    pub notification_type: Option<NotificationType>,
    pub page: Option<i64>,
    pub limit: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BulkNotificationRequest {
    pub company_id: Uuid,
    pub user_ids: Vec<Uuid>,
    pub notification_type: NotificationType,
    pub title: String,
    pub message: String,
    pub action_url: Option<String>,
    pub related_entity: Option<String>,
    pub related_entity_id: Option<Uuid>,
}
