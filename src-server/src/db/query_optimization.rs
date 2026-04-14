// Author: Quadri Atharu

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginationParams {
    pub cursor: Option<String>,
    pub limit: Option<u32>,
    pub sort_by: Option<String>,
    pub sort_order: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CursorPage<T> {
    pub data: Vec<T>,
    pub next_cursor: Option<String>,
    pub has_more: bool,
    pub total_count: Option<i64>,
}

pub struct QueryOptimizer;

impl QueryOptimizer {
    pub fn cursor_paginate(table: &str, params: &PaginationParams) -> String {
        let limit = params.limit.unwrap_or(50).min(200);
        let sort_col = params.sort_by.as_deref().unwrap_or("id");
        let sort_dir = match params.sort_order.as_deref() {
            Some("asc") | Some("ASC") => "ASC",
            Some("desc") | Some("DESC") => "DESC",
            _ => "ASC",
        };
        let fetch_limit = limit + 1;

        if let Some(ref cursor) = params.cursor {
            let operator = if sort_dir == "ASC" { ">" } else { "<" };
            format!(
                "SELECT * FROM {} WHERE {} {} $1 ORDER BY {} {} LIMIT {}",
                table, sort_col, operator, sort_col, sort_dir, fetch_limit
            )
        } else {
            format!(
                "SELECT * FROM {} ORDER BY {} {} LIMIT {}",
                table, sort_col, sort_dir, fetch_limit
            )
        }
    }

    pub fn batch_insert_sql(table: &str, columns: &[&str], row_count: usize) -> String {
        let cols = columns.join(", ");
        let mut placeholders = Vec::new();
        let mut param_idx = 1;
        for _ in 0..row_count {
            let mut row_placeholders = Vec::new();
            for _ in 0..columns.len() {
                row_placeholders.push(format!("${}", param_idx));
                param_idx += 1;
            }
            placeholders.push(format!("({})", row_placeholders.join(", ")));
        }
        format!("INSERT INTO {} ({}) VALUES {}", table, cols, placeholders.join(", "))
    }

    pub fn materialized_trial_balance_query() -> String {
        r#"
WITH account_balances AS (
    SELECT
        a.id AS account_id,
        a.code AS account_code,
        a.name AS account_name,
        a.account_type,
        a.org_id,
        COALESCE(SUM(CASE WHEN jl.debit_amount > 0 THEN jl.debit_amount ELSE 0 END), 0) AS total_debit,
        COALESCE(SUM(CASE WHEN jl.credit_amount > 0 THEN jl.credit_amount ELSE 0 END), 0) AS total_credit
    FROM chart_of_accounts a
    LEFT JOIN journal_lines jl ON jl.account_id = a.id
    LEFT JOIN journal_entries je ON je.id = jl.journal_entry_id AND je.status = 'posted'
    GROUP BY a.id, a.code, a.name, a.account_type, a.org_id
),
running_balances AS (
    SELECT
        ab.*,
        CASE
            WHEN ab.account_type IN ('asset', 'expense') THEN ab.total_debit - ab.total_credit
            ELSE ab.total_credit - ab.total_debit
        END AS running_balance
    FROM account_balances ab
)
SELECT
    rb.account_id,
    rb.account_code,
    rb.account_name,
    rb.account_type,
    rb.total_debit,
    rb.total_credit,
    rb.running_balance,
    CASE WHEN rb.running_balance >= 0 THEN rb.running_balance ELSE 0 END AS debit_balance,
    CASE WHEN rb.running_balance < 0 THEN ABS(rb.running_balance) ELSE 0 END AS credit_balance
FROM running_balances rb
ORDER BY rb.account_code
        "#.trim().to_string()
    }

    pub fn optimized_journal_query() -> String {
        r#"
SELECT
    je.id AS je_id,
    je.entry_number,
    je.description,
    je.entry_date,
    je.posting_date,
    je.status,
    je.source_type,
    je.source_id,
    je.org_id,
    je.created_by,
    je.created_at,
    je.updated_at,
    jl.id AS line_id,
    jl.account_id,
    jl.debit_amount,
    jl.credit_amount,
    jl.line_description,
    jl.line_order,
    a.code AS account_code,
    a.name AS account_name
FROM journal_entries je
LEFT JOIN journal_lines jl ON jl.journal_entry_id = je.id
    LEFT JOIN chart_of_accounts a ON a.id = jl.account_id
WHERE je.org_id = $1
ORDER BY je.entry_date DESC, je.created_at DESC, jl.line_order ASC
        "#.trim().to_string()
    }

    pub fn index_recommendations() -> Vec<String> {
        vec![
            "CREATE INDEX IF NOT EXISTS idx_journal_entries_org_date ON journal_entries (org_id, entry_date DESC)".to_string(),
            "CREATE INDEX IF NOT EXISTS idx_journal_entries_status ON journal_entries (status, org_id)".to_string(),
            "CREATE INDEX IF NOT EXISTS idx_journal_entries_source ON journal_entries (source_type, source_id)".to_string(),
            "CREATE INDEX IF NOT EXISTS idx_journal_lines_entry_id ON journal_lines (journal_entry_id)".to_string(),
            "CREATE INDEX IF NOT EXISTS idx_journal_lines_account_id ON journal_lines (account_id)".to_string(),
            "CREATE INDEX IF NOT EXISTS idx_accounts_code_org ON chart_of_accounts (code, company_id)".to_string(),
            "CREATE INDEX IF NOT EXISTS idx_accounts_type_org ON chart_of_accounts (account_type, company_id)".to_string(),
            "CREATE INDEX IF NOT EXISTS idx_posting_status_date ON journal_entries (status, posting_date) WHERE status = 'posted'".to_string(),
            "CREATE INDEX IF NOT EXISTS idx_payment_vouchers_org_date ON payment_vouchers (org_id, created_at DESC)".to_string(),
            "CREATE INDEX IF NOT EXISTS idx_sales_invoices_org_date ON sales_invoices (org_id, invoice_date DESC)".to_string(),
            "CREATE INDEX IF NOT EXISTS idx_purchase_invoices_org_date ON purchase_invoices (org_id, invoice_date DESC)".to_string(),
            "CREATE INDEX IF NOT EXISTS idx_inventory_items_org_code ON inventory_items (org_id, item_code)".to_string(),
            "CREATE INDEX IF NOT EXISTS idx_fixed_assets_org_status ON fixed_assets (org_id, status)".to_string(),
            "CREATE INDEX IF NOT EXISTS idx_users_org_active ON users (org_id, is_active) WHERE is_active = true".to_string(),
            "CREATE INDEX IF NOT EXISTS idx_audit_logs_org_date ON audit_logs (org_id, created_at DESC)".to_string(),
        ]
    }
}
