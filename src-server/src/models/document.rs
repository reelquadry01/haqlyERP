// Author: Quadri Atharu
use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type, PartialEq)]
#[sqlx(type_name = "varchar", rename_all = "snake_case")]
pub enum EntityType {
    Journal,
    Invoice,
    Bill,
    Voucher,
    Asset,
    Employee,
    Report,
    Other,
}

impl std::fmt::Display for EntityType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EntityType::Journal => write!(f, "journal"),
            EntityType::Invoice => write!(f, "invoice"),
            EntityType::Bill => write!(f, "bill"),
            EntityType::Voucher => write!(f, "voucher"),
            EntityType::Asset => write!(f, "asset"),
            EntityType::Employee => write!(f, "employee"),
            EntityType::Report => write!(f, "report"),
            EntityType::Other => write!(f, "other"),
        }
    }
}

impl std::str::FromStr for EntityType {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "journal" => Ok(EntityType::Journal),
            "invoice" => Ok(EntityType::Invoice),
            "bill" => Ok(EntityType::Bill),
            "voucher" => Ok(EntityType::Voucher),
            "asset" => Ok(EntityType::Asset),
            "employee" => Ok(EntityType::Employee),
            "report" => Ok(EntityType::Report),
            "other" => Ok(EntityType::Other),
            _ => Err(format!("Unknown entity type: {}", s)),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct DocumentAttachment {
    pub id: Uuid,
    pub company_id: Uuid,
    pub entity_type: String,
    pub entity_id: Uuid,
    pub file_name: String,
    pub file_path: String,
    pub file_size: i64,
    pub mime_type: String,
    pub description: Option<String>,
    pub uploaded_by: Option<Uuid>,
    pub is_deleted: bool,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentUploadDto {
    pub company_id: Uuid,
    pub entity_type: String,
    pub entity_id: Uuid,
    pub description: Option<String>,
}
