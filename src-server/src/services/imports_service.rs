// Author: Quadri Atharu
use anyhow::{anyhow, Result};
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Clone)]
pub struct ImportsService {
    pub pool: PgPool,
}

impl ImportsService {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    fn validate_headers(expected: &[&str], actual: &[String]) -> Result<()> {
        for expected_header in expected {
            if !actual.iter().any(|h| h.to_lowercase().replace(' ', "_") == expected_header.to_lowercase().replace(' ', "_")) {
                return Err(anyhow!("Missing required header: {}", expected_header));
            }
        }
        Ok(())
    }

    pub async fn bulk_import(&self, entity_type: &str, csv_data: &str, company_id: Uuid) -> Result<ImportResult> {
        let mut reader = csv::Reader::from_reader(csv_data.as_bytes());
        let headers = reader.headers()?.clone();
        let header_vec: Vec<String> = headers.iter().map(|h| h.to_string()).collect();

        match entity_type {
            "coa" => self.import_coa(&header_vec, reader, company_id).await,
            "customers" => self.import_customers(&header_vec, reader, company_id).await,
            "suppliers" => self.import_suppliers(&header_vec, reader, company_id).await,
            "products" => self.import_products(&header_vec, reader, company_id).await,
            "tax_configs" => self.import_tax_configs(&header_vec, reader, company_id).await,
            "branches" => self.import_branches(&header_vec, reader, company_id).await,
            "departments" => self.import_departments(&header_vec, reader, company_id).await,
            "warehouses" => self.import_warehouses(&header_vec, reader, company_id).await,
            "bank_accounts" => self.import_bank_accounts(&header_vec, reader, company_id).await,
            "asset_categories" => self.import_asset_categories(&header_vec, reader, company_id).await,
            "gl_opening_balances" => self.import_gl_opening_balances(&header_vec, reader, company_id).await,
            "ar_opening_balances" => self.import_ar_opening_balances(&header_vec, reader, company_id).await,
            "ap_opening_balances" => self.import_ap_opening_balances(&header_vec, reader, company_id).await,
            "customer_receipts" => self.import_customer_receipts(&header_vec, reader, company_id).await,
            "supplier_payments" => self.import_supplier_payments(&header_vec, reader, company_id).await,
            "fixed_assets" => self.import_fixed_assets(&header_vec, reader, company_id).await,
            "stock_opening_balances" => self.import_stock_opening_balances(&header_vec, reader, company_id).await,
            "employees" => self.import_employees(&header_vec, reader, company_id).await,
            _ => Err(anyhow!("Unknown entity type: {}", entity_type)),
        }
    }

    async fn import_coa<R: std::io::Read>(
        &self,
        headers: &[String],
        mut reader: csv::Reader<R>,
        company_id: Uuid,
    ) -> Result<ImportResult> {
        Self::validate_headers(&["code", "name", "account_type"], headers)?;
        let mut imported = 0u64;
        let mut errors = Vec::new();

        for (idx, result) in reader.records().enumerate() {
            let record = match result {
                Ok(r) => r,
                Err(e) => {
                    errors.push(format!("Row {}: parse error: {}", idx + 2, e));
                    continue;
                }
            };

            let id = Uuid::now_v7();
            let res = sqlx::query(
                r#"INSERT INTO chart_of_accounts (id, company_id, code, name, account_type, is_active, allowed_posting, is_control_account, currency_code, balance, created_at, updated_at)
                   VALUES ($1, $2, $3, $4, $5, true, true, false, 'NGN', 0, NOW(), NOW())"#,
            )
            .bind(id)
            .bind(company_id)
            .bind(&record.get(0).unwrap_or(""))
            .bind(&record.get(1).unwrap_or(""))
            .bind(&record.get(2).unwrap_or("expense"))
            .execute(&self.pool)
            .await;

            match res {
                Ok(_) => imported += 1,
                Err(e) => errors.push(format!("Row {}: {}", idx + 2, e)),
            }
        }

        Ok(ImportResult { imported, errors: errors.clone(), total: imported as i64 + errors.len() as i64 })
    }

