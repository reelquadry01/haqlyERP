// Author: Quadri Atharu
use anyhow::{anyhow, Result};
use sqlx::PgPool;
use uuid::Uuid;

use crate::models::user::{User, UserRole};

#[derive(Clone)]
pub struct UsersService {
    pub pool: PgPool,
}

impl UsersService {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn list(&self, company_id: Uuid, page: i64, limit: i64) -> Result<Vec<User>> {
        let offset = (page - 1) * limit;
        let users = sqlx::query_as::<_, User>(
            "SELECT * FROM users WHERE company_id = $1 AND is_active = true ORDER BY created_at DESC LIMIT $2 OFFSET $3",
        )
        .bind(company_id)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await?;
        Ok(users)
    }

    pub async fn get(&self, user_id: Uuid) -> Result<User> {
        let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = $1")
            .bind(user_id)
            .fetch_optional(&self.pool)
            .await?
            .ok_or_else(|| anyhow!("User not found"))?;
        Ok(user)
    }

    pub async fn create(
        &self,
        company_id: Uuid,
        email: String,
        full_name: String,
        password_hash: String,
        phone: Option<String>,
    ) -> Result<User> {
        let id = Uuid::now_v7();
        sqlx::query(
            r#"INSERT INTO users (id, company_id, email, password_hash, full_name, phone, is_active, mfa_enabled, created_at, updated_at)
               VALUES ($1, $2, $3, $4, $5, $6, true, false, NOW(), NOW())"#,
        )
        .bind(id)
        .bind(company_id)
        .bind(&email)
        .bind(&password_hash)
        .bind(&full_name)
        .bind(&phone)
        .execute(&self.pool)
        .await?;

        self.get(id).await
    }

    pub async fn update(
        &self,
        user_id: Uuid,
        full_name: Option<String>,
        email: Option<String>,
        phone: Option<String>,
        is_active: Option<bool>,
    ) -> Result<User> {
        let user = self.get(user_id).await?;
        let full_name = full_name.unwrap_or(user.full_name);
        let email = email.unwrap_or(user.email);
        let phone = phone.or(user.phone);
        let is_active = is_active.unwrap_or(user.is_active);

        sqlx::query(
            "UPDATE users SET full_name = $1, email = $2, phone = $3, is_active = $4, updated_at = NOW() WHERE id = $5",
        )
        .bind(&full_name)
        .bind(&email)
        .bind(&phone)
        .bind(is_active)
        .bind(user_id)
        .execute(&self.pool)
        .await?;

        self.get(user_id).await
    }

    pub async fn reset_password(&self, user_id: Uuid, new_password_hash: String) -> Result<()> {
        sqlx::query("UPDATE users SET password_hash = $1, updated_at = NOW() WHERE id = $2")
            .bind(&new_password_hash)
            .bind(user_id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn delete(&self, user_id: Uuid) -> Result<()> {
        sqlx::query("UPDATE users SET is_active = false, updated_at = NOW() WHERE id = $1")
            .bind(user_id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn assign_role(&self, user_id: Uuid, role_id: Uuid) -> Result<UserRole> {
        let id = Uuid::now_v7();
        sqlx::query(
            "INSERT INTO user_roles (id, user_id, role_id, assigned_at) VALUES ($1, $2, $3, NOW())",
        )
        .bind(id)
        .bind(user_id)
        .bind(role_id)
        .execute(&self.pool)
        .await?;

        Ok(UserRole {
            id,
            user_id,
            role_id,
            assigned_at: chrono::Utc::now(),
        })
    }
}
