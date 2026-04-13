// Author: Quadri Atharu
use anyhow::{anyhow, Result};
use sqlx::PgPool;
use uuid::Uuid;

use crate::models::account::{AccountingPeriod, PeriodStatus};
use crate::models::journal::{JournalHeader, JournalStatus};
use crate::models::posting::{NewJournalLine, PostingAudit, PostingContext, PostingRule};

#[derive(Clone)]
pub struct PostingService {
    pub pool: PgPool,
}

impl PostingService {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn post(&self, context: PostingContext) -> Result<JournalHeader> {
        let rule = self
            .resolve_rule(&context, &self.pool)
            .await?
            .ok_or_else(|| anyhow!("No posting rule found for module={}, type={}", context.source_module, context.transaction_type))?;

        self.validate_posting_context(&context, &rule).await?;

        if let Some(ref key) = context.idempotency_key {
            let existing: Option<Uuid> = sqlx::query_scalar(
                "SELECT journal_header_id FROM posting_audits WHERE idempotency_key = $1 AND company_id = $2",
            )
            .bind(key)
            .bind(context.company_id)
            .fetch_optional(&self.pool)
            .await?;

            if let Some(journal_id) = existing {
                let journal = sqlx::query_as::<_, JournalHeader>(
                    "SELECT * FROM journal_headers WHERE id = $1",
                )
                .bind(journal_id)
                .fetch_one(&self.pool)
                .await?;
                return Ok(journal);
            }
        }

        let journal_lines = self.generate_journal_lines(&context, &rule);
        let total_debit: bigdecimal::BigDecimal = journal_lines.iter().map(|l| l.debit.clone()).sum();
        let total_credit: bigdecimal::BigDecimal = journal_lines.iter().map(|l| l.credit.clone()).sum();

        let entry_number = self.generate_entry_number(context.company_id).await?;
        let period = self.find_open_period(context.company_id, context.posting_date).await?;

        let journal_id = Uuid::now_v7();
        let narration = context
            .narration
            .clone()
            .unwrap_or_else(|| format!("Auto-posting: {} - {}", context.source_module, context.transaction_type));

        sqlx::query(
            r#"INSERT INTO journal_headers (id, company_id, branch_id, fiscal_year_id, period_id, entry_number, reference, narration, status, journal_type, source_module, source_document_id, source_document_number, total_debit, total_credit, currency_code, posted_at, posted_by, created_by, created_at, updated_at)
               VALUES ($1, $2, $3, $4, $5, $6, $7, $8, 'posted', 'auto', $9, $10, $11, $12, $13, $14, NOW(), $15, $15, NOW(), NOW())"#,
        )
        .bind(journal_id)
        .bind(context.company_id)
        .bind(context.branch)
        .bind(period.fiscal_year_id)
        .bind(period.id)
        .bind(&entry_number)
        .bind(&context.reference_id)
        .bind(&narration)
        .bind(&context.source_module)
        .bind(context.source_document_id)
        .bind(&context.source_document_number)
        .bind(&total_debit)
        .bind(&total_credit)
        .bind(&context.currency)
        .bind(context.posted_by)
        .execute(&self.pool)
        .await?;

        for (i, line) in journal_lines.iter().enumerate() {
            let line_id = Uuid::now_v7();
            sqlx::query(
                r#"INSERT INTO journal_lines (id, journal_header_id, account_id, line_number, narration, debit, credit, currency_code, exchange_rate, cost_center_id, project_id, department_id, created_at)
                   VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, NOW())"#,
            )
            .bind(line_id)
            .bind(journal_id)
            .bind(line.account_id)
            .bind(i as i32 + 1)
            .bind(&line.narration)
            .bind(&line.debit)
            .bind(&line.credit)
            .bind(&line.currency_code)
            .bind(&line.exchange_rate)
            .bind(line.cost_center_id)
            .bind(line.project_id)
            .bind(line.department_id)
            .execute(&self.pool)
            .await?;
        }

        self.persist_audit(&context, &rule, journal_id).await?;

        let journal = sqlx::query_as::<_, JournalHeader>(
            "SELECT * FROM journal_headers WHERE id = $1",
        )
        .bind(journal_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(journal)
    }