    async fn import_customers<R: std::io::Read>(
        &self,
        headers: &[String],
        mut reader: csv::Reader<R>,
        company_id: Uuid,
    ) -> Result<ImportResult> {
        Self::validate_headers(&["code", "name"], headers)?;
        let mut imported = 0u64;
        let mut errors = Vec::new();

        for (idx, result) in reader.records().enumerate() {
            let record = match result {
                Ok(r) => r,
                Err(e) => { errors.push(format!("Row {}: {}", idx + 2, e)); continue; }
            };

            let id = Uuid::now_v7();
            let res = sqlx::query(
                r#"INSERT INTO customers (id, company_id, code, name, email, phone, outstanding_balance, currency_code, is_active, created_at, updated_at)
                   VALUES ($1, $2, $3, $4, $5, $6, 0, 'NGN', true, NOW(), NOW())"#,
            )
            .bind(id)
            .bind(company_id)
            .bind(&record.get(0).unwrap_or(""))
            .bind(&record.get(1).unwrap_or(""))
            .bind(record.get(2).unwrap_or(""))
            .bind(record.get(3).unwrap_or(""))
            .execute(&self.pool)
            .await;

            match res { Ok(_) => imported += 1, Err(e) => errors.push(format!("Row {}: {}", idx + 2, e)) }
        }

        Ok(ImportResult { imported, errors: errors.clone(), total: imported as i64 + errors.len() as i64 })
    }

    async fn import_suppliers<R: std::io::Read>(
        &self,
        headers: &[String],
        mut reader: csv::Reader<R>,
        company_id: Uuid,
    ) -> Result<ImportResult> {
        Self::validate_headers(&["code", "name"], headers)?;
        let mut imported = 0u64;
        let mut errors = Vec::new();

        for (idx, result) in reader.records().enumerate() {
            let record = match result { Ok(r) => r, Err(e) => { errors.push(format!("Row {}: {}", idx + 2, e)); continue; } };

            let id = Uuid::now_v7();
            let res = sqlx::query(
                r#"INSERT INTO suppliers (id, company_id, code, name, email, phone, outstanding_balance, currency_code, is_active, created_at, updated_at)
                   VALUES ($1, $2, $3, $4, $5, $6, 0, 'NGN', true, NOW(), NOW())"#,
            )
            .bind(id).bind(company_id)
            .bind(&record.get(0).unwrap_or(""))
            .bind(&record.get(1).unwrap_or(""))
            .bind(record.get(2).unwrap_or(""))
            .bind(record.get(3).unwrap_or(""))
            .execute(&self.pool).await;

            match res { Ok(_) => imported += 1, Err(e) => errors.push(format!("Row {}: {}", idx + 2, e)) }
        }

        Ok(ImportResult { imported, errors: errors.clone(), total: imported as i64 + errors.len() as i64 })
    }

    async fn import_products<R: std::io::Read>(
        &self,
        headers: &[String],
        mut reader: csv::Reader<R>,
        company_id: Uuid,
    ) -> Result<ImportResult> {
        Self::validate_headers(&["code", "name"], headers)?;
        let mut imported = 0u64;
        let mut errors = Vec::new();

        for (idx, result) in reader.records().enumerate() {
            let record = match result { Ok(r) => r, Err(e) => { errors.push(format!("Row {}: {}", idx + 2, e)); continue; } };

            let id = Uuid::now_v7();
            let res = sqlx::query(
                r#"INSERT INTO products (id, company_id, code, name, selling_price, cost_price, is_active, created_at, updated_at)
                   VALUES ($1, $2, $3, $4, 0, 0, true, NOW(), NOW())"#,
            )
            .bind(id).bind(company_id)
            .bind(&record.get(0).unwrap_or(""))
            .bind(&record.get(1).unwrap_or(""))
            .execute(&self.pool).await;

            match res { Ok(_) => imported += 1, Err(e) => errors.push(format!("Row {}: {}", idx + 2, e)) }
        }

        Ok(ImportResult { imported, errors: errors.clone(), total: imported as i64 + errors.len() as i64 })
    }

