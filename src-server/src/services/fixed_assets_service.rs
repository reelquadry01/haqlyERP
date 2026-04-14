// Author: Quadri Atharu
use anyhow::{anyhow, Result};
use bigdecimal::BigDecimal;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

use crate::models::fixed_asset::{
    AssetCategory, AssetStatus, DepreciationMethod, DepreciationSchedule, FixedAsset,
};
use crate::models::posting::PostingContext;
use crate::services::posting_service::PostingService;

#[derive(Clone)]
pub struct FixedAssetsService {
    pub pool: PgPool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisposalResult {
    pub asset_id: Uuid,
    pub asset_code: String,
    pub asset_name: String,
    pub acquisition_cost: BigDecimal,
    pub accumulated_depreciation: BigDecimal,
    pub net_book_value: BigDecimal,
    pub disposal_proceeds: BigDecimal,
    pub gain_or_loss: BigDecimal,
    pub is_gain: bool,
    pub journal_header_id: Option<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetRegisterEntry {
    pub asset_id: Uuid,
    pub asset_code: String,
    pub asset_name: String,
    pub category_name: String,
    pub acquisition_date: chrono::NaiveDate,
    pub acquisition_cost: BigDecimal,
    pub accumulated_depreciation: BigDecimal,
    pub net_book_value: BigDecimal,
    pub depreciation_method: DepreciationMethod,
    pub useful_life_years: i32,
    pub status: AssetStatus,
    pub location: Option<String>,
    pub custodian: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct DepreciationScheduleEntry {
    pub period_start: chrono::NaiveDate,
    pub period_end: chrono::NaiveDate,
    pub opening_book_value: BigDecimal,
    pub depreciation_amount: BigDecimal,
    pub closing_book_value: BigDecimal,
    pub cumulative_depreciation: BigDecimal,
}

impl FixedAssetsService {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn create_category(
        &self,
        company_id: Uuid,
        name: String,
        depreciation_method: DepreciationMethod,
        useful_life_years: i32,
        residual_value_percent: BigDecimal,
        depreciation_rate: BigDecimal,
        asset_account_id: Uuid,
        accumulated_dep_account_id: Uuid,
        depreciation_expense_account_id: Uuid,
        disposal_account_id: Option<Uuid>,
        capital_allowance_category: Option<String>,
    ) -> Result<AssetCategory> {
        let id = Uuid::now_v7();
        sqlx::query(
            r#"INSERT INTO asset_categories (id, company_id, name, depreciation_method, useful_life_years, residual_value_percent, depreciation_rate, asset_account_id, accumulated_dep_account_id, depreciation_expense_account_id, disposal_account_id, capital_allowance_category, created_at, updated_at)
               VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, NOW(), NOW())"#,
        )
        .bind(id)
        .bind(company_id)
        .bind(&name)
        .bind(&depreciation_method)
        .bind(useful_life_years)
        .bind(&residual_value_percent)
        .bind(&depreciation_rate)
        .bind(asset_account_id)
        .bind(accumulated_dep_account_id)
        .bind(depreciation_expense_account_id)
        .bind(disposal_account_id)
        .bind(&capital_allowance_category)
        .execute(&self.pool)
        .await?;

        sqlx::query_as::<_, AssetCategory>("SELECT * FROM asset_categories WHERE id = $1")
            .bind(id)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| anyhow!("Failed to fetch category: {}", e))
    }

    pub async fn register_asset(
        &self,
        company_id: Uuid,
        branch_id: Option<Uuid>,
        category_id: Uuid,
        asset_code: String,
        name: String,
        description: Option<String>,
        acquisition_date: chrono::NaiveDate,
        acquisition_cost: BigDecimal,
        residual_value: BigDecimal,
        useful_life_years: i32,
        depreciation_method: DepreciationMethod,
        depreciation_rate: BigDecimal,
        asset_account_id: Uuid,
        accumulated_dep_account_id: Uuid,
        depreciation_expense_account_id: Uuid,
        disposal_account_id: Option<Uuid>,
        location: Option<String>,
        custodian: Option<String>,
        serial_number: Option<String>,
        created_by: Uuid,
    ) -> Result<FixedAsset> {
        let id = Uuid::now_v7();
        let net_book_value = &acquisition_cost - &residual_value;

        sqlx::query(
            r#"INSERT INTO fixed_assets (id, company_id, branch_id, category_id, asset_code, name, description, acquisition_date, acquisition_cost, residual_value, useful_life_years, depreciation_method, depreciation_rate, accumulated_depreciation, net_book_value, status, location, custodian, serial_number, asset_account_id, accumulated_dep_account_id, depreciation_expense_account_id, disposal_account_id, created_by, created_at, updated_at)
               VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, 0, $14, 'active', $15, $16, $17, $18, $19, $20, $21, $22, NOW(), NOW())"#,
        )
        .bind(id)
        .bind(company_id)
        .bind(branch_id)
        .bind(category_id)
        .bind(&asset_code)
        .bind(&name)
        .bind(&description)
        .bind(acquisition_date)
        .bind(&acquisition_cost)
        .bind(&residual_value)
        .bind(useful_life_years)
        .bind(&depreciation_method)
        .bind(&depreciation_rate)
        .bind(&net_book_value)
        .bind(&location)
        .bind(&custodian)
        .bind(&serial_number)
        .bind(asset_account_id)
        .bind(accumulated_dep_account_id)
        .bind(depreciation_expense_account_id)
        .bind(disposal_account_id)
        .bind(created_by)
        .execute(&self.pool)
        .await?;

        sqlx::query_as::<_, FixedAsset>("SELECT * FROM fixed_assets WHERE id = $1")
            .bind(id)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| anyhow!("Failed to fetch asset: {}", e))
    }

    pub async fn list_assets(&self, company_id: Uuid) -> Result<Vec<FixedAsset>> {
        let assets = sqlx::query_as::<_, FixedAsset>(
            "SELECT * FROM fixed_assets WHERE company_id = $1 AND status != 'disposed' ORDER BY asset_code",
        )
        .bind(company_id)
        .fetch_all(&self.pool)
        .await?;
        Ok(assets)
    }

    pub async fn compute_depreciation(
        &self,
        asset_id: Uuid,
    ) -> Result<BigDecimal> {
        let asset = sqlx::query_as::<_, FixedAsset>("SELECT * FROM fixed_assets WHERE id = $1")
            .bind(asset_id)
            .fetch_one(&self.pool)
            .await?;

        if asset.status != AssetStatus::Active {
            return Ok(BigDecimal::from(0));
        }

        let depreciable_amount = &asset.acquisition_cost - &asset.residual_value;
        let months_depreciated = self.count_depreciation_months(asset.id).await?;
        let remaining_months = (asset.useful_life_years as i64 * 12) - months_depreciated;

        if remaining_months <= 0 {
            return Ok(BigDecimal::from(0));
        }

        let annual_dep = match asset.depreciation_method {
            DepreciationMethod::StraightLine => {
                &depreciable_amount / BigDecimal::from(asset.useful_life_years)
            }
            DepreciationMethod::DecliningBalance => {
                &asset.net_book_value * &asset.depreciation_rate / BigDecimal::from(100)
            }
            DepreciationMethod::SumOfYearsDigits => {
                let n = asset.useful_life_years;
                let sum_years = n * (n + 1) / 2;
                let remaining_life_years = (remaining_months as f64 / 12.0).ceil() as i32;
                &depreciable_amount * BigDecimal::from(remaining_life_years) / BigDecimal::from(sum_years)
            }
        };

        let monthly_dep = &annual_dep / BigDecimal::from(12);
        Ok(monthly_dep)
    }

    pub async fn run_depreciation(
        &self,
        company_id: Uuid,
        period_id: Uuid,
        fiscal_year_id: Uuid,
        run_by: Uuid,
    ) -> Result<Vec<DepreciationSchedule>> {
        let assets = sqlx::query_as::<_, FixedAsset>(
            "SELECT * FROM fixed_assets WHERE company_id = $1 AND status = 'active'",
        )
        .bind(company_id)
        .fetch_all(&self.pool)
        .await?;

        let mut schedules = Vec::new();

        for asset in assets {
            let dep_amount = self.compute_depreciation(asset.id).await?;
            if dep_amount == BigDecimal::from(0) {
                continue;
            }

            let new_accumulated = &asset.accumulated_depreciation + &dep_amount;
            let new_nbv = &asset.acquisition_cost - &new_accumulated;
            let closing_nbv = if new_nbv < asset.residual_value {
                asset.residual_value.clone()
            } else {
                new_nbv
            };

            let actual_dep = if closing_nbv != new_nbv {
                &asset.net_book_value - &asset.residual_value
            } else {
                dep_amount
            };
            let final_accumulated = &asset.accumulated_depreciation + &actual_dep;

            let schedule_id = Uuid::now_v7();
            sqlx::query(
                r#"INSERT INTO depreciation_schedules (id, asset_id, period_id, fiscal_year_id, depreciation_date, opening_book_value, depreciation_amount, closing_book_value, is_posted, created_at)
                   VALUES ($1, $2, $3, $4, CURRENT_DATE, $5, $6, $7, false, NOW())"#,
            )
            .bind(schedule_id)
            .bind(asset.id)
            .bind(period_id)
            .bind(fiscal_year_id)
            .bind(&asset.net_book_value)
            .bind(&actual_dep)
            .bind(&closing_nbv)
            .execute(&self.pool)
            .await?;

            sqlx::query(
                "UPDATE fixed_assets SET accumulated_depreciation = $1, net_book_value = $2, updated_at = NOW() WHERE id = $3",
            )
            .bind(&final_accumulated)
            .bind(&closing_nbv)
            .bind(asset.id)
            .execute(&self.pool)
            .await?;

            schedules.push(
                sqlx::query_as::<_, DepreciationSchedule>(
                    "SELECT * FROM depreciation_schedules WHERE id = $1",
                )
                .bind(schedule_id)
                .fetch_one(&self.pool)
                .await?,
            );
        }

        Ok(schedules)
    }

    pub async fn dispose_asset(
        &self,
        asset_id: Uuid,
        disposal_proceeds: BigDecimal,
        disposal_date: chrono::NaiveDate,
        disposed_by: Uuid,
    ) -> Result<DisposalResult> {
        let asset = sqlx::query_as::<_, FixedAsset>("SELECT * FROM fixed_assets WHERE id = $1")
            .bind(asset_id)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| anyhow!("Asset not found: {}", e))?;

        if asset.status == AssetStatus::Disposed {
            return Err(anyhow!("Asset is already disposed"));
        }

        let gain_or_loss = &disposal_proceeds - &asset.net_book_value;
        let is_gain = gain_or_loss > BigDecimal::from(0);

        let posting_service = PostingService::new(self.pool.clone());
        let context = PostingContext {
            company_id: asset.company_id,
            source_module: "fixed_assets".to_string(),
            source_document_id: Some(asset.id),
            source_document_number: Some(asset.asset_code.clone()),
            reference_id: None,
            customer_or_vendor: None,
            branch: asset.branch_id,
            department: None,
            cost_center: None,
            project: None,
            tax_code: None,
            currency: "NGN".to_string(),
            amount: disposal_proceeds.clone(),
            tax_amount: None,
            discount_amount: None,
            narration: Some(format!(
                "Disposal of {} - {} ({}: {})",
                asset.name,
                asset.asset_code,
                if is_gain { "Gain" } else { "Loss" },
                gain_or_loss.abs()
            )),
            correlation_id: None,
            idempotency_key: Some(format!("disposal-{}", asset.id)),
            transaction_type: "asset_disposal".to_string(),
            transaction_subtype: Some(if is_gain { "gain" } else { "loss" }.to_string()),
            posted_by: Some(disposed_by),
            posting_date: disposal_date,
        };

        let journal = posting_service.post(context).await.ok();

        sqlx::query(
            r#"UPDATE fixed_assets
               SET status = 'disposed',
                   net_book_value = 0,
                   accumulated_depreciation = acquisition_cost,
                   updated_at = NOW()
               WHERE id = $1"#,
        )
        .bind(asset_id)
        .execute(&self.pool)
        .await?;

        Ok(DisposalResult {
            asset_id,
            asset_code: asset.asset_code,
            asset_name: asset.name,
            acquisition_cost: asset.acquisition_cost,
            accumulated_depreciation: asset.accumulated_depreciation,
            net_book_value: asset.net_book_value.clone(),
            disposal_proceeds,
            gain_or_loss: gain_or_loss.abs(),
            is_gain,
            journal_header_id: journal.map(|j| j.id),
        })
    }

    pub async fn get_depreciation_schedule(&self, asset_id: Uuid) -> Result<Vec<DepreciationScheduleEntry>> {
        let asset = sqlx::query_as::<_, FixedAsset>("SELECT * FROM fixed_assets WHERE id = $1")
            .bind(asset_id)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| anyhow!("Asset not found: {}", e))?;

        let existing: Vec<DepreciationSchedule> = sqlx::query_as::<_, DepreciationSchedule>(
            "SELECT * FROM depreciation_schedules WHERE asset_id = $1 ORDER BY depreciation_date",
        )
        .bind(asset_id)
        .fetch_all(&self.pool)
        .await?;

        if !existing.is_empty() {
            let mut entries = Vec::new();
            let mut cumulative = BigDecimal::from(0);
            for sched in &existing {
                cumulative += &sched.depreciation_amount;
                entries.push(DepreciationScheduleEntry {
                    period_start: sched.depreciation_date,
                    period_end: sched.depreciation_date,
                    opening_book_value: sched.opening_book_value.clone(),
                    depreciation_amount: sched.depreciation_amount.clone(),
                    closing_book_value: sched.closing_book_value.clone(),
                    cumulative_depreciation: cumulative.clone(),
                });
            }

            let depreciable_amount = &asset.acquisition_cost - &asset.residual_value;
            let months_depreciated = existing.len() as i32;
            let total_months = asset.useful_life_years * 12;

            if months_depreciated < total_months && asset.status == AssetStatus::Active {
                let mut opening = entries.last().map(|e| e.closing_book_value.clone()).unwrap_or(asset.net_book_value.clone());
                for m in months_depreciated..total_months {
                    let dep = match asset.depreciation_method {
                        DepreciationMethod::StraightLine => {
                            &depreciable_amount / BigDecimal::from(total_months)
                        }
                        DepreciationMethod::DecliningBalance => {
                            &opening * &asset.depreciation_rate / BigDecimal::from(1200)
                        }
                        DepreciationMethod::SumOfYearsDigits => {
                            let remaining = total_months - m;
                            let n = asset.useful_life_years;
                            let sum_years = n * (n + 1) / 2;
                            let remaining_years = (remaining as f64 / 12.0).ceil() as i32;
                            &depreciable_amount * BigDecimal::from(remaining_years)
                                / BigDecimal::from(sum_years)
                                / BigDecimal::from(12)
                        }
                    };
                    let closing = &opening - &dep;
                    cumulative += &dep;
                    entries.push(DepreciationScheduleEntry {
                        period_start: chrono::NaiveDate::from_ymd_opt(
                            asset.acquisition_date.year() + (m / 12),
                            ((m % 12) + 1) as u32,
                            1,
                        ).unwrap_or(asset.acquisition_date),
                        period_end: chrono::NaiveDate::from_ymd_opt(
                            asset.acquisition_date.year() + (m / 12),
                            ((m % 12) + 1) as u32,
                            28,
                        ).unwrap_or(asset.acquisition_date),
                        opening_book_value: opening.clone(),
                        depreciation_amount: dep.clone(),
                        closing_book_value: closing.clone(),
                        cumulative_depreciation: cumulative.clone(),
                    });
                    opening = closing;
                }
            }

            return Ok(entries);
        }

        let depreciable_amount = &asset.acquisition_cost - &asset.residual_value;
        let total_months = asset.useful_life_years * 12;
        let mut entries = Vec::new();
        let mut opening = asset.acquisition_cost.clone();
        let mut cumulative = BigDecimal::from(0);

        for m in 0..total_months {
            let dep = match asset.depreciation_method {
                DepreciationMethod::StraightLine => {
                    &depreciable_amount / BigDecimal::from(total_months)
                }
                DepreciationMethod::DecliningBalance => {
                    &opening * &asset.depreciation_rate / BigDecimal::from(1200)
                }
                DepreciationMethod::SumOfYearsDigits => {
                    let remaining = total_months - m;
                    let n = asset.useful_life_years;
                    let sum_years = n * (n + 1) / 2;
                    let remaining_years = (remaining as f64 / 12.0).ceil() as i32;
                    &depreciable_amount * BigDecimal::from(remaining_years)
                        / BigDecimal::from(sum_years)
                        / BigDecimal::from(12)
                }
            };
            let closing = &opening - &dep;
            cumulative += &dep;
            entries.push(DepreciationScheduleEntry {
                period_start: chrono::NaiveDate::from_ymd_opt(
                    asset.acquisition_date.year() + (m / 12),
                    ((m % 12) + 1) as u32,
                    1,
                ).unwrap_or(asset.acquisition_date),
                period_end: chrono::NaiveDate::from_ymd_opt(
                    asset.acquisition_date.year() + (m / 12),
                    ((m % 12) + 1) as u32,
                    28,
                ).unwrap_or(asset.acquisition_date),
                opening_book_value: opening.clone(),
                depreciation_amount: dep.clone(),
                closing_book_value: closing.clone(),
                cumulative_depreciation: cumulative.clone(),
            });
            opening = closing;
        }

        Ok(entries)
    }

    pub async fn get_asset_register(&self, company_id: Uuid) -> Result<Vec<AssetRegisterEntry>> {
        let assets = sqlx::query_as::<_, FixedAsset>(
            "SELECT * FROM fixed_assets WHERE company_id = $1 ORDER BY asset_code",
        )
        .bind(company_id)
        .fetch_all(&self.pool)
        .await?;

        let mut entries = Vec::new();
        for asset in assets {
            let category_name: String = sqlx::query_scalar(
                "SELECT name FROM asset_categories WHERE id = $1",
            )
            .bind(asset.category_id)
            .fetch_one(&self.pool)
            .await
            .unwrap_or_else(|_| "Unknown".to_string());

            entries.push(AssetRegisterEntry {
                asset_id: asset.id,
                asset_code: asset.asset_code,
                asset_name: asset.name,
                category_name,
                acquisition_date: asset.acquisition_date,
                acquisition_cost: asset.acquisition_cost,
                accumulated_depreciation: asset.accumulated_depreciation,
                net_book_value: asset.net_book_value,
                depreciation_method: asset.depreciation_method,
                useful_life_years: asset.useful_life_years,
                status: asset.status,
                location: asset.location,
                custodian: asset.custodian,
            });
        }

        Ok(entries)
    }

    pub async fn post_depreciation_to_gl(
        &self,
        schedule_ids: Vec<Uuid>,
        posted_by: Uuid,
    ) -> Result<()> {
        let posting_service = PostingService::new(self.pool.clone());

        for schedule_id in schedule_ids {
            let schedule = sqlx::query_as::<_, DepreciationSchedule>(
                "SELECT * FROM depreciation_schedules WHERE id = $1",
            )
            .bind(schedule_id)
            .fetch_one(&self.pool)
            .await?;

            if schedule.is_posted {
                continue;
            }

            let asset = sqlx::query_as::<_, FixedAsset>("SELECT * FROM fixed_assets WHERE id = $1")
                .bind(schedule.asset_id)
                .fetch_one(&self.pool)
                .await?;

            let context = PostingContext {
                company_id: asset.company_id,
                source_module: "fixed_assets".to_string(),
                source_document_id: Some(schedule.id),
                source_document_number: Some(asset.asset_code.clone()),
                reference_id: None,
                customer_or_vendor: None,
                branch: asset.branch_id,
                department: None,
                cost_center: None,
                project: None,
                tax_code: None,
                currency: "NGN".to_string(),
                amount: schedule.depreciation_amount.clone(),
                tax_amount: None,
                discount_amount: None,
                narration: Some(format!("Depreciation for {}", asset.name)),
                correlation_id: None,
                idempotency_key: Some(format!("dep-{}", schedule.id)),
                transaction_type: "depreciation".to_string(),
                transaction_subtype: None,
                posted_by: Some(posted_by),
                posting_date: schedule.depreciation_date,
            };

            let journal = posting_service.post(context).await?;

            sqlx::query(
                "UPDATE depreciation_schedules SET is_posted = true, journal_header_id = $1 WHERE id = $2",
            )
            .bind(journal.id)
            .bind(schedule_id)
            .execute(&self.pool)
            .await?;
        }

        Ok(())
    }

    async fn count_depreciation_months(&self, asset_id: Uuid) -> Result<i64> {
        let count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM depreciation_schedules WHERE asset_id = $1",
        )
        .bind(asset_id)
        .fetch_one(&self.pool)
        .await?;
        Ok(count)
    }
}
