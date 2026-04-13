-- HAQLY ERP - Nigerian Tax Module
-- Author: Quadri Atharu

CREATE TABLE tax_configs (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    company_id UUID NOT NULL REFERENCES companies(id) ON DELETE CASCADE,
    tax_type VARCHAR(20) NOT NULL CHECK (tax_type IN ('VAT', 'WHT', 'CIT', 'EDU_TAX', 'CGT', 'STAMP_DUTY', 'PAYE')),
    name VARCHAR(100) NOT NULL,
    rate NUMERIC(5,2) NOT NULL,
    is_active BOOLEAN NOT NULL DEFAULT true,
    effective_from DATE NOT NULL,
    effective_to DATE,
    description TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE(company_id, tax_type, name)
);

CREATE TABLE tax_transactions (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    company_id UUID NOT NULL REFERENCES companies(id) ON DELETE CASCADE,
    tax_config_id UUID NOT NULL REFERENCES tax_configs(id),
    transaction_type VARCHAR(10) NOT NULL CHECK (transaction_type IN ('OUTPUT', 'INPUT')),
    source_module VARCHAR(50) NOT NULL,
    source_document_id UUID NOT NULL,
    source_document_number VARCHAR(100),
    taxable_amount NUMERIC(18,2) NOT NULL,
    tax_amount NUMERIC(18,2) NOT NULL,
    tax_rate NUMERIC(5,2) NOT NULL,
    posting_date DATE NOT NULL,
    is_reported BOOLEAN NOT NULL DEFAULT false,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- WHT rate categories for Nigeria
CREATE TABLE wht_rate_categories (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    company_id UUID NOT NULL REFERENCES companies(id) ON DELETE CASCADE,
    category_name VARCHAR(255) NOT NULL,
    rate NUMERIC(5,2) NOT NULL,
    is_active BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Capital allowance schedules
CREATE TABLE capital_allowance_categories (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    company_id UUID NOT NULL REFERENCES companies(id) ON DELETE CASCADE,
    name VARCHAR(255) NOT NULL,
    initial_rate NUMERIC(5,2) NOT NULL,
    annual_rate NUMERIC(5,2) NOT NULL,
    is_active BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_tax_configs_company ON tax_configs(company_id, tax_type);
CREATE INDEX idx_tax_transactions_company ON tax_transactions(company_id);
CREATE INDEX idx_tax_transactions_date ON tax_transactions(posting_date);
CREATE INDEX idx_tax_transactions_reported ON tax_transactions(company_id, is_reported);
