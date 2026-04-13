-- HAQLY ERP - Purchases Module
-- Author: Quadri Atharu

CREATE TABLE suppliers (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    company_id UUID NOT NULL REFERENCES companies(id) ON DELETE CASCADE,
    code VARCHAR(50) NOT NULL,
    name VARCHAR(255) NOT NULL,
    email VARCHAR(255),
    phone VARCHAR(50),
    tax_id VARCHAR(50),
    payment_terms INT NOT NULL DEFAULT 30,
    is_active BOOLEAN NOT NULL DEFAULT true,
    bank_name VARCHAR(255),
    bank_account_number VARCHAR(50),
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE(company_id, code)
);

CREATE TABLE supplier_addresses (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    supplier_id UUID NOT NULL REFERENCES suppliers(id) ON DELETE CASCADE,
    line1 VARCHAR(255) NOT NULL,
    line2 VARCHAR(255),
    city VARCHAR(100),
    state VARCHAR(100),
    country_code VARCHAR(3) NOT NULL DEFAULT 'NG',
    postal_code VARCHAR(20),
    is_default BOOLEAN NOT NULL DEFAULT false
);

CREATE TABLE purchase_bills (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    company_id UUID NOT NULL REFERENCES companies(id) ON DELETE CASCADE,
    branch_id UUID REFERENCES branches(id) ON DELETE SET NULL,
    supplier_id UUID NOT NULL REFERENCES suppliers(id),
    number VARCHAR(50) NOT NULL,
    date DATE NOT NULL,
    due_date DATE,
    status VARCHAR(20) NOT NULL DEFAULT 'DRAFT' CHECK (status IN ('DRAFT', 'SUBMITTED', 'APPROVED', 'POSTED', 'PARTIALLY_PAID', 'PAID', 'CANCELLED')),
    currency_code VARCHAR(3) NOT NULL DEFAULT 'NGN',
    exchange_rate NUMERIC(18,6) NOT NULL DEFAULT 1,
    taxable_amount NUMERIC(18,2) NOT NULL DEFAULT 0,
    tax_amount NUMERIC(18,2) NOT NULL DEFAULT 0,
    total_amount NUMERIC(18,2) NOT NULL DEFAULT 0,
    amount_paid NUMERIC(18,2) NOT NULL DEFAULT 0,
    narration TEXT,
    created_by UUID NOT NULL REFERENCES users(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE(company_id, number)
);

CREATE TABLE purchase_bill_items (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    purchase_bill_id UUID NOT NULL REFERENCES purchase_bills(id) ON DELETE CASCADE,
    line_number INT NOT NULL,
    product_id UUID,
    sku VARCHAR(100),
    description TEXT NOT NULL,
    quantity NUMERIC(18,4) NOT NULL,
    unit_price NUMERIC(18,4) NOT NULL,
    discount_percent NUMERIC(5,2) NOT NULL DEFAULT 0,
    tax_rate NUMERIC(5,2),
    taxable_amount NUMERIC(18,2) NOT NULL,
    tax_amount NUMERIC(18,2) NOT NULL DEFAULT 0,
    line_amount NUMERIC(18,2) NOT NULL,
    cost_center_id UUID REFERENCES cost_centers(id) ON DELETE SET NULL,
    project_id UUID REFERENCES projects(id) ON DELETE SET NULL
);

CREATE TABLE supplier_payments (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    company_id UUID NOT NULL REFERENCES companies(id) ON DELETE CASCADE,
    supplier_id UUID NOT NULL REFERENCES suppliers(id),
    purchase_bill_id UUID REFERENCES purchase_bills(id),
    number VARCHAR(50) NOT NULL,
    date DATE NOT NULL,
    amount NUMERIC(18,2) NOT NULL,
    payment_method VARCHAR(30) NOT NULL CHECK (payment_method IN ('CASH', 'BANK_TRANSFER', 'CHEQUE', 'CARD', 'POS', 'MOBILE_MONEY')),
    bank_account_id UUID,
    reference VARCHAR(100),
    narration TEXT,
    status VARCHAR(20) NOT NULL DEFAULT 'DRAFT' CHECK (status IN ('DRAFT', 'SUBMITTED', 'APPROVED', 'POSTED', 'CANCELLED')),
    posted_to_gl BOOLEAN NOT NULL DEFAULT false,
    gl_journal_id UUID REFERENCES journal_headers(id),
    created_by UUID NOT NULL REFERENCES users(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE(company_id, number)
);

CREATE INDEX idx_suppliers_company ON suppliers(company_id);
CREATE INDEX idx_purchase_bills_company ON purchase_bills(company_id);
CREATE INDEX idx_purchase_bills_supplier ON purchase_bills(supplier_id);
CREATE INDEX idx_supplier_payments_company ON supplier_payments(company_id);