    async fn import_tax_configs<R: std::io::Read>(
        &self,
        headers: &[String],
        mut reader: csv::Reader<R>,
        company_id: Uuid,
    ) -> Result<ImportResult> {
        Self::validate_headers(&["name", "tax_type", "rate"], headers)?;
        let mut imported = 0u64;
        let mut errors = Vec::new();

        for (idx, result) in reader.records().enumerate() {
            let record = match result { Ok(r) => r, Err(e) => { errors.push(format!("Row {}: {}", idx + 2, e)); continue; } };

            let id = Uuid::now_v7();
            let res = sqlx::query(
                r#"INSERT INTO tax_configs (id, company_id, name, tax_type, rate, effective_from, is_active, created_at, updated_at)
                   VALUES ($1, $2, $3, $4, $5, '2024-01-01', true, NOW(), NOW())"#,
            )
            .bind(id).bind(company_id)
            .bind(&record.get(0).unwrap_or(""))
            .bind(&record.get(1).unwrap_or("vat"))
            .bind(&record.get(2).unwrap_or("0"))
            .execute(&self.pool).await;

            match res { Ok(_) => imported += 1, Err(e) => errors.push(format!("Row {}: {}", idx + 2, e)) }
        }

        Ok(ImportResult { imported, errors: errors.clone(), total: imported as i64 + errors.len() as i64 })
    }

    async fn import_branches<R: std::io::Read>(
        &self,
        headers: &[String],
        mut reader: csv::Reader<R>,
        company_id: Uuid,
    ) -> Result<ImportResult> {
        Self::validate_headers(&["code", "name"], headers)?;
        let mut imported = 0u64;
        let mut errors = Vec::new();

        for (idx, result) in reader.records().enumerate() {
            let record = match result { Ok(r) => r, Err(e) => { errors.push(format!("Row {}: {}", idx + 2, e)); continue; } };

            let id = Uuid::now_v7();
            let res = sqlx::query(
                r#"INSERT INTO branches (id, company_id, code, name, is_active, created_at, updated_at)
                   VALUES ($1, $2, $3, $4, true, NOW(), NOW())"#,
            )
            .bind(id).bind(company_id)
            .bind(&record.get(0).unwrap_or(""))
            .bind(&record.get(1).unwrap_or(""))
            .execute(&self.pool).await;

            match res { Ok(_) => imported += 1, Err(e) => errors.push(format!("Row {}: {}", idx + 2, e)) }
        }

        Ok(ImportResult { imported, errors: errors.clone(), total: imported as i64 + errors.len() as i64 })
    }

    async fn import_departments<R: std::io::Read>(
        &self,
        headers: &[String],
        mut reader: csv::Reader<R>,
        company_id: Uuid,
    ) -> Result<ImportResult> {
        Self::validate_headers(&["code", "name"], headers)?;
        let mut imported = 0u64;
        let mut errors = Vec::new();

        for (idx, result) in reader.records().enumerate() {
            let record = match result { Ok(r) => r, Err(e) => { errors.push(format!("Row {}: {}", idx + 2, e)); continue; } };

            let id = Uuid::now_v7();
            let res = sqlx::query(
                r#"INSERT INTO departments (id, company_id, code, name, is_active, created_at, updated_at)
                   VALUES ($1, $2, $3, $4, true, NOW(), NOW())"#,
            )
            .bind(id).bind(company_id)
            .bind(&record.get(0).unwrap_or(""))
            .bind(&record.get(1).unwrap_or(""))
            .execute(&self.pool).await;

            match res { Ok(_) => imported += 1, Err(e) => errors.push(format!("Row {}: {}", idx + 2, e)) }
        }

        Ok(ImportResult { imported, errors: errors.clone(), total: imported as i64 + errors.len() as i64 })
    }

