// Author: Quadri Atharu
use anyhow::{anyhow, Result};
use bigdecimal::{BigDecimal, FromPrimitive};
use chrono::NaiveDate;
use sqlx::PgPool;
use uuid::Uuid;

use crate::models::fixed_asset::{
    DepreciationMethod, DepreciationRun, DepreciationSchedule, FixedAsset,
};

#[derive(Clone)]
pub struct DepreciationService {
    pub pool: PgPool,
}

impl DepreciationService {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub fn compute_straight_line(
        cost: &BigDecimal,
        residual_value: &BigDecimal,
        useful_life_years: i32,
    ) -> BigDecimal {
        if useful_life_years <= 0 {
            return BigDecimal::from(0);
        }
        (cost - residual_value) / BigDecimal::from(useful_life_years)
    }

    pub fn compute_declining_balance(
        book_value: &BigDecimal,
        rate: &BigDecimal,
        residual_value: &BigDecimal,
    ) -> BigDecimal {
        let depreciation = book_value * rate;
        let max_depreciation = book_value - residual_value;
        if depreciation > max_depreciation {
            if max_depreciation > BigDecimal::from(0) {
                max_depreciation
            } else {
                BigDecimal::from(0)
            }
        } else {
            depreciation
        }
    }

    pub fn generate_schedule(
        asset: &FixedAsset,
        fiscal_year_id: Uuid,
        periods: i32,
    ) -> Vec<DepreciationScheduleEntry> {
        let mut schedule = Vec::new();
        let annual_depreciation = match asset.depreciation_method {
            DepreciationMethod::StraightLine => Self::compute_straight_line(
                &asset.acquisition_cost,
                &asset.residual_value,
                asset.useful_life_years,
            ),
            DepreciationMethod::DecliningBalance => Self::compute_declining_balance(
                &asset.net_book_value,
                &asset.depreciation_rate,
                &asset.residual_value,
            ),
            DepreciationMethod::SumOfYearsDigits => {
                let n = asset.useful_life_years;
                let sum_of_years = (n * (n + 1)) / 2;
                let depreciable = &asset.acquisition_cost - &asset.residual_value;
                &depreciable * (BigDecimal::from(n) / BigDecimal::from(sum_of_years))
            }
        };

        let monthly_depreciation = if periods > 0 {
            &annual_depreciation / BigDecimal::from(periods)
        } else {
            annual_depreciation.clone()
        };

        let mut opening = asset.net_book_value.clone();
        for period_num in 1..=periods {
            let depreciation = if opening <= asset.residual_value {
                BigDecimal::from(0)
            } else if &opening - &monthly_depreciation < asset.residual_value {
                &opening - &asset.residual_value
            } else {
                monthly_depreciation.clone()
            };

            let closing = &opening - &depreciation;

            schedule.push(DepreciationScheduleEntry {
                id: Uuid::now_v7(),
                asset_id: asset.id,
                period_id: Uuid::nil(),
                fiscal_year_id,
                depreciation_date: NaiveDate::from_ymd_opt(1, 1, 1).unwrap(),
                opening_book_value: opening.clone(),
                depreciation_amount: depreciation.clone(),
                closing_book_value: closing.clone(),
                is_posted: false,
                journal_header_id: None,
                created_at: chrono::Utc::now().naive_utc(),
            });

            opening = closing;
        }

        schedule
    }

