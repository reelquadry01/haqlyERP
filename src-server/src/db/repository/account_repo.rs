// Author: Quadri Atharu
use bigdecimal::BigDecimal;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Account {
    pub id: Uuid,
    pub company_id: Uuid,
    pub code: String,
    pub name: String,
    pub account_type: String,
    pub sub_type: Option<String>,
    pub parent_id: Option<Uuid>,
    pub is_control_account: bool,
    pub is_active: bool,
    pub allowed_posting: bool,
    pub currency_code: String,
    pub balance: BigDecimal,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewAccount {
    pub company_id: Uuid,
    pub code: String,
    pub name: String,
    pub account_type: String,
    pub sub_type: Option<String>,
    pub parent_id: Option<Uuid>,
    pub is_control_account: bool,
    pub allowed_posting: bool,
    pub currency_code: String,
}

pub struct AccountRepo {
    pool: PgPool,
}

impl AccountRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn find_by_company(&self, company_id: Uuid) -> Result<Vec<Account>, sqlx::Error> {
        sqlx::query_as::<_, Account>(
            "SELECT id, company_id, code, name, account_type, sub_type, parent_id, is_control_account, is_active, allowed_posting, currency_code, balance, created_at, updated_at FROM chart_of_accounts WHERE company_id = $1 ORDER BY code",
        )
        .bind(company_id)
        .fetch_all(&self.pool)
        .await
    }

    pub async fn find_by_code(
        &self,
        company_id: Uuid,
        code: &str,
    ) -> Result<Option<Account>, sqlx::Error> {
        sqlx::query_as::<_, Account>(
            "SELECT id, company_id, code, name, account_type, sub_type, parent_id, is_control_account, is_active, allowed_posting, currency_code, balance, created_at, updated_at FROM chart_of_accounts WHERE company_id = $1 AND code = $2",
        )
        .bind(company_id)
        .bind(code)
        .fetch_optional(&self.pool)
        .await
    }

    pub async fn create(&self, new_account: NewAccount) -> Result<Account, sqlx::Error> {
        sqlx::query_as::<_, Account>(
            r#"INSERT INTO chart_of_accounts (company_id, code, name, account_type, sub_type, parent_id, is_control_account, allowed_posting, currency_code)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            RETURNING id, company_id, code, name, account_type, sub_type, parent_id, is_control_account, is_active, allowed_posting, currency_code, balance, created_at, updated_at"#,
        )
        .bind(new_account.company_id)
        .bind(&new_account.code)
        .bind(&new_account.name)
        .bind(&new_account.account_type)
        .bind(&new_account.sub_type)
        .bind(new_account.parent_id)
        .bind(new_account.is_control_account)
        .bind(new_account.allowed_posting)
        .bind(&new_account.currency_code)
        .fetch_one(&self.pool)
        .await
    }
}