    pub async fn resolve_rule(
        &self,
        context: &PostingContext,
        pool: &PgPool,
    ) -> Result<Option<PostingRule>> {
        let rules = sqlx::query_as::<_, PostingRule>(
            r#"SELECT * FROM posting_rules
               WHERE company_id = $1
               AND module = $2
               AND transaction_type = $3
               AND ($4::text IS NULL OR transaction_subtype = $4)
               AND is_active = true
               AND effective_from <= $5
               AND ($6::date IS NULL OR effective_to >= $5)
               ORDER BY priority DESC, effective_from DESC"#,
        )
        .bind(context.company_id)
        .bind(&context.source_module)
        .bind(&context.transaction_type)
        .bind(&context.transaction_subtype)
        .bind(context.posting_date)
        .bind(None::<chrono::NaiveDate>)
        .fetch_all(pool)
        .await?;

        let mut matching: Vec<&PostingRule> = rules
            .iter()
            .filter(|r| {
                let branch_match = r.branch_id.is_none() || context.branch == r.branch_id;
                let dept_match = r.department_id.is_none() || context.department == r.department_id;
                let cc_match = r.cost_center_id.is_none() || context.cost_center == r.cost_center_id;
                let proj_match = r.project_id.is_none() || context.project == r.project_id;
                branch_match && dept_match && cc_match && proj_match
            })
            .collect();

        if matching.len() > 1 {
            let exact = matching
                .iter()
                .filter(|r| {
                    r.branch_id.is_some()
                        || r.department_id.is_some()
                        || r.cost_center_id.is_some()
                        || r.project_id.is_some()
                })
                .cloned()
                .collect::<Vec<_>>();

            if exact.len() == 1 {
                return Ok(Some(exact.into_iter().next().unwrap().clone()));
            }

            return Err(anyhow!(
                "Ambiguous posting rules found ({} matches) for module={}, type={}. Refine rules.",
                matching.len(),
                context.source_module,
                context.transaction_type
            ));
        }

        Ok(matching.into_iter().next().cloned())
    }

    pub async fn validate_posting_context(
        &self,
        context: &PostingContext,
        rule: &PostingRule,
    ) -> Result<()> {
        if rule.requires_explicit_rule {
            let _ = rule;
        }

        let period = self.find_open_period(context.company_id, context.posting_date).await?;
        if period.status != PeriodStatus::Open {
            return Err(anyhow!("Accounting period is not open for date {}", context.posting_date));
        }

        if let Some(branch_id) = context.branch {
            let exists: bool = sqlx::query_scalar(
                "SELECT EXISTS(SELECT 1 FROM branches WHERE id = $1 AND is_active = true)",
            )
            .bind(branch_id)
            .fetch_one(&self.pool)
            .await?;
            if !exists {
                return Err(anyhow!("Branch {} not found or inactive", branch_id));
            }
        }

        for account_id in &[rule.debit_account_id, rule.credit_account_id] {
            let account: Option<(bool, bool, bool)> = sqlx::query_as(
                "SELECT is_active, allowed_posting, is_control_account FROM accounts WHERE id = $1",
            )
            .bind(*account_id)
            .fetch_optional(&self.pool)
            .await?;

            match account {
                Some((is_active, allowed_posting, _is_control)) => {
                    if !is_active {
                        return Err(anyhow!("Account {} is not active", account_id));
                    }
                    if !allowed_posting {
                        return Err(anyhow!("Account {} does not allow direct posting", account_id));
                    }
                }
                None => {
                    return Err(anyhow!("Account {} not found", account_id));
                }
            }
        }

        if let Some(tax_account_id) = rule.tax_account_id {
            let is_active: bool = sqlx::query_scalar(
                "SELECT is_active FROM accounts WHERE id = $1",
            )
            .bind(tax_account_id)
            .fetch_one(&self.pool)
            .await
            .unwrap_or(false);

            if !is_active {
                return Err(anyhow!("Tax account {} is not active", tax_account_id));
            }
        }

        if let Some(ref key) = context.idempotency_key {
            let is_dup: bool = sqlx::query_scalar(
                "SELECT EXISTS(SELECT 1 FROM posting_audits WHERE idempotency_key = $1 AND status = 'success')",
            )
            .bind(key)
            .fetch_one(&self.pool)
            .await?;

            if is_dup {
                return Err(anyhow!("Duplicate posting detected for idempotency key {}", key));
            }
        }

        Ok(())
    }

