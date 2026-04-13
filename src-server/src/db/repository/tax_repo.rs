// Author: Quadri Atharu
use bigdecimal::BigDecimal;
use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct TaxConfig {
    pub id: Uuid,
    pub company_id: Uuid,
    pub tax_type: String,
    pub name: String,
    pub rate: BigDecimal,
    pub is_active: bool,
    pub effective_from: NaiveDate,
    pub effective_to: Option<NaiveDate>,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct TaxTransaction {
    pub id: Uuid,
    pub company_id: Uuid,
    pub tax_config_id: Uuid,
    pub transaction_type: String,
    pub source_module: String,
    pub source_document_id: Uuid,
    pub source_document_number: Option<String>,
    pub taxable_amount: BigDecimal,
    pub tax_amount: BigDecimal,
    pub tax_rate: BigDecimal,
    pub posting_date: NaiveDate,
    pub is_reported: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewTaxConfig {
    pub company_id: Uuid,
    pub tax_type: String,
    pub name: String,
    pub rate: BigDecimal,
    pub effective_from: NaiveDate,
    pub effective_to: Option<NaiveDate>,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewTaxTransaction {
    pub company_id: Uuid,
    pub tax_config_id: Uuid,
    pub transaction_type: String,
    pub source_module: String,
    pub source_document_id: Uuid,
    pub source_document_number: Option<String>,
    pub taxable_amount: BigDecimal,
    pub tax_amount: BigDecimal,
    pub tax_rate: BigDecimal,
    pub posting_date: NaiveDate,
}

pub struct TaxRepo {
    pool: PgPool,
}

impl TaxRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn create_config(&self, new_config: NewTaxConfig) -> Result<TaxConfig, sqlx::Error> {
        sqlx::query_as::<_, TaxConfig>(
            r#"INSERT INTO tax_configs (company_id, tax_type, name, rate, effective_from, effective_to, description)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            RETURNING id, company_id, tax_type, name, rate, is_active, effective_from, effective_to, description, created_at, updated_at"#,
        )
        .bind(new_config.company_id)
        .bind(&new_config.tax_type)
        .bind(&new_config.name)
        .bind(&new_config.rate)
        .bind(new_config.effective_from)
        .bind(new_config.effective_to)
        .bind(&new_config.description)
        .fetch_one(&self.pool)
        .await
    }

    pub async fn find_configs(&self, company_id: Uuid) -> Result<Vec<TaxConfig>, sqlx::Error> {
        sqlx::query_as::<_, TaxConfig>(
            "SELECT id, company_id, tax_type, name, rate, is_active, effective_from, effective_to, description, created_at, updated_at FROM tax_configs WHERE company_id = $1 ORDER BY tax_type, name",
        )
        .bind(company_id)
        .fetch_all(&self.pool)
        .await
    }

    pub async fn record_tax_transaction(
        &self,
        new_txn: NewTaxTransaction,
    ) -> Result<TaxTransaction, sqlx::Error> {
        sqlx::query_as::<_, TaxTransaction>(
            r#"INSERT INTO tax_transactions (company_id, tax_config_id, transaction_type, source_module, source_document_id, source_document_number, taxable_amount, tax_amount, tax_rate, posting_date)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            RETURNING id, company_id, tax_config_id, transaction_type, source_module, source_document_id, source_document_number, taxable_amount, tax_amount, tax_rate, posting_date, is_reported, created_at"#,
        )
        .bind(new_txn.company_id)
        .bind(new_txn.tax_config_id)
        .bind(&new_txn.transaction_type)
        .bind(&new_txn.source_module)
        .bind(new_txn.source_document_id)
        .bind(&new_txn.source_document_number)
        .bind(&new_txn.taxable_amount)
        .bind(&new_txn.tax_amount)
        .bind(&new_txn.tax_rate)
        .bind(new_txn.posting_date)
        .fetch_one(&self.pool)
        .await
    }

    pub async fn find_tax_transactions(
        &self,
        company_id: Uuid,
    ) -> Result<Vec<TaxTransaction>, sqlx::Error> {
        sqlx::query_as::<_, TaxTransaction>(
            "SELECT id, company_id, tax_config_id, transaction_type, source_module, source_document_id, source_document_number, taxable_amount, tax_amount, tax_rate, posting_date, is_reported, created_at FROM tax_transactions WHERE company_id = $1 ORDER BY posting_date DESC",
        )
        .bind(company_id)
        .fetch_all(&self.pool)
        .await
    }
}
