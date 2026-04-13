// Author: Quadri Atharu
use anyhow::{anyhow, Result};
use bigdecimal::BigDecimal;
use sqlx::PgPool;
use uuid::Uuid;

use crate::dtos::purchase_dto::{CreateBillRequest, CreatePaymentRequest, CreateSupplierRequest};
use crate::models::posting::PostingContext;
use crate::models::purchase::{PurchaseBill, PurchaseBillItem, Supplier, SupplierPayment};
use crate::services::posting_service::PostingService;

#[derive(Clone)]
pub struct PurchasesService {
    pub pool: PgPool,
}

impl PurchasesService {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn create_supplier(&self, req: CreateSupplierRequest) -> Result<Supplier> {
        let id = Uuid::now_v7();
        let code = req.code.unwrap_or_else(|| format!("SUP-{}", Uuid::now_v7().as_simple().to_string().chars().take(8).collect::<String>()));
        let currency = req.currency_code.unwrap_or_else(|| "NGN".to_string());

        sqlx::query(
            r#"INSERT INTO suppliers (id, company_id, code, name, email, phone, tax_identification_number, rc_number, contact_person, payment_terms, bank_name, bank_account_number, bank_sort_code, outstanding_balance, currency_code, is_active, branch_id, created_at, updated_at)
               VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, 0, $14, true, $15, NOW(), NOW())"#,
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
        .bind(&req.payment_terms)
        .bind(&req.bank_name)
        .bind(&req.bank_account_number)
        .bind(&req.bank_sort_code)
        .bind(&currency)
        .bind(req.branch_id)
        .execute(&self.pool)
        .await?;

        if let Some(addr) = req.address {
            let addr_id = Uuid::now_v7();
            sqlx::query(
                r#"INSERT INTO supplier_addresses (id, supplier_id, address_type, line1, line2, city, state, country, postal_code, is_default, created_at)
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

        sqlx::query_as::<_, Supplier>("SELECT * FROM suppliers WHERE id = $1")
            .bind(id)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| anyhow!("Failed to fetch supplier: {}", e))
    }

    pub async fn list_suppliers(&self, company_id: Uuid) -> Result<Vec<Supplier>> {
        let suppliers = sqlx::query_as::<_, Supplier>(
            "SELECT * FROM suppliers WHERE company_id = $1 AND is_active = true ORDER BY name",
        )
        .bind(company_id)
        .fetch_all(&self.pool)
        .await?;
        Ok(suppliers)
    }

    pub async fn create_bill(
        &self,
        req: CreateBillRequest,
        created_by: Uuid,
    ) -> Result<PurchaseBill> {
        let id = Uuid::now_v7();
        let bill_number = self.generate_bill_number(req.company_id).await?;
        let currency = req.currency_code.unwrap_or_else(|| "NGN".to_string());

        let mut subtotal = BigDecimal::from(0);
        let mut total_tax = BigDecimal::from(0);
        let mut total_wht = BigDecimal::from(0);

        for item in &req.items {
            let line_total = &item.quantity * &item.unit_price;
            let discount = match &item.discount_percent {
                Some(pct) => &line_total * pct / BigDecimal::from(100),
                None => BigDecimal::from(0),
            };
            let net = &line_total - &discount;
            let tax = match &item.tax_rate {
                Some(rate) => &net * rate / BigDecimal::from(100),
                None => BigDecimal::from(0),
            };
            let wht = match &item.wht_rate {
                Some(rate) => &net * rate / BigDecimal::from(100),
                None => BigDecimal::from(0),
            };
            subtotal += net.clone();
            total_tax += tax;
            total_wht += wht;
        }

        let total_amount = &subtotal + &total_tax - &total_wht;

        sqlx::query(
            r#"INSERT INTO purchase_bills (id, company_id, branch_id, supplier_id, bill_number, bill_date, due_date, subtotal, tax_amount, withholding_amount, total_amount, amount_paid, currency_code, status, narration, created_by, created_at, updated_at)
               VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, 0, $12, 'draft', $13, $14, NOW(), NOW())"#,
        )
        .bind(id)
        .bind(req.company_id)
        .bind(req.branch_id)
        .bind(req.supplier_id)
        .bind(&bill_number)
        .bind(&req.bill_date)
        .bind(&req.due_date)
        .bind(&subtotal)
        .bind(&total_tax)
        .bind(&total_wht)
        .bind(&total_amount)
        .bind(&currency)
        .bind(&req.narration)
        .bind(created_by)
        .execute(&self.pool)
        .await?;

        for (i, item) in req.items.iter().enumerate() {
            let line_id = Uuid::now_v7();
            let line_total = &item.quantity * &item.unit_price;
            let discount = match &item.discount_percent {
                Some(pct) => &line_total * pct / BigDecimal::from(100),
                None => BigDecimal::from(0),
            };
            let net = &line_total - &discount;
            let tax = match &item.tax_rate {
                Some(rate) => &net * rate / BigDecimal::from(100),
                None => BigDecimal::from(0),
            };
            let wht = match &item.wht_rate {
                Some(rate) => &net * rate / BigDecimal::from(100),
                None => BigDecimal::from(0),
            };
            let line_total_final = &net + &tax - &wht;

            sqlx::query(
                r#"INSERT INTO purchase_bill_items (id, bill_id, product_id, line_number, description, quantity, unit_price, discount_percent, tax_rate, tax_amount, wht_rate, wht_amount, line_total, cost_center_id, created_at)
                   VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, NOW())"#,
            )
            .bind(line_id)
            .bind(id)
            .bind(item.product_id)
            .bind(i as i32 + 1)
            .bind(&item.description)
            .bind(&item.quantity)
            .bind(&item.unit_price)
            .bind(&item.discount_percent)
            .bind(&item.tax_rate)
            .bind(&tax)
            .bind(&item.wht_rate)
            .bind(&wht)
            .bind(&line_total_final)
            .bind(item.cost_center_id)
            .execute(&self.pool)
            .await?;
        }

