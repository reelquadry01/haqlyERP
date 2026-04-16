// Author: Quadri Atharu
use anyhow::{anyhow, Result};
use base64::engine::general_purpose::STANDARD as BASE64;
use base64::Engine;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use sqlx::FromRow;
use sqlx::PgPool;
use uuid::Uuid;

use crate::services::encryption_service;

fn get_encryption_key() -> Result<[u8; 32]> {
    let key_b64 = std::env::var("HAQLY_ENCRYPTION_KEY")
        .map_err(|_| anyhow!("HAQLY_ENCRYPTION_KEY not set"))?;
    let key_bytes = BASE64.decode(&key_b64)
        .map_err(|e| anyhow!("Invalid encryption key: {}", e))?;
    if key_bytes.len() != 32 {
        return Err(anyhow!("Encryption key must be 32 bytes"));
    }
    let mut key = [0u8; 32];
    key.copy_from_slice(&key_bytes);
    Ok(key)
}

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
    pub api_key_nonce: Option<Vec<u8>>,
    pub api_key_tag: Option<Vec<u8>>,
    pub api_secret_nonce: Option<Vec<u8>>,
    pub api_secret_tag: Option<Vec<u8>>,
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
        let mut result = sqlx::query_as::<_, EInvoiceCredential>(
            r#"SELECT id, company_id, api_key, api_secret, crypto_key, base_url, environment,
                is_active, last_tested_at, api_key_nonce, api_key_tag, api_secret_nonce, api_secret_tag,
                created_at, updated_at
            FROM einvoice_credentials WHERE company_id = $1"#,
        )
        .bind(company_id)
        .fetch_optional(&self.pool)
        .await?;

        if let Some(ref mut cred) = result {
            if let Ok(key) = get_encryption_key() {
                if let (Some(ref nonce), Some(ref tag)) = (cred.api_key_nonce.clone(), cred.api_key_tag.clone()) {
                    let nonce_b64 = BASE64.encode(nonce);
                    let tag_b64 = BASE64.encode(tag);
                    if let Ok(plain) = encryption_service::decrypt_field(&cred.api_key, &key, &nonce_b64, &tag_b64) {
                        cred.api_key = plain;
                    }
                }
                if let (Some(ref nonce), Some(ref tag)) = (cred.api_secret_nonce.clone(), cred.api_secret_tag.clone()) {
                    let nonce_b64 = BASE64.encode(nonce);
                    let tag_b64 = BASE64.encode(tag);
                    if let Ok(plain) = encryption_service::decrypt_field(&cred.api_secret, &key, &nonce_b64, &tag_b64) {
                        cred.api_secret = plain;
                    }
                }
            }
        }

        Ok(result)
    }

    pub async fn save_credentials(
        &self,
        cred: NewEInvoiceCredential,
    ) -> Result<EInvoiceCredential, sqlx::Error> {
        let key = get_encryption_key().map_err(|e| sqlx::Error::Configuration(e.into()))?;
        let enc_api_key = encryption_service::encrypt_field(&cred.api_key, &key)
            .map_err(|e| sqlx::Error::Configuration(e.into()))?;
        let enc_api_secret = encryption_service::encrypt_field(&cred.api_secret, &key)
            .map_err(|e| sqlx::Error::Configuration(e.into()))?;

        let api_key_nonce = BASE64.decode(&enc_api_key.nonce)
            .map_err(|e| sqlx::Error::Configuration(e.into()))?;
        let api_key_tag = BASE64.decode(&enc_api_key.tag)
            .map_err(|e| sqlx::Error::Configuration(e.into()))?;
        let api_secret_nonce = BASE64.decode(&enc_api_secret.nonce)
            .map_err(|e| sqlx::Error::Configuration(e.into()))?;
        let api_secret_tag = BASE64.decode(&enc_api_secret.tag)
            .map_err(|e| sqlx::Error::Configuration(e.into()))?;

        let mut result = sqlx::query_as::<_, EInvoiceCredential>(
            r#"INSERT INTO einvoice_credentials (company_id, api_key, api_secret, crypto_key, base_url, environment, api_key_nonce, api_key_tag, api_secret_nonce, api_secret_tag)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            ON CONFLICT (company_id) DO UPDATE SET
                api_key = EXCLUDED.api_key, api_secret = EXCLUDED.api_secret,
                crypto_key = EXCLUDED.crypto_key, base_url = EXCLUDED.base_url,
                environment = EXCLUDED.environment,
                api_key_nonce = EXCLUDED.api_key_nonce, api_key_tag = EXCLUDED.api_key_tag,
                api_secret_nonce = EXCLUDED.api_secret_nonce, api_secret_tag = EXCLUDED.api_secret_tag,
                updated_at = now()
            RETURNING id, company_id, api_key, api_secret, crypto_key, base_url, environment,
                is_active, last_tested_at, api_key_nonce, api_key_tag, api_secret_nonce, api_secret_tag,
                created_at, updated_at"#,
        )
        .bind(cred.company_id)
        .bind(&enc_api_key.ciphertext)
        .bind(&enc_api_secret.ciphertext)
        .bind(&cred.crypto_key)
        .bind(&cred.base_url)
        .bind(&cred.environment)
        .bind(&api_key_nonce)
        .bind(&api_key_tag)
        .bind(&api_secret_nonce)
        .bind(&api_secret_tag)
        .fetch_one(&self.pool)
        .await?;

        result.api_key = cred.api_key;
        result.api_secret = cred.api_secret;
        Ok(result)
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