    pub async fn run_depreciation(
        &self,
        company_id: Uuid,
        period_id: Uuid,
    ) -> Result<DepreciationRun> {
        let assets = sqlx::query_as::<_, FixedAsset>(
            "SELECT * FROM fixed_assets WHERE company_id = $1 AND status = 'active'",
        )
        .bind(company_id)
        .fetch_all(&self.pool)
        .await?;

        let period: crate::models::account::AccountingPeriod =
            sqlx::query_as("SELECT * FROM accounting_periods WHERE id = $1")
                .bind(period_id)
                .fetch_one(&self.pool)
                .await?;

        let run_id = Uuid::now_v7();
        let mut total_depreciation = BigDecimal::from(0);
        let mut asset_count = 0i32;

        for asset in &assets {
            let schedule = Self::generate_schedule(asset, period.fiscal_year_id, 12);

            let monthly_dep = if schedule.is_empty() {
                BigDecimal::from(0)
            } else {
                schedule[0].depreciation_amount.clone()
            };

            if monthly_dep == BigDecimal::from(0) {
                continue;
            }

            let schedule_id = Uuid::now_v7();
            sqlx::query(
                r#"INSERT INTO depreciation_schedules (id, asset_id, period_id, fiscal_year_id, depreciation_date, opening_book_value, depreciation_amount, closing_book_value, is_posted, created_at)
                   VALUES ($1, $2, $3, $4, $5, $6, $7, $8, false, NOW())"#,
            )
            .bind(schedule_id)
            .bind(asset.id)
            .bind(period_id)
            .bind(period.fiscal_year_id)
            .bind(period.start_date)
            .bind(&asset.net_book_value)
            .bind(&monthly_dep)
            .bind(&(&asset.net_book_value - &monthly_dep))
            .execute(&self.pool)
            .await?;

            sqlx::query(
                r#"UPDATE fixed_assets SET accumulated_depreciation = accumulated_depreciation + $1, net_book_value = net_book_value - $1, updated_at = NOW() WHERE id = $2"#,
            )
            .bind(&monthly_dep)
            .bind(asset.id)
            .execute(&self.pool)
            .await?;

            total_depreciation = total_depreciation + monthly_dep;
            asset_count += 1;
        }

        sqlx::query(
            r#"INSERT INTO depreciation_runs (id, company_id, fiscal_year_id, period_id, run_date, total_depreciation, asset_count, status, created_by, created_at)
               VALUES ($1, $2, $3, $4, $5, $6, $7, 'processed', $8, NOW())"#,
        )
        .bind(run_id)
        .bind(company_id)
        .bind(period.fiscal_year_id)
        .bind(period_id)
        .bind(period.start_date)
        .bind(&total_depreciation)
        .bind(asset_count)
        .bind(Uuid::nil())
        .execute(&self.pool)
        .await?;

        sqlx::query_as::<_, DepreciationRun>("SELECT * FROM depreciation_runs WHERE id = $1")
            .bind(run_id)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| anyhow!("Failed to fetch depreciation run: {}", e))
    }

    pub async fn post_run_to_gl(
        &self,
        run_id: Uuid,
        posted_by: Uuid,
    ) -> Result<DepreciationRun> {
        let run = sqlx::query_as::<_, DepreciationRun>("SELECT * FROM depreciation_runs WHERE id = $1")
            .bind(run_id)
            .fetch_one(&self.pool)
            .await?;

        if run.status != "processed" {
            return Err(anyhow!("Depreciation run must be in processed status to post"));
        }

        if run.total_depreciation == BigDecimal::from(0) {
            return Err(anyhow!("No depreciation to post"));
        }

        let schedules = sqlx::query_as::<_, DepreciationSchedule>(
            "SELECT * FROM depreciation_schedules WHERE fiscal_year_id = $1 AND period_id = $2 AND is_posted = false",
        )
        .bind(run.fiscal_year_id)
        .bind(run.period_id)
        .fetch_all(&self.pool)
        .await?;

        let mut total_debit = BigDecimal::from(0);
        let mut total_credit = BigDecimal::from(0);

        let journal_id = Uuid::now_v7();
        sqlx::query(
            r#"INSERT INTO journal_headers (id, company_id, fiscal_year_id, period_id, entry_number, narration, status, journal_type, source_module, total_debit, total_credit, currency_code, posted_at, posted_by, created_by, created_at, updated_at)
               VALUES ($1, $2, $3, $4, $5, 'Depreciation posting', 'posted', 'auto', 'fixed_assets', $6, $7, 'NGN', NOW(), $8, $8, NOW(), NOW())"#,
        )
        .bind(journal_id)
        .bind(run.company_id)
        .bind(run.fiscal_year_id)
        .bind(run.period_id)
        .bind(format!("DEP-{}", run_id))
        .bind(&run.total_depreciation)
        .bind(&run.total_depreciation)
        .bind(posted_by)
        .execute(&self.pool)
        .await?;

        for schedule in &schedules {
            let asset = sqlx::query_as::<_, FixedAsset>("SELECT * FROM fixed_assets WHERE id = $1")
                .bind(schedule.asset_id)
                .fetch_one(&self.pool)
                .await?;

            sqlx::query(
                r#"INSERT INTO journal_lines (id, journal_header_id, account_id, line_number, narration, debit, credit, currency_code, created_at)
                   VALUES ($1, $2, $3, 1, 'Depreciation expense', $4, 0, 'NGN', NOW())"#,
            )
            .bind(Uuid::now_v7())
            .bind(journal_id)
            .bind(asset.depreciation_expense_account_id)
            .bind(&schedule.depreciation_amount)
            .execute(&self.pool)
            .await?;

            sqlx::query(
                r#"INSERT INTO journal_lines (id, journal_header_id, account_id, line_number, narration, debit, credit, currency_code, created_at)
                   VALUES ($1, $2, $3, 2, 'Accumulated depreciation', 0, $4, 'NGN', NOW())"#,
            )
            .bind(Uuid::now_v7())
            .bind(journal_id)
            .bind(asset.accumulated_dep_account_id)
            .bind(&schedule.depreciation_amount)
            .execute(&self.pool)
            .await?;

            total_debit = total_debit + schedule.depreciation_amount.clone();
            total_credit = total_credit + schedule.depreciation_amount.clone();

            sqlx::query("UPDATE depreciation_schedules SET is_posted = true, journal_header_id = $1 WHERE id = $2")
                .bind(journal_id)
                .bind(schedule.id)
                .execute(&self.pool)
                .await?;
        }

        sqlx::query(
            "UPDATE journal_headers SET total_debit = $1, total_credit = $2 WHERE id = $3",
        )
        .bind(&total_debit)
        .bind(&total_credit)
        .bind(journal_id)
        .execute(&self.pool)
        .await?;

        sqlx::query("UPDATE depreciation_runs SET status = 'posted', journal_header_id = $1 WHERE id = $2")
            .bind(journal_id)
            .bind(run_id)
            .execute(&self.pool)
            .await?;

        sqlx::query_as::<_, DepreciationRun>("SELECT * FROM depreciation_runs WHERE id = $1")
            .bind(run_id)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| anyhow!("Failed to fetch depreciation run: {}", e))
    }
}

struct DepreciationScheduleEntry {
    id: Uuid,
    asset_id: Uuid,
    period_id: Uuid,
    fiscal_year_id: Uuid,
    depreciation_date: NaiveDate,
    opening_book_value: BigDecimal,
    depreciation_amount: BigDecimal,
    closing_book_value: BigDecimal,
    is_posted: bool,
    journal_header_id: Option<Uuid>,
    created_at: chrono::NaiveDateTime,
}
