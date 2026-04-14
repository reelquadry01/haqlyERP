// Author: Quadri Atharu
use anyhow::{anyhow, Result};
use serde_json::json;
use sqlx::PgPool;
use uuid::Uuid;

const AI_SERVICE_URL: &str = "http://localhost:8200";

#[derive(Clone)]
pub struct AiBridgeService {
    pub pool: PgPool,
}

impl AiBridgeService {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn check_health(&self) -> Result<serde_json::Value> {
        let client = reqwest::Client::new();
        let response = client
            .get(format!("{}/health", AI_SERVICE_URL))
            .timeout(std::time::Duration::from_secs(10))
            .send()
            .await
            .map_err(|e| anyhow!("AI service unreachable: {}", e))?;

        let status = response.status().as_u16();
        let body = response.text().await.unwrap_or_default();

        Ok(json!({
            "service": "ai_bridge",
            "upstream_status": status,
            "upstream_response": body,
        }))
    }

    pub async fn analyze_financials(
        &self,
        company_id: Uuid,
        period_id: Option<Uuid>,
    ) -> Result<serde_json::Value> {
        let client = reqwest::Client::new();

        let revenue: bigdecimal::BigDecimal = sqlx::query_scalar(
            r#"SELECT COALESCE(SUM(jl.credit - jl.debit), 0)
               FROM journal_lines jl
               JOIN journal_headers jh ON jl.journal_header_id = jh.id
               JOIN accounts a ON jl.account_id = a.id
               WHERE jh.company_id = $1 AND a.account_type = 'revenue' AND jh.status = 'posted'"#,
        )
        .bind(company_id)
        .fetch_one(&self.pool)
        .await
        .unwrap_or(bigdecimal::BigDecimal::from(0));

        let expenses: bigdecimal::BigDecimal = sqlx::query_scalar(
            r#"SELECT COALESCE(SUM(jl.debit - jl.credit), 0)
               FROM journal_lines jl
               JOIN journal_headers jh ON jl.journal_header_id = jh.id
               JOIN accounts a ON jl.account_id = a.id
               WHERE jh.company_id = $1 AND a.account_type = 'expense' AND jh.status = 'posted'"#,
        )
        .bind(company_id)
        .fetch_one(&self.pool)
        .await
        .unwrap_or(bigdecimal::BigDecimal::from(0));

        let payload = json!({
            "company_id": company_id.to_string(),
            "period_id": period_id.map(|p| p.to_string()),
            "revenue": revenue.to_string(),
            "expenses": expenses.to_string(),
        });

        let response = client
            .post(format!("{}/analyze-financials", AI_SERVICE_URL))
            .json(&payload)
            .timeout(std::time::Duration::from_secs(60))
            .send()
            .await
            .map_err(|e| anyhow!("AI service request failed: {}", e))?;

        let result: serde_json::Value = response
            .json()
            .await
            .map_err(|e| anyhow!("Failed to parse AI response: {}", e))?;

        Ok(result)
    }

    pub async fn compute_tax(
        &self,
        company_id: Uuid,
        tax_type: &str,
        amount: bigdecimal::BigDecimal,
    ) -> Result<serde_json::Value> {
        let client = reqwest::Client::new();

        let payload = json!({
            "company_id": company_id.to_string(),
            "tax_type": tax_type,
            "amount": amount.to_string(),
        });

        let response = client
            .post(format!("{}/compute-tax", AI_SERVICE_URL))
            .json(&payload)
            .timeout(std::time::Duration::from_secs(30))
            .send()
            .await
            .map_err(|e| anyhow!("AI service request failed: {}", e))?;

        let result: serde_json::Value = response
            .json()
            .await
            .map_err(|e| anyhow!("Failed to parse AI response: {}", e))?;

        Ok(result)
    }

    pub async fn generate_report(
        &self,
        company_id: Uuid,
        report_type: &str,
        parameters: serde_json::Value,
    ) -> Result<serde_json::Value> {
        let client = reqwest::Client::new();

        let payload = json!({
            "company_id": company_id.to_string(),
            "report_type": report_type,
            "parameters": parameters,
        });

        let response = client
            .post(format!("{}/generate-report", AI_SERVICE_URL))
            .json(&payload)
            .timeout(std::time::Duration::from_secs(120))
            .send()
            .await
            .map_err(|e| anyhow!("AI service request failed: {}", e))?;

        let result: serde_json::Value = response
            .json()
            .await
            .map_err(|e| anyhow!("Failed to parse AI response: {}", e))?;

        Ok(result)
    }

    pub async fn proxy(&self, path: &str, body: serde_json::Value) -> Result<serde_json::Value> {
        let client = reqwest::Client::new();

        let response = client
            .post(format!("{}/{}", AI_SERVICE_URL, path))
            .json(&body)
            .timeout(std::time::Duration::from_secs(60))
            .send()
            .await
            .map_err(|e| anyhow!("AI proxy request failed for path {}: {}", path, e))?;

        let result: serde_json::Value = response
            .json()
            .await
            .map_err(|e| anyhow!("Failed to parse AI response: {}", e))?;

        Ok(result)
    }
}
