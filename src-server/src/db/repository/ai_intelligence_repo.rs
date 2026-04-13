// Author: Quadri Atharu
use bigdecimal::BigDecimal;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use sqlx::FromRow;
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct AiAnalysisResult {
    pub id: Uuid,
    pub company_id: Uuid,
    pub analysis_type: String,
    pub source_module: String,
    pub source_document_id: Option<Uuid>,
    pub input_data: Option<JsonValue>,
    pub result_data: JsonValue,
    pub confidence_score: Option<BigDecimal>,
    pub model_used: Option<String>,
    pub processing_time_ms: Option<i32>,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct AiTaxComputation {
    pub id: Uuid,
    pub company_id: Uuid,
    pub tax_type: String,
    pub fiscal_year_id: Option<Uuid>,
    pub period_id: Option<Uuid>,
    pub computation_data: JsonValue,
    pub result_data: JsonValue,
    pub is_accepted: bool,
    pub accepted_by: Option<Uuid>,
    pub accepted_at: Option<DateTime<Utc>>,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct AiAgentLog {
    pub id: Uuid,
    pub company_id: Uuid,
    pub agent_name: String,
    pub action: String,
    pub input_summary: Option<String>,
    pub output_summary: Option<String>,
    pub duration_ms: Option<i32>,
    pub status: String,
    pub error_message: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewAiAnalysisResult {
    pub company_id: Uuid,
    pub analysis_type: String,
    pub source_module: String,
    pub source_document_id: Option<Uuid>,
    pub input_data: Option<JsonValue>,
    pub result_data: JsonValue,
    pub confidence_score: Option<BigDecimal>,
    pub model_used: Option<String>,
    pub processing_time_ms: Option<i32>,
    pub created_by: Option<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewAiTaxComputation {
    pub company_id: Uuid,
    pub tax_type: String,
    pub fiscal_year_id: Option<Uuid>,
    pub period_id: Option<Uuid>,
    pub computation_data: JsonValue,
    pub result_data: JsonValue,
    pub created_by: Option<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewAiAgentLog {
    pub company_id: Uuid,
    pub agent_name: String,
    pub action: String,
    pub input_summary: Option<String>,
    pub output_summary: Option<String>,
    pub duration_ms: Option<i32>,
    pub status: String,
    pub error_message: Option<String>,
}

pub struct AiIntelligenceRepo {
    pool: PgPool,
}

impl AiIntelligenceRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn save_analysis_result(
        &self,
        new_result: NewAiAnalysisResult,
    ) -> Result<AiAnalysisResult, sqlx::Error> {
        sqlx::query_as::<_, AiAnalysisResult>(
            r#"INSERT INTO ai_analysis_results (company_id, analysis_type, source_module, source_document_id, input_data, result_data, confidence_score, model_used, processing_time_ms, created_by)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            RETURNING id, company_id, analysis_type, source_module, source_document_id, input_data, result_data, confidence_score, model_used, processing_time_ms, created_by, created_at"#,
        )
        .bind(new_result.company_id)
        .bind(&new_result.analysis_type)
        .bind(&new_result.source_module)
        .bind(new_result.source_document_id)
        .bind(&new_result.input_data)
        .bind(&new_result.result_data)
        .bind(&new_result.confidence_score)
        .bind(&new_result.model_used)
        .bind(new_result.processing_time_ms)
        .bind(new_result.created_by)
        .fetch_one(&self.pool)
        .await
    }

    pub async fn find_analysis_results(
        &self,
        company_id: Uuid,
        analysis_type: Option<&str>,
    ) -> Result<Vec<AiAnalysisResult>, sqlx::Error> {
        match analysis_type {
            Some(at) => {
                sqlx::query_as::<_, AiAnalysisResult>(
                    "SELECT id, company_id, analysis_type, source_module, source_document_id, input_data, result_data, confidence_score, model_used, processing_time_ms, created_by, created_at FROM ai_analysis_results WHERE company_id = $1 AND analysis_type = $2 ORDER BY created_at DESC",
                )
                .bind(company_id)
                .bind(at)
                .fetch_all(&self.pool)
                .await
            }
            None => {
                sqlx::query_as::<_, AiAnalysisResult>(
                    "SELECT id, company_id, analysis_type, source_module, source_document_id, input_data, result_data, confidence_score, model_used, processing_time_ms, created_by, created_at FROM ai_analysis_results WHERE company_id = $1 ORDER BY created_at DESC",
                )
                .bind(company_id)
                .fetch_all(&self.pool)
                .await
            }
        }
    }

    pub async fn save_tax_computation(
        &self,
        new_comp: NewAiTaxComputation,
    ) -> Result<AiTaxComputation, sqlx::Error> {
        sqlx::query_as::<_, AiTaxComputation>(
            r#"INSERT INTO ai_tax_computations (company_id, tax_type, fiscal_year_id, period_id, computation_data, result_data, created_by)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            RETURNING id, company_id, tax_type, fiscal_year_id, period_id, computation_data, result_data, is_accepted, accepted_by, accepted_at, created_by, created_at"#,
        )
        .bind(new_comp.company_id)
        .bind(&new_comp.tax_type)
        .bind(new_comp.fiscal_year_id)
        .bind(new_comp.period_id)
        .bind(&new_comp.computation_data)
        .bind(&new_comp.result_data)
        .bind(new_comp.created_by)
        .fetch_one(&self.pool)
        .await
    }

    pub async fn find_tax_computations(
        &self,
        company_id: Uuid,
        tax_type: Option<&str>,
    ) -> Result<Vec<AiTaxComputation>, sqlx::Error> {
        match tax_type {
            Some(tt) => {
                sqlx::query_as::<_, AiTaxComputation>(
                    "SELECT id, company_id, tax_type, fiscal_year_id, period_id, computation_data, result_data, is_accepted, accepted_by, accepted_at, created_by, created_at FROM ai_tax_computations WHERE company_id = $1 AND tax_type = $2 ORDER BY created_at DESC",
                )
                .bind(company_id)
                .bind(tt)
                .fetch_all(&self.pool)
                .await
            }
            None => {
                sqlx::query_as::<_, AiTaxComputation>(
                    "SELECT id, company_id, tax_type, fiscal_year_id, period_id, computation_data, result_data, is_accepted, accepted_by, accepted_at, created_by, created_at FROM ai_tax_computations WHERE company_id = $1 ORDER BY created_at DESC",
                )
                .bind(company_id)
                .fetch_all(&self.pool)
                .await
            }
        }
    }

    pub async fn log_agent_activity(
        &self,
        new_log: NewAiAgentLog,
    ) -> Result<AiAgentLog, sqlx::Error> {
        sqlx::query_as::<_, AiAgentLog>(
            r#"INSERT INTO ai_agent_logs (company_id, agent_name, action, input_summary, output_summary, duration_ms, status, error_message)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            RETURNING id, company_id, agent_name, action, input_summary, output_summary, duration_ms, status, error_message, created_at"#,
        )
        .bind(new_log.company_id)
        .bind(&new_log.agent_name)
        .bind(&new_log.action)
        .bind(&new_log.input_summary)
        .bind(&new_log.output_summary)
        .bind(new_log.duration_ms)
        .bind(&new_log.status)
        .bind(&new_log.error_message)
        .fetch_one(&self.pool)
        .await
    }
}