    async fn import_warehouses<R: std::io::Read>(
        &self,
        headers: &[String],
        mut reader: csv::Reader<R>,
        company_id: Uuid,
    ) -> Result<ImportResult> {
        Self::validate_headers(&["code", "name"], headers)?;
        let mut imported = 0u64;
        let mut errors = Vec::new();

        for (idx, result) in reader.records().enumerate() {
            let record = match result { Ok(r) => r, Err(e) => { errors.push(format!("Row {}: {}", idx + 2, e)); continue; } };

            let id = Uuid::now_v7();
            let res = sqlx::query(
                r#"INSERT INTO warehouses (id, company_id, code, name, is_active, created_at, updated_at)
                   VALUES ($1, $2, $3, $4, true, NOW(), NOW())"#,
            )
            .bind(id).bind(company_id)
            .bind(&record.get(0).unwrap_or(""))
            .bind(&record.get(1).unwrap_or(""))
            .execute(&self.pool).await;

            match res { Ok(_) => imported += 1, Err(e) => errors.push(format!("Row {}: {}", idx + 2, e)) }
        }

        Ok(ImportResult { imported, errors: errors.clone(), total: imported as i64 + errors.len() as i64 })
    }

    async fn import_bank_accounts<R: std::io::Read>(
        &self,
        headers: &[String],
        mut reader: csv::Reader<R>,
        company_id: Uuid,
    ) -> Result<ImportResult> {
        Self::validate_headers(&["bank_name", "account_name", "account_number"], headers)?;
        let mut imported = 0u64;
        let mut errors = Vec::new();

        for (idx, result) in reader.records().enumerate() {
            let record = match result { Ok(r) => r, Err(e) => { errors.push(format!("Row {}: {}", idx + 2, e)); continue; } };

            let id = Uuid::now_v7();
            let res = sqlx::query(
                r#"INSERT INTO bank_accounts (id, company_id, bank_name, account_name, account_number, currency_code, is_active, opening_balance, current_balance, created_at, updated_at)
                   VALUES ($1, $2, $3, $4, $5, 'NGN', true, 0, 0, NOW(), NOW())"#,
            )
            .bind(id).bind(company_id)
            .bind(&record.get(0).unwrap_or(""))
            .bind(&record.get(1).unwrap_or(""))
            .bind(&record.get(2).unwrap_or(""))
            .execute(&self.pool).await;

            match res { Ok(_) => imported += 1, Err(e) => errors.push(format!("Row {}: {}", idx + 2, e)) }
        }

        Ok(ImportResult { imported, errors: errors.clone(), total: imported as i64 + errors.len() as i64 })
    }

    async fn import_asset_categories<R: std::io::Read>(
        &self,
        headers: &[String],
        mut reader: csv::Reader<R>,
        company_id: Uuid,
    ) -> Result<ImportResult> {
        Self::validate_headers(&["name", "depreciation_method"], headers)?;
        let mut imported = 0u64;
        let mut errors = Vec::new();

        for (idx, result) in reader.records().enumerate() {
            let record = match result { Ok(r) => r, Err(e) => { errors.push(format!("Row {}: {}", idx + 2, e)); continue; } };

            let id = Uuid::now_v7();
            let res = sqlx::query(
                r#"INSERT INTO asset_categories (id, company_id, name, depreciation_method, useful_life_years, residual_value_percent, depreciation_rate, created_at, updated_at)
                   VALUES ($1, $2, $3, $4, 5, 0, 0.2, NOW(), NOW())"#,
            )
            .bind(id).bind(company_id)
            .bind(&record.get(0).unwrap_or(""))
            .bind(&record.get(1).unwrap_or("straight_line"))
            .execute(&self.pool).await;

            match res { Ok(_) => imported += 1, Err(e) => errors.push(format!("Row {}: {}", idx + 2, e)) }
        }

        Ok(ImportResult { imported, errors: errors.clone(), total: imported as i64 + errors.len() as i64 })
    }

