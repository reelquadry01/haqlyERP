// Author: Quadri Atharu
use anyhow::{anyhow, Result};
use bigdecimal::BigDecimal;
use sqlx::PgPool;
use uuid::Uuid;

use crate::models::inventory::{
    MovementType, Product, ProductType, StockLevel, StockMovement, Warehouse,
};

#[derive(Clone)]
pub struct InventoryService {
    pub pool: PgPool,
}

impl InventoryService {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn create_product(
        &self,
        company_id: Uuid,
        code: String,
        name: String,
        product_type: ProductType,
        category: Option<String>,
        unit_of_measure: Option<String>,
        sales_price: Option<BigDecimal>,
        purchase_price: Option<BigDecimal>,
        cost_price: Option<BigDecimal>,
        tax_rate: Option<BigDecimal>,
        is_taxable: bool,
        revenue_account_id: Option<Uuid>,
        inventory_account_id: Option<Uuid>,
        cogs_account_id: Option<Uuid>,
        reorder_level: Option<BigDecimal>,
        reorder_quantity: Option<BigDecimal>,
    ) -> Result<Product> {
        let id = Uuid::now_v7();
        sqlx::query(
            r#"INSERT INTO products (id, company_id, code, name, product_type, category, unit_of_measure, sales_price, purchase_price, cost_price, tax_rate, is_taxable, revenue_account_id, inventory_account_id, cogs_account_id, is_active, reorder_level, reorder_quantity, created_at, updated_at)
               VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, true, $16, $17, NOW(), NOW())"#,
        )
        .bind(id)
        .bind(company_id)
        .bind(&code)
        .bind(&name)
        .bind(&product_type)
        .bind(&category)
        .bind(&unit_of_measure)
        .bind(&sales_price)
        .bind(&purchase_price)
        .bind(&cost_price)
        .bind(&tax_rate)
        .bind(is_taxable)
        .bind(revenue_account_id)
        .bind(inventory_account_id)
        .bind(cogs_account_id)
        .bind(&reorder_level)
        .bind(&reorder_quantity)
        .execute(&self.pool)
        .await?;

