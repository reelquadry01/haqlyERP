// Author: Quadri Atharu
use anyhow::{anyhow, Result};
use bigdecimal::BigDecimal;
use num_traits::Num;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

use crate::dtos::sales_dto::{
    CreateCreditNoteRequest, CreateCustomerRequest, CreateInvoiceRequest, CreateReceiptRequest,
    RecordPaymentRequest,
};
use crate::models::posting::PostingContext;
use crate::models::sales::{
    Customer, CustomerReceipt, InvoiceType, PaymentMethod, ProformaInvoice, SalesInvoice,
    SalesInvoiceItem,
};
use crate::services::posting_service::PostingService;

const NGN_VAT_RATE: &str = "7.5";

#[derive(Clone)]
pub struct SalesService {
    pub pool: PgPool,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct InvoiceAgingBucket {
    pub bucket: String,
    pub invoice_count: i64,
    pub total_outstanding: BigDecimal,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct RevenueSummary {
    pub total_revenue: BigDecimal,
    pub net_revenue: BigDecimal,
    pub total_vat: BigDecimal,
    pub invoice_count: i64,
    pub vat_rate_used: String,
}

impl SalesService {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn create_customer(&self, req: CreateCustomerRequest) -> Result<Customer> {
        let id = Uuid::now_v7();
        let code = req.code.unwrap_or_else(|| format!("CUST-{}", Uuid::now_v7().as_simple().to_string().chars().take(8).collect::<String>()));
        let currency = req.currency_code.unwrap_or_else(|| "NGN".to_string());

        sqlx::query(
            r#"INSERT INTO customers (id, company_id, code, name, email, phone, tax_identification_number, rc_number, contact_person, credit_limit, outstanding_balance, currency_code, is_active, branch_id, created_at, updated_at)
               VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, 0, $11, true, $12, NOW(), NOW())"#,
        )
        .bind(id)
        .bind(req.company_id)
        .bind(&code)
        .bind(&req.name)
        .bind(&req.email)
        .bind(&req.phone)
        .bind(&req.tax_identification_number)
        .bind(&req.rc_number)
        .bind(&req.contact_person)
        .bind(&req.credit_limit)
        .bind(&currency)
        .bind(req.branch_id)
        .execute(&self.pool)
        .await?;

        if let Some(addr) = req.address {
            let addr_id = Uuid::now_v7();
            sqlx::query(
                r#"INSERT INTO customer_addresses (id, customer_id, address_type, line1, line2, city, state, country, postal_code, is_default, created_at)
                   VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, NOW())"#,
            )
            .bind(addr_id)
            .bind(id)
            .bind(&addr.address_type)
            .bind(&addr.line1)
            .bind(&addr.line2)
            .bind(&addr.city)
            .bind(&addr.state)
            .bind(&addr.country)
            .bind(&addr.postal_code)
            .bind(addr.is_default)
            .execute(&self.pool)
            .await?;
        }

        sqlx::query_as::<_, Customer>("SELECT * FROM customers WHERE id = $1")
            .bind(id)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| anyhow!("Failed to fetch customer: {}", e))
    }

    pub async fn list_customers(&self, company_id: Uuid) -> Result<Vec<Customer>> {
        let customers = sqlx::query_as::<_, Customer>(
            "SELECT * FROM customers WHERE company_id = $1 AND is_active = true ORDER BY name",
        )
        .bind(company_id)
        .fetch_all(&self.pool)
        .await?;
        Ok(customers)
    }

    pub async fn create_invoice(
        &self,
        req: CreateInvoiceRequest,
        created_by: Uuid,
    ) -> Result<SalesInvoice> {
        let id = Uuid::now_v7();
        let invoice_number = self.generate_invoice_number(req.company_id).await?;
        let currency = req.currency_code.unwrap_or_else(|| "NGN".to_string());

        let mut subtotal = BigDecimal::from(0);
        let mut total_tax = BigDecimal::from(0);
        let mut total_discount = BigDecimal::from(0);

        let mut line_items = Vec::new();
        for (i, item) in req.items.iter().enumerate() {
            let line_total_before_discount = &item.quantity * &item.unit_price;
            let discount = match &item.discount_percent {
                Some(pct) => &line_total_before_discount * pct / BigDecimal::from(100),
                None => BigDecimal::from(0),
            };
            let net = &line_total_before_discount - &discount;
            let tax_rate = item.tax_rate.clone().unwrap_or_else(|| BigDecimal::from_str_radix(NGN_VAT_RATE, 10).unwrap_or(BigDecimal::from(0)));
            let tax_amount = &net * &tax_rate / BigDecimal::from(100);
            let line_total = &net + &tax_amount;

            subtotal += net.clone();
            total_tax += tax_amount.clone();
            total_discount += discount.clone();

            line_items.push(SalesInvoiceItem {
                id: Uuid::now_v7(),
                invoice_id: id,
                product_id: item.product_id,
                line_number: i as i32 + 1,
                description: item.description.clone(),
                quantity: item.quantity.clone(),
                unit_price: item.unit_price.clone(),
                discount_percent: item.discount_percent.clone(),
                tax_rate: Some(tax_rate),
                tax_amount: tax_amount.clone(),
                line_total: line_total.clone(),
                cost_center_id: item.cost_center_id,
                created_at: chrono::Utc::now().naive_utc(),
            });
        }

        let total_amount = &subtotal + &total_tax;

        sqlx::query(
            r#"INSERT INTO sales_invoices (id, company_id, branch_id, customer_id, invoice_number, invoice_type, invoice_date, due_date, subtotal, tax_amount, discount_amount, total_amount, amount_paid, currency_code, status, linked_invoice_id, narration, created_by, created_at, updated_at)
               VALUES ($1, $2, $3, $4, $5, 'standard', $6, $7, $8, $9, $10, $11, 0, $12, 'draft', NULL, $13, $14, NOW(), NOW())"#,
        )
        .bind(id)
        .bind(req.company_id)
        .bind(req.branch_id)
        .bind(req.customer_id)
        .bind(&invoice_number)
        .bind(&req.invoice_date)
        .bind(&req.due_date)
        .bind(&subtotal)
        .bind(&total_tax)
        .bind(&total_discount)
        .bind(&total_amount)
        .bind(&currency)
        .bind(&req.narration)
        .bind(created_by)
        .execute(&self.pool)
        .await?;

        for item in &line_items {
            sqlx::query(
                r#"INSERT INTO sales_invoice_items (id, invoice_id, product_id, line_number, description, quantity, unit_price, discount_percent, tax_rate, tax_amount, line_total, cost_center_id, created_at)
                   VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)"#,
            )
            .bind(item.id)
            .bind(item.invoice_id)
            .bind(item.product_id)
            .bind(item.line_number)
            .bind(&item.description)
            .bind(&item.quantity)
            .bind(&item.unit_price)
            .bind(&item.discount_percent)
            .bind(&item.tax_rate)
            .bind(&item.tax_amount)
            .bind(&item.line_total)
            .bind(item.cost_center_id)
            .bind(item.created_at)
            .execute(&self.pool)
            .await?;
        }

        sqlx::query_as::<_, SalesInvoice>("SELECT * FROM sales_invoices WHERE id = $1")
            .bind(id)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| anyhow!("Failed to fetch invoice: {}", e))
    }

    pub async fn send_invoice(&self, id: Uuid) -> Result<SalesInvoice> {
        let invoice = sqlx::query_as::<_, SalesInvoice>("SELECT * FROM sales_invoices WHERE id = $1")
            .bind(id)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| anyhow!("Invoice not found: {}", e))?;

        if invoice.status != "draft" {
            return Err(anyhow!("Invoice must be in Draft status to send, current: {}", invoice.status));
        }

        sqlx::query("UPDATE sales_invoices SET status = 'sent', updated_at = NOW() WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;

        sqlx::query_as::<_, SalesInvoice>("SELECT * FROM sales_invoices WHERE id = $1")
            .bind(id)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| anyhow!("Failed to fetch invoice: {}", e))
    }

    pub async fn record_payment(
        &self,
        req: RecordPaymentRequest,
        created_by: Uuid,
    ) -> Result<CustomerReceipt> {
        let invoice = sqlx::query_as::<_, SalesInvoice>("SELECT * FROM sales_invoices WHERE id = $1")
            .bind(req.invoice_id)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| anyhow!("Invoice not found: {}", e))?;

        if invoice.status != "sent" && invoice.status != "partial" {
            return Err(anyhow!("Invoice must be Sent or Partial to record payment, current: {}", invoice.status));
        }

        let new_amount_paid = &invoice.amount_paid + &req.amount;
        let new_status = if new_amount_paid >= invoice.total_amount {
            "paid"
        } else {
            "partial"
        };

        let receipt_id = Uuid::now_v7();
        let receipt_number = self.generate_receipt_number(invoice.company_id).await?;
        let receipt_date = req.receipt_date.unwrap_or_else(|| chrono::Utc::now().naive_utc().date().to_string());
        let currency = invoice.currency_code.clone();
        let payment_method = match req.payment_method.as_str() {
            "bank_transfer" => PaymentMethod::BankTransfer,
            "cheque" => PaymentMethod::Cheque,
            "card" => PaymentMethod::Card,
            "mobile_money" => PaymentMethod::MobileMoney,
            "ussd" => PaymentMethod::Ussd,
            _ => PaymentMethod::Cash,
        };

        sqlx::query(
            r#"INSERT INTO customer_receipts (id, company_id, branch_id, customer_id, receipt_number, receipt_date, amount, currency_code, payment_method, reference, bank_account_id, invoice_id, created_by, created_at, updated_at)
               VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, NOW(), NOW())"#,
        )
        .bind(receipt_id)
        .bind(invoice.company_id)
        .bind(invoice.branch_id)
        .bind(invoice.customer_id)
        .bind(&receipt_number)
        .bind(&receipt_date)
        .bind(&req.amount)
        .bind(&currency)
        .bind(&payment_method)
        .bind(&req.reference)
        .bind(req.bank_account_id)
        .bind(req.invoice_id)
        .bind(created_by)
        .execute(&self.pool)
        .await?;

        sqlx::query(
            "UPDATE sales_invoices SET amount_paid = $1, status = $2, updated_at = NOW() WHERE id = $3",
        )
        .bind(&new_amount_paid)
        .bind(new_status)
        .bind(req.invoice_id)
        .execute(&self.pool)
        .await?;

        sqlx::query(
            "UPDATE customers SET outstanding_balance = outstanding_balance - $1, updated_at = NOW() WHERE id = $2",
        )
        .bind(&req.amount)
        .bind(invoice.customer_id)
        .execute(&self.pool)
        .await?;

        sqlx::query_as::<_, CustomerReceipt>("SELECT * FROM customer_receipts WHERE id = $1")
            .bind(receipt_id)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| anyhow!("Failed to fetch receipt: {}", e))
    }

    pub async fn create_credit_note(
        &self,
        req: CreateCreditNoteRequest,
        created_by: Uuid,
    ) -> Result<SalesInvoice> {
        let original = sqlx::query_as::<_, SalesInvoice>("SELECT * FROM sales_invoices WHERE id = $1")
            .bind(req.invoice_id)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| anyhow!("Original invoice not found: {}", e))?;

        let id = Uuid::now_v7();
        let cn_number = self.generate_credit_note_number(req.company_id).await?;
        let currency = req.currency_code.unwrap_or(original.currency_code.clone());

        let mut subtotal = BigDecimal::from(0);
        let mut total_tax = BigDecimal::from(0);
        let mut total_discount = BigDecimal::from(0);

        for line in &req.lines {
            let line_total_before_discount = &line.quantity * &line.unit_price;
            let discount = match &line.discount_percent {
                Some(pct) => &line_total_before_discount * pct / BigDecimal::from(100),
                None => BigDecimal::from(0),
            };
            let net = &line_total_before_discount - &discount;
            let tax_rate = line.tax_rate.clone().unwrap_or_else(|| BigDecimal::from_str_radix(NGN_VAT_RATE, 10).unwrap_or(BigDecimal::from(0)));
            let tax_amount = &net * &tax_rate / BigDecimal::from(100);

            subtotal += net.clone();
            total_tax += tax_amount.clone();
            total_discount += discount.clone();
        }

        let total_amount = &subtotal + &total_tax;

        sqlx::query(
            r#"INSERT INTO sales_invoices (id, company_id, branch_id, customer_id, invoice_number, invoice_type, invoice_date, due_date, subtotal, tax_amount, discount_amount, total_amount, amount_paid, currency_code, status, linked_invoice_id, narration, created_by, created_at, updated_at)
               VALUES ($1, $2, $3, $4, $5, 'credit_note', CURRENT_DATE, CURRENT_DATE, $6, $7, $8, $9, 0, $10, 'draft', $11, $12, $13, NOW(), NOW())"#,
        )
        .bind(id)
        .bind(req.company_id)
        .bind(req.branch_id)
        .bind(original.customer_id)
        .bind(&cn_number)
        .bind(&subtotal)
        .bind(&total_tax)
        .bind(&total_discount)
        .bind(&total_amount)
        .bind(&currency)
        .bind(req.invoice_id)
        .bind(&req.reason)
        .bind(created_by)
        .execute(&self.pool)
        .await?;

        for (i, line) in req.lines.iter().enumerate() {
            let line_id = Uuid::now_v7();
            let line_total_before_discount = &line.quantity * &line.unit_price;
            let discount = match &line.discount_percent {
                Some(pct) => &line_total_before_discount * pct / BigDecimal::from(100),
                None => BigDecimal::from(0),
            };
            let net = &line_total_before_discount - &discount;
            let tax_rate = line.tax_rate.clone().unwrap_or_else(|| BigDecimal::from_str_radix(NGN_VAT_RATE, 10).unwrap_or(BigDecimal::from(0)));
            let tax_amount = &net * &tax_rate / BigDecimal::from(100);
            let line_total = &net + &tax_amount;

            sqlx::query(
                r#"INSERT INTO sales_invoice_items (id, invoice_id, product_id, line_number, description, quantity, unit_price, discount_percent, tax_rate, tax_amount, line_total, cost_center_id, created_at)
                   VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, NOW())"#,
            )
            .bind(line_id)
            .bind(id)
            .bind(line.product_id)
            .bind(i as i32 + 1)
            .bind(&line.description)
            .bind(&line.quantity)
            .bind(&line.unit_price)
            .bind(&line.discount_percent)
            .bind(&tax_rate)
            .bind(&tax_amount)
            .bind(&line_total)
            .bind(line.cost_center_id)
            .execute(&self.pool)
            .await?;
        }

        sqlx::query_as::<_, SalesInvoice>("SELECT * FROM sales_invoices WHERE id = $1")
            .bind(id)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| anyhow!("Failed to fetch credit note: {}", e))
    }

    pub async fn get_invoice_aging(&self, company_id: Uuid) -> Result<Vec<InvoiceAgingBucket>> {
        let rows = sqlx::query_as::<_, InvoiceAgingBucket>(
            r#"SELECT
                CASE
                    WHEN due_date >= CURRENT_DATE THEN 'current'
                    WHEN due_date >= CURRENT_DATE - INTERVAL '30 days' THEN '30'
                    WHEN due_date >= CURRENT_DATE - INTERVAL '60 days' THEN '60'
                    ELSE '90+'
                END AS bucket,
                COUNT(*) AS invoice_count,
                SUM(total_amount - amount_paid) AS total_outstanding
               FROM sales_invoices
               WHERE company_id = $1
               AND status IN ('sent', 'partial')
               AND total_amount > amount_paid
               AND invoice_type = 'standard'
               GROUP BY bucket
               ORDER BY array_position(ARRAY['current', '30', '60', '90+'], bucket)"#,
        )
        .bind(company_id)
        .fetch_all(&self.pool)
        .await?;
        Ok(rows)
    }

    pub async fn get_revenue_summary(
        &self,
        company_id: Uuid,
        period_start: chrono::NaiveDate,
        period_end: chrono::NaiveDate,
    ) -> Result<RevenueSummary> {
        let row = sqlx::query_as::<_, RevenueSummary>(
            r#"SELECT
                COALESCE(SUM(total_amount), 0) AS total_revenue,
                COALESCE(SUM(subtotal), 0) AS net_revenue,
                COALESCE(SUM(tax_amount), 0) AS total_vat,
                COUNT(*) AS invoice_count,
                $3::text AS vat_rate_used
               FROM sales_invoices
               WHERE company_id = $1
               AND invoice_date BETWEEN $2 AND $3
               AND status NOT IN ('draft', 'cancelled')
               AND invoice_type = 'standard'"#,
        )
        .bind(company_id)
        .bind(period_start)
        .bind(period_end)
        .fetch_one(&self.pool)
        .await?;
        Ok(row)
    }

    pub async fn list_invoices(&self, company_id: Uuid) -> Result<Vec<SalesInvoice>> {
        let invoices = sqlx::query_as::<_, SalesInvoice>(
            "SELECT * FROM sales_invoices WHERE company_id = $1 ORDER BY created_at DESC",
        )
        .bind(company_id)
        .fetch_all(&self.pool)
        .await?;
        Ok(invoices)
    }

    pub async fn create_receipt(
        &self,
        req: CreateReceiptRequest,
        created_by: Uuid,
    ) -> Result<CustomerReceipt> {
        let id = Uuid::now_v7();
        let receipt_number = self.generate_receipt_number(req.company_id).await?;
        let currency = req.currency_code.unwrap_or_else(|| "NGN".to_string());
        let payment_method = match req.payment_method.as_str() {
            "bank_transfer" => PaymentMethod::BankTransfer,
            "cheque" => PaymentMethod::Cheque,
            "card" => PaymentMethod::Card,
            "mobile_money" => PaymentMethod::MobileMoney,
            "ussd" => PaymentMethod::Ussd,
            _ => PaymentMethod::Cash,
        };

        sqlx::query(
            r#"INSERT INTO customer_receipts (id, company_id, branch_id, customer_id, receipt_number, receipt_date, amount, currency_code, payment_method, reference, bank_account_id, invoice_id, created_by, created_at, updated_at)
               VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, NOW(), NOW())"#,
        )
        .bind(id)
        .bind(req.company_id)
        .bind(req.branch_id)
        .bind(req.customer_id)
        .bind(&receipt_number)
        .bind(&req.receipt_date)
        .bind(&req.amount)
        .bind(&currency)
        .bind(&payment_method)
        .bind(&req.reference)
        .bind(req.bank_account_id)
        .bind(req.invoice_id)
        .bind(created_by)
        .execute(&self.pool)
        .await?;

        if let Some(invoice_id) = req.invoice_id {
            sqlx::query(
                "UPDATE sales_invoices SET amount_paid = amount_paid + $1, updated_at = NOW() WHERE id = $2",
            )
            .bind(&req.amount)
            .bind(invoice_id)
            .execute(&self.pool)
            .await?;
        }

        sqlx::query(
            "UPDATE customers SET outstanding_balance = outstanding_balance - $1, updated_at = NOW() WHERE id = $2",
        )
        .bind(&req.amount)
        .bind(req.customer_id)
        .execute(&self.pool)
        .await?;

        sqlx::query_as::<_, CustomerReceipt>("SELECT * FROM customer_receipts WHERE id = $1")
            .bind(id)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| anyhow!("Failed to fetch receipt: {}", e))
    }

    pub async fn post_receipt_to_gl(&self, receipt_id: Uuid, posted_by: Uuid) -> Result<()> {
        let receipt = sqlx::query_as::<_, CustomerReceipt>(
            "SELECT * FROM customer_receipts WHERE id = $1",
        )
        .bind(receipt_id)
        .fetch_one(&self.pool)
        .await?;

        let posting_service = PostingService::new(self.pool.clone());
        let context = PostingContext {
            company_id: receipt.company_id,
            source_module: "sales".to_string(),
            source_document_id: Some(receipt.id),
            source_document_number: Some(receipt.receipt_number.clone()),
            reference_id: None,
            customer_or_vendor: Some("customer".to_string()),
            branch: receipt.branch_id,
            department: None,
            cost_center: None,
            project: None,
            tax_code: None,
            currency: receipt.currency_code.clone(),
            amount: receipt.amount.clone(),
            tax_amount: None,
            discount_amount: None,
            narration: Some(format!("Customer receipt {}", receipt.receipt_number)),
            correlation_id: None,
            idempotency_key: Some(format!("receipt-{}", receipt.id)),
            transaction_type: "customer_receipt".to_string(),
            transaction_subtype: None,
            posted_by: Some(posted_by),
            posting_date: chrono::Utc::now().naive_utc().date(),
        };

        let journal = posting_service.post(context).await?;

        sqlx::query(
            "UPDATE customer_receipts SET journal_header_id = $1, updated_at = NOW() WHERE id = $2",
        )
        .bind(journal.id)
        .bind(receipt_id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn generate_invoice_number(&self, company_id: Uuid) -> Result<String> {
        let count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM sales_invoices WHERE company_id = $1",
        )
        .bind(company_id)
        .fetch_one(&self.pool)
        .await?;
        Ok(format!("INV-{:06}", count + 1))
    }

    async fn generate_receipt_number(&self, company_id: Uuid) -> Result<String> {
        let count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM customer_receipts WHERE company_id = $1",
        )
        .bind(company_id)
        .fetch_one(&self.pool)
        .await?;
        Ok(format!("RCPT-{:06}", count + 1))
    }

    async fn generate_credit_note_number(&self, company_id: Uuid) -> Result<String> {
        let count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM sales_invoices WHERE company_id = $1 AND invoice_type = 'credit_note'",
        )
        .bind(company_id)
        .fetch_one(&self.pool)
        .await?;
        Ok(format!("CN-{:06}", count + 1))
    }
}
