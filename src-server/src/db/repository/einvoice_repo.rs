// Author: Quadri Atharu
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use sqlx::FromRow;
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct EInvoiceProfile {
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
pub struct EInvoiceCredential {
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
pub struct EInvoiceDocument {
    pub id: Uuid,
    pub company_id: Uuid,
    pub sales_invoice_id: Uuid,
    pub irn: Option<String>,
    pub status: String,
    pub invoice_category: Option<String>,
    pub validation_result: Option<JsonValue>,
    pub signing_result: Option<JsonValue>,
    pub confirmation_result: Option<JsonValue>,
    pub download_data: Option<JsonValue>,
    pub firs_submitted_at: Option<DateTime<Utc>>,
    pub firs_confirmed_at: Option<DateTime<Utc>>,
    pub error_message: Option<String>,
    pub retry_count: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewEInvoiceProfile {
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
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewEInvoiceCredential {
    pub company_id: Uuid,
    pub api_key: String,
    pub api_secret: String,
    pub crypto_key: Option<String>,
    pub base_url: String,
    pub environment: String,
}

pub struct EInvoiceRepo {
    pool: PgPool,
}

impl EInvoiceRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn find_profile(
        &self,
        company_id: Uuid,
    ) -> Result<Option<EInvoiceProfile>, sqlx::Error> {
        sqlx::query_as::<_, EInvoiceProfile>(
            r#"SELECT id, company_id, tin, legal_name, trade_name, business_email, business_phone,
                country_code, state, city, address_line1, address_line2, postal_code,
                access_point_provider_name, access_point_provider_code,
                default_currency_code, is_complete, created_at, updated_at
            FROM einvoice_profiles WHERE company_id = $1"#,
        )
        .bind(company_id)
        .fetch_optional(&self.pool)
        .await
    }

    pub async fn save_profile(
        &self,
        profile: NewEInvoiceProfile,
    ) -> Result<EInvoiceProfile, sqlx::Error> {
        sqlx::query_as::<_, EInvoiceProfile>(
            r#"INSERT INTO einvoice_profiles (company_id, tin, legal_name, trade_name, business_email, business_phone, country_code, state, city, address_line1, address_line2, postal_code, access_point_provider_name, access_point_provider_code, default_currency_code)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15)
            ON CONFLICT (company_id) DO UPDATE SET
                tin = EXCLUDED.tin, legal_name = EXCLUDED.legal_name, trade_name = EXCLUDED.trade_name,
                business_email = EXCLUDED.business_email, business_phone = EXCLUDED.business_phone,
                country_code = EXCLUDED.country_code, state = EXCLUDED.state, city = EXCLUDED.city,
                address_line1 = EXCLUDED.address_line1, address_line2 = EXCLUDED.address_line2,
                postal_code = EXCLUDED.postal_code,
                access_point_provider_name = EXCLUDED.access_point_provider_name,
                access_point_provider_code = EXCLUDED.access_point_provider_code,
                default_currency_code = EXCLUDED.default_currency_code,
                updated_at = now()
            RETURNING id, company_id, tin, legal_name, trade_name, business_email, business_phone,
                country_code, state, city, address_line1, address_line2, postal_code,
                access_point_provider_name, access_point_provider_code,
                default_currency_code, is_complete, created_at, updated_at"#,
        )
        .bind(profile.company_id)
        .bind(&profile.tin)
        .bind(&profile.legal_name)
        .bind(&profile.trade_name)
        .bind(&profile.business_email)
        .bind(&profile.business_phone)
        .bind(&profile.country_code)
        .bind(&profile.state)
        .bind(&profile.city)
        .bind(&profile.address_line1)
        .bind(&profile.address_line2)
        .bind(&profile.postal_code)
        .bind(&profile.access_point_provider_name)
        .bind(&profile.access_point_provider_code)
        .bind(&profile.default_currency_code)
        .fetch_one(&self.pool)
        .await
    }

    pub async fn find_credentials(
        &self,
        company_id: Uuid,
    ) -> Result<Option<EInvoiceCredential>, sqlx::Error> {
        sqlx::query_as::<_, EInvoiceCredential>(
            r#"SELECT id, company_id, api_key, api_secret, crypto_key, base_url, environment,
                is_active, last_tested_at, created_at, updated_at
            FROM einvoice_credentials WHERE company_id = $1"#,
        )
        .bind(company_id)
        .fetch_optional(&self.pool)
        .await
    }

    pub async fn save_credentials(
        &self,
        cred: NewEInvoiceCredential,
    ) -> Result<EInvoiceCredential, sqlx::Error> {
        sqlx::query_as::<_, EInvoiceCredential>(
            r#"INSERT INTO einvoice_credentials (company_id, api_key, api_secret, crypto_key, base_url, environment)
            VALUES ($1, $2, $3, $4, $5, $6)
            ON CONFLICT (company_id) DO UPDATE SET
                api_key = EXCLUDED.api_key, api_secret = EXCLUDED.api_secret,
                crypto_key = EXCLUDED.crypto_key, base_url = EXCLUDED.base_url,
                environment = EXCLUDED.environment, updated_at = now()
            RETURNING id, company_id, api_key, api_secret, crypto_key, base_url, environment,
                is_active, last_tested_at, created_at, updated_at"#,
        )
        .bind(cred.company_id)
        .bind(&cred.api_key)
        .bind(&cred.api_secret)
        .bind(&cred.crypto_key)
        .bind(&cred.base_url)
        .bind(&cred.environment)
        .fetch_one(&self.pool)
        .await
    }

    pub async fn create_document(
        &self,
        company_id: Uuid,
        sales_invoice_id: Uuid,
        invoice_category: Option<&str>,
    ) -> Result<EInvoiceDocument, sqlx::Error> {
        sqlx::query_as::<_, EInvoiceDocument>(
            r#"INSERT INTO einvoice_documents (company_id, sales_invoice_id, invoice_category)
            VALUES ($1, $2, $3)
            RETURNING id, company_id, sales_invoice_id, irn, status, invoice_category,
                validation_result, signing_result, confirmation_result, download_data,
                firs_submitted_at, firs_confirmed_at, error_message, retry_count,
                created_at, updated_at"#,
        )
        .bind(company_id)
        .bind(sales_invoice_id)
        .bind(invoice_category)
        .fetch_one(&self.pool)
        .await
    }

    pub async fn find_document_by_invoice(
        &self,
        sales_invoice_id: Uuid,
    ) -> Result<Option<EInvoiceDocument>, sqlx::Error> {
        sqlx::query_as::<_, EInvoiceDocument>(
            r#"SELECT id, company_id, sales_invoice_id, irn, status, invoice_category,
                validation_result, signing_result, confirmation_result, download_data,
                firs_submitted_at, firs_confirmed_at, error_message, retry_count,
                created_at, updated_at
            FROM einvoice_documents WHERE sales_invoice_id = $1"#,
        )
        .bind(sales_invoice_id)
        .fetch_optional(&self.pool)
        .await
    }

    pub async fn update_document_irn(
        &self,
        id: Uuid,
        irn: &str,
    ) -> Result<(), sqlx::Error> {
        sqlx::query("UPDATE einvoice_documents SET irn = $1, updated_at = now() WHERE id = $2")
            .bind(irn)
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn update_document_status(
        &self,
        id: Uuid,
        status: &str,
        error_message: Option<&str>,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            "UPDATE einvoice_documents SET status = $1, error_message = $2, updated_at = now() WHERE id = $3",
        )
        .bind(status)
        .bind(error_message)
        .bind(id)
        .execute(&self.pool)
        .await?;
        Ok(())
    }
}