        sqlx::query_as::<_, Product>("SELECT * FROM products WHERE id = $1")
            .bind(id)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| anyhow!("Failed to fetch product: {}", e))
    }

    pub async fn list_products(&self, company_id: Uuid) -> Result<Vec<Product>> {
        let products = sqlx::query_as::<_, Product>(
            "SELECT * FROM products WHERE company_id = $1 AND is_active = true ORDER BY code",
        )
        .bind(company_id)
        .fetch_all(&self.pool)
        .await?;
        Ok(products)
    }

    pub async fn create_warehouse(
        &self,
        company_id: Uuid,
        branch_id: Option<Uuid>,
        code: String,
        name: String,
        location: Option<String>,
    ) -> Result<Warehouse> {
        let id = Uuid::now_v7();
        sqlx::query(
            r#"INSERT INTO warehouses (id, company_id, branch_id, code, name, location, is_active, created_at, updated_at)
               VALUES ($1, $2, $3, $4, $5, $6, true, NOW(), NOW())"#,
        )
        .bind(id)
        .bind(company_id)
        .bind(branch_id)
        .bind(&code)
        .bind(&name)
        .bind(&location)
        .execute(&self.pool)
        .await?;

        sqlx::query_as::<_, Warehouse>("SELECT * FROM warehouses WHERE id = $1")
            .bind(id)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| anyhow!("Failed to fetch warehouse: {}", e))
    }

    pub async fn record_stock_movement(
        &self,
        company_id: Uuid,
        product_id: Uuid,
        warehouse_id: Uuid,
        movement_type: MovementType,
        quantity: BigDecimal,
        unit_cost: Option<BigDecimal>,
        reference: Option<String>,
        source_document_id: Option<Uuid>,
        source_document_type: Option<String>,
        narration: Option<String>,
        created_by: Uuid,
    ) -> Result<StockMovement> {
        let id = Uuid::now_v7();
        sqlx::query(
            r#"INSERT INTO stock_movements (id, company_id, product_id, warehouse_id, movement_type, quantity, unit_cost, reference, source_document_id, source_document_type, narration, created_by, created_at)
               VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, NOW())"#,
        )
        .bind(id)
        .bind(company_id)
        .bind(product_id)
        .bind(warehouse_id)
        .bind(&movement_type)
        .bind(&quantity)
        .bind(&unit_cost)
        .bind(&reference)
        .bind(source_document_id)
        .bind(&source_document_type)
        .bind(&narration)
        .bind(created_by)
        .execute(&self.pool)
        .await?;

        self.update_stock_level(product_id, warehouse_id, &movement_type, &quantity, &unit_cost)
            .await?;

        sqlx::query_as::<_, StockMovement>("SELECT * FROM stock_movements WHERE id = $1")
            .bind(id)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| anyhow!("Failed to fetch movement: {}", e))
    }

    pub async fn get_stock_levels(&self, company_id: Uuid) -> Result<Vec<StockLevel>> {
        let levels = sqlx::query_as::<_, StockLevel>(
            r#"SELECT sl.* FROM stock_levels sl
               JOIN products p ON sl.product_id = p.id
               WHERE p.company_id = $1
               ORDER BY p.code"#,
        )
        .bind(company_id)
        .fetch_all(&self.pool)
        .await?;
        Ok(levels)
    }

    pub async fn check_reorder_alerts(&self, company_id: Uuid) -> Result<Vec<Product>> {
        let products = sqlx::query_as::<_, Product>(
            r#"SELECT p.* FROM products p
               JOIN stock_levels sl ON sl.product_id = p.id
               WHERE p.company_id = $1
               AND p.is_active = true
               AND p.reorder_level IS NOT NULL
               AND sl.quantity_available <= p.reorder_level
               ORDER BY p.code"#,
        )
        .bind(company_id)
        .fetch_all(&self.pool)
        .await?;
        Ok(products)
    }

    async fn update_stock_level(
        &self,
        product_id: Uuid,
        warehouse_id: Uuid,
        movement_type: &MovementType,
        quantity: &BigDecimal,
        unit_cost: &Option<BigDecimal>,
    ) -> Result<()> {
        let existing = sqlx::query_as::<_, StockLevel>(
            "SELECT * FROM stock_levels WHERE product_id = $1 AND warehouse_id = $2",
        )
        .bind(product_id)
        .bind(warehouse_id)
        .fetch_optional(&self.pool)
        .await?;

        let is_increase = matches!(
            movement_type,
            MovementType::PurchaseReceipt
                | MovementType::TransferIn
                | MovementType::AdjustmentIncrease
                | MovementType::ReturnFromCustomer
                | MovementType::OpeningBalance
        );

        match existing {
            Some(mut sl) => {
                if is_increase {
                    let total_cost = &sl.quantity_on_hand * &sl.average_cost + quantity * unit_cost.as_ref().unwrap_or(&BigDecimal::from(0));
                    sl.quantity_on_hand += quantity;
                    sl.quantity_available = &sl.quantity_on_hand - &sl.quantity_reserved;
                    sl.average_cost = if sl.quantity_on_hand > BigDecimal::from(0) {
                        &total_cost / &sl.quantity_on_hand
                    } else {
                        BigDecimal::from(0)
                    };
                } else {
                    sl.quantity_on_hand -= quantity;
                    sl.quantity_available = &sl.quantity_on_hand - &sl.quantity_reserved;
                }
                if let Some(cost) = unit_cost {
                    sl.last_cost = Some(cost.clone());
                }

                sqlx::query(
                    "UPDATE stock_levels SET quantity_on_hand = $1, quantity_available = $2, average_cost = $3, last_cost = $4, updated_at = NOW() WHERE id = $5",
                )
                .bind(&sl.quantity_on_hand)
                .bind(&sl.quantity_available)
                .bind(&sl.average_cost)
                .bind(&sl.last_cost)
                .bind(sl.id)
                .execute(&self.pool)
                .await?;
            }
            None => {
                let id = Uuid::now_v7();
                let on_hand = if is_increase { quantity.clone() } else { BigDecimal::from(0) };
                sqlx::query(
                    r#"INSERT INTO stock_levels (id, product_id, warehouse_id, quantity_on_hand, quantity_reserved, quantity_available, average_cost, last_cost, updated_at)
                       VALUES ($1, $2, $3, $4, 0, $4, $5, $6, NOW())"#,
                )
                .bind(id)
                .bind(product_id)
                .bind(warehouse_id)
                .bind(&on_hand)
                .bind(unit_cost.as_ref().unwrap_or(&BigDecimal::from(0)))
                .bind(unit_cost)
                .execute(&self.pool)
                .await?;
            }
        }

        Ok(())
    }
}
