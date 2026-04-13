// Author: Quadri Atharu
use bigdecimal::BigDecimal;
use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Customer {
    pub id: Uuid,
    pub company_id: Uuid,
    pub code: String,
    pub name: String,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub tax_id: Option<String>,
    pub customer_type: String,
    pub credit_limit: BigDecimal,
    pub payment_terms: i32,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct SalesInvoice {
    pub id: Uuid,
    pub company_id: Uuid,
    pub branch_id: Option<Uuid>,
    pub customer_id: Uuid,
    pub number: String,
    pub date: NaiveDate,
    pub due_date: Option<NaiveDate>,
    pub invoice_type: String,
    pub status: String,
    pub currency_code: String,
    pub exchange_rate: BigDecimal,
    pub taxable_amount: BigDecimal,
    pub tax_amount: BigDecimal,
    pub total_amount: BigDecimal,
    pub amount_paid: BigDecimal,
    pub narration: Option<String>,
    pub is_einvoice_eligible: bool,
    pub einvoice_irn: Option<String>,
    pub created_by: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct SalesInvoiceItem {
    pub id: Uuid,
    pub sales_invoice_id: Uuid,
    pub line_number: i32,
    pub product_id: Option<Uuid>,
    pub sku: Option<String>,
    pub description: String,
    pub quantity: BigDecimal,
    pub unit_price: BigDecimal,
    pub discount_percent: BigDecimal,
    pub tax_rate: Option<BigDecimal>,
    pub taxable_amount: BigDecimal,
    pub tax_amount: BigDecimal,
    pub line_amount: BigDecimal,
    pub cost_center_id: Option<Uuid>,
    pub project_id: Option<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct CustomerReceipt {
    pub id: Uuid,
    pub company_id: Uuid,
    pub customer_id: Uuid,
    pub number: String,
    pub date: NaiveDate,
    pub amount: BigDecimal,
    pub payment_method: String,
    pub bank_account_id: Option<Uuid>,
    pub reference: Option<String>,
    pub narration: Option<String>,
    pub status: String,
    pub posted_to_gl: bool,
    pub gl_journal_id: Option<Uuid>,
    pub created_by: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewCustomer {
    pub company_id: Uuid,
    pub code: String,
    pub name: String,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub tax_id: Option<String>,
    pub customer_type: String,
    pub credit_limit: BigDecimal,
    pub payment_terms: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewSalesInvoice {
    pub company_id: Uuid,
    pub branch_id: Option<Uuid>,
    pub customer_id: Uuid,
    pub number: String,
    pub date: NaiveDate,
    pub due_date: Option<NaiveDate>,
    pub invoice_type: String,
    pub currency_code: String,
    pub exchange_rate: BigDecimal,
    pub taxable_amount: BigDecimal,
    pub tax_amount: BigDecimal,
    pub total_amount: BigDecimal,
    pub narration: Option<String>,
    pub created_by: Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewCustomerReceipt {
    pub company_id: Uuid,
    pub customer_id: Uuid,
    pub number: String,
    pub date: NaiveDate,
    pub amount: BigDecimal,
    pub payment_method: String,
    pub bank_account_id: Option<Uuid>,
    pub reference: Option<String>,
    pub narration: Option<String>,
    pub created_by: Uuid,
}

pub struct SalesRepo {
    pool: PgPool,
}

impl SalesRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn create_customer(&self, new_customer: NewCustomer) -> Result<Customer, sqlx::Error> {
        sqlx::query_as::<_, Customer>(
            r#"INSERT INTO customers (company_id, code, name, email, phone, tax_id, customer_type, credit_limit, payment_terms)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            RETURNING id, company_id, code, name, email, phone, tax_id, customer_type, credit_limit, payment_terms, is_active, created_at, updated_at"#,
        )
        .bind(new_customer.company_id)
        .bind(&new_customer.code)
        .bind(&new_customer.name)
        .bind(&new_customer.email)
        .bind(&new_customer.phone)
        .bind(&new_customer.tax_id)
        .bind(&new_customer.customer_type)
        .bind(&new_customer.credit_limit)
        .bind(new_customer.payment_terms)
        .fetch_one(&self.pool)
        .await
    }

    pub async fn find_customers(&self, company_id: Uuid) -> Result<Vec<Customer>, sqlx::Error> {
        sqlx::query_as::<_, Customer>(
            "SELECT id, company_id, code, name, email, phone, tax_id, customer_type, credit_limit, payment_terms, is_active, created_at, updated_at FROM customers WHERE company_id = $1 ORDER BY code",
        )
        .bind(company_id)
        .fetch_all(&self.pool)
        .await
    }

    pub async fn create_invoice(&self, inv: NewSalesInvoice) -> Result<SalesInvoice, sqlx::Error> {
        sqlx::query_as::<_, SalesInvoice>(
            r#"INSERT INTO sales_invoices (company_id, branch_id, customer_id, number, date, due_date, invoice_type, currency_code, exchange_rate, taxable_amount, tax_amount, total_amount, narration, created_by)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14)
            RETURNING id, company_id, branch_id, customer_id, number, date, due_date, invoice_type, status, currency_code, exchange_rate, taxable_amount, tax_amount, total_amount, amount_paid, narration, is_einvoice_eligible, einvoice_irn, created_by, created_at, updated_at"#,
        )
        .bind(inv.company_id)
        .bind(inv.branch_id)
        .bind(inv.customer_id)
        .bind(&inv.number)
        .bind(inv.date)
        .bind(inv.due_date)
        .bind(&inv.invoice_type)
        .bind(&inv.currency_code)
        .bind(&inv.exchange_rate)
        .bind(&inv.taxable_amount)
        .bind(&inv.tax_amount)
        .bind(&inv.total_amount)
        .bind(&inv.narration)
        .bind(inv.created_by)
        .fetch_one(&self.pool)
        .await
    }

    pub async fn find_invoices(&self, company_id: Uuid) -> Result<Vec<SalesInvoice>, sqlx::Error> {
        sqlx::query_as::<_, SalesInvoice>(
            "SELECT id, company_id, branch_id, customer_id, number, date, due_date, invoice_type, status, currency_code, exchange_rate, taxable_amount, tax_amount, total_amount, amount_paid, narration, is_einvoice_eligible, einvoice_irn, created_by, created_at, updated_at FROM sales_invoices WHERE company_id = $1 ORDER BY date DESC",
        )
        .bind(company_id)
        .fetch_all(&self.pool)
        .await
    }

    pub async fn create_receipt(
        &self,
        new_receipt: NewCustomerReceipt,
    ) -> Result<CustomerReceipt, sqlx::Error> {
        sqlx::query_as::<_, CustomerReceipt>(
            r#"INSERT INTO customer_receipts (company_id, customer_id, number, date, amount, payment_method, bank_account_id, reference, narration, created_by)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            RETURNING id, company_id, customer_id, number, date, amount, payment_method, bank_account_id, reference, narration, status, posted_to_gl, gl_journal_id, created_by, created_at, updated_at"#,
        )
        .bind(new_receipt.company_id)
        .bind(new_receipt.customer_id)
        .bind(&new_receipt.number)
        .bind(new_receipt.date)
        .bind(&new_receipt.amount)
        .bind(&new_receipt.payment_method)
        .bind(new_receipt.bank_account_id)
        .bind(&new_receipt.reference)
        .bind(&new_receipt.narration)
        .bind(new_receipt.created_by)
        .fetch_one(&self.pool)
        .await
    }

    pub async fn post_receipt(&self, receipt_id: Uuid, journal_id: Uuid) -> Result<(), sqlx::Error> {
        sqlx::query(
            "UPDATE customer_receipts SET posted_to_gl = true, gl_journal_id = $1, status = 'POSTED', updated_at = now() WHERE id = $2",
        )
        .bind(journal_id)
        .bind(receipt_id)
        .execute(&self.pool)
        .await?;
        Ok(())
    }
}