        sqlx::query_as::<_, PurchaseBill>("SELECT * FROM purchase_bills WHERE id = $1")
            .bind(id)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| anyhow!("Failed to fetch bill: {}", e))
    }

    pub async fn list_bills(&self, company_id: Uuid) -> Result<Vec<PurchaseBill>> {
        let bills = sqlx::query_as::<_, PurchaseBill>(
            "SELECT * FROM purchase_bills WHERE company_id = $1 ORDER BY created_at DESC",
        )
        .bind(company_id)
        .fetch_all(&self.pool)
        .await?;
        Ok(bills)
    }

    pub async fn create_payment(
        &self,
        req: CreatePaymentRequest,
        created_by: Uuid,
    ) -> Result<SupplierPayment> {
        let id = Uuid::now_v7();
        let payment_number = self.generate_payment_number(req.company_id).await?;
        let currency = req.currency_code.unwrap_or_else(|| "NGN".to_string());

        sqlx::query(
            r#"INSERT INTO supplier_payments (id, company_id, branch_id, supplier_id, payment_number, payment_date, amount, currency_code, payment_method, reference, bank_account_id, bill_id, created_by, created_at, updated_at)
               VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, NOW(), NOW())"#,
        )
        .bind(id)
        .bind(req.company_id)
        .bind(req.branch_id)
        .bind(req.supplier_id)
        .bind(&payment_number)
        .bind(&req.payment_date)
        .bind(&req.amount)
        .bind(&currency)
        .bind(&req.payment_method)
        .bind(&req.reference)
        .bind(req.bank_account_id)
        .bind(req.bill_id)
        .bind(created_by)
        .execute(&self.pool)
        .await?;

        if let Some(bill_id) = req.bill_id {
            sqlx::query(
                "UPDATE purchase_bills SET amount_paid = amount_paid + $1, updated_at = NOW() WHERE id = $2",
            )
            .bind(&req.amount)
            .bind(bill_id)
            .execute(&self.pool)
            .await?;
        }

        sqlx::query(
            "UPDATE suppliers SET outstanding_balance = outstanding_balance - $1, updated_at = NOW() WHERE id = $2",
        )
        .bind(&req.amount)
        .bind(req.supplier_id)
        .execute(&self.pool)
        .await?;

        sqlx::query_as::<_, SupplierPayment>("SELECT * FROM supplier_payments WHERE id = $1")
            .bind(id)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| anyhow!("Failed to fetch payment: {}", e))
    }

    pub async fn post_payment_to_gl(&self, payment_id: Uuid, posted_by: Uuid) -> Result<()> {
        let payment = sqlx::query_as::<_, SupplierPayment>(
            "SELECT * FROM supplier_payments WHERE id = $1",
        )
        .bind(payment_id)
        .fetch_one(&self.pool)
        .await?;

        let posting_service = PostingService::new(self.pool.clone());
        let context = PostingContext {
            company_id: payment.company_id,
            source_module: "purchases".to_string(),
            source_document_id: Some(payment.id),
            source_document_number: Some(payment.payment_number.clone()),
            reference_id: None,
            customer_or_vendor: Some("supplier".to_string()),
            branch: payment.branch_id,
            department: None,
            cost_center: None,
            project: None,
            tax_code: None,
            currency: payment.currency_code.clone(),
            amount: payment.amount.clone(),
            tax_amount: None,
            discount_amount: None,
            narration: Some(format!("Supplier payment {}", payment.payment_number)),
            correlation_id: None,
            idempotency_key: Some(format!("supplier-payment-{}", payment.id)),
            transaction_type: "supplier_payment".to_string(),
            transaction_subtype: None,
            posted_by: Some(posted_by),
            posting_date: chrono::Utc::now().naive_utc().date(),
        };

        let journal = posting_service.post(context).await?;

        sqlx::query(
            "UPDATE supplier_payments SET journal_header_id = $1, updated_at = NOW() WHERE id = $2",
        )
        .bind(journal.id)
        .bind(payment_id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn generate_bill_number(&self, company_id: Uuid) -> Result<String> {
        let count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM purchase_bills WHERE company_id = $1",
        )
        .bind(company_id)
        .fetch_one(&self.pool)
        .await?;
        Ok(format!("BILL-{:06}", count + 1))
    }

    async fn generate_payment_number(&self, company_id: Uuid) -> Result<String> {
        let count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM supplier_payments WHERE company_id = $1",
        )
        .bind(company_id)
        .fetch_one(&self.pool)
        .await?;
        Ok(format!("SPAY-{:06}", count + 1))
    }
}
