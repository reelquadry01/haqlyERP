// Author: Quadri Atharu
use bigdecimal::BigDecimal;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct User {
    pub id: Uuid,
    pub company_id: Option<Uuid>,
    pub email: String,
    pub password_hash: String,
    pub full_name: String,
    pub phone: Option<String>,
    pub is_active: bool,
    pub mfa_secret: Option<String>,
    pub mfa_enabled: bool,
    pub last_login_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewUser {
    pub company_id: Option<Uuid>,
    pub email: String,
    pub password_hash: String,
    pub full_name: String,
    pub phone: Option<String>,
}

pub struct UserRepo {
    pool: PgPool,
}

impl UserRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn find_by_email(&self, email: &str) -> Result<Option<User>, sqlx::Error> {
        sqlx::query_as::<_, User>(
            "SELECT id, company_id, email, password_hash, full_name, phone, is_active, mfa_secret, mfa_enabled, last_login_at, created_at, updated_at FROM users WHERE email = $1",
        )
        .bind(email)
        .fetch_optional(&self.pool)
        .await
    }

    pub async fn find_by_id(&self, id: Uuid) -> Result<Option<User>, sqlx::Error> {
        sqlx::query_as::<_, User>(
            "SELECT id, company_id, email, password_hash, full_name, phone, is_active, mfa_secret, mfa_enabled, last_login_at, created_at, updated_at FROM users WHERE id = $1",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
    }

    pub async fn create(&self, new_user: NewUser) -> Result<User, sqlx::Error> {
        sqlx::query_as::<_, User>(
            r#"INSERT INTO users (company_id, email, password_hash, full_name, phone)
            VALUES ($1, $2, $3, $4, $5)
            RETURNING id, company_id, email, password_hash, full_name, phone, is_active, mfa_secret, mfa_enabled, last_login_at, created_at, updated_at"#,
        )
        .bind(&new_user.company_id)
        .bind(&new_user.email)
        .bind(&new_user.password_hash)
        .bind(&new_user.full_name)
        .bind(&new_user.phone)
        .fetch_one(&self.pool)
        .await
    }

    pub async fn update_last_login(&self, id: Uuid) -> Result<(), sqlx::Error> {
        sqlx::query("UPDATE users SET last_login_at = now(), updated_at = now() WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }
}
