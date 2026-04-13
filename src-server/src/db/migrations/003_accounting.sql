-- HAQLY ERP - Accounting Foundation
-- Author: Quadri Atharu

CREATE TABLE chart_of_accounts (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    company_id UUID NOT NULL REFERENCES companies(id) ON DELETE CASCADE,
    code VARCHAR(50) NOT NULL,
    name VARCHAR(255) NOT NULL,
    account_type VARCHAR(20) NOT NULL CHECK (account_type IN ('Asset', 'Liability', 'Equity', 'Revenue', 'Expense')),
    sub_type VARCHAR(50),
    parent_id UUID REFERENCES chart_of_accounts(id) ON DELETE SET NULL,
    is_control_account BOOLEAN NOT NULL DEFAULT false,
    is_active BOOLEAN NOT NULL DEFAULT true,
    allowed_posting BOOLEAN NOT NULL DEFAULT true,
    currency_code VARCHAR(3) NOT NULL DEFAULT 'NGN',
    balance NUMERIC(18,2) NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE(company_id, code)
);

CREATE TABLE fiscal_years (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    company_id UUID NOT NULL REFERENCES companies(id) ON DELETE CASCADE,
    name VARCHAR(100) NOT NULL,
    start_date DATE NOT NULL,
    end_date DATE NOT NULL,
    is_closed BOOLEAN NOT NULL DEFAULT false,
    closed_by UUID REFERENCES users(id),
    closed_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE(company_id, name)
);

