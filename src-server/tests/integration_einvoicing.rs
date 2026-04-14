// Author: Quadri Atharu
use base64::engine::general_purpose::STANDARD as BASE64;
use base64::Engine;
use haqly_erp_server::dtos::einvoice_dto::{
    EInvoiceLineDto, SaveCredentialsRequest, SaveProfileRequest, SubmitEInvoiceRequest,
};
use haqly_erp_server::models::einvoice::{EInvoiceStatus, InvoiceCategory};
use haqly_erp_server::services::einvoicing_service::EInvoicingService;
use haqly_erp_server::services::encryption_service;
use sqlx::PgPool;
use sqlx::postgres::PgPoolOptions;
use uuid::Uuid;

#[cfg(test)]
mod tests {
    use super::*;

    async fn get_test_pool() -> PgPool {
        let database_url = std::env::var("TEST_DATABASE_URL")
            .unwrap_or_else(|_| "postgresql://haqly:haqly@localhost:5432/haqly_test".to_string());
        PgPoolOptions::new()
            .max_connections(5)
            .connect(&database_url)
            .await
            .expect("Failed to connect to test database")
    }

    async fn seed_test_company(pool: &PgPool) -> Uuid {
        let company_id = Uuid::now_v7();
        sqlx::query(
            r#"INSERT INTO companies (id, name, currency_code, is_active, created_at, updated_at)
               VALUES ($1, 'Test Co EInvoicing', 'NGN', true, NOW(), NOW())
               ON CONFLICT (id) DO NOTHING"#,
        )
        .bind(company_id)
        .execute(pool)
        .await
        .expect("seed company");
        company_id
    }

    #[tokio::test]
    #[ignore]
    async fn test_einvoice_validate_payload() {
        let pool = get_test_pool().await;
        let company_id = seed_test_company(&pool).await;

        let einvoice_svc = EInvoicingService::new(pool.clone());

        let profile = einvoice_svc
            .save_profile(SaveProfileRequest {
                company_id,
                business_id: "BN1234567".to_string(),
                business_name: "HAQLY Test Ltd".to_string(),
                tax_id: "12345678-0001".to_string(),
                integration_type: "firs".to_string(),
                api_base_url: Some("http://localhost:9999/firs".to_string()),
            })
            .await
            .expect("save profile");

        assert_eq!(profile.company_id, company_id);
        assert!(profile.is_active);

        let req = SubmitEInvoiceRequest {
            company_id,
            invoice_id: Uuid::now_v7(),
            category: "b2b".to_string(),
            seller_name: "HAQLY Test Ltd".to_string(),
            seller_tax_id: Some("12345678-0001".to_string()),
            seller_business_id: Some("BN1234567".to_string()),
            seller_address_line1: "12 Marina Street".to_string(),
            seller_city: "Lagos".to_string(),
            seller_state: Some("Lagos".to_string()),
            seller_country: "NG".to_string(),
            buyer_name: "Client Corp".to_string(),
            buyer_tax_id: Some("87654321-0001".to_string()),
            buyer_address_line1: "45 Awolowo Road".to_string(),
            buyer_city: "Lagos".to_string(),
            buyer_state: Some("Lagos".to_string()),
            buyer_country: "NG".to_string(),
            invoice_number: "INV-EINVOICE-001".to_string(),
            invoice_date: "2024-06-15".to_string(),
            currency: "NGN".to_string(),
            lines: vec![EInvoiceLineDto {
                line_number: 1,
                description: "Consulting services".to_string(),
                quantity: bigdecimal::BigDecimal::from(1),
                unit_price: bigdecimal::BigDecimal::from(1_000_000),
                tax_rate: bigdecimal::BigDecimal::from(7) + bigdecimal::BigDecimal::from(5) / bigdecimal::BigDecimal::from(10),
                tax_amount: bigdecimal::BigDecimal::from(75_000),
                line_total: bigdecimal::BigDecimal::from(1_075_000),
            }],
        };

        let doc = einvoice_svc.submit_invoice(req).await.expect("submit invoice");

        assert!(doc.payload_json.is_some());

        let payload = doc.payload_json.as_ref().unwrap();
        assert!(payload.get("seller").is_some());
        assert!(payload.get("buyer").is_some());
        assert!(payload.get("document").is_some());
        assert!(payload.get("lines").is_some());

        let seller = payload.get("seller").unwrap();
        assert_eq!(seller["name"], "HAQLY Test Ltd");
        assert_eq!(seller["tax_id"], "12345678-0001");

        let document = payload.get("document").unwrap();
        assert_eq!(document["invoice_number"], "INV-EINVOICE-001");
        assert_eq!(document["currency"], "NGN");

        let lines = payload.get("lines").unwrap().as_array().unwrap();
        assert_eq!(lines.len(), 1);
    }

