// Author: Quadri Atharu
use bigdecimal::BigDecimal;
use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct JournalHeader {
    pub id: Uuid,
    pub company_id: Uuid,
    pub branch_id: Option<Uuid>,
    pub department_id: Option<Uuid>,
    pub fiscal_year_id: Uuid,
    pub period_id: Uuid,
    pub number: String,
    pub date: NaiveDate,
    pub narration: Option<String>,
    pub source_module: Option<String>,
    pub source_type: Option<String>,
    pub source_document_id: Option<Uuid>,
    pub reference_id: Option<String>,
    pub currency_code: String,
    pub exchange_rate: BigDecimal,
    pub status: String,
    pub total_debit: BigDecimal,
    pub total_credit: BigDecimal,
    pub is_balanced: bool,
    pub created_by: Uuid,
    pub submitted_by: Option<Uuid>,
    pub approved_by: Option<Uuid>,
    pub posted_by: Option<Uuid>,
    pub submitted_at: Option<DateTime<Utc>>,
    pub approved_at: Option<DateTime<Utc>>,
    pub posted_at: Option<DateTime<Utc>>,
    pub reversal_of: Option<Uuid>,
    pub reversal_reason: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct JournalLine {
    pub id: Uuid,
    pub journal_header_id: Uuid,
    pub line_number: i32,
    pub account_id: Uuid,
    pub debit: BigDecimal,
    pub credit: BigDecimal,
    pub narration: Option<String>,
    pub cost_center_id: Option<Uuid>,
    pub project_id: Option<Uuid>,
    pub subledger_party: Option<String>,
    pub tax_code: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JournalHeaderWithLines {
    pub header: JournalHeader,
    pub lines: Vec<JournalLine>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewJournalHeader {
    pub company_id: Uuid,
    pub branch_id: Option<Uuid>,
    pub department_id: Option<Uuid>,
    pub fiscal_year_id: Uuid,
    pub period_id: Uuid,
    pub number: String,
    pub date: NaiveDate,
    pub narration: Option<String>,
    pub source_module: Option<String>,
    pub source_type: Option<String>,
    pub source_document_id: Option<Uuid>,
    pub reference_id: Option<String>,
    pub currency_code: String,
    pub exchange_rate: BigDecimal,
    pub created_by: Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewJournalLine {
    pub line_number: i32,
    pub account_id: Uuid,
    pub debit: BigDecimal,
    pub credit: BigDecimal,
    pub narration: Option<String>,
    pub cost_center_id: Option<Uuid>,
    pub project_id: Option<Uuid>,
    pub subledger_party: Option<String>,
    pub tax_code: Option<String>,
}

pub struct JournalRepo {
    pool: PgPool,
}

impl JournalRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn create_header(
        &self,
        new_header: NewJournalHeader,
    ) -> Result<JournalHeader, sqlx::Error> {
        sqlx::query_as::<_, JournalHeader>(
            r#"INSERT INTO journal_headers (
                company_id, branch_id, department_id, fiscal_year_id, period_id,
                number, date, narration, source_module, source_type,
                source_document_id, reference_id, currency_code, exchange_rate, created_by
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15)
            RETURNING id, company_id, branch_id, department_id, fiscal_year_id, period_id,
                number, date, narration, source_module, source_type, source_document_id,
                reference_id, currency_code, exchange_rate, status, total_debit, total_credit,
                is_balanced, created_by, submitted_by, approved_by, posted_by,
                submitted_at, approved_at, posted_at, reversal_of, reversal_reason,
                created_at, updated_at"#,
        )
        .bind(new_header.company_id)
        .bind(new_header.branch_id)
        .bind(new_header.department_id)
        .bind(new_header.fiscal_year_id)
        .bind(new_header.period_id)
        .bind(&new_header.number)
        .bind(new_header.date)
        .bind(&new_header.narration)
        .bind(&new_header.source_module)
        .bind(&new_header.source_type)
        .bind(new_header.source_document_id)
        .bind(&new_header.reference_id)
        .bind(&new_header.currency_code)
        .bind(&new_header.exchange_rate)
        .bind(new_header.created_by)
        .fetch_one(&self.pool)
        .await
    }

    pub async fn create_lines(
        &self,
        header_id: Uuid,
        lines: Vec<NewJournalLine>,
    ) -> Result<Vec<JournalLine>, sqlx::Error> {
        let mut created = Vec::with_capacity(lines.len());
        for line in lines {
            let row = sqlx::query_as::<_, JournalLine>(
                r#"INSERT INTO journal_lines (
                    journal_header_id, line_number, account_id, debit, credit,
                    narration, cost_center_id, project_id, subledger_party, tax_code
                ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
                RETURNING id, journal_header_id, line_number, account_id, debit, credit,
                    narration, cost_center_id, project_id, subledger_party, tax_code, created_at"#,
            )
            .bind(header_id)
            .bind(line.line_number)
            .bind(line.account_id)
            .bind(&line.debit)
            .bind(&line.credit)
            .bind(&line.narration)
            .bind(line.cost_center_id)
            .bind(line.project_id)
            .bind(&line.subledger_party)
            .bind(&line.tax_code)
            .fetch_one(&self.pool)
            .await?;
            created.push(row);
        }
        Ok(created)
    }

    pub async fn find_by_id(&self, id: Uuid) -> Result<Option<JournalHeaderWithLines>, sqlx::Error> {
        let header = sqlx::query_as::<_, JournalHeader>(
            "SELECT id, company_id, branch_id, department_id, fiscal_year_id, period_id, number, date, narration, source_module, source_type, source_document_id, reference_id, currency_code, exchange_rate, status, total_debit, total_credit, is_balanced, created_by, submitted_by, approved_by, posted_by, submitted_at, approved_at, posted_at, reversal_of, reversal_reason, created_at, updated_at FROM journal_headers WHERE id = $1",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        match header {
            Some(h) => {
                let lines = sqlx::query_as::<_, JournalLine>(
                    "SELECT id, journal_header_id, line_number, account_id, debit, credit, narration, cost_center_id, project_id, subledger_party, tax_code, created_at FROM journal_lines WHERE journal_header_id = $1 ORDER BY line_number",
                )
                .bind(id)
                .fetch_all(&self.pool)
                .await?;

                Ok(Some(JournalHeaderWithLines { header: h, lines }))
            }
            None => Ok(None),
        }
    }

    pub async fn find_by_company(
        &self,
        company_id: Uuid,
        status: Option<&str>,
    ) -> Result<Vec<JournalHeader>, sqlx::Error> {
        match status {
            Some(s) => {
                sqlx::query_as::<_, JournalHeader>(
                    "SELECT id, company_id, branch_id, department_id, fiscal_year_id, period_id, number, date, narration, source_module, source_type, source_document_id, reference_id, currency_code, exchange_rate, status, total_debit, total_credit, is_balanced, created_by, submitted_by, approved_by, posted_by, submitted_at, approved_at, posted_at, reversal_of, reversal_reason, created_at, updated_at FROM journal_headers WHERE company_id = $1 AND status = $2 ORDER BY date DESC, number",
                )
                .bind(company_id)
                .bind(s)
                .fetch_all(&self.pool)
                .await
            }
            None => {
                sqlx::query_as::<_, JournalHeader>(
                    "SELECT id, company_id, branch_id, department_id, fiscal_year_id, period_id, number, date, narration, source_module, source_type, source_document_id, reference_id, currency_code, exchange_rate, status, total_debit, total_credit, is_balanced, created_by, submitted_by, approved_by, posted_by, submitted_at, approved_at, posted_at, reversal_of, reversal_reason, created_at, updated_at FROM journal_headers WHERE company_id = $1 ORDER BY date DESC, number",
                )
                .bind(company_id)
                .fetch_all(&self.pool)
                .await
            }
        }
    }

    pub async fn update_status(&self, id: Uuid, status: &str) -> Result<(), sqlx::Error> {
        sqlx::query("UPDATE journal_headers SET status = $1, updated_at = now() WHERE id = $2")
            .bind(status)
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }
}
