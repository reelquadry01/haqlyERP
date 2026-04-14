// Author: Quadri Atharu
use anyhow::{anyhow, Result};
use bigdecimal::BigDecimal;
use sqlx::PgPool;
use uuid::Uuid;

use crate::dtos::journal_dto::{CreateJournalRequest, CreateTemplateRequest, JournalFilterParams, UpdateJournalRequest};
use crate::models::account::{AccountingPeriod, PeriodStatus};
use crate::models::journal::{
    JournalHeader, JournalLine, JournalStatus, JournalTemplate, JournalTemplateLine,
    JournalHeaderWithLines,
};

#[derive(Clone)]
pub struct JournalsService {
    pub pool: PgPool,
}

impl JournalsService {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn create_draft(
        &self,
        company_id: Uuid,
        req: CreateJournalRequest,
        created_by: Uuid,
    ) -> Result<JournalHeader> {
        let total_debit: BigDecimal = req.lines.iter().filter_map(|l| l.debit.clone()).sum();
        let total_credit: BigDecimal = req.lines.iter().filter_map(|l| l.credit.clone()).sum();

        if total_debit != total_credit {
            return Err(anyhow!("Journal must balance: debit {} != credit {}", total_debit, total_credit));
        }

        let period = self.find_open_period(company_id, req.reference.as_deref()).await?;
        let entry_number = self.generate_entry_number(company_id).await?;
        let id = Uuid::now_v7();
        let currency = req.currency_code.unwrap_or_else(|| "NGN".to_string());

        let mut tx = self.pool.begin().await.map_err(|e| anyhow!("Failed to begin transaction: {}", e))?;

        sqlx::query(
            r#"INSERT INTO journal_headers (id, company_id, branch_id, fiscal_year_id, period_id, entry_number, reference, narration, status, journal_type, total_debit, total_credit, currency_code, created_by, created_at, updated_at)
               VALUES ($1, $2, $3, $4, $5, $6, $7, $8, 'draft', $9, $10, $11, $12, $13, NOW(), NOW())"#,
        )
        .bind(id)
        .bind(company_id)
        .bind(req.branch_id)
        .bind(period.fiscal_year_id)
        .bind(period.id)
        .bind(&entry_number)
        .bind(&req.reference)
        .bind(&req.narration)
        .bind(&req.journal_type)
        .bind(&total_debit)
        .bind(&total_credit)
        .bind(&currency)
        .bind(created_by)
        .execute(&mut *tx)
        .await?;

        for (i, line) in req.lines.iter().enumerate() {
            let line_id = Uuid::now_v7();
            sqlx::query(
                r#"INSERT INTO journal_lines (id, journal_header_id, account_id, line_number, narration, debit, credit, currency_code, cost_center_id, project_id, department_id, created_at)
                   VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, NOW())"#,
            )
            .bind(line_id)
            .bind(id)
            .bind(line.account_id)
            .bind(i as i32 + 1)
            .bind(&line.narration)
            .bind(&line.debit)
            .bind(&line.credit)
            .bind(&currency)
            .bind(line.cost_center_id)
            .bind(line.project_id)
            .bind(line.department_id)
            .execute(&mut *tx)
            .await?;
        }

        tx.commit().await.map_err(|e| anyhow!("Failed to commit journal creation: {}", e))?;