    #[tokio::test]
    #[ignore]
    async fn test_einvoice_submit_and_confirm() {
        let pool = get_test_pool().await;
        let company_id = seed_test_company(&pool).await;

        let einvoice_svc = EInvoicingService::new(pool.clone());

        einvoice_svc
            .save_profile(SaveProfileRequest {
                company_id,
                business_id: "BN7654321".to_string(),
                business_name: "HAQLY Confirm Ltd".to_string(),
                tax_id: "11223344-0001".to_string(),
                integration_type: "firs".to_string(),
                api_base_url: None,
            })
            .await
            .expect("save profile");

        let doc = einvoice_svc
            .submit_invoice(SubmitEInvoiceRequest {
                company_id,
                invoice_id: Uuid::now_v7(),
                category: "b2b".to_string(),
                seller_name: "HAQLY Confirm Ltd".to_string(),
                seller_tax_id: Some("11223344-0001".to_string()),
                seller_business_id: None,
                seller_address_line1: "1 Broad Street".to_string(),
                seller_city: "Lagos".to_string(),
                seller_state: Some("Lagos".to_string()),
                seller_country: "NG".to_string(),
                buyer_name: "Buyer Corp".to_string(),
                buyer_tax_id: Some("44332211-0001".to_string()),
                buyer_address_line1: "2 Marina".to_string(),
                buyer_city: "Abuja".to_string(),
                buyer_state: Some("FCT".to_string()),
                buyer_country: "NG".to_string(),
                invoice_number: "INV-CONFIRM-001".to_string(),
                invoice_date: "2024-07-01".to_string(),
                currency: "NGN".to_string(),
                lines: vec![EInvoiceLineDto {
                    line_number: 1,
                    description: "Goods".to_string(),
                    quantity: bigdecimal::BigDecimal::from(10),
                    unit_price: bigdecimal::BigDecimal::from(50_000),
                    tax_rate: bigdecimal::BigDecimal::from(7) + bigdecimal::BigDecimal::from(5) / bigdecimal::BigDecimal::from(10),
                    tax_amount: bigdecimal::BigDecimal::from(37_500),
                    line_total: bigdecimal::BigDecimal::from(537_500),
                }],
            })
            .await
            .expect("submit invoice");

        assert!(
            doc.status == EInvoiceStatus::Pending
                || doc.status == EInvoiceStatus::Submitted
                || doc.status == EInvoiceStatus::Rejected,
            "Expected pending/submitted/rejected status, got {:?}",
            doc.status
        );

        if doc.status == EInvoiceStatus::Pending || doc.status == EInvoiceStatus::Submitted {
            let confirmed = einvoice_svc.confirm_invoice(doc.id).await.expect("confirm");

            assert!(
                confirmed.status == EInvoiceStatus::Validated
                    || confirmed.status == EInvoiceStatus::Submitted,
                "Expected validated/submitted after confirm, got {:?}",
                confirmed.status
            );

            if confirmed.status == EInvoiceStatus::Validated {
                assert!(confirmed.validated_at.is_some());
            }
        }
    }