CREATE TABLE accounting_periods (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    fiscal_year_id UUID NOT NULL REFERENCES fiscal_years(id) ON DELETE CASCADE,
    name VARCHAR(100) NOT NULL,
    start_date DATE NOT NULL,
    end_date DATE NOT NULL,
    period_number INT NOT NULL,
    status VARCHAR(20) NOT NULL DEFAULT 'OPEN' CHECK (status IN ('OPEN', 'CLOSED', 'LOCKED')),
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE journal_headers (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    company_id UUID NOT NULL REFERENCES companies(id) ON DELETE CASCADE,
    branch_id UUID REFERENCES branches(id) ON DELETE SET NULL,
    department_id UUID REFERENCES departments(id) ON DELETE SET NULL,
    fiscal_year_id UUID NOT NULL REFERENCES fiscal_years(id),
    period_id UUID NOT NULL REFERENCES accounting_periods(id),
    number VARCHAR(50) NOT NULL,
    date DATE NOT NULL,
    narration TEXT,
    source_module VARCHAR(50),
    source_type VARCHAR(50),
    source_document_id UUID,
    reference_id VARCHAR(100),
    currency_code VARCHAR(3) NOT NULL DEFAULT 'NGN',
    exchange_rate NUMERIC(18,6) NOT NULL DEFAULT 1,
    status VARCHAR(20) NOT NULL DEFAULT 'DRAFT' CHECK (status IN ('DRAFT', 'VALIDATED', 'SUBMITTED', 'APPROVED', 'POSTED', 'REVERSED', 'CANCELLED')),
    total_debit NUMERIC(18,2) NOT NULL DEFAULT 0,
    total_credit NUMERIC(18,2) NOT NULL DEFAULT 0,
    is_balanced BOOLEAN NOT NULL DEFAULT false,
    created_by UUID NOT NULL REFERENCES users(id),
    submitted_by UUID REFERENCES users(id),
    approved_by UUID REFERENCES users(id),
    posted_by UUID REFERENCES users(id),
    submitted_at TIMESTAMPTZ,
    approved_at TIMESTAMPTZ,
    posted_at TIMESTAMPTZ,
    reversal_of UUID REFERENCES journal_headers(id),
    reversal_reason TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE(company_id, number)
);

CREATE TABLE journal_lines (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    journal_header_id UUID NOT NULL REFERENCES journal_headers(id) ON DELETE CASCADE,
    line_number INT NOT NULL,
    account_id UUID NOT NULL REFERENCES chart_of_accounts(id),
    debit NUMERIC(18,2) NOT NULL DEFAULT 0,
    credit NUMERIC(18,2) NOT NULL DEFAULT 0,
    narration TEXT,
    cost_center_id UUID REFERENCES cost_centers(id) ON DELETE SET NULL,
    project_id UUID REFERENCES projects(id) ON DELETE SET NULL,
    subledger_party VARCHAR(255),
    tax_code VARCHAR(50),
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE posting_rules (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    company_id UUID NOT NULL REFERENCES companies(id) ON DELETE CASCADE,
    module VARCHAR(50) NOT NULL,
    transaction_type VARCHAR(50) NOT NULL,
    transaction_subtype VARCHAR(50),
    legal_entity_id UUID REFERENCES companies(id),
    branch_id UUID REFERENCES branches(id),
    department_id UUID REFERENCES departments(id),
    product_category VARCHAR(100),
    customer_group VARCHAR(100),
    vendor_group VARCHAR(100),
    tax_code VARCHAR(50),
    currency_code VARCHAR(3),
    condition_expression TEXT,
    debit_account_id UUID NOT NULL REFERENCES chart_of_accounts(id),
    credit_account_id UUID NOT NULL REFERENCES chart_of_accounts(id),
    tax_account_id UUID REFERENCES chart_of_accounts(id),
    rounding_account_id UUID REFERENCES chart_of_accounts(id),
    exchange_gain_account_id UUID REFERENCES chart_of_accounts(id),
    exchange_loss_account_id UUID REFERENCES chart_of_accounts(id),
    suspense_account_id UUID REFERENCES chart_of_accounts(id),
    posting_description_template TEXT,
    require_branch BOOLEAN NOT NULL DEFAULT false,
    require_department BOOLEAN NOT NULL DEFAULT false,
    require_cost_center BOOLEAN NOT NULL DEFAULT false,
    require_project BOOLEAN NOT NULL DEFAULT false,
    require_subledger BOOLEAN NOT NULL DEFAULT false,
    require_tax BOOLEAN NOT NULL DEFAULT false,
    effective_from DATE NOT NULL,
    effective_to DATE,
    is_active BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE posting_audit (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    source_module VARCHAR(50) NOT NULL,
    source_table VARCHAR(100) NOT NULL,
    source_document_id UUID NOT NULL,
    source_document_number VARCHAR(100),
    reference_id VARCHAR(100),
    customer_or_vendor VARCHAR(255),
    triggering_event VARCHAR(50) NOT NULL,
    posting_rule_id UUID REFERENCES posting_rules(id),
    user_id UUID REFERENCES users(id),
    approval_reference VARCHAR(100),
    posting_timestamp TIMESTAMPTZ NOT NULL DEFAULT now(),
    period_id UUID REFERENCES accounting_periods(id),
    branch_id UUID REFERENCES branches(id),
    legal_entity_id UUID REFERENCES companies(id),
    department_id UUID REFERENCES departments(id),
    cost_center_id UUID REFERENCES cost_centers(id),
    project_id UUID REFERENCES projects(id),
    tax_code VARCHAR(50),
    currency_code VARCHAR(3),
    narration TEXT,
    correlation_id UUID,
    idempotency_key UUID NOT NULL UNIQUE DEFAULT uuid_generate_v4(),
    reversal_of_audit_id UUID REFERENCES posting_audit(id),
    rule_snapshot JSONB,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_coa_company ON chart_of_accounts(company_id);
CREATE INDEX idx_coa_type ON chart_of_accounts(company_id, account_type);
CREATE INDEX idx_journal_headers_company ON journal_headers(company_id);
CREATE INDEX idx_journal_headers_status ON journal_headers(company_id, status);
CREATE INDEX idx_journal_lines_header ON journal_lines(journal_header_id);
CREATE INDEX idx_journal_lines_account ON journal_lines(account_id);
CREATE INDEX idx_posting_audit_source ON posting_audit(source_module, source_table, source_document_id);
CREATE INDEX idx_posting_audit_idempotency ON posting_audit(idempotency_key);
CREATE INDEX idx_posting_rules_module ON posting_rules(company_id, module, transaction_type);