    async fn import_gl_opening_balances<R: std::io::Read>(
        &self,
        headers: &[String],
        mut reader: csv::Reader<R>,
        company_id: Uuid,
    ) -> Result<ImportResult> {
        Self::validate_headers(&["account_code", "debit", "credit"], headers)?;
        let mut imported = 0u64;
        let mut errors = Vec::new();

        for (idx, result) in reader.records().enumerate() {
            let record = match result { Ok(r) => r, Err(e) => { errors.push(format!("Row {}: {}", idx + 2, e)); continue; } };

            let account_code = record.get(0).unwrap_or("");
            let account_id: Option<Uuid> = sqlx::query_scalar(
                "SELECT id FROM chart_of_accounts WHERE company_id = $1 AND code = $2"
            )
            .bind(company_id).bind(account_code)
            .fetch_optional(&self.pool).await.unwrap_or(None);

            if let Some(acc_id) = account_id {
                let res = sqlx::query(
                    "UPDATE chart_of_accounts SET balance = $1 WHERE id = $2"
                )
                .bind(record.get(1).unwrap_or("0"))
                .bind(acc_id)
                .execute(&self.pool).await;

                match res { Ok(_) => imported += 1, Err(e) => errors.push(format!("Row {}: {}", idx + 2, e)) }
            } else {
                errors.push(format!("Row {}: Account {} not found", idx + 2, account_code));
            }
        }

        Ok(ImportResult { imported, errors: errors.clone(), total: imported as i64 + errors.len() as i64 })
    }

    async fn import_ar_opening_balances<R: std::io::Read>(
        &self,
        headers: &[String],
        mut reader: csv::Reader<R>,
        company_id: Uuid,
    ) -> Result<ImportResult> {
        Self::validate_headers(&["customer_code", "amount"], headers)?;
        let mut imported = 0u64;
        let mut errors = Vec::new();

        for (idx, result) in reader.records().enumerate() {
            let record = match result { Ok(r) => r, Err(e) => { errors.push(format!("Row {}: {}", idx + 2, e)); continue; } };

            let customer_code = record.get(0).unwrap_or("");
            let res = sqlx::query(
                "UPDATE customers SET outstanding_balance = $1 WHERE company_id = $2 AND code = $3"
            )
            .bind(record.get(1).unwrap_or("0"))
            .bind(company_id)
            .bind(customer_code)
            .execute(&self.pool).await;

            match res { Ok(r) if r.rows_affected() > 0 => imported += 1, Ok(_) => errors.push(format!("Row {}: Customer {} not found", idx + 2, customer_code)), Err(e) => errors.push(format!("Row {}: {}", idx + 2, e)) }
        }

        Ok(ImportResult { imported, errors: errors.clone(), total: imported as i64 + errors.len() as i64 })
    }

    async fn import_ap_opening_balances<R: std::io::Read>(
        &self,
        headers: &[String],
        mut reader: csv::Reader<R>,
        company_id: Uuid,
    ) -> Result<ImportResult> {
        Self::validate_headers(&["supplier_code", "amount"], headers)?;
        let mut imported = 0u64;
        let mut errors = Vec::new();

        for (idx, result) in reader.records().enumerate() {
            let record = match result { Ok(r) => r, Err(e) => { errors.push(format!("Row {}: {}", idx + 2, e)); continue; } };

            let supplier_code = record.get(0).unwrap_or("");
            let res = sqlx::query(
                "UPDATE suppliers SET outstanding_balance = $1 WHERE company_id = $2 AND code = $3"
            )
            .bind(record.get(1).unwrap_or("0"))
            .bind(company_id)
            .bind(supplier_code)
            .execute(&self.pool).await;

            match res { Ok(r) if r.rows_affected() > 0 => imported += 1, Ok(_) => errors.push(format!("Row {}: Supplier {} not found", idx + 2, supplier_code)), Err(e) => errors.push(format!("Row {}: {}", idx + 2, e)) }
        }

        Ok(ImportResult { imported, errors: errors.clone(), total: imported as i64 + errors.len() as i64 })
    }