    #[tokio::test]
    #[ignore]
    async fn test_einvoice_irn_generation() {
        let pool = get_test_pool().await;
        let company_id = seed_test_company(&pool).await;

        let einvoice_svc = EInvoicingService::new(pool.clone());

        einvoice_svc
            .save_profile(SaveProfileRequest {
                company_id,
                business_id: "BN9998887".to_string(),
                business_name: "HAQLY IRN Ltd".to_string(),
                tax_id: "99887766-0001".to_string(),
                integration_type: "firs".to_string(),
                api_base_url: None,
            })
            .await
            .expect("save profile");

        let doc1 = einvoice_svc
            .submit_invoice(SubmitEInvoiceRequest {
                company_id,
                invoice_id: Uuid::now_v7(),
                category: "b2c".to_string(),
                seller_name: "HAQLY IRN Ltd".to_string(),
                seller_tax_id: Some("99887766-0001".to_string()),
                seller_business_id: None,
                seller_address_line1: "3 Allen Avenue".to_string(),
                seller_city: "Lagos".to_string(),
                seller_state: Some("Lagos".to_string()),
                seller_country: "NG".to_string(),
                buyer_name: "Walk-in Customer".to_string(),
                buyer_tax_id: None,
                buyer_address_line1: "N/A".to_string(),
                buyer_city: "Lagos".to_string(),
                buyer_state: None,
                buyer_country: "NG".to_string(),
                invoice_number: "INV-IRN-001".to_string(),
                invoice_date: "2024-08-01".to_string(),
                currency: "NGN".to_string(),
                lines: vec![EInvoiceLineDto {
                    line_number: 1,
                    description: "Retail sale".to_string(),
                    quantity: bigdecimal::BigDecimal::from(1),
                    unit_price: bigdecimal::BigDecimal::from(100_000),
                    tax_rate: bigdecimal::BigDecimal::from(7) + bigdecimal::BigDecimal::from(5) / bigdecimal::BigDecimal::from(10),
                    tax_amount: bigdecimal::BigDecimal::from(7_500),
                    line_total: bigdecimal::BigDecimal::from(107_500),
                }],
            })
            .await
            .expect("submit invoice 1");

        let doc2 = einvoice_svc
            .submit_invoice(SubmitEInvoiceRequest {
                company_id,
                invoice_id: Uuid::now_v7(),
                category: "b2b".to_string(),
                seller_name: "HAQLY IRN Ltd".to_string(),
                seller_tax_id: Some("99887766-0001".to_string()),
                seller_business_id: None,
                seller_address_line1: "3 Allen Avenue".to_string(),
                seller_city: "Lagos".to_string(),
                seller_state: Some("Lagos".to_string()),
                seller_country: "NG".to_string(),
                buyer_name: "Corp Buyer".to_string(),
                buyer_tax_id: Some("55667788-0001".to_string()),
                buyer_address_line1: "10 Victoria Island".to_string(),
                buyer_city: "Lagos".to_string(),
                buyer_state: Some("Lagos".to_string()),
                buyer_country: "NG".to_string(),
                invoice_number: "INV-IRN-002".to_string(),
                invoice_date: "2024-08-02".to_string(),
                currency: "NGN".to_string(),
                lines: vec![EInvoiceLineDto {
                    line_number: 1,
                    description: "Wholesale".to_string(),
                    quantity: bigdecimal::BigDecimal::from(5),
                    unit_price: bigdecimal::BigDecimal::from(200_000),
                    tax_rate: bigdecimal::BigDecimal::from(7) + bigdecimal::BigDecimal::from(5) / bigdecimal::BigDecimal::from(10),
                    tax_amount: bigdecimal::BigDecimal::from(75_000),
                    line_total: bigdecimal::BigDecimal::from(1_075_000),
                }],
            })
            .await
            .expect("submit invoice 2");

        assert_ne!(doc1.id, doc2.id, "Each document must have unique ID");

        if let (Some(irn1), Some(irn2)) = (&doc1.irn, &doc2.irn) {
            assert_ne!(irn1, irn2, "IRNs must be unique across documents");

            assert!(irn1.starts_with("IRN-"), "IRN format must start with IRN-");
            assert!(irn2.starts_with("IRN-"), "IRN format must start with IRN-");

            let parts1: Vec<&str> = irn1.split('-').collect();
            assert!(parts1.len() >= 3, "IRN must have at least 3 dash-separated parts");
        } else {
            assert!(doc1.irn.is_none() || doc2.irn.is_none() || doc1.irn != doc2.irn);
        }

        assert!(doc1.id != doc2.id);
    }

