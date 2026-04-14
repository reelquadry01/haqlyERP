// Author: Quadri Atharu
use anyhow::{anyhow, Result};
use serde_json::json;
use sqlx::PgPool;
use uuid::Uuid;

use crate::models::account::{AccountingPeriod, FiscalYear, PeriodStatus};
use crate::models::company::CompanySettings;
use crate::models::user::{Permission, Role, RolePermission, UserRole};

#[derive(Clone)]
pub struct AdminService {
    pub pool: PgPool,
}

impl AdminService {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn create_permission(
        &self,
        name: String,
        module: String,
        action: String,
        description: Option<String>,
    ) -> Result<Permission> {
        let id = Uuid::now_v7();
        sqlx::query(
            r#"INSERT INTO permissions (id, name, module, action, description, created_at)
               VALUES ($1, $2, $3, $4, $5, NOW())"#,
        )
        .bind(id)
        .bind(&name)
        .bind(&module)
        .bind(&action)
        .bind(&description)
        .execute(&self.pool)
        .await?;

        sqlx::query_as::<_, Permission>("SELECT * FROM permissions WHERE id = $1")
            .bind(id)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| anyhow!("Failed to fetch permission: {}", e))
    }

    pub async fn create_role(
        &self,
        company_id: Uuid,
        name: String,
        description: Option<String>,
        is_system: bool,
    ) -> Result<Role> {
        let id = Uuid::now_v7();
        sqlx::query(
            r#"INSERT INTO roles (id, company_id, name, description, is_system, created_at, updated_at)
               VALUES ($1, $2, $3, $4, $5, NOW(), NOW())"#,
        )
        .bind(id)
        .bind(company_id)
        .bind(&name)
        .bind(&description)
        .bind(is_system)
        .execute(&self.pool)
        .await?;

        sqlx::query_as::<_, Role>("SELECT * FROM roles WHERE id = $1")
            .bind(id)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| anyhow!("Failed to fetch role: {}", e))
    }

    pub async fn assign_roles(&self, user_id: Uuid, role_ids: Vec<Uuid>) -> Result<Vec<UserRole>> {
        for role_id in &role_ids {
            let id = Uuid::now_v7();
            sqlx::query(
                r#"INSERT INTO user_roles (id, user_id, role_id, assigned_at)
                   VALUES ($1, $2, $3, NOW())
                   ON CONFLICT DO NOTHING"#,
            )
            .bind(id)
            .bind(user_id)
            .bind(role_id)
            .execute(&self.pool)
            .await?;
        }

        let user_roles = sqlx::query_as::<_, UserRole>(
            "SELECT * FROM user_roles WHERE user_id = $1",
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(user_roles)
    }

    pub async fn list_fiscal_years(&self, company_id: Uuid) -> Result<Vec<FiscalYear>> {
        sqlx::query_as::<_, FiscalYear>(
            "SELECT * FROM fiscal_years WHERE company_id = $1 ORDER BY start_date DESC",
        )
        .bind(company_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| anyhow!("Failed to list fiscal years: {}", e))
    }

    pub async fn create_fiscal_year(
        &self,
        company_id: Uuid,
        name: String,
        start_date: chrono::NaiveDate,
        end_date: chrono::NaiveDate,
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

    pub async fn close_fiscal_year(&self, fiscal_year_id: Uuid) -> Result<FiscalYear> {
        let fy = sqlx::query_as::<_, FiscalYear>("SELECT * FROM fiscal_years WHERE id = $1")
            .bind(fiscal_year_id)
            .fetch_one(&self.pool)
            .await?;

        if fy.is_closed {
            return Err(anyhow!("Fiscal year is already closed"));
        }

        sqlx::query(
            "UPDATE accounting_periods SET status = 'closed', updated_at = NOW() WHERE fiscal_year_id = $1",
        )
        .bind(fiscal_year_id)
        .execute(&self.pool)
        .await?;

        sqlx::query(
            "UPDATE fiscal_years SET is_closed = true, updated_at = NOW() WHERE id = $1",
        )
        .bind(fiscal_year_id)
        .execute(&self.pool)
        .await?;

        sqlx::query_as::<_, FiscalYear>("SELECT * FROM fiscal_years WHERE id = $1")
            .bind(fiscal_year_id)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| anyhow!("Failed to fetch fiscal year: {}", e))
    }

    pub async fn lock_fiscal_year(&self, fiscal_year_id: Uuid) -> Result<FiscalYear> {
        sqlx::query(
            "UPDATE accounting_periods SET status = 'locked', updated_at = NOW() WHERE fiscal_year_id = $1",
        )
        .bind(fiscal_year_id)
        .execute(&self.pool)
        .await?;

        sqlx::query(
            "UPDATE fiscal_years SET is_closed = true, updated_at = NOW() WHERE id = $1",
        )
        .bind(fiscal_year_id)
        .execute(&self.pool)
        .await?;

        sqlx::query_as::<_, FiscalYear>("SELECT * FROM fiscal_years WHERE id = $1")
            .bind(fiscal_year_id)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| anyhow!("Failed to fetch fiscal year: {}", e))
    }

    pub async fn update_period_status(
        &self,
        period_id: Uuid,
        status: PeriodStatus,
    ) -> Result<AccountingPeriod> {
        sqlx::query(
            "UPDATE accounting_periods SET status = $1, updated_at = NOW() WHERE id = $2",
        )
        .bind(&status)
        .bind(period_id)
        .execute(&self.pool)
        .await?;

        sqlx::query_as::<_, AccountingPeriod>("SELECT * FROM accounting_periods WHERE id = $1")
            .bind(period_id)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| anyhow!("Failed to fetch period: {}", e))
    }

    pub async fn get_settings(&self, company_id: Uuid) -> Result<CompanySettings> {
        sqlx::query_as::<_, CompanySettings>(
            "SELECT * FROM company_settings WHERE company_id = $1",
        )
        .bind(company_id)
        .fetch_optional(&self.pool)
        .await?
        .ok_or_else(|| anyhow!("Company settings not found"))
    }

    pub async fn update_settings(
        &self,
        company_id: Uuid,
        updates: serde_json::Value,
    ) -> Result<CompanySettings> {
        let existing = self.get_settings(company_id).await.ok();

        match existing {
            Some(_) => {
                if let Some(obj) = updates.as_object() {
                    for (key, value) in obj {
                        match key.as_str() {
                            "default_payment_terms" => {
                                if let Some(v) = value.as_i64() {
                                    sqlx::query("UPDATE company_settings SET default_payment_terms = $1, updated_at = NOW() WHERE company_id = $2")
                                        .bind(v as i32).bind(company_id).execute(&self.pool).await?;
                                }
                            }
                            "tax_inclusive_pricing" => {
                                if let Some(v) = value.as_bool() {
                                    sqlx::query("UPDATE company_settings SET tax_inclusive_pricing = $1, updated_at = NOW() WHERE company_id = $2")
                                        .bind(v).bind(company_id).execute(&self.pool).await?;
                                }
                            }
                            "multi_currency_enabled" => {
                                if let Some(v) = value.as_bool() {
                                    sqlx::query("UPDATE company_settings SET multi_currency_enabled = $1, updated_at = NOW() WHERE company_id = $2")
                                        .bind(v).bind(company_id).execute(&self.pool).await?;
                                }
                            }
                            "einvoicing_enabled" => {
                                if let Some(v) = value.as_bool() {
                                    sqlx::query("UPDATE company_settings SET einvoicing_enabled = $1, updated_at = NOW() WHERE company_id = $2")
                                        .bind(v).bind(company_id).execute(&self.pool).await?;
                                }
                            }
                            "approval_workflow_enabled" => {
                                if let Some(v) = value.as_bool() {
                                    sqlx::query("UPDATE company_settings SET approval_workflow_enabled = $1, updated_at = NOW() WHERE company_id = $2")
                                        .bind(v).bind(company_id).execute(&self.pool).await?;
                                }
                            }
                            "min_approvers" => {
                                if let Some(v) = value.as_i64() {
                                    sqlx::query("UPDATE company_settings SET min_approvers = $1, updated_at = NOW() WHERE company_id = $2")
                                        .bind(v as i32).bind(company_id).execute(&self.pool).await?;
                                }
                            }
                            "settings_json" => {
                                sqlx::query("UPDATE company_settings SET settings_json = $1, updated_at = NOW() WHERE company_id = $2")
                                    .bind(value).bind(company_id).execute(&self.pool).await?;
                            }
                            _ => {}
                        }
                    }
                }
            }
            None => {
                let id = Uuid::now_v7();
                sqlx::query(
                    r#"INSERT INTO company_settings (id, company_id, tax_inclusive_pricing, multi_currency_enabled, einvoicing_enabled, document_retention_years, approval_workflow_enabled, min_approvers, fiscal_year_start_month, base_currency, created_at, updated_at)
                       VALUES ($1, $2, false, false, false, 7, false, 1, 1, 'NGN', NOW(), NOW())"#,
                )
                .bind(id)
                .bind(company_id)
                .execute(&self.pool)
                .await?;
            }
        }

        self.get_settings(company_id).await
    }

    pub async fn list_approval_rules(&self, company_id: Uuid) -> Result<Vec<ApprovalRule>> {
        let rows = sqlx::query_as::<_, ApprovalRuleRow>(
            "SELECT * FROM approval_rules WHERE company_id = $1 ORDER BY created_at",
        )
        .bind(company_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| ApprovalRule {
            id: r.id,
            company_id: r.company_id,
            module: r.module,
            transaction_type: r.transaction_type,
            min_amount: r.min_amount,
            max_amount: r.max_amount,
            required_approvers: r.required_approvers,
            is_active: r.is_active,
            created_at: r.created_at,
        }).collect())
    }

    pub async fn create_approval_rule(
        &self,
        company_id: Uuid,
        module: String,
        transaction_type: String,
        min_amount: Option<bigdecimal::BigDecimal>,
        max_amount: Option<bigdecimal::BigDecimal>,
        required_approvers: i32,
    ) -> Result<ApprovalRule> {
        let id = Uuid::now_v7();
        sqlx::query(
            r#"INSERT INTO approval_rules (id, company_id, module, transaction_type, min_amount, max_amount, required_approvers, is_active, created_at)
               VALUES ($1, $2, $3, $4, $5, $6, $7, true, NOW())"#,
        )
        .bind(id)
        .bind(company_id)
        .bind(&module)
        .bind(&transaction_type)
        .bind(&min_amount)
        .bind(&max_amount)
        .bind(required_approvers)
        .execute(&self.pool)
        .await?;

        Ok(ApprovalRule {
            id,
            company_id,
            module,
            transaction_type,
            min_amount,
            max_amount,
            required_approvers,
            is_active: true,
            created_at: chrono::Utc::now().naive_utc(),
        })
    }
}

pub struct ApprovalRule {
    pub id: Uuid,
    pub company_id: Uuid,
    pub module: String,
    pub transaction_type: String,
    pub min_amount: Option<bigdecimal::BigDecimal>,
    pub max_amount: Option<bigdecimal::BigDecimal>,
    pub required_approvers: i32,
    pub is_active: bool,
    pub created_at: chrono::NaiveDateTime,
}

#[derive(Debug, Clone, sqlx::FromRow)]
struct ApprovalRuleRow {
    pub id: Uuid,
    pub company_id: Uuid,
    pub module: String,
    pub transaction_type: String,
    pub min_amount: Option<bigdecimal::BigDecimal>,
    pub max_amount: Option<bigdecimal::BigDecimal>,
    pub required_approvers: i32,
    pub is_active: bool,
    pub created_at: chrono::NaiveDateTime,
}
