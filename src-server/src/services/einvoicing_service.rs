// Author: Quadri Atharu
use anyhow::{anyhow, Result};
use base64::engine::general_purpose::STANDARD as BASE64;
use base64::Engine;
use sqlx::PgPool;
use uuid::Uuid;

use crate::dtos::einvoice_dto::{EInvoiceReadinessCheck, SaveCredentialsRequest, SaveProfileRequest, SubmitEInvoiceRequest};
use crate::models::einvoice::{
    EInvoiceCredential, EInvoiceDocument, EInvoiceProfile, EInvoiceStatus, InvoiceCategory,
};
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

#[derive(Clone)]
pub struct EInvoicingService {
    pub pool: PgPool,
}

impl EInvoicingService {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn save_profile(&self, req: SaveProfileRequest) -> Result<EInvoiceProfile> {
        let id = Uuid::now_v7();
        sqlx::query(
            r#"INSERT INTO einvoice_profiles (id, company_id, business_id, business_name, tax_id, integration_type, api_base_url, is_active, created_at, updated_at)
               VALUES ($1, $2, $3, $4, $5, $6, $7, true, NOW(), NOW())
               ON CONFLICT (company_id) DO UPDATE SET business_id = $3, business_name = $4, tax_id = $5, integration_type = $6, api_base_url = $7, updated_at = NOW()"#,
        )
        .bind(id)
        .bind(req.company_id)
        .bind(&req.business_id)
        .bind(&req.business_name)
        .bind(&req.tax_id)
        .bind(&req.integration_type)
        .bind(&req.api_base_url)
        .execute(&self.pool)
        .await?;