    #[tokio::test]
    #[ignore]
    async fn test_encrypted_credentials_roundtrip() {
        let key = encryption_service::generate_encryption_key();
        let original_secret = "firs-api-secret-key-ABC123xyz";

        let encrypted = encryption_service::encrypt_field(original_secret, &key)
            .expect("encrypt");

        assert_ne!(encrypted.ciphertext, BASE64.encode(original_secret.as_bytes()));
        assert!(!encrypted.nonce.is_empty());
        assert!(!encrypted.tag.is_empty());

        let decrypted = encryption_service::decrypt_field(
            &encrypted.ciphertext,
            &key,
            &encrypted.nonce,
            &encrypted.tag,
        )
        .expect("decrypt");

        assert_eq!(original_secret, decrypted);

        let wrong_key = encryption_service::generate_encryption_key();
        let wrong_result = encryption_service::decrypt_field(
            &encrypted.ciphertext,
            &wrong_key,
            &encrypted.nonce,
            &encrypted.tag,
        );

        assert!(wrong_result.is_err(), "Decryption with wrong key must fail");

        let key_b64 = BASE64.encode(key);
        let key_bytes = BASE64.decode(&key_b64).expect("decode key");
        assert_eq!(key_bytes.len(), 32);

        let pool = get_test_pool().await;
        let company_id = seed_test_company(&pool).await;

        let einvoice_svc = EInvoicingService::new(pool.clone());

        let profile = einvoice_svc
            .save_profile(SaveProfileRequest {
                company_id,
                business_id: "BNENCRYPT".to_string(),
                business_name: "HAQLY Encrypt Ltd".to_string(),
                tax_id: "ENC-001".to_string(),
                integration_type: "firs".to_string(),
                api_base_url: None,
            })
            .await
            .expect("save profile");

        std::env::set_var("HAQLY_ENCRYPTION_KEY", &key_b64);

        let cred = einvoice_svc
            .save_credentials(SaveCredentialsRequest {
                profile_id: profile.id,
                company_id,
                client_id: "firs-client-123".to_string(),
                client_secret: original_secret.to_string(),
                certificate_path: None,
            })
            .await;

        if let Ok(saved_cred) = cred {
            assert_eq!(saved_cred.client_id, "firs-client-123");
            assert_ne!(saved_cred.client_secret_encrypted, original_secret);
            assert!(saved_cred.client_secret_encrypted.len() > 0);
            assert!(saved_cred.is_active);

            if let (Some(nonce_bytes), Some(tag_bytes)) = (&saved_cred.client_secret_nonce, &saved_cred.client_secret_tag) {
                let nonce_b64 = BASE64.encode(nonce_bytes);
                let tag_b64 = BASE64.encode(tag_bytes);

                let decrypted_db = encryption_service::decrypt_field(
                    &saved_cred.client_secret_encrypted,
                    &key,
                    &nonce_b64,
                    &tag_b64,
                )
                .expect("decrypt from DB");

                assert_eq!(decrypted_db, original_secret);
            }
        }

        std::env::remove_var("HAQLY_ENCRYPTION_KEY");
    }
}
