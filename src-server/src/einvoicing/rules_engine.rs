// Author: Quadri Atharu
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

use super::{
    CustomerRow, EInvoiceCredentialRow, EInvoiceProfileRow, InvoiceCategory, SalesInvoiceRow,
};

const B2C_SIMPLIFIED_THRESHOLD: f64 = 50_000.0;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReadinessCheck {
    pub ready: bool,
    pub missing_fields: Vec<String>,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EligibilityResult {
    pub eligible: bool,
    pub category: InvoiceCategory,
    pub reason: String,
}

pub struct EInvoicingRulesService;

impl EInvoicingRulesService {
    pub fn is_einvoice_required(
        invoice: &SalesInvoiceRow,
        profile: &EInvoiceProfileRow,
        customer_type: &str,
    ) -> EligibilityResult {
        if invoice.invoice_type != "STANDARD" {
            return EligibilityResult {
                eligible: false,
                category: InvoiceCategory::B2C,
                reason: "Only commercial/standard invoices are eligible for e-invoicing".to_string(),
            };
        }

        if profile.tin.is_empty() {
            return EligibilityResult {
                eligible: false,
                category: InvoiceCategory::B2C,
                reason: "Seller TIN is required for e-invoicing".to_string(),
            };
        }

        let category = Self::resolve_invoice_category(customer_type);
        let _ = category;

        EligibilityResult {
            eligible: true,
            category: InvoiceCategory::B2C,
            reason: String::new(),
        }
    }

    pub fn is_einvoice_required_sync(
        invoice: &SalesInvoiceRow,
        customer: &CustomerRow,
        profile: &EInvoiceProfileRow,
    ) -> EligibilityResult {
        if invoice.invoice_type != "STANDARD" {
            return EligibilityResult {
                eligible: false,
                category: InvoiceCategory::B2C,
                reason: "Only commercial/standard invoices are eligible for e-invoicing".to_string(),
            };
        }

        if profile.tin.is_empty() {
            return EligibilityResult {
                eligible: false,
                category: InvoiceCategory::B2C,
                reason: "Seller TIN is required for e-invoicing".to_string(),
            };
        }

        let category = Self::resolve_invoice_category(&customer.customer_type);

        match category {
            InvoiceCategory::B2B => EligibilityResult {
                eligible: true,
                category,
                reason: "B2B invoices are always e-invoice eligible".to_string(),
            },
            InvoiceCategory::B2C => {
                let gross = super::bd_to_f64(&invoice.total_amount);
                if gross > B2C_SIMPLIFIED_THRESHOLD {
                    EligibilityResult {
                        eligible: true,
                        category,
                        reason: format!("B2C invoice exceeds threshold ({})", B2C_SIMPLIFIED_THRESHOLD),
                    }
                } else {
                    EligibilityResult {
                        eligible: true,
                        category,
                        reason: "B2C invoice is eligible for e-invoicing".to_string(),
                    }
                }
            }
            InvoiceCategory::Simplified => {
                let gross = super::bd_to_f64(&invoice.total_amount);
                if gross > B2C_SIMPLIFIED_THRESHOLD {
                    EligibilityResult {
                        eligible: true,
                        category,
                        reason: format!("Simplified invoice exceeds threshold ({})", B2C_SIMPLIFIED_THRESHOLD),
                    }
                } else {
                    EligibilityResult {
                        eligible: false,
                        category,
                        reason: format!("Simplified invoice below threshold ({})", B2C_SIMPLIFIED_THRESHOLD),
                    }
                }
            }
        }
    }

    pub fn resolve_invoice_category(customer_type: &str) -> InvoiceCategory {
        let normalized = customer_type.trim().to_uppercase();
        match normalized.as_str() {
            "BUSINESS" | "GOVERNMENT" | "B2B" => InvoiceCategory::B2B,
            "INDIVIDUAL" | "B2C" => InvoiceCategory::B2C,
            "WALK_IN" | "RETAIL" | "CASH" => InvoiceCategory::Simplified,
            _ => InvoiceCategory::B2C,
        }
    }

    pub fn validate_readiness(
        profile: &EInvoiceProfileRow,
        credentials: &EInvoiceCredentialRow,
    ) -> ReadinessCheck {
        let mut missing = Vec::new();
        let mut warnings = Vec::new();

        if profile.tin.is_empty() {
            missing.push("seller_tin".to_string());
        }
        if profile.legal_name.is_empty() {
            missing.push("seller_legal_name".to_string());
        }
        if profile.country_code.is_empty() {
            missing.push("seller_country_code".to_string());
        }
        if !profile.is_complete {
            warnings.push("profile_not_marked_complete".to_string());
        }
        if profile.address_line1.is_empty() {
            missing.push("seller_address".to_string());
        }
        if profile.access_point_provider_name.is_none() && profile.access_point_provider_code.is_none() {
            missing.push("access_point_provider".to_string());
        }

        if credentials.api_key.is_empty() {
            missing.push("api_key".to_string());
        }
        if credentials.api_secret.is_empty() {
            missing.push("api_secret".to_string());
        }
        if credentials.base_url.is_empty() {
            missing.push("base_url".to_string());
        }
        if !credentials.is_active {
            warnings.push("credentials_not_active".to_string());
        }
        if credentials.crypto_key.is_none() {
            warnings.push("crypto_key_not_set_downloads_will_fail".to_string());
        }

        ReadinessCheck {
            ready: missing.is_empty(),
            missing_fields: missing,
            warnings,
        }
    }

    pub async fn evaluate_sales_invoice(
        invoice_id: Uuid,
        pool: &PgPool,
    ) -> Result<EvaluateResult, anyhow::Error> {
        let invoice = sqlx::query_as::<_, SalesInvoiceRow>(
            r#"SELECT id, company_id, branch_id, customer_id, number, date, due_date,
                      invoice_type, status, currency_code, exchange_rate, taxable_amount,
                      tax_amount, total_amount, amount_paid, narration, is_einvoice_eligible,
                      einvoice_irn, created_by, created_at, updated_at
               FROM sales_invoices WHERE id = $1"#,
        )
        .bind(invoice_id)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| anyhow::anyhow!("Sales invoice not found"))?;

        let customer = sqlx::query_as::<_, CustomerRow>(
            r#"SELECT id, company_id, code, name, email, phone, tax_id, customer_type,
                      credit_limit, payment_terms, is_active, created_at, updated_at
               FROM customers WHERE id = $1"#,
        )
        .bind(invoice.customer_id)
        .fetch_one(pool)
        .await?;

        let profile = sqlx::query_as::<_, EInvoiceProfileRow>(
            r#"SELECT id, company_id, tin, legal_name, trade_name, business_email,
                      business_phone, country_code, state, city, address_line1, address_line2,
                      postal_code, access_point_provider_name, access_point_provider_code,
                      default_currency_code, is_complete, created_at, updated_at
               FROM einvoice_profiles WHERE company_id = $1"#,
        )
        .bind(invoice.company_id)
        .fetch_optional(pool)
        .await?;

        let credential = sqlx::query_as::<_, EInvoiceCredentialRow>(
            r#"SELECT id, company_id, api_key, api_secret, crypto_key, base_url,
                      environment, is_active, last_tested_at, created_at, updated_at
               FROM einvoice_credentials WHERE company_id = $1 AND is_active = true
               ORDER BY created_at DESC LIMIT 1"#,
        )
        .bind(invoice.company_id)
        .fetch_optional(pool)
        .await?;

        let category = Self::resolve_invoice_category(&customer.customer_type);

        let mut checks = Vec::new();
        let mut missing = Vec::new();

        checks.push(("commercial_invoice".to_string(), invoice.invoice_type == "STANDARD"));
        checks.push(("seller_tin".to_string(), profile.as_ref().map_or(false, |p| !p.tin.is_empty())));
        checks.push(("seller_legal_name".to_string(), profile.as_ref().map_or(false, |p| !p.legal_name.is_empty())));
        checks.push(("seller_country".to_string(), profile.as_ref().map_or(false, |p| !p.country_code.is_empty())));
        checks.push(("invoice_number".to_string(), !invoice.number.is_empty()));
        checks.push(("invoice_date".to_string(), true));
        checks.push(("invoice_total".to_string(), super::bd_to_f64(&invoice.total_amount) != 0.0));
        checks.push(("customer_name".to_string(), !customer.name.is_empty()));
        checks.push(("api_key_present".to_string(), credential.as_ref().map_or(false, |c| !c.api_key.is_empty())));

        if category == InvoiceCategory::B2B {
            checks.push(("buyer_tax_id".to_string(), customer.tax_id.as_ref().map_or(false, |t| !t.is_empty())));
        }

        for (key, passed) in &checks {
            if !passed {
                missing.push(key.to_string());
            }
        }

        Ok(EvaluateResult {
            sales_invoice_id: invoice.id,
            invoice_number: invoice.number,
            customer_id: invoice.customer_id,
            customer_type: Some(customer.customer_type),
            invoice_category: Some(category),
            ready: missing.is_empty(),
            checks,
            missing,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvaluateResult {
    pub sales_invoice_id: Uuid,
    pub invoice_number: String,
    pub customer_id: Uuid,
    pub customer_type: Option<String>,
    pub invoice_category: Option<InvoiceCategory>,
    pub ready: bool,
    pub checks: Vec<(String, bool)>,
    pub missing: Vec<String>,
}
