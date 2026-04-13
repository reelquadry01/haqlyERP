// Author: Quadri Atharu
use anyhow::{anyhow, Result};
use bigdecimal::BigDecimal;
use sqlx::PgPool;
use uuid::Uuid;

use crate::dtos::sales_dto::{CreateCustomerRequest, CreateInvoiceRequest, CreateReceiptRequest};
use crate::models::posting::PostingContext;
use crate::models::sales::{
    Customer, CustomerReceipt, InvoiceType, PaymentMethod, ProformaInvoice, SalesInvoice,
    SalesInvoiceItem,
};
use crate::services::posting_service::PostingService;

#[derive(Clone)]
pub struct SalesService {
    pub pool: PgPool,
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
            let tax_rate = item.tax_rate.unwrap_or(BigDecimal::from(0));
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
            r#"INSERT INTO sales_invoices (id, company_id, branch_id, customer_id, invoice_number, invoice_type, invoice_date, due_date, subtotal, tax_amount, discount_amount, total_amount, amount_paid, currency_code, status, narration, created_by, created_at, updated_at)
               VALUES ($1, $2, $3, $4, $5, 'standard', $6, $7, $8, $9, $10, $11, 0, $12, 'draft', $13, $14, NOW(), NOW())"#,
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
}
