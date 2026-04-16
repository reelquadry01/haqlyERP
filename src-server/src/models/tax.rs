// Author: Quadri Atharu
// Tax rates updated per Nigeria Tax Reform Acts 2025 (effective 2026)
// FIRS renamed to NRS (Nigeria Revenue Service)
use bigdecimal::BigDecimal;
use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type, PartialEq)]
#[sqlx(type_name = "tax_type", rename_all = "snake_case")]
pub enum TaxType {
    Vat,
    Wht,
    Cit,
    EduTax,
    Cgt,
    StampDuty,
    Paye,
}

impl std::fmt::Display for TaxType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TaxType::Vat => write!(f, "vat"),
            TaxType::Wht => write!(f, "wht"),
            TaxType::Cit => write!(f, "cit"),
            TaxType::EduTax => write!(f, "edu_tax"),
            TaxType::Cgt => write!(f, "cgt"),
            TaxType::StampDuty => write!(f, "stamp_duty"),
            TaxType::Paye => write!(f, "paye"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type, PartialEq)]
#[sqlx(type_name = "wht_rate_category", rename_all = "snake_case")]
pub enum WhtRateCategory {
    ContractGeneral,
    ContractConstruction,
    Consultancy,
    Management,
    Dividend,
    Interest,
    Royalty,
    Rent,
    Commission,
}

impl WhtRateCategory {
    pub fn rate(&self) -> BigDecimal {
        match self {
            WhtRateCategory::ContractGeneral => BigDecimal::from(5),
            WhtRateCategory::ContractConstruction => BigDecimal::from(5),
            WhtRateCategory::Consultancy => BigDecimal::from(5),
            WhtRateCategory::Management => BigDecimal::from(5),
            WhtRateCategory::Dividend => BigDecimal::from(10),
            WhtRateCategory::Interest => BigDecimal::from(10),
            WhtRateCategory::Royalty => BigDecimal::from(5),
            WhtRateCategory::Rent => BigDecimal::from(10),
            WhtRateCategory::Commission => BigDecimal::from(5),
        }
    }
    pub fn rate_for_individual(&self) -> BigDecimal {
        match self {
            WhtRateCategory::Dividend => BigDecimal::from(5),
            WhtRateCategory::Interest => BigDecimal::from(5),
            WhtRateCategory::Rent => BigDecimal::from(5),
            _ => self.rate(),
        }
    }
}

impl std::fmt::Display for WhtRateCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WhtRateCategory::ContractGeneral => write!(f, "contract_general"),
            WhtRateCategory::ContractConstruction => write!(f, "contract_construction"),
            WhtRateCategory::Consultancy => write!(f, "consultancy"),
            WhtRateCategory::Management => write!(f, "management"),
            WhtRateCategory::Dividend => write!(f, "dividend"),
            WhtRateCategory::Interest => write!(f, "interest"),
            WhtRateCategory::Royalty => write!(f, "royalty"),
            WhtRateCategory::Rent => write!(f, "rent"),
            WhtRateCategory::Commission => write!(f, "commission"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type, PartialEq)]
#[sqlx(type_name = "capital_allowance_category", rename_all = "snake_case")]
pub enum CapitalAllowanceCategory {
    Building,
    PlantAndMachinery,
    FurnitureAndFittings,
    MotorVehicle,
    ComputerAndItEquipment,
}

impl std::fmt::Display for CapitalAllowanceCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CapitalAllowanceCategory::Building => write!(f, "building"),
            CapitalAllowanceCategory::PlantAndMachinery => write!(f, "plant_and_machinery"),
            CapitalAllowanceCategory::FurnitureAndFittings => write!(f, "furniture_and_fittings"),
            CapitalAllowanceCategory::MotorVehicle => write!(f, "motor_vehicle"),
            CapitalAllowanceCategory::ComputerAndItEquipment => {
                write!(f, "computer_and_it_equipment")
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct TaxConfig {
    pub id: Uuid,
    pub company_id: Uuid,
    pub tax_type: TaxType,
    pub name: String,
    pub rate: BigDecimal,
    pub effective_from: chrono::NaiveDate,
    pub effective_to: Option<chrono::NaiveDate>,
    pub is_active: bool,
    pub account_id: Option<Uuid>,
    pub wht_category: Option<WhtRateCategory>,
    pub description: Option<String>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct TaxTransaction {
    pub id: Uuid,
    pub company_id: Uuid,
    pub tax_type: TaxType,
    pub tax_config_id: Uuid,
    pub source_document_id: Option<Uuid>,
    pub source_document_type: Option<String>,
    pub base_amount: BigDecimal,
    pub tax_amount: BigDecimal,
    pub currency_code: String,
    pub period_id: Option<Uuid>,
    pub is_reported: bool,
    pub created_at: NaiveDateTime,
}
