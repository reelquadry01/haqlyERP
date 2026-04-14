// Author: Quadri Atharu
use anyhow::{anyhow, Result};
use sqlx::PgPool;
use uuid::Uuid;

use crate::models::notification::{Notification, NotificationPreference, NotificationType};

#[derive(Clone)]
pub struct NotificationService {
    pub pool: PgPool,
}

impl NotificationService {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn create_notification(
        &self,
        company_id: Uuid,
        user_id: Uuid,
        notification_type: NotificationType,
        title: String,
        message: String,
        action_url: Option<String>,
        related_entity: Option<String>,
        related_entity_id: Option<Uuid>,
    ) -> Result<Notification> {
        let id = Uuid::now_v7();
        sqlx::query(
            r#"INSERT INTO notifications (id, company_id, user_id, notification_type, title, message, is_read, action_url, related_entity, related_entity_id, created_at)
               VALUES ($1, $2, $3, $4, $5, $6, false, $7, $8, $9, NOW())"#,
        )
        .bind(id)
        .bind(company_id)
        .bind(user_id)
        .bind(&notification_type)
        .bind(&title)
        .bind(&message)
        .bind(&action_url)
        .bind(&related_entity)
        .bind(related_entity_id)
        .execute(&self.pool)
        .await?;

        sqlx::query_as::<_, Notification>("SELECT * FROM notifications WHERE id = $1")
            .bind(id)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| anyhow!("Failed to fetch notification: {}", e))
    }

    pub async fn get_user_notifications(
        &self,
        user_id: Uuid,
        page: i64,
        limit: i64,
    ) -> Result<Vec<Notification>> {
        let offset = (page - 1) * limit;
        sqlx::query_as::<_, Notification>(
            "SELECT * FROM notifications WHERE user_id = $1 ORDER BY created_at DESC LIMIT $2 OFFSET $3",
        )
        .bind(user_id)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| anyhow!("Failed to get notifications: {}", e))
    }

    pub async fn mark_as_read(&self, notification_ids: Vec<Uuid>) -> Result<()> {
        for id in notification_ids {
            sqlx::query("UPDATE notifications SET is_read = true WHERE id = $1")
                .bind(id)
                .execute(&self.pool)
                .await?;
        }
        Ok(())
    }

    pub async fn mark_all_read(&self, user_id: Uuid) -> Result<u64> {
        let result = sqlx::query(
            "UPDATE notifications SET is_read = true WHERE user_id = $1 AND is_read = false",
        )
        .bind(user_id)
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected())
    }

    pub async fn get_preferences(&self, user_id: Uuid) -> Result<Vec<NotificationPreference>> {
        sqlx::query_as::<_, NotificationPreference>(
            "SELECT * FROM notification_preferences WHERE user_id = $1",
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| anyhow!("Failed to get preferences: {}", e))
    }

    pub async fn update_preferences(
        &self,
        user_id: Uuid,
        notification_type: NotificationType,
        email_enabled: Option<bool>,
        in_app_enabled: Option<bool>,
        push_enabled: Option<bool>,
    ) -> Result<NotificationPreference> {
        let existing = sqlx::query_as::<_, NotificationPreference>(
            "SELECT * FROM notification_preferences WHERE user_id = $1 AND notification_type = $2",
        )
        .bind(user_id)
        .bind(&notification_type)
        .fetch_optional(&self.pool)
        .await?;

        match existing {
            Some(pref) => {
                let email = email_enabled.unwrap_or(pref.email_enabled);
                let in_app = in_app_enabled.unwrap_or(pref.in_app_enabled);
                let push = push_enabled.unwrap_or(pref.push_enabled);

                sqlx::query(
                    "UPDATE notification_preferences SET email_enabled = $1, in_app_enabled = $2, push_enabled = $3, updated_at = NOW() WHERE id = $4",
                )
                .bind(email)
                .bind(in_app)
                .bind(push)
                .bind(pref.id)
                .execute(&self.pool)
                .await?;

                sqlx::query_as::<_, NotificationPreference>(
                    "SELECT * FROM notification_preferences WHERE id = $1",
                )
                .bind(pref.id)
                .fetch_one(&self.pool)
                .await
                .map_err(|e| anyhow!("Failed to fetch preference: {}", e))
            }
            None => {
                let id = Uuid::now_v7();
                sqlx::query(
                    r#"INSERT INTO notification_preferences (id, user_id, notification_type, email_enabled, in_app_enabled, push_enabled, created_at, updated_at)
                       VALUES ($1, $2, $3, $4, $5, $6, NOW(), NOW())"#,
                )
                .bind(id)
                .bind(user_id)
                .bind(&notification_type)
                .bind(email_enabled.unwrap_or(true))
                .bind(in_app_enabled.unwrap_or(true))
                .bind(push_enabled.unwrap_or(false))
                .execute(&self.pool)
                .await?;

                sqlx::query_as::<_, NotificationPreference>(
                    "SELECT * FROM notification_preferences WHERE id = $1",
                )
                .bind(id)
                .fetch_one(&self.pool)
                .await
                .map_err(|e| anyhow!("Failed to fetch preference: {}", e))
            }
        }
    }

    pub async fn send_bulk_notifications(
        &self,
        company_id: Uuid,
        user_ids: Vec<Uuid>,
        notification_type: NotificationType,
        title: String,
        message: String,
        action_url: Option<String>,
        related_entity: Option<String>,
        related_entity_id: Option<Uuid>,
    ) -> Result<Vec<Notification>> {
        let mut notifications = Vec::new();
        for user_id in user_ids {
            let notif = self
                .create_notification(
                    company_id,
                    user_id,
                    notification_type.clone(),
                    title.clone(),
                    message.clone(),
                    action_url.clone(),
                    related_entity.clone(),
                    related_entity_id,
                )
                .await?;
            notifications.push(notif);
        }
        Ok(notifications)
    }
}
