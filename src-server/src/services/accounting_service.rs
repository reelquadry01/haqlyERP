// Author: Quadri Atharu
use anyhow::{anyhow, Result};
use bigdecimal::BigDecimal;
use chrono::NaiveDate;
use sqlx::PgPool;
use uuid::Uuid;

use crate::models::account::{
    Account, AccountType, AccountingPeriod, FiscalYear, PeriodStatus,
};

#[derive(Clone)]
pub struct AccountingService {
    pub pool: PgPool,
}

impl AccountingService {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn create_account(
        &self,
        company_id: Uuid,
        code: String,
        name: String,
        account_type: AccountType,
        sub_type: Option<String>,
        parent_id: Option<Uuid>,
        is_control_account: bool,
        allowed_posting: bool,
        currency_code: String,
    ) -> Result<Account> {
        let id = Uuid::now_v7();
        sqlx::query(
            r#"INSERT INTO chart_of_accounts (id, company_id, code, name, account_type, sub_type, parent_id, is_control_account, is_active, allowed_posting, currency_code, balance, created_at, updated_at)
               VALUES ($1, $2, $3, $4, $5, $6, $7, $8, true, $9, $10, 0, NOW(), NOW())"#,
        )
        .bind(id)
        .bind(company_id)
        .bind(&code)
        .bind(&name)
        .bind(&account_type)
        .bind(&sub_type)
        .bind(parent_id)
        .bind(is_control_account)
        .bind(allowed_posting)
        .bind(&currency_code)
        .execute(&self.pool)
        .await?;

        self.get_account(id).await
    }

    pub async fn get_account(&self, id: Uuid) -> Result<Account> {
        sqlx::query_as::<_, Account>("SELECT * FROM chart_of_accounts WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await?
            .ok_or_else(|| anyhow!("Account not found"))
    }

    pub async fn list_accounts(&self, company_id: Uuid) -> Result<Vec<Account>> {
        let accounts = sqlx::query_as::<_, Account>(
            "SELECT * FROM chart_of_accounts WHERE company_id = $1 AND is_active = true ORDER BY code",
        )
        .bind(company_id)
        .fetch_all(&self.pool)
        .await?;
        Ok(accounts)
    }

    pub async fn create_fiscal_year(
        &self,
        company_id: Uuid,
        name: String,
        start_date: NaiveDate,
        end_date: NaiveDate,
    ) -> Result<FiscalYear> {
        let id = Uuid::now_v7();
        sqlx::query(
            r#"INSERT INTO fiscal_years (id, company_id, name, start_date, end_date, is_closed, created_at, updated_at)
               VALUES ($1, $2, $3, $4, $5, false, NOW(), NOW())"#,
        )
        .bind(id)
        .bind(company_id)
        .bind(&name)
        .bind(start_date)
        .bind(end_date)
        .execute(&self.pool)
        .await?;

        sqlx::query_as::<_, FiscalYear>("SELECT * FROM fiscal_years WHERE id = $1")
            .bind(id)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| anyhow!("Failed to fetch fiscal year: {}", e))
    }

    pub async fn create_periods(&self, fiscal_year_id: Uuid) -> Result<Vec<AccountingPeriod>> {
        let fy = sqlx::query_as::<_, FiscalYear>("SELECT * FROM fiscal_years WHERE id = $1")
            .bind(fiscal_year_id)
            .fetch_one(&self.pool)
            .await?;

        let mut periods = Vec::new();
        let mut current = fy.start_date;
        let mut period_number = 1;

        while current <= fy.end_date {
            let month_end = {
                let next_month = if current.month() == 12 {
                    NaiveDate::from_ymd_opt(current.year() + 1, 1, 1).unwrap()
                } else {
                    NaiveDate::from_ymd_opt(current.year(), current.month() + 1, 1).unwrap()
                };
                next_month - chrono::Duration::days(1)
            };
            let end = if month_end > fy.end_date {
                fy.end_date
            } else {
                month_end
            };

            let id = Uuid::now_v7();
            let name = format!("Period {} - {}", current.format("%b %Y"), end.format("%b %Y"));

            sqlx::query(
                r#"INSERT INTO accounting_periods (id, fiscal_year_id, company_id, name, period_number, start_date, end_date, status, created_at, updated_at)
                   VALUES ($1, $2, $3, $4, $5, $6, $7, 'open', NOW(), NOW())"#,
            )
            .bind(id)
            .bind(fiscal_year_id)
            .bind(fy.company_id)
            .bind(&name)
            .bind(period_number)
            .bind(current)
            .bind(end)
            .execute(&self.pool)
            .await?;

            periods.push(
                sqlx::query_as::<_, AccountingPeriod>("SELECT * FROM accounting_periods WHERE id = $1")
                    .bind(id)
                    .fetch_one(&self.pool)
                    .await?,
            );

            period_number += 1;
            let next = end + chrono::Duration::days(1);
            if next > fy.end_date {
                break;
            }
            current = next;
        }

        Ok(periods)
    }

    pub async fn close_period(&self, period_id: Uuid) -> Result<AccountingPeriod> {
        let period = sqlx::query_as::<_, AccountingPeriod>(
            "SELECT * FROM accounting_periods WHERE id = $1",
        )
        .bind(period_id)
        .fetch_one(&self.pool)
        .await?;

        if period.status == PeriodStatus::Locked {
            return Err(anyhow!("Period is already locked"));
        }

        sqlx::query(
            "UPDATE accounting_periods SET status = 'closed', updated_at = NOW() WHERE id = $1",
        )
        .bind(period_id)
        .execute(&self.pool)
        .await?;

        sqlx::query_as::<_, AccountingPeriod>("SELECT * FROM accounting_periods WHERE id = $1")
            .bind(period_id)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| anyhow!("Failed to fetch period: {}", e))
    }

    pub async fn lock_period(&self, period_id: Uuid) -> Result<AccountingPeriod> {
        sqlx::query(
            "UPDATE accounting_periods SET status = 'locked', updated_at = NOW() WHERE id = $1",
        )
        .bind(period_id)
        .execute(&self.pool)
        .await?;

        sqlx::query_as::<_, AccountingPeriod>("SELECT * FROM accounting_periods WHERE id = $1")
            .bind(period_id)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| anyhow!("Failed to fetch period: {}", e))
    }
}