        self.get_journal(id).await
    }

    pub async fn update_draft(
        &self,
        journal_id: Uuid,
        req: UpdateJournalRequest,
    ) -> Result<JournalHeader> {
        let journal = self.get_journal(journal_id).await?;
        if journal.status != JournalStatus::Draft {
            return Err(anyhow!("Only draft journals can be updated"));
        }

        let mut tx = self.pool.begin().await.map_err(|e| anyhow!("Failed to begin transaction: {}", e))?;

        if let Some(narration) = req.narration {
            sqlx::query("UPDATE journal_headers SET narration = $1, updated_at = NOW() WHERE id = $2")
                .bind(&narration)
                .bind(journal_id)
                .execute(&mut *tx)
                .await?;
        }

        if let Some(lines) = req.lines {
            sqlx::query("DELETE FROM journal_lines WHERE journal_header_id = $1")
                .bind(journal_id)
                .execute(&mut *tx)
                .await?;

            let total_debit: BigDecimal = lines.iter().filter_map(|l| l.debit.clone()).sum();
            let total_credit: BigDecimal = lines.iter().filter_map(|l| l.credit.clone()).sum();

            for (i, line) in lines.iter().enumerate() {
                let line_id = Uuid::now_v7();
                sqlx::query(
                    r#"INSERT INTO journal_lines (id, journal_header_id, account_id, line_number, narration, debit, credit, currency_code, cost_center_id, project_id, department_id, created_at)
                       VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, NOW())"#,
                )
                .bind(line_id)
                .bind(journal_id)
                .bind(line.account_id)
                .bind(i as i32 + 1)
                .bind(&line.narration)
                .bind(&line.debit)
                .bind(&line.credit)
                .bind(&journal.currency_code)
                .bind(line.cost_center_id)
                .bind(line.project_id)
                .bind(line.department_id)
                .execute(&mut *tx)
                .await?;
            }

            sqlx::query(
                "UPDATE journal_headers SET total_debit = $1, total_credit = $2, updated_at = NOW() WHERE id = $3",
            )
            .bind(&total_debit)
            .bind(&total_credit)
            .bind(journal_id)
            .execute(&mut *tx)
            .await?;
        }

        tx.commit().await.map_err(|e| anyhow!("Failed to commit journal update: {}", e))?;

        self.get_journal(journal_id).await
    }

    pub async fn validate(&self, journal_id: Uuid) -> Result<JournalHeader> {
        let journal = self.get_journal(journal_id).await?;
        if journal.status != JournalStatus::Draft {
            return Err(anyhow!("Only draft journals can be validated"));
        }

        if journal.total_debit != journal.total_credit {
            return Err(anyhow!("Journal does not balance"));
        }

        let lines = sqlx::query_as::<_, JournalLine>(
            "SELECT * FROM journal_lines WHERE journal_header_id = $1 ORDER BY line_number",
        )
        .bind(journal_id)
        .fetch_all(&self.pool)
        .await?;

        if lines.len() < 2 {
            return Err(anyhow!("Journal must have at least 2 lines"));
        }

        for line in &lines {
            let account_exists: bool = sqlx::query_scalar(
                "SELECT EXISTS(SELECT 1 FROM accounts WHERE id = $1 AND is_active = true AND allowed_posting = true)",
            )
            .bind(line.account_id)
            .fetch_one(&self.pool)
            .await?;

            if !account_exists {
                return Err(anyhow!("Account {} is not active or does not allow posting", line.account_id));
            }
        }

        let period = sqlx::query_as::<_, AccountingPeriod>(
            "SELECT * FROM accounting_periods WHERE id = $1",
        )
        .bind(journal.period_id)
        .fetch_one(&self.pool)
        .await?;

        if period.status != PeriodStatus::Open {
            return Err(anyhow!("Period is not open"));
        }

        sqlx::query(
            "UPDATE journal_headers SET status = 'validated', updated_at = NOW() WHERE id = $1",
        )
        .bind(journal_id)
        .execute(&self.pool)
        .await?;

        self.get_journal(journal_id).await
    }

    pub async fn submit_for_approval(&self, journal_id: Uuid) -> Result<JournalHeader> {
        let journal = self.get_journal(journal_id).await?;
        if journal.status != JournalStatus::Validated {
            return Err(anyhow!("Only validated journals can be submitted"));
        }

        sqlx::query(
            "UPDATE journal_headers SET status = 'submitted', updated_at = NOW() WHERE id = $1",
        )
        .bind(journal_id)
        .execute(&self.pool)
        .await?;

        self.get_journal(journal_id).await
    }

    pub async fn approve(&self, journal_id: Uuid, approved_by: Uuid) -> Result<JournalHeader> {
        let journal = self.get_journal(journal_id).await?;
        if journal.status != JournalStatus::Submitted {
            return Err(anyhow!("Only submitted journals can be approved"));
        }

        sqlx::query(
            "UPDATE journal_headers SET status = 'approved', approved_by = $1, updated_at = NOW() WHERE id = $2",
        )
        .bind(approved_by)
        .bind(journal_id)
        .execute(&self.pool)
        .await?;

        self.get_journal(journal_id).await
    }

    pub async fn reject(&self, journal_id: Uuid) -> Result<JournalHeader> {
        let journal = self.get_journal(journal_id).await?;
        if journal.status != JournalStatus::Submitted {
            return Err(anyhow!("Only submitted journals can be rejected"));
        }

        sqlx::query(
            "UPDATE journal_headers SET status = 'draft', updated_at = NOW() WHERE id = $1",
        )
        .bind(journal_id)
        .execute(&self.pool)
        .await?;

        self.get_journal(journal_id).await
    }

    pub async fn recall(&self, journal_id: Uuid) -> Result<JournalHeader> {
        let journal = self.get_journal(journal_id).await?;
        if journal.status != JournalStatus::Submitted {
            return Err(anyhow!("Only submitted journals can be recalled"));
        }

        sqlx::query(
            "UPDATE journal_headers SET status = 'validated', updated_at = NOW() WHERE id = $1",
        )
        .bind(journal_id)
        .execute(&self.pool)
        .await?;

        self.get_journal(journal_id).await
    }

    pub async fn post_to_gl(&self, journal_id: Uuid, posted_by: Uuid) -> Result<JournalHeader> {
        let journal = self.get_journal(journal_id).await?;
        if journal.status != JournalStatus::Approved {
            return Err(anyhow!("Only approved journals can be posted"));
        }

        let lines = sqlx::query_as::<_, JournalLine>(
            "SELECT * FROM journal_lines WHERE journal_header_id = $1",
        )
        .bind(journal_id)
        .fetch_all(&self.pool)
        .await?;

        let mut tx = self.pool.begin().await.map_err(|e| anyhow!("Failed to begin transaction: {}", e))?;

        for line in &lines {
            let debit: BigDecimal = line.debit.clone();
            let credit: BigDecimal = line.credit.clone();
            let net = &debit - &credit;

            let account_type: String = sqlx::query_scalar(
                "SELECT account_type::text FROM accounts WHERE id = $1",
            )
            .bind(line.account_id)
            .fetch_one(&mut *tx)
            .await?;

            let balance_change = match account_type.as_str() {
                "asset" | "expense" => net,
                "liability" | "equity" | "revenue" => -net.clone(),
                _ => net,
            };

            if balance_change != BigDecimal::from(0) {
                sqlx::query(
                    "UPDATE accounts SET balance = balance + $1, updated_at = NOW() WHERE id = $2",
                )
                .bind(&balance_change)
                .bind(line.account_id)
                .execute(&mut *tx)
                .await?;
            }
        }

        sqlx::query(
            "UPDATE journal_headers SET status = 'posted', posted_at = NOW(), posted_by = $1, updated_at = NOW() WHERE id = $2",
        )
        .bind(posted_by)
        .bind(journal_id)
        .execute(&mut *tx)
        .await?;

        tx.commit().await.map_err(|e| anyhow!("Failed to commit GL posting: {}", e))?;

        self.get_journal(journal_id).await
    }

    pub async fn reverse(&self, journal_id: Uuid, reason: String, reversed_by: Uuid) -> Result<JournalHeader> {
        let journal = self.get_journal(journal_id).await?;
        if journal.status != JournalStatus::Posted {
            return Err(anyhow!("Only posted journals can be reversed"));
        }

        let lines = sqlx::query_as::<_, JournalLine>(
            "SELECT * FROM journal_lines WHERE journal_header_id = $1",
        )
        .bind(journal_id)
        .fetch_all(&self.pool)
        .await?;

        let reversal_entry_number = self.generate_entry_number(journal.company_id).await?;
        let reversal_id = Uuid::now_v7();
        let reversal_narration = format!("Reversal of {} - {}", journal.entry_number, reason);

        let mut tx = self.pool.begin().await.map_err(|e| anyhow!("Failed to begin transaction: {}", e))?;

        sqlx::query(
            r#"INSERT INTO journal_headers (id, company_id, branch_id, fiscal_year_id, period_id, entry_number, reference, narration, status, journal_type, source_module, reversal_of, total_debit, total_credit, currency_code, created_by, created_at, updated_at)
               VALUES ($1, $2, $3, $4, $5, $6, $7, $8, 'posted', $9, $10, $11, $12, $13, $14, $15, NOW(), NOW())"#,
        )
        .bind(reversal_id)
        .bind(journal.company_id)
        .bind(journal.branch_id)
        .bind(journal.fiscal_year_id)
        .bind(journal.period_id)
        .bind(&reversal_entry_number)
        .bind(&journal.reference)
        .bind(&reversal_narration)
        .bind(&journal.journal_type)
        .bind(journal.source_module.as_deref())
        .bind(journal_id)
        .bind(&journal.total_credit)
        .bind(&journal.total_debit)
        .bind(&journal.currency_code)
        .bind(reversed_by)
        .execute(&mut *tx)
        .await?;

        for (i, line) in lines.iter().enumerate() {
            let line_id = Uuid::now_v7();
            sqlx::query(
                r#"INSERT INTO journal_lines (id, journal_header_id, account_id, line_number, narration, debit, credit, currency_code, cost_center_id, project_id, department_id, created_at)
                   VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, NOW())"#,
            )
            .bind(line_id)
            .bind(reversal_id)
            .bind(line.account_id)
            .bind(i as i32 + 1)
            .bind(&line.narration)
            .bind(&line.credit)
            .bind(&line.debit)
            .bind(&line.currency_code)
            .bind(line.cost_center_id)
            .bind(line.project_id)
            .bind(line.department_id)
            .execute(&mut *tx)
            .await?;
        }

        sqlx::query(
            "UPDATE journal_headers SET status = 'reversed', updated_at = NOW() WHERE id = $1",
        )
        .bind(journal_id)
        .execute(&mut *tx)
        .await?;

        tx.commit().await.map_err(|e| anyhow!("Failed to commit reversal: {}", e))?;

        self.get_journal(reversal_id).await
    }

    pub async fn cancel(&self, journal_id: Uuid) -> Result<JournalHeader> {
        let journal = self.get_journal(journal_id).await?;
        if journal.status == JournalStatus::Posted || journal.status == JournalStatus::Reversed {
            return Err(anyhow!("Posted or reversed journals cannot be cancelled"));
        }

        sqlx::query(
            "UPDATE journal_headers SET status = 'cancelled', updated_at = NOW() WHERE id = $1",
        )
        .bind(journal_id)
        .execute(&self.pool)
        .await?;

        self.get_journal(journal_id).await
    }

    pub async fn list_journals(&self, params: JournalFilterParams) -> Result<Vec<JournalHeaderWithLines>> {
        let limit = params.limit.unwrap_or(50);
        let offset = (params.page.unwrap_or(1) - 1) * limit;

        let headers = sqlx::query_as::<_, JournalHeader>(
            "SELECT * FROM journal_headers WHERE company_id = $1 AND ($2::text IS NULL OR status::text = $2) AND ($3::uuid IS NULL OR branch_id = $3) ORDER BY created_at DESC LIMIT $4 OFFSET $5",
        )
        .bind(params.company_id)
        .bind(&params.status)
        .bind(params.branch_id)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await?;

        if headers.is_empty() {
            return Ok(Vec::new());
        }

        let header_ids: Vec<Uuid> = headers.iter().map(|h| h.id).collect();

        let all_lines = sqlx::query_as::<_, JournalLine>(
            "SELECT * FROM journal_lines WHERE journal_header_id = ANY($1) ORDER BY line_number",
        )
        .bind(&header_ids)
        .fetch_all(&self.pool)
        .await?;

        let mut lines_by_header: std::collections::HashMap<Uuid, Vec<JournalLine>> = std::collections::HashMap::new();
        for line in all_lines {
            lines_by_header
                .entry(line.journal_header_id)
                .or_default()
                .push(line);
        }

        let result = headers
            .into_iter()
            .map(|header| {
                let lines = lines_by_header.remove(&header.id).unwrap_or_default();
                JournalHeaderWithLines { header, lines }
            })
            .collect();

        Ok(result)
    }

    pub async fn get_journal(&self, id: Uuid) -> Result<JournalHeader> {
        sqlx::query_as::<_, JournalHeader>("SELECT * FROM journal_headers WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await?
            .ok_or_else(|| anyhow!("Journal not found"))
    }

    pub async fn get_journal_with_lines(&self, id: Uuid) -> Result<JournalHeaderWithLines> {
        let header = self.get_journal(id).await?;
        let lines = sqlx::query_as::<_, JournalLine>(
            "SELECT * FROM journal_lines WHERE journal_header_id = $1 ORDER BY line_number",
        )
        .bind(id)
        .fetch_all(&self.pool)
        .await?;
        Ok(JournalHeaderWithLines { header, lines })
    }

    pub async fn create_template(
        &self,
        req: CreateTemplateRequest,
        created_by: Uuid,
    ) -> Result<JournalTemplate> {
        let id = Uuid::now_v7();

        let mut tx = self.pool.begin().await.map_err(|e| anyhow!("Failed to begin transaction: {}", e))?;

        sqlx::query(
            r#"INSERT INTO journal_templates (id, company_id, name, narration_template, journal_type, recurrence, is_active, created_by, created_at, updated_at)
               VALUES ($1, $2, $3, $4, $5, $6, true, $7, NOW(), NOW())"#,
        )
        .bind(id)
        .bind(req.company_id)
        .bind(&req.name)
        .bind(&req.narration_template)
        .bind(&req.journal_type)
        .bind(&req.recurrence)
        .bind(created_by)
        .execute(&mut *tx)
        .await?;

        for (i, line) in req.lines.iter().enumerate() {
            let line_id = Uuid::now_v7();
            sqlx::query(
                r#"INSERT INTO journal_template_lines (id, template_id, account_id, line_number, narration_template, debit_expression, credit_expression, cost_center_id, project_id)
                   VALUES ($1, $2, $3, $4, $5, NULL, NULL, $6, $7)"#,
            )
            .bind(line_id)
            .bind(id)
            .bind(line.account_id)
            .bind(i as i32 + 1)
            .bind(&line.narration)
            .bind(line.cost_center_id)
            .bind(line.project_id)
            .execute(&mut *tx)
            .await?;
        }

        tx.commit().await.map_err(|e| anyhow!("Failed to commit template creation: {}", e))?;

        sqlx::query_as::<_, JournalTemplate>("SELECT * FROM journal_templates WHERE id = $1")
            .bind(id)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| anyhow!("Failed to fetch template: {}", e))
    }

    pub async fn list_templates(&self, company_id: Uuid) -> Result<Vec<JournalTemplate>> {
        let templates = sqlx::query_as::<_, JournalTemplate>(
            "SELECT * FROM journal_templates WHERE company_id = $1 AND is_active = true ORDER BY name",
        )
        .bind(company_id)
        .fetch_all(&self.pool)
        .await?;
        Ok(templates)
    }

    pub async fn generate_from_template(
        &self,
        template_id: Uuid,
        amounts: Vec<(usize, BigDecimal, BigDecimal)>,
        created_by: Uuid,
    ) -> Result<JournalHeader> {
        let template = sqlx::query_as::<_, JournalTemplate>(
            "SELECT * FROM journal_templates WHERE id = $1",
        )
        .bind(template_id)
        .fetch_one(&self.pool)
        .await?;

        let template_lines = sqlx::query_as::<_, JournalTemplateLine>(
            "SELECT * FROM journal_template_lines WHERE template_id = $1 ORDER BY line_number",
        )
        .bind(template_id)
        .fetch_all(&self.pool)
        .await?;

        let mut journal_lines_dto = Vec::new();
        for tl in &template_lines {
            let idx = (tl.line_number - 1) as usize;
            let (debit, credit) = amounts.get(idx).cloned().unwrap_or((BigDecimal::from(0), BigDecimal::from(0)));
            journal_lines_dto.push(crate::dtos::journal_dto::JournalLineDto {
                account_id: tl.account_id,
                narration: tl.narration_template.clone(),
                debit: Some(debit),
                credit: Some(credit),
                cost_center_id: tl.cost_center_id,
                project_id: tl.project_id,
                department_id: None,
            });
        }

        self.create_draft(
            template.company_id,
            CreateJournalRequest {
                company_id: template.company_id,
                branch_id: None,
                narration: template.narration_template.clone(),
                reference: None,
                journal_type: template.journal_type.clone(),
                currency_code: None,
                lines: journal_lines_dto,
            },
            created_by,
        )
        .await
    }

    async fn find_open_period(
        &self,
        company_id: Uuid,
        _reference: Option<&str>,
    ) -> Result<AccountingPeriod> {
        sqlx::query_as::<_, AccountingPeriod>(
            "SELECT * FROM accounting_periods WHERE company_id = $1 AND status = 'open' ORDER BY start_date DESC LIMIT 1",
        )
        .bind(company_id)
        .fetch_optional(&self.pool)
        .await?
        .ok_or_else(|| anyhow!("No open accounting period found"))
    }

    async fn generate_entry_number(&self, company_id: Uuid) -> Result<String> {
        let count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM journal_headers WHERE company_id = $1",
        )
        .bind(company_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(format!("JE-{}", count + 1))
    }
}
