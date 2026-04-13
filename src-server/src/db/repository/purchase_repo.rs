// Author: Quadri Atharu
use bigdecimal::BigDecimal;
use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Supplier {
    pub id: Uuid,
    pub company_id: Uuid,
    pub code: String,
    pub name: String,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub tax_id: Option<String>,
    pub payment_terms: i32,
    pub is_active: bool,
    pub bank_name: Option<String>,
    pub bank_account_number: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct PurchaseBill {
    pub id: Uuid,
    pub company_id: Uuid,
    pub branch_id: Option<Uuid>,
    pub supplier_id: Uuid,
    pub number: String,
    pub date: NaiveDate,
    pub due_date: Option<NaiveDate>,
    pub status: String,
    pub currency_code: String,
    pub exchange_rate: BigDecimal,
    pub taxable_amount: BigDecimal,
    pub tax_amount: BigDecimal,
    pub total_amount: BigDecimal,
    pub amount_paid: BigDecimal,
    pub narration: Option<String>,
    pub created_by: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct PurchaseBillItem {
    pub id: Uuid,
    pub purchase_bill_id: Uuid,
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
pub struct SupplierPayment {
    pub id: Uuid,
    pub company_id: Uuid,
    pub supplier_id: Uuid,
    pub purchase_bill_id: Option<Uuid>,
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
pub struct NewSupplier {
    pub company_id: Uuid,
    pub code: String,
    pub name: String,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub tax_id: Option<String>,
    pub payment_terms: i32,
    pub bank_name: Option<String>,
    pub bank_account_number: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewPurchaseBill {
    pub company_id: Uuid,
    pub branch_id: Option<Uuid>,
    pub supplier_id: Uuid,
    pub number: String,
    pub date: NaiveDate,
    pub due_date: Option<NaiveDate>,
    pub currency_code: String,
    pub exchange_rate: BigDecimal,
    pub taxable_amount: BigDecimal,
    pub tax_amount: BigDecimal,
    pub total_amount: BigDecimal,
    pub narration: Option<String>,
    pub created_by: Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewSupplierPayment {
    pub company_id: Uuid,
    pub supplier_id: Uuid,
    pub purchase_bill_id: Option<Uuid>,
    pub number: String,
    pub date: NaiveDate,
    pub amount: BigDecimal,
    pub payment_method: String,
    pub bank_account_id: Option<Uuid>,
    pub reference: Option<String>,
    pub narration: Option<String>,
    pub created_by: Uuid,
}

pub struct PurchaseRepo {
    pool: PgPool,
}

impl PurchaseRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn create_supplier(&self, new_supplier: NewSupplier) -> Result<Supplier, sqlx::Error> {
        sqlx::query_as::<_, Supplier>(
            r#"INSERT INTO suppliers (company_id, code, name, email, phone, tax_id, payment_terms, bank_name, bank_account_number)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            RETURNING id, company_id, code, name, email, phone, tax_id, payment_terms, is_active, bank_name, bank_account_number, created_at, updated_at"#,
        )
        .bind(new_supplier.company_id)
        .bind(&new_supplier.code)
        .bind(&new_supplier.name)
        .bind(&new_supplier.email)
        .bind(&new_supplier.phone)
        .bind(&new_supplier.tax_id)
        .bind(new_supplier.payment_terms)
        .bind(&new_supplier.bank_name)
        .bind(&new_supplier.bank_account_number)
        .fetch_one(&self.pool)
        .await
    }

    pub async fn find_suppliers(&self, company_id: Uuid) -> Result<Vec<Supplier>, sqlx::Error> {
        sqlx::query_as::<_, Supplier>(
            "SELECT id, company_id, code, name, email, phone, tax_id, payment_terms, is_active, bank_name, bank_account_number, created_at, updated_at FROM suppliers WHERE company_id = $1 ORDER BY code",
        )
        .bind(company_id)
        .fetch_all(&self.pool)
        .await
    }

    pub async fn create_bill(&self, bill: NewPurchaseBill) -> Result<PurchaseBill, sqlx::Error> {
        sqlx::query_as::<_, PurchaseBill>(
            r#"INSERT INTO purchase_bills (company_id, branch_id, supplier_id, number, date, due_date, currency_code, exchange_rate, taxable_amount, tax_amount, total_amount, narration, created_by)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14)
            RETURNING id, company_id, branch_id, supplier_id, number, date, due_date, status, currency_code, exchange_rate, taxable_amount, tax_amount, total_amount, amount_paid, narration, created_by, created_at, updated_at"#,
        )
        .bind(bill.company_id)
        .bind(bill.branch_id)
        .bind(bill.supplier_id)
        .bind(&bill.number)
        .bind(bill.date)
        .bind(bill.due_date)
        .bind(&bill.currency_code)
        .bind(&bill.exchange_rate)
        .bind(&bill.taxable_amount)
        .bind(&bill.tax_amount)
        .bind(&bill.total_amount)
        .bind(&bill.narration)
        .bind(bill.created_by)
        .fetch_one(&self.pool)
        .await
    }

    pub async fn find_bills(&self, company_id: Uuid) -> Result<Vec<PurchaseBill>, sqlx::Error> {
        sqlx::query_as::<_, PurchaseBill>(
            "SELECT id, company_id, branch_id, supplier_id, number, date, due_date, status, currency_code, exchange_rate, taxable_amount, tax_amount, total_amount, amount_paid, narration, created_by, created_at, updated_at FROM purchase_bills WHERE company_id = $1 ORDER BY date DESC",
        )
        .bind(company_id)
        .fetch_all(&self.pool)
        .await
    }

    pub async fn create_payment(
        &self,
        new_payment: NewSupplierPayment,
    ) -> Result<SupplierPayment, sqlx::Error> {
        sqlx::query_as::<_, SupplierPayment>(
            r#"INSERT INTO supplier_payments (company_id, supplier_id, purchase_bill_id, number, date, amount, payment_method, bank_account_id, reference, narration, created_by)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
            RETURNING id, company_id, supplier_id, purchase_bill_id, number, date, amount, payment_method, bank_account_id, reference, narration, status, posted_to_gl, gl_journal_id, created_by, created_at, updated_at"#,
        )
        .bind(new_payment.company_id)
        .bind(new_payment.supplier_id)
        .bind(new_payment.purchase_bill_id)
        .bind(&new_payment.number)
        .bind(new_payment.date)
        .bind(&new_payment.amount)
        .bind(&new_payment.payment_method)
        .bind(new_payment.bank_account_id)
        .bind(&new_payment.reference)
        .bind(&new_payment.narration)
        .bind(new_payment.created_by)
        .fetch_one(&self.pool)
        .await
    }
}