    pub fn generate_journal_lines(
        &self,
        context: &PostingContext,
        rule: &PostingRule,
    ) -> Vec<NewJournalLine> {
        let mut lines = Vec::new();

        let narration = context.narration.clone().unwrap_or_else(|| {
            format!(
                "{} - {} {}",
                context.source_module, context.transaction_type, context.amount
            )
        });

        lines.push(NewJournalLine {
            account_id: rule.debit_account_id,
            narration: Some(narration.clone()),
            debit: context.amount.clone(),
            credit: bigdecimal::BigDecimal::from(0),
            currency_code: context.currency.clone(),
            exchange_rate: None,
            cost_center_id: context.cost_center.or(rule.cost_center_id),
            project_id: context.project.or(rule.project_id),
            department_id: context.department.or(rule.department_id),
        });

        lines.push(NewJournalLine {
            account_id: rule.credit_account_id,
            narration: Some(narration.clone()),
            debit: bigdecimal::BigDecimal::from(0),
            credit: context.amount.clone(),
            currency_code: context.currency.clone(),
            exchange_rate: None,
            cost_center_id: context.cost_center.or(rule.cost_center_id),
            project_id: context.project.or(rule.project_id),
            department_id: context.department.or(rule.department_id),
        });

        if let Some(ref tax_amount) = context.tax_amount {
            if *tax_amount != bigdecimal::BigDecimal::from(0) {
                if let Some(tax_account_id) = rule.tax_account_id {
                    lines.push(NewJournalLine {
                        account_id: tax_account_id,
                        narration: Some(format!("{} - Tax", narration)),
                        debit: tax_amount.clone(),
                        credit: bigdecimal::BigDecimal::from(0),
                        currency_code: context.currency.clone(),
                        exchange_rate: None,
                        cost_center_id: context.cost_center,
                        project_id: context.project,
                        department_id: context.department,
                    });

                    lines.push(NewJournalLine {
                        account_id: rule.credit_account_id,
                        narration: Some(format!("{} - Tax credit adjustment", narration)),
                        debit: bigdecimal::BigDecimal::from(0),
                        credit: tax_amount.clone(),
                        currency_code: context.currency.clone(),
                        exchange_rate: None,
                        cost_center_id: context.cost_center.or(rule.cost_center_id),
                        project_id: context.project.or(rule.project_id),
                        department_id: context.department.or(rule.department_id),
                    });
                }
            }
        }

        lines
    }

    pub async fn persist_audit(
        &self,
        context: &PostingContext,
        rule: &PostingRule,
        journal_id: Uuid,
    ) -> Result<PostingAudit> {
        let id = Uuid::now_v7();
        sqlx::query(
            r#"INSERT INTO posting_audits (id, company_id, journal_header_id, posting_rule_id, source_module, source_document_id, source_document_number, correlation_id, idempotency_key, status, posted_by, posted_at)
               VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, 'success', $10, NOW())"#,
        )
        .bind(id)
        .bind(context.company_id)
        .bind(journal_id)
        .bind(rule.id)
        .bind(&context.source_module)
        .bind(context.source_document_id)
        .bind(&context.source_document_number)
        .bind(&context.correlation_id)
        .bind(&context.idempotency_key)
        .bind(context.posted_by)
        .execute(&self.pool)
        .await?;

        Ok(PostingAudit {
            id,
            company_id: context.company_id,
            journal_header_id: journal_id,
            posting_rule_id: rule.id,
            source_module: context.source_module.clone(),
            source_document_id: context.source_document_id,
            source_document_number: context.source_document_number.clone(),
            correlation_id: context.correlation_id.clone(),
            idempotency_key: context.idempotency_key.clone(),
            status: "success".to_string(),
            error_message: None,
            posted_by: context.posted_by,
            posted_at: chrono::Utc::now().naive_utc(),
        })
    }

    async fn find_open_period(
        &self,
        company_id: Uuid,
        posting_date: chrono::NaiveDate,
    ) -> Result<AccountingPeriod> {
        sqlx::query_as::<_, AccountingPeriod>(
            "SELECT * FROM accounting_periods WHERE company_id = $1 AND start_date <= $2 AND end_date >= $2 AND status = 'open'",
        )
        .bind(company_id)
        .bind(posting_date)
        .fetch_optional(&self.pool)
        .await?
        .ok_or_else(|| anyhow!("No open accounting period found for date {}", posting_date))
    }

    async fn generate_entry_number(&self, company_id: Uuid) -> Result<String> {
        let count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM journal_headers WHERE company_id = $1",
        )
        .bind(company_id)
        .fetch_one(&self.pool)
        .await?;
        Ok(format!("AUTO-{}", count + 1))
    }
}
