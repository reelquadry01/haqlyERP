-- HAQLY ERP - Inventory Module
-- Author: Quadri Atharu

CREATE TABLE products (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    company_id UUID NOT NULL REFERENCES companies(id) ON DELETE CASCADE,
    sku VARCHAR(100) NOT NULL,
    name VARCHAR(255) NOT NULL,
    description TEXT,
    category VARCHAR(100),
    product_type VARCHAR(20) NOT NULL DEFAULT 'FINISHED' CHECK (product_type IN ('FINISHED', 'RAW_MATERIAL', 'SEMI_FINISHED', 'SERVICE', 'CONSUMABLE')),
    unit_of_measure VARCHAR(20) NOT NULL DEFAULT 'PCS',
    cost_price NUMERIC(18,2) NOT NULL DEFAULT 0,
    selling_price NUMERIC(18,2) NOT NULL DEFAULT 0,
    tax_code VARCHAR(50),
    is_active BOOLEAN NOT NULL DEFAULT true,
    track_inventory BOOLEAN NOT NULL DEFAULT true,
    reorder_point NUMERIC(18,2) NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE(company_id, sku)
);

CREATE TABLE warehouses (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    company_id UUID NOT NULL REFERENCES companies(id) ON DELETE CASCADE,
    branch_id UUID REFERENCES branches(id) ON DELETE SET NULL,
    code VARCHAR(50) NOT NULL,
    name VARCHAR(255) NOT NULL,
    location TEXT,
    is_active BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE(company_id, code)
);

CREATE TABLE inventory_stock_levels (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    product_id UUID NOT NULL REFERENCES products(id) ON DELETE CASCADE,
    warehouse_id UUID NOT NULL REFERENCES warehouses(id) ON DELETE CASCADE,
    quantity_on_hand NUMERIC(18,2) NOT NULL DEFAULT 0,
    quantity_reserved NUMERIC(18,2) NOT NULL DEFAULT 0,
    quantity_available NUMERIC(18,2) NOT NULL DEFAULT 0,
    average_cost NUMERIC(18,4) NOT NULL DEFAULT 0,
    last_cost NUMERIC(18,4) NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE(product_id, warehouse_id)
);

CREATE TABLE stock_movements (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    company_id UUID NOT NULL REFERENCES companies(id) ON DELETE CASCADE,
    product_id UUID NOT NULL REFERENCES products(id),
    warehouse_id UUID NOT NULL REFERENCES warehouses(id),
    movement_type VARCHAR(20) NOT NULL CHECK (movement_type IN ('RECEIPT', 'ISSUE', 'TRANSFER', 'ADJUSTMENT', 'RETURN')),
    quantity NUMERIC(18,2) NOT NULL,
    unit_cost NUMERIC(18,4) NOT NULL DEFAULT 0,
    reference_type VARCHAR(50),
    reference_id UUID,
    narration TEXT,
    date DATE NOT NULL,
    created_by UUID NOT NULL REFERENCES users(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_products_company ON products(company_id);
CREATE INDEX idx_warehouses_company ON warehouses(company_id);
CREATE INDEX idx_stock_levels_product ON inventory_stock_levels(product_id);
CREATE INDEX idx_stock_levels_warehouse ON inventory_stock_levels(warehouse_id);
CREATE INDEX idx_stock_movements_product ON stock_movements(product_id);
CREATE INDEX idx_stock_movements_date ON stock_movements(date);
