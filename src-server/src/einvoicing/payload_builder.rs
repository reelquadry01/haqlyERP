// Author: Quadri Atharu
use bigdecimal::BigDecimal;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sqlx::PgPool;
use uuid::Uuid;

use super::{
    bd_to_f64, opt_bd_to_f64, round_money, CustomerAddressRow, CustomerRow, EInvoiceProfileRow,
    InvoiceCategory, SalesInvoiceItemRow, SalesInvoiceRow,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddressPayload {
    pub line1: Option<String>,
    pub line2: Option<String>,
    pub city: Option<String>,
    pub state: Option<String>,
    pub country: Option<String>,
    pub postal_code: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SellerPayload {
    pub tin: String,
    pub legal_name: String,
    pub trade_name: Option<String>,
    pub business_email: Option<String>,
    pub business_phone: Option<String>,
    pub country_code: String,
    pub state: Option<String>,
    pub city: Option<String>,
    pub address: AddressPayload,
    pub access_point_provider_name: Option<String>,
    pub access_point_provider_code: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuyerPayload {
    pub name: String,
    pub tax_id: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub customer_type: Option<String>,
    pub address: AddressPayload,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentPayload {
    pub invoice_number: String,
    pub invoice_date: String,
    pub due_date: Option<String>,
    pub invoice_type: String,
    pub invoice_category: String,
    pub currency_code: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaxTotalEntry {
    pub rate: f64,
    pub taxable_amount: f64,
    pub tax_amount: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TotalsPayload {
    pub taxable_amount: f64,
    pub tax_amount: f64,
    pub gross_amount: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LinePayload {
    pub line_number: i32,
    pub product_id: String,
    pub sku: String,
    pub description: String,
    pub quantity: f64,
    pub unit_price: f64,
    pub tax_rate: Option<f64>,
    pub taxable_amount: f64,
    pub tax_amount: f64,
    pub line_amount: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EInvoicePayload {
    pub seller: SellerPayload,
    pub buyer: BuyerPayload,
    pub document: DocumentPayload,
    pub tax_totals: Vec<TaxTotalEntry>,
    pub totals: TotalsPayload,
    pub lines: Vec<LinePayload>,
}

pub struct EInvoicingPayloadBuilder;

impl EInvoicingPayloadBuilder {
    pub async fn build_sales_invoice_payload(
        invoice_id: Uuid,
        pool: &PgPool,
    ) -> Result<EInvoicePayload, anyhow::Error> {
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
        .ok_or_else(|| anyhow::anyhow!("Sales invoice not found: {}", invoice_id))?;

        let customer = sqlx::query_as::<_, CustomerRow>(
            r#"SELECT id, company_id, code, name, email, phone, tax_id, customer_type,
                      credit_limit, payment_terms, is_active, created_at, updated_at
               FROM customers WHERE id = $1"#,
        )
        .bind(invoice.customer_id)
        .fetch_one(pool)
        .await?;

        let address = sqlx::query_as::<_, CustomerAddressRow>(
            r#"SELECT id, customer_id, line1, line2, city, state, country_code,
                      postal_code, is_default
               FROM customer_addresses WHERE customer_id = $1 AND is_default = true
               LIMIT 1"#,
        )
        .bind(invoice.customer_id)
        .fetch_optional(pool)
        .await?;

        let items = sqlx::query_as::<_, SalesInvoiceItemRow>(
            r#"SELECT id, sales_invoice_id, line_number, product_id, sku, description,
                      quantity, unit_price, discount_percent, tax_rate, taxable_amount,
                      tax_amount, line_amount, cost_center_id, project_id
               FROM sales_invoice_items WHERE sales_invoice_id = $1
               ORDER BY line_number"#,
        )
        .bind(invoice_id)
        .fetch_all(pool)
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
        .await?
        .ok_or_else(|| anyhow::anyhow!("E-invoicing profile not found for company {}", invoice.company_id))?;

        let invoice_category = Self::resolve_invoice_category(&customer.customer_type);
        let lines = Self::build_lines(&items);
        let tax_totals = Self::build_tax_totals(&lines);
        let totals = TotalsPayload {
            taxable_amount: round_money(lines.iter().map(|l| l.taxable_amount).sum()),
            tax_amount: round_money(lines.iter().map(|l| l.tax_amount).sum()),
            gross_amount: round_money(lines.iter().map(|l| l.line_amount).sum()),
        };

        Ok(EInvoicePayload {
            seller: SellerPayload {
                tin: profile.tin,
                legal_name: profile.legal_name,
                trade_name: profile.trade_name,
                business_email: profile.business_email,
                business_phone: profile.business_phone,
                country_code: profile.country_code,
                state: profile.state,
                city: profile.city,
                address: Self::build_address(
                    Some(&profile.address_line1),
                    profile.address_line2.as_deref(),
                    profile.city.as_deref(),
                    profile.state.as_deref(),
                    Some(&profile.country_code),
                    profile.postal_code.as_deref(),
                ),
                access_point_provider_name: profile.access_point_provider_name,
                access_point_provider_code: profile.access_point_provider_code,
            },
            buyer: BuyerPayload {
                name: customer.name.clone(),
                tax_id: customer.tax_id,
                email: customer.email,
                phone: customer.phone,
                customer_type: Some(customer.customer_type.clone()),
                address: Self::build_address(
                    address.as_ref().map(|a| a.line1.as_str()),
                    address.as_ref().and_then(|a| a.line2.as_deref()),
                    address.as_ref().and_then(|a| a.city.as_deref()),
                    address.as_ref().and_then(|a| a.state.as_deref()),
                    address.as_ref().map(|a| a.country_code.as_str()),
                    address.as_ref().and_then(|a| a.postal_code.as_deref()),
                ),
            },
            document: DocumentPayload {
                invoice_number: invoice.number,
                invoice_date: invoice.date.to_string(),
                due_date: invoice.due_date.map(|d| d.to_string()),
                invoice_type: invoice.invoice_type,
                invoice_category: invoice_category.as_str().to_string(),
                currency_code: invoice.currency_code,
            },
            tax_totals,
            totals,
            lines,
        })
    }

    pub fn resolve_invoice_category(customer_type: &str) -> InvoiceCategory {
        let normalized = customer_type.trim().to_uppercase();
        match normalized.as_str() {
            "BUSINESS" | "GOVERNMENT" => InvoiceCategory::B2B,
            "INDIVIDUAL" => InvoiceCategory::B2C,
            "WALK_IN" | "RETAIL" | "CASH" => InvoiceCategory::Simplified,
            _ => InvoiceCategory::B2C,
        }
    }

    fn build_lines(items: &[SalesInvoiceItemRow]) -> Vec<LinePayload> {
        items
            .iter()
            .map(|item| {
                let quantity = bd_to_f64(&item.quantity);
                let unit_price = bd_to_f64(&item.unit_price);
                let tax_rate = item.tax_rate.as_ref().map(bd_to_f64);
                let taxable_amount = round_money(quantity * unit_price);
                let tax_amount = match tax_rate {
                    Some(rate) => round_money(taxable_amount * rate / 100.0),
                    None => 0.0,
                };
                LinePayload {
                    line_number: item.line_number,
                    product_id: item.product_id.map_or_else(|| "0".to_string(), |id| id.to_string()),
                    sku: item.sku.clone().unwrap_or_else(|| format!("ITEM-{}", item.product_id.unwrap_or_else(Uuid::nil))),
                    description: item.description.clone(),
                    quantity,
                    unit_price,
                    tax_rate,
                    taxable_amount,
                    tax_amount,
                    line_amount: round_money(taxable_amount + tax_amount),
                }
            })
            .collect()
    }

    fn build_tax_totals(lines: &[LinePayload]) -> Vec<TaxTotalEntry> {
        let mut map: std::collections::BTreeMap<u64, TaxTotalEntry> = std::collections::BTreeMap::new();
        for line in lines {
            let rate = match line.tax_rate {
                Some(r) => r,
                None => continue,
            };
            let key = (rate * 100.0).round() as u64;
            let entry = map.entry(key).or_insert_with(|| TaxTotalEntry {
                rate,
                taxable_amount: 0.0,
                tax_amount: 0.0,
            });
            entry.taxable_amount = round_money(entry.taxable_amount + line.taxable_amount);
            entry.tax_amount = round_money(entry.tax_amount + line.tax_amount);
        }
        map.into_values().collect()
    }

    fn build_address(
        line1: Option<&str>,
        line2: Option<&str>,
        city: Option<&str>,
        state: Option<&str>,
        country: Option<&str>,
        postal_code: Option<&str>,
    ) -> AddressPayload {
        AddressPayload {
            line1: line1.map(|s| s.to_string()),
            line2: line2.map(|s| s.to_string()),
            city: city.map(|s| s.to_string()),
            state: state.map(|s| s.to_string()),
            country: country.map(|s| s.to_string()),
            postal_code: postal_code.map(|s| s.to_string()),
        }
    }
}
