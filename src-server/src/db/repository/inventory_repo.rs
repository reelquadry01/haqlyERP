// Author: Quadri Atharu
use bigdecimal::BigDecimal;
use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Product {
    pub id: Uuid,
    pub company_id: Uuid,
    pub sku: String,
    pub name: String,
    pub description: Option<String>,
    pub category: Option<String>,
    pub product_type: String,
    pub unit_of_measure: String,
    pub cost_price: BigDecimal,
    pub selling_price: BigDecimal,
    pub tax_code: Option<String>,
    pub is_active: bool,
    pub track_inventory: bool,
    pub reorder_point: BigDecimal,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
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
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct StockMovement {
    pub id: Uuid,
    pub company_id: Uuid,
    pub product_id: Uuid,
    pub warehouse_id: Uuid,
    pub movement_type: String,
    pub quantity: BigDecimal,
    pub unit_cost: BigDecimal,
    pub reference_type: Option<String>,
    pub reference_id: Option<Uuid>,
    pub narration: Option<String>,
    pub date: NaiveDate,
    pub created_by: Uuid,
    pub created_at: DateTime<Utc>,
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
    pub last_cost: BigDecimal,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewProduct {
    pub company_id: Uuid,
    pub sku: String,
    pub name: String,
    pub description: Option<String>,
    pub category: Option<String>,
    pub product_type: String,
    pub unit_of_measure: String,
    pub cost_price: BigDecimal,
    pub selling_price: BigDecimal,
    pub tax_code: Option<String>,
    pub track_inventory: bool,
    pub reorder_point: BigDecimal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewWarehouse {
    pub company_id: Uuid,
    pub branch_id: Option<Uuid>,
    pub code: String,
    pub name: String,
    pub location: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewStockMovement {
    pub company_id: Uuid,
    pub product_id: Uuid,
    pub warehouse_id: Uuid,
    pub movement_type: String,
    pub quantity: BigDecimal,
    pub unit_cost: BigDecimal,
    pub reference_type: Option<String>,
    pub reference_id: Option<Uuid>,
    pub narration: Option<String>,
    pub date: NaiveDate,
    pub created_by: Uuid,
}

pub struct InventoryRepo {
    pool: PgPool,
}

impl InventoryRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn create_product(&self, new_product: NewProduct) -> Result<Product, sqlx::Error> {
        sqlx::query_as::<_, Product>(
            r#"INSERT INTO products (company_id, sku, name, description, category, product_type, unit_of_measure, cost_price, selling_price, tax_code, track_inventory, reorder_point)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
            RETURNING id, company_id, sku, name, description, category, product_type, unit_of_measure, cost_price, selling_price, tax_code, is_active, track_inventory, reorder_point, created_at, updated_at"#,
        )
        .bind(new_product.company_id)
        .bind(&new_product.sku)
        .bind(&new_product.name)
        .bind(&new_product.description)
        .bind(&new_product.category)
        .bind(&new_product.product_type)
        .bind(&new_product.unit_of_measure)
        .bind(&new_product.cost_price)
        .bind(&new_product.selling_price)
        .bind(&new_product.tax_code)
        .bind(new_product.track_inventory)
        .bind(&new_product.reorder_point)
        .fetch_one(&self.pool)
        .await
    }

    pub async fn find_products(&self, company_id: Uuid) -> Result<Vec<Product>, sqlx::Error> {
        sqlx::query_as::<_, Product>(
            "SELECT id, company_id, sku, name, description, category, product_type, unit_of_measure, cost_price, selling_price, tax_code, is_active, track_inventory, reorder_point, created_at, updated_at FROM products WHERE company_id = $1 ORDER BY sku",
        )
        .bind(company_id)
        .fetch_all(&self.pool)
        .await
    }

    pub async fn create_warehouse(
        &self,
        new_warehouse: NewWarehouse,
    ) -> Result<Warehouse, sqlx::Error> {
        sqlx::query_as::<_, Warehouse>(
            r#"INSERT INTO warehouses (company_id, branch_id, code, name, location)
            VALUES ($1, $2, $3, $4, $5)
            RETURNING id, company_id, branch_id, code, name, location, is_active, created_at, updated_at"#,
        )
        .bind(new_warehouse.company_id)
        .bind(new_warehouse.branch_id)
        .bind(&new_warehouse.code)
        .bind(&new_warehouse.name)
        .bind(&new_warehouse.location)
        .fetch_one(&self.pool)
        .await
    }

    pub async fn find_warehouses(&self, company_id: Uuid) -> Result<Vec<Warehouse>, sqlx::Error> {
        sqlx::query_as::<_, Warehouse>(
            "SELECT id, company_id, branch_id, code, name, location, is_active, created_at, updated_at FROM warehouses WHERE company_id = $1 ORDER BY code",
        )
        .bind(company_id)
        .fetch_all(&self.pool)
        .await
    }

    pub async fn record_movement(
        &self,
        movement: NewStockMovement,
    ) -> Result<StockMovement, sqlx::Error> {
        sqlx::query_as::<_, StockMovement>(
            r#"INSERT INTO stock_movements (company_id, product_id, warehouse_id, movement_type, quantity, unit_cost, reference_type, reference_id, narration, date, created_by)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
            RETURNING id, company_id, product_id, warehouse_id, movement_type, quantity, unit_cost, reference_type, reference_id, narration, date, created_by, created_at"#,
        )
        .bind(movement.company_id)
        .bind(movement.product_id)
        .bind(movement.warehouse_id)
        .bind(&movement.movement_type)
        .bind(&movement.quantity)
        .bind(&movement.unit_cost)
        .bind(&movement.reference_type)
        .bind(movement.reference_id)
        .bind(&movement.narration)
        .bind(movement.date)
        .bind(movement.created_by)
        .fetch_one(&self.pool)
        .await
    }

    pub async fn get_stock_levels(
        &self,
        warehouse_id: Uuid,
    ) -> Result<Vec<StockLevel>, sqlx::Error> {
        sqlx::query_as::<_, StockLevel>(
            "SELECT id, product_id, warehouse_id, quantity_on_hand, quantity_reserved, quantity_available, average_cost, last_cost, created_at, updated_at FROM inventory_stock_levels WHERE warehouse_id = $1",
        )
        .bind(warehouse_id)
        .fetch_all(&self.pool)
        .await
    }
}
