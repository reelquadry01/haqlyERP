// Author: Quadri Atharu
use anyhow::{anyhow, Result};
use bigdecimal::BigDecimal;
use sqlx::PgPool;
use uuid::Uuid;

use crate::models::bi::{BiDashboard, BiDataset, DatasetSourceType as BiDatasetSourceType, BiWidget, WidgetType};
use crate::dtos::bi_dto::{FinancialSummary, KpiData};

#[derive(Clone)]
pub struct BiService {
    pub pool: PgPool,
}

impl BiService {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn create_dashboard(
        &self,
        company_id: Uuid,
        name: String,
        description: Option<String>,
        layout_config: serde_json::Value,
        is_default: bool,
        created_by: Uuid,
    ) -> Result<BiDashboard> {
        let id = Uuid::now_v7();
        sqlx::query(
            r#"INSERT INTO bi_dashboards (id, company_id, name, description, layout_config, is_default, created_by, created_at, updated_at)
               VALUES ($1, $2, $3, $4, $5, $6, $7, NOW(), NOW())"#,
        )
        .bind(id)
        .bind(company_id)
        .bind(&name)
        .bind(&description)
        .bind(&layout_config)
        .bind(is_default)
        .bind(created_by)
        .execute(&self.pool)
        .await?;

        sqlx::query_as::<_, BiDashboard>("SELECT * FROM bi_dashboards WHERE id = $1")
            .bind(id)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| anyhow!("Failed to fetch dashboard: {}", e))
    }

    pub async fn list_dashboards(&self, company_id: Uuid) -> Result<Vec<BiDashboard>> {
        sqlx::query_as::<_, BiDashboard>(
            "SELECT * FROM bi_dashboards WHERE company_id = $1 ORDER BY created_at DESC",
        )
        .bind(company_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| anyhow!("Failed to list dashboards: {}", e))
    }

    pub async fn add_widget(
        &self,
        dashboard_id: Uuid,
        widget_type: WidgetType,
        title: String,
        data_source_config: serde_json::Value,
        position_config: serde_json::Value,
        refresh_interval_seconds: Option<i32>,
    ) -> Result<BiWidget> {
        let id = Uuid::now_v7();
        sqlx::query(
            r#"INSERT INTO bi_widgets (id, dashboard_id, widget_type, title, data_source_config, position_config, refresh_interval_seconds, created_at, updated_at)
               VALUES ($1, $2, $3, $4, $5, $6, $7, NOW(), NOW())"#,
        )
        .bind(id)
        .bind(dashboard_id)
        .bind(&widget_type)
        .bind(&title)
        .bind(&data_source_config)
        .bind(&position_config)
        .bind(refresh_interval_seconds)
        .execute(&self.pool)
        .await?;

        sqlx::query_as::<_, BiWidget>("SELECT * FROM bi_widgets WHERE id = $1")
            .bind(id)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| anyhow!("Failed to fetch widget: {}", e))
    }

    pub async fn execute_query(
        &self,
        dataset_id: Uuid,
        query_text: String,
        parameters: serde_json::Value,
        cache_ttl_seconds: i32,
    ) -> Result<serde_json::Value> {
        let id = Uuid::now_v7();
        sqlx::query(
            r#"INSERT INTO bi_queries (id, dataset_id, name, query_text, parameters, cache_ttl_seconds, created_at, updated_at)
               VALUES ($1, $2, $3, $4, $5, $6, NOW(), NOW())"#,
        )
        .bind(id)
        .bind(dataset_id)
        .bind("adhoc")
        .bind(&query_text)
        .bind(&parameters)
        .bind(cache_ttl_seconds)
        .execute(&self.pool)
        .await?;

        let dataset = self.get_dataset(dataset_id).await?;

        match dataset.source_type {
            BiDatasetSourceType::PostgreSQL => {
                let rows = sqlx::query(&query_text)
                    .fetch_all(&self.pool)
                    .await
                    .map_err(|e| anyhow!("Query execution failed: {}", e))?;

                let row_count = rows.len();

                Ok(serde_json::json!({
                    "query_id": id,
                    "row_count": row_count,
                    "message": format!("Query returned {} rows", row_count),
                }))
            }
            BiDatasetSourceType::API | BiDatasetSourceType::PythonEngine => {
                let client = reqwest::Client::new();
                let response = client
                    .post(format!("{}/query", dataset.source_config.get("url").and_then(|v| v.as_str()).unwrap_or("http://localhost:8200")))
                    .json(&serde_json::json!({"query": query_text, "parameters": parameters}))
                    .timeout(std::time::Duration::from_secs(60))
                    .send()
                    .await
                    .map_err(|e| anyhow!("External query failed: {}", e))?;

                let result: serde_json::Value = response.json().await
                    .map_err(|e| anyhow!("Failed to parse response: {}", e))?;

                Ok(result)
            }
            BiDatasetSourceType::File => Ok(serde_json::json!({
                "query_id": id,
                "message": "File source type queries are not supported via SQL",
            })),
        }
    }

    pub async fn get_dataset(&self, dataset_id: Uuid) -> Result<BiDataset> {
        sqlx::query_as::<_, BiDataset>("SELECT * FROM bi_datasets WHERE id = $1")
            .bind(dataset_id)
            .fetch_optional(&self.pool)
            .await?
            .ok_or_else(|| anyhow!("Dataset not found"))
    }

    pub async fn refresh_dataset(&self, dataset_id: Uuid) -> Result<BiDataset> {
        sqlx::query(
            "UPDATE bi_datasets SET last_refreshed = NOW(), updated_at = NOW() WHERE id = $1",
        )
        .bind(dataset_id)
        .execute(&self.pool)
        .await?;

        self.get_dataset(dataset_id).await
    }

    pub async fn generate_kpi_data(&self, company_id: Uuid, period: String) -> Result<KpiData> {
        let revenue: BigDecimal = sqlx::query_scalar(
            r#"SELECT COALESCE(SUM(jl.credit - jl.debit), 0)
               FROM journal_lines jl
               JOIN journal_headers jh ON jl.journal_header_id = jh.id
               JOIN accounts a ON jl.account_id = a.id
               WHERE jh.company_id = $1 AND a.account_type = 'revenue' AND jh.status = 'posted'"#,
        )
        .bind(company_id)
        .fetch_one(&self.pool)
        .await
        .unwrap_or(BigDecimal::from(0));

        let expenses: BigDecimal = sqlx::query_scalar(
            r#"SELECT COALESCE(SUM(jl.debit - jl.credit), 0)
               FROM journal_lines jl
               JOIN journal_headers jh ON jl.journal_header_id = jh.id
               JOIN accounts a ON jl.account_id = a.id
               WHERE jh.company_id = $1 AND a.account_type = 'expense' AND jh.status = 'posted'"#,
        )
        .bind(company_id)
        .fetch_one(&self.pool)
        .await
        .unwrap_or(BigDecimal::from(0));

        let net_income = &revenue - &expenses;

        let cash_balance: BigDecimal = sqlx::query_scalar(
            r#"SELECT COALESCE(SUM(jl.debit - jl.credit), 0)
               FROM journal_lines jl
               JOIN journal_headers jh ON jl.journal_header_id = jh.id
               JOIN accounts a ON jl.account_id = a.id
               WHERE jh.company_id = $1 AND a.account_type = 'asset' AND a.code LIKE '1%' AND jh.status = 'posted'"#,
        )
        .bind(company_id)
        .fetch_one(&self.pool)
        .await
        .unwrap_or(BigDecimal::from(0));

        let zero = BigDecimal::from(0);

        Ok(KpiData {
            company_id,
            period,
            revenue,
            expenses,
            net_income,
            cash_balance,
            ar_aging_current: zero.clone(),
            ar_aging_30: zero.clone(),
            ar_aging_60: zero.clone(),
            ar_aging_90: zero.clone(),
            ar_aging_over_90: zero.clone(),
            ap_aging_current: zero.clone(),
            ap_aging_30: zero.clone(),
            ap_aging_60: zero.clone(),
            ap_aging_90: zero.clone(),
            ap_aging_over_90: zero,
        })
    }

    pub async fn generate_financial_summary(&self, company_id: Uuid) -> Result<FinancialSummary> {
        let total_revenue: BigDecimal = sqlx::query_scalar(
            r#"SELECT COALESCE(SUM(jl.credit - jl.debit), 0)
               FROM journal_lines jl
               JOIN journal_headers jh ON jl.journal_header_id = jh.id
               JOIN accounts a ON jl.account_id = a.id
               WHERE jh.company_id = $1 AND a.account_type = 'revenue' AND jh.status = 'posted'"#,
        )
        .bind(company_id)
        .fetch_one(&self.pool)
        .await
        .unwrap_or(BigDecimal::from(0));

        let total_expenses: BigDecimal = sqlx::query_scalar(
            r#"SELECT COALESCE(SUM(jl.debit - jl.credit), 0)
               FROM journal_lines jl
               JOIN journal_headers jh ON jl.journal_header_id = jh.id
               JOIN accounts a ON jl.account_id = a.id
               WHERE jh.company_id = $1 AND a.account_type = 'expense' AND jh.status = 'posted'"#,
        )
        .bind(company_id)
        .fetch_one(&self.pool)
        .await
        .unwrap_or(BigDecimal::from(0));

        let gross_profit = &total_revenue - &total_expenses;
        let net_income = gross_profit.clone();

        let total_assets: BigDecimal = sqlx::query_scalar(
            r#"SELECT COALESCE(SUM(jl.debit - jl.credit), 0)
               FROM journal_lines jl
               JOIN journal_headers jh ON jl.journal_header_id = jh.id
               JOIN accounts a ON jl.account_id = a.id
               WHERE jh.company_id = $1 AND a.account_type = 'asset' AND jh.status = 'posted'"#,
        )
        .bind(company_id)
        .fetch_one(&self.pool)
        .await
        .unwrap_or(BigDecimal::from(0));

        let total_liabilities: BigDecimal = sqlx::query_scalar(
            r#"SELECT COALESCE(SUM(jl.credit - jl.debit), 0)
               FROM journal_lines jl
               JOIN journal_headers jh ON jl.journal_header_id = jh.id
               JOIN accounts a ON jl.account_id = a.id
               WHERE jh.company_id = $1 AND a.account_type = 'liability' AND jh.status = 'posted'"#,
        )
        .bind(company_id)
        .fetch_one(&self.pool)
        .await
        .unwrap_or(BigDecimal::from(0));

        let total_equity = &total_assets - &total_liabilities;

        let zero = BigDecimal::from(0);

        Ok(FinancialSummary {
            company_id,
            total_revenue,
            total_expenses,
            gross_profit,
            net_income,
            total_assets,
            total_liabilities,
            total_equity,
            cash_and_equivalents: zero.clone(),
            accounts_receivable: zero.clone(),
            accounts_payable: zero.clone(),
            inventory_value: zero.clone(),
            fixed_assets_net: zero,
        })
    }
}
