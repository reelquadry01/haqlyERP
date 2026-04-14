// Author: Quadri Atharu
use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type, PartialEq)]
#[sqlx(type_name = "widget_type", rename_all = "snake_case")]
pub enum WidgetType {
    KpiCard,
    Chart,
    Table,
    Gauge,
    Heatmap,
    Funnel,
}

impl std::fmt::Display for WidgetType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WidgetType::KpiCard => write!(f, "kpi_card"),
            WidgetType::Chart => write!(f, "chart"),
            WidgetType::Table => write!(f, "table"),
            WidgetType::Gauge => write!(f, "gauge"),
            WidgetType::Heatmap => write!(f, "heatmap"),
            WidgetType::Funnel => write!(f, "funnel"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type, PartialEq)]
#[sqlx(type_name = "dataset_source_type", rename_all = "snake_case")]
pub enum DatasetSourceType {
    PostgreSQL,
    API,
    PythonEngine,
    File,
}

impl std::fmt::Display for DatasetSourceType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DatasetSourceType::PostgreSQL => write!(f, "postgre_sql"),
            DatasetSourceType::API => write!(f, "api"),
            DatasetSourceType::PythonEngine => write!(f, "python_engine"),
            DatasetSourceType::File => write!(f, "file"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct BiDashboard {
    pub id: Uuid,
    pub company_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub layout_config: serde_json::Value,
    pub is_default: bool,
    pub created_by: Uuid,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct BiWidget {
    pub id: Uuid,
    pub dashboard_id: Uuid,
    pub widget_type: WidgetType,
    pub title: String,
    pub data_source_config: serde_json::Value,
    pub position_config: serde_json::Value,
    pub refresh_interval_seconds: Option<i32>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct BiDataset {
    pub id: Uuid,
    pub company_id: Uuid,
    pub name: String,
    pub source_type: DatasetSourceType,
    pub source_config: serde_json::Value,
    pub last_refreshed: Option<NaiveDateTime>,
    pub is_active: bool,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct BiQuery {
    pub id: Uuid,
    pub dataset_id: Uuid,
    pub name: String,
    pub query_text: String,
    pub parameters: serde_json::Value,
    pub cache_ttl_seconds: i32,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}
