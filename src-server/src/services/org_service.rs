// Author: Quadri Atharu
use anyhow::{anyhow, Result};
use sqlx::PgPool;
use uuid::Uuid;

use crate::models::company::{
    BankAccount, Branch, Company, CompanySettings, CostCenter, Department, Project,
};

#[derive(Clone)]
pub struct OrgService {
    pub pool: PgPool,
}

impl OrgService {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn create_company(
        &self,
        name: String,
        registration_number: Option<String>,
        tax_identification_number: Option<String>,
        base_currency: String,
    ) -> Result<Company> {
        let id = Uuid::now_v7();
        sqlx::query(
            r#"INSERT INTO companies (id, name, registration_number, tax_identification_number, base_currency, fiscal_year_start_month, is_active, created_at, updated_at)
               VALUES ($1, $2, $3, $4, $5, 1, true, NOW(), NOW())"#,
        )
        .bind(id)
        .bind(&name)
        .bind(&registration_number)
        .bind(&tax_identification_number)
        .bind(&base_currency)
        .execute(&self.pool)
        .await?;

        let settings_id = Uuid::now_v7();
        sqlx::query(
            r#"INSERT INTO company_settings (id, company_id, tax_inclusive_pricing, multi_currency_enabled, einvoicing_enabled, document_retention_years, approval_workflow_enabled, min_approvers, fiscal_year_start_month, base_currency, created_at, updated_at)
               VALUES ($1, $2, false, false, false, 6, false, 1, 1, $3, NOW(), NOW())"#,
        )
        .bind(settings_id)
        .bind(id)
        .bind(&base_currency)
        .execute(&self.pool)
        .await?;

        self.get_company(id).await
    }

    pub async fn get_company(&self, id: Uuid) -> Result<Company> {
        sqlx::query_as::<_, Company>("SELECT * FROM companies WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await?
            .ok_or_else(|| anyhow!("Company not found"))
    }

    pub async fn list_companies(&self) -> Result<Vec<Company>> {
        let companies = sqlx::query_as::<_, Company>(
            "SELECT * FROM companies WHERE is_active = true ORDER BY name",
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(companies)
    }

    pub async fn update_company(
        &self,
        id: Uuid,
        name: Option<String>,
        email: Option<String>,
        phone: Option<String>,
        website: Option<String>,
        address_line1: Option<String>,
        city: Option<String>,
        state: Option<String>,
        country: Option<String>,
    ) -> Result<Company> {
        sqlx::query(
            r#"UPDATE companies SET name = COALESCE($2, name), email = COALESCE($3, email),
               phone = COALESCE($4, phone), website = COALESCE($5, website),
               address_line1 = COALESCE($6, address_line1), city = COALESCE($7, city),
               state = COALESCE($8, state), country = COALESCE($9, country), updated_at = NOW()
               WHERE id = $1"#,
        )
        .bind(id)
        .bind(&name)
        .bind(&email)
        .bind(&phone)
        .bind(&website)
        .bind(&address_line1)
        .bind(&city)
        .bind(&state)
        .bind(&country)
        .execute(&self.pool)
        .await?;

        self.get_company(id).await
    }

    pub async fn create_branch(
        &self,
        company_id: Uuid,
        code: String,
        name: String,
    ) -> Result<Branch> {
        let id = Uuid::now_v7();
        sqlx::query(
            r#"INSERT INTO branches (id, company_id, code, name, is_active, created_at, updated_at)
               VALUES ($1, $2, $3, $4, true, NOW(), NOW())"#,
        )
        .bind(id)
        .bind(company_id)
        .bind(&code)
        .bind(&name)
        .execute(&self.pool)
        .await?;

        sqlx::query_as::<_, Branch>("SELECT * FROM branches WHERE id = $1")
            .bind(id)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| anyhow!("Failed to fetch branch: {}", e))
    }

    pub async fn create_department(
        &self,
        company_id: Uuid,
        branch_id: Option<Uuid>,
        code: String,
        name: String,
    ) -> Result<Department> {
        let id = Uuid::now_v7();
        sqlx::query(
            r#"INSERT INTO departments (id, company_id, branch_id, code, name, is_active, created_at, updated_at)
               VALUES ($1, $2, $3, $4, $5, true, NOW(), NOW())"#,
        )
        .bind(id)
        .bind(company_id)
        .bind(branch_id)
        .bind(&code)
        .bind(&name)
        .execute(&self.pool)
        .await?;

        sqlx::query_as::<_, Department>("SELECT * FROM departments WHERE id = $1")
            .bind(id)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| anyhow!("Failed to fetch department: {}", e))
    }

    pub async fn create_cost_center(
        &self,
        company_id: Uuid,
        branch_id: Option<Uuid>,
        department_id: Option<Uuid>,
        code: String,
        name: String,
    ) -> Result<CostCenter> {
        let id = Uuid::now_v7();
        sqlx::query(
            r#"INSERT INTO cost_centers (id, company_id, branch_id, department_id, code, name, is_active, created_at, updated_at)
               VALUES ($1, $2, $3, $4, $5, $6, true, NOW(), NOW())"#,
        )
        .bind(id)
        .bind(company_id)
        .bind(branch_id)
        .bind(department_id)
        .bind(&code)
        .bind(&name)
        .execute(&self.pool)
        .await?;

        sqlx::query_as::<_, CostCenter>("SELECT * FROM cost_centers WHERE id = $1")
            .bind(id)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| anyhow!("Failed to fetch cost center: {}", e))
    }

    pub async fn create_project(
        &self,
        company_id: Uuid,
        code: String,
        name: String,
        description: Option<String>,
        start_date: Option<chrono::NaiveDate>,
        end_date: Option<chrono::NaiveDate>,
    ) -> Result<Project> {
        let id = Uuid::now_v7();
        sqlx::query(
            r#"INSERT INTO projects (id, company_id, code, name, description, start_date, end_date, status, is_active, created_at, updated_at)
               VALUES ($1, $2, $3, $4, $5, $6, $7, 'active', true, NOW(), NOW())"#,
        )
        .bind(id)
        .bind(company_id)
        .bind(&code)
        .bind(&name)
        .bind(&description)
        .bind(start_date)
        .bind(end_date)
        .execute(&self.pool)
        .await?;

        sqlx::query_as::<_, Project>("SELECT * FROM projects WHERE id = $1")
            .bind(id)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| anyhow!("Failed to fetch project: {}", e))
    }

    pub async fn update_settings(
        &self,
        company_id: Uuid,
        settings: serde_json::Value,
    ) -> Result<CompanySettings> {
        sqlx::query(
            r#"UPDATE company_settings SET settings_json = $2, updated_at = NOW() WHERE company_id = $1"#,
        )
        .bind(company_id)
        .bind(&settings)
        .execute(&self.pool)
        .await?;

        sqlx::query_as::<_, CompanySettings>(
            "SELECT * FROM company_settings WHERE company_id = $1",
        )
        .bind(company_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| anyhow!("Failed to fetch settings: {}", e))
    }
}
