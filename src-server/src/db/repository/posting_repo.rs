// Author: Quadri Atharu
use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use sqlx::FromRow;
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct PostingRule {
    pub id: Uuid,
    pub company_id: Uuid,
    pub module: String,
    pub transaction_type: String,
    pub transaction_subtype: Option<String>,
    pub legal_entity_id: Option<Uuid>,
    pub branch_id: Option<Uuid>,
    pub department_id: Option<Uuid>,
    pub product_category: Option<String>,
    pub customer_group: Option<String>,
    pub vendor_group: Option<String>,
    pub tax_code: Option<String>,
    pub currency_code: Option<String>,
    pub condition_expression: Option<String>,
    pub debit_account_id: Uuid,
    pub credit_account_id: Uuid,
    pub tax_account_id: Option<Uuid>,
    pub rounding_account_id: Option<Uuid>,
    pub exchange_gain_account_id: Option<Uuid>,
    pub exchange_loss_account_id: Option<Uuid>,
    pub suspense_account_id: Option<Uuid>,
    pub posting_description_template: Option<String>,
    pub require_branch: bool,
    pub require_department: bool,
    pub require_cost_center: bool,
    pub require_project: bool,
    pub require_subledger: bool,
    pub require_tax: bool,
    pub effective_from: NaiveDate,
    pub effective_to: Option<NaiveDate>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct PostingAudit {
    pub id: Uuid,
    pub source_module: String,
    pub source_table: String,
    pub source_document_id: Uuid,
    pub source_document_number: Option<String>,
    pub reference_id: Option<String>,
    pub customer_or_vendor: Option<String>,
    pub triggering_event: String,
    pub posting_rule_id: Option<Uuid>,
    pub user_id: Option<Uuid>,
    pub approval_reference: Option<String>,
    pub posting_timestamp: DateTime<Utc>,
    pub period_id: Option<Uuid>,
    pub branch_id: Option<Uuid>,
    pub legal_entity_id: Option<Uuid>,
    pub department_id: Option<Uuid>,
    pub cost_center_id: Option<Uuid>,
    pub project_id: Option<Uuid>,
    pub tax_code: Option<String>,
    pub currency_code: Option<String>,
    pub narration: Option<String>,
    pub correlation_id: Option<Uuid>,
    pub idempotency_key: Uuid,
    pub reversal_of_audit_id: Option<Uuid>,
    pub rule_snapshot: Option<JsonValue>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewPostingAudit {
    pub source_module: String,
    pub source_table: String,
    pub source_document_id: Uuid,
    pub source_document_number: Option<String>,
    pub reference_id: Option<String>,
    pub customer_or_vendor: Option<String>,
    pub triggering_event: String,
    pub posting_rule_id: Option<Uuid>,
    pub user_id: Option<Uuid>,
    pub approval_reference: Option<String>,
    pub period_id: Option<Uuid>,
    pub branch_id: Option<Uuid>,
    pub legal_entity_id: Option<Uuid>,
    pub department_id: Option<Uuid>,
    pub cost_center_id: Option<Uuid>,
    pub project_id: Option<Uuid>,
    pub tax_code: Option<String>,
    pub currency_code: Option<String>,
    pub narration: Option<String>,
    pub correlation_id: Option<Uuid>,
    pub idempotency_key: Uuid,
    pub rule_snapshot: Option<JsonValue>,
}

pub struct PostingRepo {
    pool: PgPool,
}

impl PostingRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn find_rule_by_context(
        &self,
        company_id: Uuid,
        module: &str,
        transaction_type: &str,
        transaction_subtype: Option<&str>,
        branch_id: Option<Uuid>,
        department_id: Option<Uuid>,
    ) -> Result<Option<PostingRule>, sqlx::Error> {
        sqlx::query_as::<_, PostingRule>(
            r#"SELECT id, company_id, module, transaction_type, transaction_subtype,
                legal_entity_id, branch_id, department_id, product_category,
                customer_group, vendor_group, tax_code, currency_code,
                condition_expression, debit_account_id, credit_account_id,
                tax_account_id, rounding_account_id, exchange_gain_account_id,
                exchange_loss_account_id, suspense_account_id,
                posting_description_template, require_branch, require_department,
                require_cost_center, require_project, require_subledger,
                require_tax, effective_from, effective_to, is_active,
                created_at, updated_at
            FROM posting_rules
            WHERE company_id = $1 AND module = $2 AND transaction_type = $3
                AND is_active = true
                AND effective_from <= CURRENT_DATE
                AND (effective_to IS NULL OR effective_to >= CURRENT_DATE)
            ORDER BY
                CASE WHEN transaction_subtype = $4 THEN 0 ELSE 1 END,
                CASE WHEN branch_id = $5 THEN 0 ELSE 1 END,
                CASE WHEN department_id = $6 THEN 0 ELSE 1 END
            LIMIT 1"#,
        )
        .bind(company_id)
        .bind(module)
        .bind(transaction_type)
        .bind(transaction_subtype)
        .bind(branch_id)
        .bind(department_id)
        .fetch_optional(&self.pool)
        .await
    }

    pub async fn create_audit(&self, entry: NewPostingAudit) -> Result<PostingAudit, sqlx::Error> {
        sqlx::query_as::<_, PostingAudit>(
            r#"INSERT INTO posting_audits (
                source_module, source_table, source_document_id, source_document_number,
                reference_id, customer_or_vendor, triggering_event, posting_rule_id,
                user_id, approval_reference, period_id, branch_id, legal_entity_id,
                department_id, cost_center_id, project_id, tax_code, currency_code,
                narration, correlation_id, idempotency_key, rule_snapshot
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19, $20, $21, $22)
            RETURNING id, source_module, source_table, source_document_id, source_document_number,
                reference_id, customer_or_vendor, triggering_event, posting_rule_id,
                user_id, approval_reference, posting_timestamp, period_id, branch_id,
                legal_entity_id, department_id, cost_center_id, project_id, tax_code,
                currency_code, narration, correlation_id, idempotency_key,
                reversal_of_audit_id, rule_snapshot, created_at"#,
        )
        .bind(&entry.source_module)
        .bind(&entry.source_table)
        .bind(entry.source_document_id)
        .bind(&entry.source_document_number)
        .bind(&entry.reference_id)
        .bind(&entry.customer_or_vendor)
        .bind(&entry.triggering_event)
        .bind(entry.posting_rule_id)
        .bind(entry.user_id)
        .bind(&entry.approval_reference)
        .bind(entry.period_id)
        .bind(entry.branch_id)
        .bind(entry.legal_entity_id)
        .bind(entry.department_id)
        .bind(entry.cost_center_id)
        .bind(entry.project_id)
        .bind(&entry.tax_code)
        .bind(&entry.currency_code)
        .bind(&entry.narration)
        .bind(entry.correlation_id)
        .bind(entry.idempotency_key)
        .bind(&entry.rule_snapshot)
        .fetch_one(&self.pool)
        .await
    }

    pub async fn find_audit_by_idempotency_key(
        &self,
        key: Uuid,
    ) -> Result<Option<PostingAudit>, sqlx::Error> {
        sqlx::query_as::<_, PostingAudit>(
            r#"SELECT id, source_module, source_table, source_document_id, source_document_number,
                reference_id, customer_or_vendor, triggering_event, posting_rule_id,
                user_id, approval_reference, posting_timestamp, period_id, branch_id,
                legal_entity_id, department_id, cost_center_id, project_id, tax_code,
                currency_code, narration, correlation_id, idempotency_key,
                reversal_of_audit_id, rule_snapshot, created_at
            FROM posting_audits WHERE idempotency_key = $1"#,
        )
        .bind(key)
        .fetch_optional(&self.pool)
        .await
    }
}