        sqlx::query_as::<_, EInvoiceProfile>("SELECT * FROM einvoice_profiles WHERE company_id = $1")
            .bind(req.company_id)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| anyhow!("Failed to fetch e-invoice profile: {}", e))
    }

    pub async fn save_credentials(&self, req: SaveCredentialsRequest) -> Result<EInvoiceCredential> {
        let key = get_encryption_key()?;
        let encrypted = encryption_service::encrypt_field(&req.client_secret, &key)
            .map_err(|e| anyhow!("Encryption failed: {}", e))?;
        let nonce_bytes = BASE64.decode(&encrypted.nonce)
            .map_err(|e| anyhow!("Invalid nonce: {}", e))?;
        let tag_bytes = BASE64.decode(&encrypted.tag)
            .map_err(|e| anyhow!("Invalid tag: {}", e))?;

        let id = Uuid::now_v7();
        sqlx::query(
            r#"INSERT INTO einvoice_credentials (id, profile_id, company_id, client_id, client_secret_encrypted, client_secret_nonce, client_secret_tag, certificate_path, is_active, created_at, updated_at)
               VALUES ($1, $2, $3, $4, $5, $6, $7, $8, true, NOW(), NOW())"#,
        )
        .bind(id)
        .bind(req.profile_id)
        .bind(req.company_id)
        .bind(&req.client_id)
        .bind(&encrypted.ciphertext)
        .bind(&nonce_bytes)
        .bind(&tag_bytes)
        .bind(&req.certificate_path)
        .execute(&self.pool)
        .await?;

        sqlx::query_as::<_, EInvoiceCredential>("SELECT * FROM einvoice_credentials WHERE id = $1")
            .bind(id)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| anyhow!("Failed to fetch credentials: {}", e))
    }

    pub async fn check_readiness(&self, company_id: Uuid) -> Result<EInvoiceReadinessCheck> {
        let profile = sqlx::query_as::<_, EInvoiceProfile>(
            "SELECT * FROM einvoice_profiles WHERE company_id = $1 AND is_active = true",
        )
        .bind(company_id)
        .fetch_optional(&self.pool)
        .await?;

        let credentials = if let Some(ref p) = profile {
            sqlx::query_as::<_, EInvoiceCredential>(
                "SELECT * FROM einvoice_credentials WHERE profile_id = $1 AND is_active = true LIMIT 1",
            )
            .bind(p.id)
            .fetch_optional(&self.pool)
            .await?
        } else {
            None
        };

        let profile_configured = profile.is_some();
        let credentials_valid = credentials.is_some();
        let api_reachable = if let Some(ref p) = profile {
            if let Some(ref url) = p.api_base_url {
                reqwest::Client::new()
                    .get(format!("{}/health", url))
                    .timeout(std::time::Duration::from_secs(5))
                    .send()
                    .await
                    .is_ok()
            } else {
                false
            }
        } else {
            false
        };

        let mut issues = Vec::new();
        if !profile_configured {
            issues.push("E-invoice profile not configured".to_string());
        }
        if !credentials_valid {
            issues.push("API credentials not configured".to_string());
        }
        if !api_reachable {
            issues.push("API is not reachable".to_string());
        }

        let ready = profile_configured && credentials_valid;

        Ok(EInvoiceReadinessCheck {
            profile_configured,
            credentials_valid,
            api_reachable,
            ready,
            issues,
        })
    }

    pub async fn submit_invoice(&self, req: SubmitEInvoiceRequest) -> Result<EInvoiceDocument> {
        let profile = sqlx::query_as::<_, EInvoiceProfile>(
            "SELECT * FROM einvoice_profiles WHERE company_id = $1 AND is_active = true",
        )
        .bind(req.company_id)
        .fetch_one(&self.pool)
        .await?;

        let category = match req.category.as_str() {
            "b2b" => InvoiceCategory::B2b,
            "b2c" => InvoiceCategory::B2c,
            _ => InvoiceCategory::Simplified,
        };

        let id = Uuid::now_v7();
        let payload = serde_json::json!({
            "seller": {
                "name": req.seller_name,
                "tax_id": req.seller_tax_id,
                "business_id": req.seller_business_id,
                "address": {
                    "line1": req.seller_address_line1,
                    "city": req.seller_city,
                    "state": req.seller_state,
                    "country": req.seller_country,
                }
            },
            "buyer": {
                "name": req.buyer_name,
                "tax_id": req.buyer_tax_id,
                "address": {
                    "line1": req.buyer_address_line1,
                    "city": req.buyer_city,
                    "state": req.buyer_state,
                    "country": req.buyer_country,
                }
            },
            "document": {
                "invoice_number": req.invoice_number,
                "invoice_date": req.invoice_date,
                "currency": req.currency,
            },
            "lines": req.lines,
        });

        sqlx::query(
            r#"INSERT INTO einvoice_documents (id, company_id, invoice_id, category, status, payload_json, created_by, created_at, updated_at)
               VALUES ($1, $2, $3, $4, 'pending', $5, $6, NOW(), NOW())"#,
        )
        .bind(id)
        .bind(req.company_id)
        .bind(req.invoice_id)
        .bind(&category)
        .bind(&payload)
        .bind(Uuid::nil())
        .execute(&self.pool)
        .await?;

        if let Some(ref url) = profile.api_base_url {
            let credentials = sqlx::query_as::<_, EInvoiceCredential>(
                "SELECT * FROM einvoice_credentials WHERE profile_id = $1 AND is_active = true LIMIT 1",
            )
            .bind(profile.id)
            .fetch_optional(&self.pool)
            .await?;

            if let Some(cred) = credentials {
                let key = get_encryption_key()?;
                let nonce_b64 = cred.client_secret_nonce
                    .as_ref()
                    .map(|n| BASE64.encode(n))
                    .unwrap_or_default();
                let tag_b64 = cred.client_secret_tag
                    .as_ref()
                    .map(|t| BASE64.encode(t))
                    .unwrap_or_default();
                let decrypted_secret = encryption_service::decrypt_field(
                    &cred.client_secret_encrypted, &key, &nonce_b64, &tag_b64,
                ).map_err(|e| anyhow!("Decryption failed: {}", e))?;

                let client = reqwest::Client::new();
                let response = client
                    .post(format!("{}/invoices", url))
                    .header("Authorization", format!("Bearer {}:{}", cred.client_id, decrypted_secret))
                    .json(&payload)
                    .timeout(std::time::Duration::from_secs(30))
                    .send()
                    .await;

                match response {
                    Ok(resp) => {
                        let status_code = resp.status().as_u16();
                        let body = resp.text().await.unwrap_or_default();

                        if status_code >= 200 && status_code < 300 {
                            let response_json: serde_json::Value = serde_json::from_str(&body).unwrap_or(serde_json::Value::Null);
                            let irn = response_json.get("irn").and_then(|v| v.as_str()).map(|s| s.to_string());

                            sqlx::query(
                                r#"UPDATE einvoice_documents SET status = 'submitted', response_json = $1, irn = $2, submitted_at = NOW(), updated_at = NOW() WHERE id = $3"#,
                            )
                            .bind(&serde_json::from_str::<serde_json::Value>(&body).unwrap_or(serde_json::Value::Null))
                            .bind(&irn)
                            .bind(id)
                            .execute(&self.pool)
                            .await?;
                        } else {
                            sqlx::query(
                                r#"UPDATE einvoice_documents SET status = 'rejected', error_message = $1, response_json = $2, updated_at = NOW() WHERE id = $3"#,
                            )
                            .bind(format!("HTTP {}: {}", status_code, &body[..body.len().min(500)]))
                            .bind(serde_json::from_str::<serde_json::Value>(&body).unwrap_or(serde_json::Value::Null))
                            .bind(id)
                            .execute(&self.pool)
                            .await?;
                        }
                    }
                    Err(e) => {
                        sqlx::query(
                            r#"UPDATE einvoice_documents SET status = 'rejected', error_message = $1, updated_at = NOW() WHERE id = $2"#,
                        )
                        .bind(e.to_string())
                        .bind(id)
                        .execute(&self.pool)
                        .await?;
                    }
                }
            }
        }

        sqlx::query_as::<_, EInvoiceDocument>("SELECT * FROM einvoice_documents WHERE id = $1")
            .bind(id)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| anyhow!("Failed to fetch e-invoice document: {}", e))
    }

    pub async fn get_document_status(&self, document_id: Uuid) -> Result<EInvoiceDocument> {
        sqlx::query_as::<_, EInvoiceDocument>("SELECT * FROM einvoice_documents WHERE id = $1")
            .bind(document_id)
            .fetch_optional(&self.pool)
            .await?
            .ok_or_else(|| anyhow!("E-invoice document not found"))
    }

    pub async fn confirm_invoice(&self, document_id: Uuid) -> Result<EInvoiceDocument> {
        let doc = self.get_document_status(document_id).await?;

        if doc.status != EInvoiceStatus::Submitted && doc.status != EInvoiceStatus::Validated {
            return Err(anyhow!("Document must be in submitted or validated status to confirm"));
        }

        sqlx::query(
            "UPDATE einvoice_documents SET status = 'validated', validated_at = NOW(), updated_at = NOW() WHERE id = $1",
        )
        .bind(document_id)
        .execute(&self.pool)
        .await?;

        self.get_document_status(document_id).await
    }

    pub async fn download_invoice(&self, document_id: Uuid) -> Result<EInvoiceDocument> {
        let doc = self.get_document_status(document_id).await?;

        if doc.status != EInvoiceStatus::Validated {
            return Err(anyhow!("Invoice must be validated before download"));
        }

        Ok(doc)
    }

    pub async fn orchestrate_einvoice_flow(&self, req: SubmitEInvoiceRequest) -> Result<EInvoiceDocument> {
        let readiness = self.check_readiness(req.company_id).await?;
        if !readiness.ready {
            return Err(anyhow!("E-invoicing not ready: {}", readiness.issues.join(", ")));
        }

        let doc = self.submit_invoice(req).await?;

        if doc.status == EInvoiceStatus::Submitted {
            if let Some(ref irn) = doc.irn {
                let _ = irn;
            }
        }

        Ok(doc)
    }
}
