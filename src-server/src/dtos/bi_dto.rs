// Author: Quadri Atharu
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

use crate::models::bi::{DatasetSourceType, WidgetType};

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct CreateDashboardRequest {
    pub company_id: Uuid,
    #[validate(length(min = 1, message = "name is required"))]
    pub name: String,
    pub description: Option<String>,
    pub layout_config: serde_json::Value,
    pub is_default: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct AddWidgetRequest {
    pub dashboard_id: Uuid,
    pub widget_type: WidgetType,
    #[validate(length(min = 1, message = "title is required"))]
    pub title: String,
    pub data_source_config: serde_json::Value,
    pub position_config: serde_json::Value,
    pub refresh_interval_seconds: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct CreateDatasetRequest {
    pub company_id: Uuid,
    #[validate(length(min = 1, message = "name is required"))]
    pub name: String,
    pub source_type: DatasetSourceType,
    pub source_config: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct ExecuteQueryRequest {
    pub dataset_id: Uuid,
    #[validate(length(min = 1, message = "query text is required"))]
    pub query_text: String,
    pub parameters: Option<serde_json::Value>,
    pub cache_ttl_seconds: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KpiData {
    pub company_id: Uuid,
    pub period: String,
    pub revenue: bigdecimal::BigDecimal,
    pub expenses: bigdecimal::BigDecimal,
    pub net_income: bigdecimal::BigDecimal,
    pub cash_balance: bigdecimal::BigDecimal,
    pub ar_aging_current: bigdecimal::BigDecimal,
    pub ar_aging_30: bigdecimal::BigDecimal,
    pub ar_aging_60: bigdecimal::BigDecimal,
    pub ar_aging_90: bigdecimal::BigDecimal,
    pub ar_aging_over_90: bigdecimal::BigDecimal,
    pub ap_aging_current: bigdecimal::BigDecimal,
    pub ap_aging_30: bigdecimal::BigDecimal,
    pub ap_aging_60: bigdecimal::BigDecimal,
    pub ap_aging_90: bigdecimal::BigDecimal,
    pub ap_aging_over_90: bigdecimal::BigDecimal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FinancialSummary {
    pub company_id: Uuid,
    pub total_revenue: bigdecimal::BigDecimal,
    pub total_expenses: bigdecimal::BigDecimal,
    pub gross_profit: bigdecimal::BigDecimal,
    pub net_income: bigdecimal::BigDecimal,
    pub total_assets: bigdecimal::BigDecimal,
    pub total_liabilities: bigdecimal::BigDecimal,
    pub total_equity: bigdecimal::BigDecimal,
    pub cash_and_equivalents: bigdecimal::BigDecimal,
    pub accounts_receivable: bigdecimal::BigDecimal,
    pub accounts_payable: bigdecimal::BigDecimal,
    pub inventory_value: bigdecimal::BigDecimal,
    pub fixed_assets_net: bigdecimal::BigDecimal,
}