    async fn import_customer_receipts<R: std::io::Read>(
        &self,
        headers: &[String],
        mut reader: csv::Reader<R>,
        company_id: Uuid,
    ) -> Result<ImportResult> {
        Self::validate_headers(&["customer_code", "amount", "receipt_date"], headers)?;
        let mut imported = 0u64;
        let mut errors = Vec::new();

        for (idx, result) in reader.records().enumerate() {
            let record = match result { Ok(r) => r, Err(e) => { errors.push(format!("Row {}: {}", idx + 2, e)); continue; } };

            let id = Uuid::now_v7();
            let receipt_number = format!("RCPT-{}", imported + 1);
            let res = sqlx::query(
                r#"INSERT INTO customer_receipts (id, company_id, customer_id, receipt_number, receipt_date, amount, currency_code, payment_method, created_by, created_at, updated_at)
                   SELECT $1, $2, c.id, $3, $4, $5, 'NGN', 'bank_transfer', '00000000-0000-0000-0000-000000000000', NOW(), NOW()
                   FROM customers c WHERE c.company_id = $2 AND c.code = $6"#,
            )
            .bind(id).bind(company_id).bind(&receipt_number)
            .bind(record.get(2).unwrap_or(""))
            .bind(record.get(1).unwrap_or("0"))
            .bind(record.get(0).unwrap_or(""))
            .execute(&self.pool).await;

            match res { Ok(r) if r.rows_affected() > 0 => imported += 1, Ok(_) => errors.push(format!("Row {}: Customer not found", idx + 2)), Err(e) => errors.push(format!("Row {}: {}", idx + 2, e)) }
        }

        Ok(ImportResult { imported, errors: errors.clone(), total: imported as i64 + errors.len() as i64 })
    }

    async fn import_supplier_payments<R: std::io::Read>(
        &self,
        headers: &[String],
        mut reader: csv::Reader<R>,
        company_id: Uuid,
    ) -> Result<ImportResult> {
        Self::validate_headers(&["supplier_code", "amount", "payment_date"], headers)?;
        let mut imported = 0u64;
        let mut errors = Vec::new();

        for (idx, result) in reader.records().enumerate() {
            let record = match result { Ok(r) => r, Err(e) => { errors.push(format!("Row {}: {}", idx + 2, e)); continue; } };

            let id = Uuid::now_v7();
            let payment_number = format!("SPAY-{}", imported + 1);
            let res = sqlx::query(
                r#"INSERT INTO supplier_payments (id, company_id, supplier_id, payment_number, payment_date, amount, currency_code, payment_method, created_by, created_at, updated_at)
                   SELECT $1, $2, s.id, $3, $4, $5, 'NGN', 'bank_transfer', '00000000-0000-0000-0000-000000000000', NOW(), NOW()
                   FROM suppliers s WHERE s.company_id = $2 AND s.code = $6"#,
            )
            .bind(id).bind(company_id).bind(&payment_number)
            .bind(record.get(2).unwrap_or(""))
            .bind(record.get(1).unwrap_or("0"))
            .bind(record.get(0).unwrap_or(""))
            .execute(&self.pool).await;

            match res { Ok(r) if r.rows_affected() > 0 => imported += 1, Ok(_) => errors.push(format!("Row {}: Supplier not found", idx + 2)), Err(e) => errors.push(format!("Row {}: {}", idx + 2, e)) }
        }

        Ok(ImportResult { imported, errors: errors.clone(), total: imported as i64 + errors.len() as i64 })
    }

