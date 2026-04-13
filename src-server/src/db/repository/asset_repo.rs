// Author: Quadri Atharu
use bigdecimal::BigDecimal;
use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct AssetCategory {
    pub id: Uuid,
    pub company_id: Uuid,
    pub name: String,
    pub depreciation_method: String,
    pub useful_life_years: i32,
    pub residual_rate: BigDecimal,
    pub debit_account_id: Uuid,
    pub credit_account_id: Uuid,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct FixedAsset {
    pub id: Uuid,
    pub company_id: Uuid,
    pub branch_id: Option<Uuid>,
    pub category_id: Uuid,
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    pub acquisition_date: NaiveDate,
    pub acquisition_cost: BigDecimal,
    pub accumulated_depreciation: BigDecimal,
    pub net_book_value: BigDecimal,
    pub residual_value: BigDecimal,
    pub useful_life_years: i32,
    pub depreciation_method: String,
    pub status: String,
    pub location: Option<String>,
    pub created_by: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct DepreciationSchedule {
    pub id: Uuid,
    pub asset_id: Uuid,
    pub fiscal_year_id: Uuid,
    pub period_id: Uuid,
    pub depreciation_amount: BigDecimal,
    pub accumulated_depreciation: BigDecimal,
    pub net_book_value: BigDecimal,
    pub is_posted: bool,
    pub posted_journal_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewAssetCategory {
    pub company_id: Uuid,
    pub name: String,
    pub depreciation_method: String,
    pub useful_life_years: i32,
    pub residual_rate: BigDecimal,
    pub debit_account_id: Uuid,
    pub credit_account_id: Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewFixedAsset {
    pub company_id: Uuid,
    pub branch_id: Option<Uuid>,
    pub category_id: Uuid,
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    pub acquisition_date: NaiveDate,
    pub acquisition_cost: BigDecimal,
    pub net_book_value: BigDecimal,
    pub residual_value: BigDecimal,
    pub useful_life_years: i32,
    pub depreciation_method: String,
    pub location: Option<String>,
    pub created_by: Uuid,
}

pub struct AssetRepo {
    pool: PgPool,
}

impl AssetRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn create_category(
        &self,
        new_cat: NewAssetCategory,
    ) -> Result<AssetCategory, sqlx::Error> {
        sqlx::query_as::<_, AssetCategory>(
            r#"INSERT INTO asset_categories (company_id, name, depreciation_method, useful_life_years, residual_rate, debit_account_id, credit_account_id)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            RETURNING id, company_id, name, depreciation_method, useful_life_years, residual_rate, debit_account_id, credit_account_id, is_active, created_at, updated_at"#,
        )
        .bind(new_cat.company_id)
        .bind(&new_cat.name)
        .bind(&new_cat.depreciation_method)
        .bind(new_cat.useful_life_years)
        .bind(&new_cat.residual_rate)
        .bind(new_cat.debit_account_id)
        .bind(new_cat.credit_account_id)
        .fetch_one(&self.pool)
        .await
    }

    pub async fn find_categories(
        &self,
        company_id: Uuid,
    ) -> Result<Vec<AssetCategory>, sqlx::Error> {
        sqlx::query_as::<_, AssetCategory>(
            "SELECT id, company_id, name, depreciation_method, useful_life_years, residual_rate, debit_account_id, credit_account_id, is_active, created_at, updated_at FROM asset_categories WHERE company_id = $1 ORDER BY name",
        )
        .bind(company_id)
        .fetch_all(&self.pool)
        .await
    }

    pub async fn create_asset(&self, new_asset: NewFixedAsset) -> Result<FixedAsset, sqlx::Error> {
        sqlx::query_as::<_, FixedAsset>(
            r#"INSERT INTO fixed_assets (company_id, branch_id, category_id, code, name, description, acquisition_date, acquisition_cost, net_book_value, residual_value, useful_life_years, depreciation_method, location, created_by)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14)
            RETURNING id, company_id, branch_id, category_id, code, name, description, acquisition_date, acquisition_cost, accumulated_depreciation, net_book_value, residual_value, useful_life_years, depreciation_method, status, location, created_by, created_at, updated_at"#,
        )
        .bind(new_asset.company_id)
        .bind(new_asset.branch_id)
        .bind(new_asset.category_id)
        .bind(&new_asset.code)
        .bind(&new_asset.name)
        .bind(&new_asset.description)
        .bind(new_asset.acquisition_date)
        .bind(&new_asset.acquisition_cost)
        .bind(&new_asset.net_book_value)
        .bind(&new_asset.residual_value)
        .bind(new_asset.useful_life_years)
        .bind(&new_asset.depreciation_method)
        .bind(&new_asset.location)
        .bind(new_asset.created_by)
        .fetch_one(&self.pool)
        .await
    }

    pub async fn find_assets(&self, company_id: Uuid) -> Result<Vec<FixedAsset>, sqlx::Error> {
        sqlx::query_as::<_, FixedAsset>(
            "SELECT id, company_id, branch_id, category_id, code, name, description, acquisition_date, acquisition_cost, accumulated_depreciation, net_book_value, residual_value, useful_life_years, depreciation_method, status, location, created_by, created_at, updated_at FROM fixed_assets WHERE company_id = $1 ORDER BY code",
        )
        .bind(company_id)
        .fetch_all(&self.pool)
        .await
    }

    pub async fn create_depreciation_schedule(
        &self,
        asset_id: Uuid,
        fiscal_year_id: Uuid,
        period_id: Uuid,
        depreciation_amount: BigDecimal,
        accumulated_depreciation: BigDecimal,
        net_book_value: BigDecimal,
    ) -> Result<DepreciationSchedule, sqlx::Error> {
        sqlx::query_as::<_, DepreciationSchedule>(
            r#"INSERT INTO depreciation_schedules (asset_id, fiscal_year_id, period_id, depreciation_amount, accumulated_depreciation, net_book_value)
            VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING id, asset_id, fiscal_year_id, period_id, depreciation_amount, accumulated_depreciation, net_book_value, is_posted, posted_journal_id, created_at"#,
        )
        .bind(asset_id)
        .bind(fiscal_year_id)
        .bind(period_id)
        .bind(&depreciation_amount)
        .bind(&accumulated_depreciation)
        .bind(&net_book_value)
        .fetch_one(&self.pool)
        .await
    }
}
