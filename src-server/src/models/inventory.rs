// Author: Quadri Atharu
use bigdecimal::BigDecimal;
use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type, PartialEq)]
#[sqlx(type_name = "product_type", rename_all = "snake_case")]
pub enum ProductType {
    Goods,
    Service,
}

impl std::fmt::Display for ProductType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProductType::Goods => write!(f, "goods"),
            ProductType::Service => write!(f, "service"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type, PartialEq)]
#[sqlx(type_name = "movement_type", rename_all = "snake_case")]
pub enum MovementType {
    PurchaseReceipt,
    SaleIssue,
    TransferIn,
    TransferOut,
    AdjustmentIncrease,
    AdjustmentDecrease,
    ReturnToSupplier,
    ReturnFromCustomer,
    OpeningBalance,
}

impl std::fmt::Display for MovementType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MovementType::PurchaseReceipt => write!(f, "purchase_receipt"),
            MovementType::SaleIssue => write!(f, "sale_issue"),
            MovementType::TransferIn => write!(f, "transfer_in"),
            MovementType::TransferOut => write!(f, "transfer_out"),
            MovementType::AdjustmentIncrease => write!(f, "adjustment_increase"),
            MovementType::AdjustmentDecrease => write!(f, "adjustment_decrease"),
            MovementType::ReturnToSupplier => write!(f, "return_to_supplier"),
            MovementType::ReturnFromCustomer => write!(f, "return_from_customer"),
            MovementType::OpeningBalance => write!(f, "opening_balance"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Product {
    pub id: Uuid,
    pub company_id: Uuid,
    pub code: String,
    pub name: String,
    pub product_type: ProductType,
    pub category: Option<String>,
    pub unit_of_measure: Option<String>,
    pub sales_price: Option<BigDecimal>,
    pub purchase_price: Option<BigDecimal>,
    pub cost_price: Option<BigDecimal>,
    pub tax_rate: Option<BigDecimal>,
    pub is_taxable: bool,
    pub revenue_account_id: Option<Uuid>,
    pub inventory_account_id: Option<Uuid>,
    pub cogs_account_id: Option<Uuid>,
    pub is_active: bool,
    pub reorder_level: Option<BigDecimal>,
    pub reorder_quantity: Option<BigDecimal>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Warehouse {
    pub id: Uuid,
    pub company_id: Uuid,
    pub branch_id: Option<Uuid>,
    pub code: String,
    pub name: String,
    pub location: Option<String>,
    pub is_active: bool,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct StockMovement {
    pub id: Uuid,
    pub company_id: Uuid,
    pub product_id: Uuid,
    pub warehouse_id: Uuid,
    pub movement_type: MovementType,
    pub quantity: BigDecimal,
    pub unit_cost: Option<BigDecimal>,
    pub reference: Option<String>,
    pub source_document_id: Option<Uuid>,
    pub source_document_type: Option<String>,
    pub narration: Option<String>,
    pub created_by: Uuid,
    pub created_at: NaiveDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct StockLevel {
    pub id: Uuid,
    pub product_id: Uuid,
    pub warehouse_id: Uuid,
    pub quantity_on_hand: BigDecimal,
    pub quantity_reserved: BigDecimal,
    pub quantity_available: BigDecimal,
    pub average_cost: BigDecimal,
    pub last_cost: Option<BigDecimal>,
    pub updated_at: NaiveDateTime,
}