    async fn import_fixed_assets<R: std::io::Read>(
        &self,
        headers: &[String],
        mut reader: csv::Reader<R>,
        company_id: Uuid,
    ) -> Result<ImportResult> {
        Self::validate_headers(&["asset_code", "name", "acquisition_cost"], headers)?;
        let mut imported = 0u64;
        let mut errors = Vec::new();

        for (idx, result) in reader.records().enumerate() {
            let record = match result { Ok(r) => r, Err(e) => { errors.push(format!("Row {}: {}", idx + 2, e)); continue; } };

            let id = Uuid::now_v7();
            let res = sqlx::query(
                r#"INSERT INTO fixed_assets (id, company_id, asset_code, name, acquisition_cost, status, created_by, created_at, updated_at)
                   VALUES ($1, $2, $3, $4, $5, 'draft', '00000000-0000-0000-0000-000000000000', NOW(), NOW())"#,
            )
            .bind(id).bind(company_id)
            .bind(&record.get(0).unwrap_or(""))
            .bind(&record.get(1).unwrap_or(""))
            .bind(record.get(2).unwrap_or("0"))
            .execute(&self.pool).await;

            match res { Ok(_) => imported += 1, Err(e) => errors.push(format!("Row {}: {}", idx + 2, e)) }
        }

        Ok(ImportResult { imported, errors: errors.clone(), total: imported as i64 + errors.len() as i64 })
    }

    async fn import_stock_opening_balances<R: std::io::Read>(
        &self,
        headers: &[String],
        mut reader: csv::Reader<R>,
        company_id: Uuid,
    ) -> Result<ImportResult> {
        Self::validate_headers(&["product_code", "quantity", "unit_cost"], headers)?;
        let mut imported = 0u64;
        let mut errors = Vec::new();

        for (idx, result) in reader.records().enumerate() {
            let record = match result { Ok(r) => r, Err(e) => { errors.push(format!("Row {}: {}", idx + 2, e)); continue; } };

            let id = Uuid::now_v7();
            let res = sqlx::query(
                r#"INSERT INTO stock_balances (id, company_id, product_id, quantity, unit_cost, created_at, updated_at)
                   SELECT $1, $2, p.id, $3, $4, NOW(), NOW()
                   FROM products p WHERE p.company_id = $2 AND p.code = $5"#,
            )
            .bind(id).bind(company_id)
            .bind(record.get(1).unwrap_or("0"))
            .bind(record.get(2).unwrap_or("0"))
            .bind(record.get(0).unwrap_or(""))
            .execute(&self.pool).await;

            match res { Ok(r) if r.rows_affected() > 0 => imported += 1, Ok(_) => errors.push(format!("Row {}: Product not found", idx + 2)), Err(e) => errors.push(format!("Row {}: {}", idx + 2, e)) }
        }

        Ok(ImportResult { imported, errors: errors.clone(), total: imported as i64 + errors.len() as i64 })
    }

    async fn import_employees<R: std::io::Read>(
        &self,
        headers: &[String],
        mut reader: csv::Reader<R>,
        company_id: Uuid,
    ) -> Result<ImportResult> {
        Self::validate_headers(&["employee_number", "first_name", "last_name", "salary_amount"], headers)?;
        let mut imported = 0u64;
        let mut errors = Vec::new();

        for (idx, result) in reader.records().enumerate() {
            let record = match result { Ok(r) => r, Err(e) => { errors.push(format!("Row {}: {}", idx + 2, e)); continue; } };

            let id = Uuid::now_v7();
            let res = sqlx::query(
                r#"INSERT INTO employees (id, company_id, employee_number, first_name, last_name, salary_amount, currency_code, employment_type, is_active, hire_date, created_at, updated_at)
                   VALUES ($1, $2, $3, $4, $5, $6, 'NGN', 'full_time', true, CURRENT_DATE, NOW(), NOW())"#,
            )
            .bind(id).bind(company_id)
            .bind(&record.get(0).unwrap_or(""))
            .bind(&record.get(1).unwrap_or(""))
            .bind(&record.get(2).unwrap_or(""))
            .bind(record.get(3).unwrap_or("0"))
            .execute(&self.pool).await;

            match res { Ok(_) => imported += 1, Err(e) => errors.push(format!("Row {}: {}", idx + 2, e)) }
        }

        Ok(ImportResult { imported, errors: errors.clone(), total: imported as i64 + errors.len() as i64 })
    }
}

pub struct ImportResult {
    pub imported: u64,
    pub errors: Vec<String>,
    pub total: i64,
}
