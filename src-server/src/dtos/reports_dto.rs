// Author: Quadri Atharu
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type, PartialEq)]
#[sqlx(type_name = "statement_type", rename_all = "snake_case")]
pub enum StatementType {
    ProfitAndLoss,
    BalanceSheet,
    CashFlow,
}

impl std::fmt::Display for StatementType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StatementType::ProfitAndLoss => write!(f, "profit_and_loss"),
            StatementType::BalanceSheet => write!(f, "balance_sheet"),
            StatementType::CashFlow => write!(f, "cash_flow"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type, PartialEq)]
#[sqlx(type_name = "cash_flow_method", rename_all = "snake_case")]
pub enum CashFlowMethod {
    Direct,
    Indirect,
}

impl std::fmt::Display for CashFlowMethod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CashFlowMethod::Direct => write!(f, "direct"),
            CashFlowMethod::Indirect => write!(f, "indirect"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct TrialBalanceRequest {
    pub company_id: Uuid,
    pub period_id: Option<Uuid>,
    pub as_of_date: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct FinancialStatementRequest {
    pub company_id: Uuid,
    pub statement_type: StatementType,
    pub period_id: Option<Uuid>,
    pub from_date: Option<String>,
    pub to_date: Option<String>,
    pub as_of_date: Option<String>,
    pub method: Option<CashFlowMethod>,
    pub comparative: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RatioAnalysisRequest {
    pub company_id: Uuid,
    pub period_id: Option<Uuid>,
    pub from_date: Option<String>,
    pub to_date: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct DateRangeRequest {
    pub company_id: Uuid,
    pub from_date: String,
    pub to_date: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrialBalanceRow {
    pub account_code: String,
    pub account_name: String,
    pub account_type: String,
    pub debit: bigdecimal::BigDecimal,
    pub credit: bigdecimal::BigDecimal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrialBalance {
    pub rows: Vec<TrialBalanceRow>,
    pub total_debit: bigdecimal::BigDecimal,
    pub total_credit: bigdecimal::BigDecimal,
    pub is_balanced: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountStatementRow {
    pub date: String,
    pub entry_number: String,
    pub narration: String,
    pub debit: bigdecimal::BigDecimal,
    pub credit: bigdecimal::BigDecimal,
    pub balance: bigdecimal::BigDecimal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountStatement {
    pub account_id: Uuid,
    pub account_code: String,
    pub account_name: String,
    pub opening_balance: bigdecimal::BigDecimal,
    pub rows: Vec<AccountStatementRow>,
    pub closing_balance: bigdecimal::BigDecimal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IncomeStatement {
    pub company_id: Uuid,
    pub from_date: String,
    pub to_date: String,
    pub revenue: Vec<StatementLine>,
    pub total_revenue: bigdecimal::BigDecimal,
    pub expenses: Vec<StatementLine>,
    pub total_expenses: bigdecimal::BigDecimal,
    pub gross_profit: bigdecimal::BigDecimal,
    pub net_income: bigdecimal::BigDecimal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatementLine {
    pub account_code: String,
    pub account_name: String,
    pub amount: bigdecimal::BigDecimal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BalanceSheet {
    pub company_id: Uuid,
    pub as_of_date: String,
    pub assets: Vec<BalanceSheetSection>,
    pub total_assets: bigdecimal::BigDecimal,
    pub liabilities: Vec<BalanceSheetSection>,
    pub total_liabilities: bigdecimal::BigDecimal,
    pub equity: Vec<BalanceSheetSection>,
    pub total_equity: bigdecimal::BigDecimal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BalanceSheetSection {
    pub category: String,
    pub lines: Vec<StatementLine>,
    pub subtotal: bigdecimal::BigDecimal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CashFlowStatement {
    pub company_id: Uuid,
    pub from_date: String,
    pub to_date: String,
    pub method: String,
    pub operating_activities: Vec<StatementLine>,
    pub net_operating: bigdecimal::BigDecimal,
    pub investing_activities: Vec<StatementLine>,
    pub net_investing: bigdecimal::BigDecimal,
    pub financing_activities: Vec<StatementLine>,
    pub net_financing: bigdecimal::BigDecimal,
    pub net_change: bigdecimal::BigDecimal,
    pub opening_cash: bigdecimal::BigDecimal,
    pub closing_cash: bigdecimal::BigDecimal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RatioAnalysis {
    pub company_id: Uuid,
    pub period_id: Option<Uuid>,
    pub current_ratio: bigdecimal::BigDecimal,
    pub quick_ratio: bigdecimal::BigDecimal,
    pub cash_ratio: bigdecimal::BigDecimal,
    pub debt_to_equity: bigdecimal::BigDecimal,
    pub interest_coverage: bigdecimal::BigDecimal,
    pub roe: bigdecimal::BigDecimal,
    pub roa: bigdecimal::BigDecimal,
    pub roce: bigdecimal::BigDecimal,
    pub gross_margin: bigdecimal::BigDecimal,
    pub net_margin: bigdecimal::BigDecimal,
    pub asset_turnover: bigdecimal::BigDecimal,
    pub inventory_turnover: bigdecimal::BigDecimal,
    pub receivable_turnover: bigdecimal::BigDecimal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportRequest {
    pub format: String,
    pub report_type: String,
    pub company_id: Uuid,
    pub params: serde_json::Value,
}
