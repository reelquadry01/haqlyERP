-- HAQLY ERP - Sales Module
-- Author: Quadri Atharu

CREATE TABLE customers (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    company_id UUID NOT NULL REFERENCES companies(id) ON DELETE CASCADE,
    code VARCHAR(50) NOT NULL,
    name VARCHAR(255) NOT NULL,
    email VARCHAR(255),
    phone VARCHAR(50),
    tax_id VARCHAR(50),
    customer_type VARCHAR(10) NOT NULL DEFAULT 'B2B' CHECK (customer_type IN ('B2B', 'B2C', 'GOVERNMENT')),
    credit_limit NUMERIC(18,2) NOT NULL DEFAULT 0,
    payment_terms INT NOT NULL DEFAULT 30,
    is_active BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE(company_id, code)
);

CREATE TABLE customer_addresses (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    customer_id UUID NOT NULL REFERENCES customers(id) ON DELETE CASCADE,
    line1 VARCHAR(255) NOT NULL,
    line2 VARCHAR(255),
    city VARCHAR(100),
    state VARCHAR(100),
    country_code VARCHAR(3) NOT NULL DEFAULT 'NG',
    postal_code VARCHAR(20),
    is_default BOOLEAN NOT NULL DEFAULT false
);

CREATE TABLE sales_invoices (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    company_id UUID NOT NULL REFERENCES companies(id) ON DELETE CASCADE,
    branch_id UUID REFERENCES branches(id) ON DELETE SET NULL,
    customer_id UUID NOT NULL REFERENCES customers(id),
    number VARCHAR(50) NOT NULL,
    date DATE NOT NULL,
    due_date DATE,
    invoice_type VARCHAR(20) NOT NULL DEFAULT 'STANDARD' CHECK (invoice_type IN ('STANDARD', 'CREDIT_NOTE', 'DEBIT_NOTE', 'PROFORMA')),
    status VARCHAR(20) NOT NULL DEFAULT 'DRAFT' CHECK (status IN ('DRAFT', 'SUBMITTED', 'APPROVED', 'POSTED', 'PARTIALLY_PAID', 'PAID', 'CANCELLED', 'VOID')),
    currency_code VARCHAR(3) NOT NULL DEFAULT 'NGN',
    exchange_rate NUMERIC(18,6) NOT NULL DEFAULT 1,
    taxable_amount NUMERIC(18,2) NOT NULL DEFAULT 0,
    tax_amount NUMERIC(18,2) NOT NULL DEFAULT 0,
    total_amount NUMERIC(18,2) NOT NULL DEFAULT 0,
    amount_paid NUMERIC(18,2) NOT NULL DEFAULT 0,
    narration TEXT,
    is_einvoice_eligible BOOLEAN NOT NULL DEFAULT false,
    einvoice_irn VARCHAR(100),
    created_by UUID NOT NULL REFERENCES users(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE(company_id, number)
);

CREATE TABLE sales_invoice_items (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    sales_invoice_id UUID NOT NULL REFERENCES sales_invoices(id) ON DELETE CASCADE,
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

CREATE TABLE customer_receipts (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    company_id UUID NOT NULL REFERENCES companies(id) ON DELETE CASCADE,
    customer_id UUID NOT NULL REFERENCES customers(id),
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

CREATE TABLE proforma_invoices (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    company_id UUID NOT NULL REFERENCES companies(id) ON DELETE CASCADE,
    customer_id UUID NOT NULL REFERENCES customers(id),
    number VARCHAR(50) NOT NULL,
    date DATE NOT NULL,
    valid_until DATE,
    status VARCHAR(20) NOT NULL DEFAULT 'DRAFT',
    total_amount NUMERIC(18,2) NOT NULL DEFAULT 0,
    converted_to_invoice BOOLEAN NOT NULL DEFAULT false,
    sales_invoice_id UUID REFERENCES sales_invoices(id),
    created_by UUID NOT NULL REFERENCES users(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE(company_id, number)
);

CREATE INDEX idx_customers_company ON customers(company_id);
CREATE INDEX idx_sales_invoices_company ON sales_invoices(company_id);
CREATE INDEX idx_sales_invoices_customer ON sales_invoices(customer_id);
CREATE INDEX idx_sales_invoices_status ON sales_invoices(company_id, status);
CREATE INDEX idx_sales_invoice_items_invoice ON sales_invoice_items(sales_invoice_id);
CREATE INDEX idx_customer_receipts_company ON customer_receipts(company_id);
