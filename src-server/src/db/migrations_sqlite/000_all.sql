-- HAQLY ERP - SQLite Combined Migration
-- Author: Quadri Atharu
-- All PostgreSQL tables adapted for SQLite with schema_reconcile changes inlined

-- 001_init: Users & Authentication
CREATE TABLE IF NOT EXISTS users (
    id TEXT PRIMARY KEY,
    email TEXT NOT NULL UNIQUE,
    password_hash TEXT NOT NULL,
    full_name TEXT NOT NULL,
    is_active INTEGER NOT NULL DEFAULT 1,
    mfa_secret TEXT,
    mfa_enabled INTEGER NOT NULL DEFAULT 0,
    last_login_at TEXT,
    company_id TEXT REFERENCES companies(id),
    phone TEXT,
    avatar_url TEXT,
    mfa_recovery_codes TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS sessions (
    id TEXT PRIMARY KEY,
    user_id TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    refresh_token TEXT NOT NULL,
    token TEXT NOT NULL DEFAULT '',
    ip_address TEXT,
    user_agent TEXT,
    expires_at TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE UNIQUE INDEX IF NOT EXISTS idx_sessions_token ON sessions(token);

-- 001_init: RBAC
CREATE TABLE IF NOT EXISTS roles (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL UNIQUE,
    description TEXT
);

CREATE TABLE IF NOT EXISTS permissions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL UNIQUE,
    description TEXT
);

CREATE TABLE IF NOT EXISTS role_permissions (
    role_id INTEGER NOT NULL REFERENCES roles(id) ON DELETE CASCADE,
    permission_id INTEGER NOT NULL REFERENCES permissions(id) ON DELETE CASCADE,
    PRIMARY KEY (role_id, permission_id)
);

CREATE TABLE IF NOT EXISTS user_roles (
    id TEXT PRIMARY KEY,
    user_id TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    role_id INTEGER NOT NULL REFERENCES roles(id) ON DELETE CASCADE,
    assigned_at TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(user_id, role_id)
);

-- 001_init: Audit
CREATE TABLE IF NOT EXISTS audit_logs (
    id TEXT PRIMARY KEY,
    user_id TEXT REFERENCES users(id),
    action TEXT NOT NULL,
    entity TEXT NOT NULL,
    entity_id TEXT,
    details TEXT,
    ip_address TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_audit_logs_user ON audit_logs(user_id);
CREATE INDEX IF NOT EXISTS idx_audit_logs_entity ON audit_logs(entity, entity_id);
CREATE INDEX IF NOT EXISTS idx_audit_logs_created ON audit_logs(created_at);

-- 001_init: Seed roles
INSERT OR IGNORE INTO roles (name, description) VALUES
    ('SuperAdmin', 'Full system access'),
    ('Admin', 'Company administration'),
    ('Accountant', 'Accounting and finance operations'),
    ('Sales', 'Sales module access'),
    ('Purchaser', 'Purchase module access'),
    ('InventoryMgr', 'Inventory management'),
    ('HR', 'Human resources and payroll'),
    ('Treasury', 'Treasury and cash management'),
    ('Viewer', 'Read-only access');

-- 001_init: Seed permissions
INSERT OR IGNORE INTO permissions (name, description) VALUES
    ('users:view', 'View users'),
    ('users:create', 'Create users'),
    ('users:update', 'Update users'),
    ('org:view', 'View organization'),
    ('org:create', 'Create organization entities'),
    ('accounting:coa', 'Chart of accounts management'),
    ('accounting:journal', 'Journal entry management'),
    ('accounting:voucher', 'Payment voucher management'),
    ('sales:view', 'View sales data'),
    ('sales:create', 'Create sales documents'),
    ('purchases:view', 'View purchase data'),
    ('purchases:create', 'Create purchase documents'),
    ('inventory:view', 'View inventory'),
    ('inventory:create', 'Manage inventory'),
    ('fixed_assets:view', 'View fixed assets'),
    ('fixed_assets:create', 'Manage fixed assets'),
    ('loans:view', 'View loans'),
    ('loans:create', 'Manage loans'),
    ('finance:view', 'View financial reports'),
    ('finance:create', 'Generate financial reports'),
    ('admin:roles', 'Manage roles and permissions'),
    ('einvoicing:manage', 'Manage e-invoicing');

-- 002_org: Organization Structure
CREATE TABLE IF NOT EXISTS companies (
    id TEXT PRIMARY KEY,
    code TEXT NOT NULL UNIQUE,
    name TEXT NOT NULL,
    tin TEXT,
    rc_number TEXT,
    address TEXT,
    city TEXT,
    state TEXT,
    country_code TEXT NOT NULL DEFAULT 'NGN',
    currency_code TEXT NOT NULL DEFAULT 'NGN',
    phone TEXT,
    email TEXT,
    website TEXT,
    is_active INTEGER NOT NULL DEFAULT 1,
    fiscal_year_start TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS branches (
    id TEXT PRIMARY KEY,
    company_id TEXT NOT NULL REFERENCES companies(id) ON DELETE CASCADE,
    code TEXT NOT NULL,
    name TEXT NOT NULL,
    address TEXT,
    city TEXT,
    state TEXT,
    is_active INTEGER NOT NULL DEFAULT 1,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(company_id, code)
);

CREATE TABLE IF NOT EXISTS departments (
    id TEXT PRIMARY KEY,
    company_id TEXT NOT NULL REFERENCES companies(id) ON DELETE CASCADE,
    branch_id TEXT REFERENCES branches(id) ON DELETE SET NULL,
    code TEXT NOT NULL,
    name TEXT NOT NULL,
    is_active INTEGER NOT NULL DEFAULT 1,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(company_id, code)
);

CREATE TABLE IF NOT EXISTS cost_centers (
    id TEXT PRIMARY KEY,
    company_id TEXT NOT NULL REFERENCES companies(id) ON DELETE CASCADE,
    code TEXT NOT NULL,
    name TEXT NOT NULL,
    is_active INTEGER NOT NULL DEFAULT 1,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(company_id, code)
);

CREATE TABLE IF NOT EXISTS projects (
    id TEXT PRIMARY KEY,
    company_id TEXT NOT NULL REFERENCES companies(id) ON DELETE CASCADE,
    code TEXT NOT NULL,
    name TEXT NOT NULL,
    start_date TEXT NOT NULL,
    end_date TEXT,
    is_active INTEGER NOT NULL DEFAULT 1,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(company_id, code)
);

CREATE TABLE IF NOT EXISTS company_settings (
    id TEXT PRIMARY KEY,
    company_id TEXT NOT NULL UNIQUE REFERENCES companies(id) ON DELETE CASCADE,
    default_currency TEXT NOT NULL DEFAULT 'NGN',
    tax_id_number TEXT,
    einvoicing_enabled INTEGER NOT NULL DEFAULT 0,
    approval_rules TEXT NOT NULL DEFAULT '{}',
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS bank_accounts (
    id TEXT PRIMARY KEY,
    company_id TEXT NOT NULL REFERENCES companies(id) ON DELETE CASCADE,
    branch_id TEXT REFERENCES branches(id) ON DELETE SET NULL,
    bank_name TEXT NOT NULL,
    account_name TEXT NOT NULL,
    account_number TEXT NOT NULL,
    currency_code TEXT NOT NULL DEFAULT 'NGN',
    is_active INTEGER NOT NULL DEFAULT 1,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_branches_company ON branches(company_id);
CREATE INDEX IF NOT EXISTS idx_departments_company ON departments(company_id);
CREATE INDEX IF NOT EXISTS idx_cost_centers_company ON cost_centers(company_id);
CREATE INDEX IF NOT EXISTS idx_projects_company ON projects(company_id);

-- 003_accounting: Accounting Foundation
CREATE TABLE IF NOT EXISTS chart_of_accounts (
    id TEXT PRIMARY KEY,
    company_id TEXT NOT NULL REFERENCES companies(id) ON DELETE CASCADE,
    code TEXT NOT NULL,
    name TEXT NOT NULL,
    account_type TEXT NOT NULL CHECK (account_type IN ('Asset', 'Liability', 'Equity', 'Revenue', 'Expense')),
    sub_type TEXT,
    parent_id TEXT REFERENCES chart_of_accounts(id) ON DELETE SET NULL,
    is_control_account INTEGER NOT NULL DEFAULT 0,
    is_active INTEGER NOT NULL DEFAULT 1,
    allowed_posting INTEGER NOT NULL DEFAULT 1,
    currency_code TEXT NOT NULL DEFAULT 'NGN',
    balance REAL NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(company_id, code)
);

CREATE TABLE IF NOT EXISTS fiscal_years (
    id TEXT PRIMARY KEY,
    company_id TEXT NOT NULL REFERENCES companies(id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    start_date TEXT NOT NULL,
    end_date TEXT NOT NULL,
    is_closed INTEGER NOT NULL DEFAULT 0,
    closed_by TEXT REFERENCES users(id),
    closed_at TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(company_id, name)
);

CREATE TABLE IF NOT EXISTS accounting_periods (
    id TEXT PRIMARY KEY,
    company_id TEXT REFERENCES companies(id),
    fiscal_year_id TEXT NOT NULL REFERENCES fiscal_years(id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    start_date TEXT NOT NULL,
    end_date TEXT NOT NULL,
    period_number INTEGER NOT NULL,
    status TEXT NOT NULL DEFAULT 'open' CHECK (status IN ('open', 'closed', 'locked')),
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS journal_headers (
    id TEXT PRIMARY KEY,
    company_id TEXT NOT NULL REFERENCES companies(id) ON DELETE CASCADE,
    branch_id TEXT REFERENCES branches(id) ON DELETE SET NULL,
    department_id TEXT REFERENCES departments(id) ON DELETE SET NULL,
    fiscal_year_id TEXT NOT NULL REFERENCES fiscal_years(id),
    period_id TEXT NOT NULL REFERENCES accounting_periods(id),
    entry_number TEXT NOT NULL,
    date TEXT NOT NULL,
    narration TEXT,
    journal_type TEXT,
    source_module TEXT,
    source_type TEXT,
    source_document_id TEXT,
    source_document_number TEXT,
    reference TEXT,
    currency_code TEXT NOT NULL DEFAULT 'NGN',
    exchange_rate REAL NOT NULL DEFAULT 1,
    status TEXT NOT NULL DEFAULT 'draft' CHECK (status IN ('draft', 'validated', 'submitted', 'approved', 'posted', 'reversed', 'cancelled')),
    total_debit REAL NOT NULL DEFAULT 0,
    total_credit REAL NOT NULL DEFAULT 0,
    is_balanced INTEGER NOT NULL DEFAULT 0,
    created_by TEXT NOT NULL REFERENCES users(id),
    submitted_by TEXT REFERENCES users(id),
    approved_by TEXT REFERENCES users(id),
    posted_by TEXT REFERENCES users(id),
    submitted_at TEXT,
    approved_at TEXT,
    posted_at TEXT,
    reversal_of TEXT REFERENCES journal_headers(id),
    reversal_reason TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(company_id, entry_number)
);

CREATE TABLE IF NOT EXISTS journal_lines (
    id TEXT PRIMARY KEY,
    journal_header_id TEXT NOT NULL REFERENCES journal_headers(id) ON DELETE CASCADE,
    line_number INTEGER NOT NULL,
    account_id TEXT NOT NULL REFERENCES chart_of_accounts(id),
    debit REAL NOT NULL DEFAULT 0,
    credit REAL NOT NULL DEFAULT 0,
    narration TEXT,
    cost_center_id TEXT REFERENCES cost_centers(id) ON DELETE SET NULL,
    project_id TEXT REFERENCES projects(id) ON DELETE SET NULL,
    department_id TEXT REFERENCES departments(id) ON DELETE SET NULL,
    subledger_party TEXT,
    tax_code TEXT,
    currency_code TEXT NOT NULL DEFAULT 'NGN',
    exchange_rate REAL NOT NULL DEFAULT 1,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS posting_rules (
    id TEXT PRIMARY KEY,
    company_id TEXT NOT NULL REFERENCES companies(id) ON DELETE CASCADE,
    module TEXT NOT NULL,
    transaction_type TEXT NOT NULL,
    transaction_subtype TEXT,
    legal_entity_id TEXT REFERENCES companies(id),
    branch_id TEXT REFERENCES branches(id),
    department_id TEXT REFERENCES departments(id),
    product_category TEXT,
    customer_group TEXT,
    vendor_group TEXT,
    tax_code TEXT,
    currency_code TEXT,
    condition_expression TEXT,
    debit_account_id TEXT NOT NULL REFERENCES chart_of_accounts(id),
    credit_account_id TEXT NOT NULL REFERENCES chart_of_accounts(id),
    tax_account_id TEXT REFERENCES chart_of_accounts(id),
    rounding_account_id TEXT REFERENCES chart_of_accounts(id),
    exchange_gain_account_id TEXT REFERENCES chart_of_accounts(id),
    exchange_loss_account_id TEXT REFERENCES chart_of_accounts(id),
    suspense_account_id TEXT REFERENCES chart_of_accounts(id),
    posting_description_template TEXT,
    require_branch INTEGER NOT NULL DEFAULT 0,
    require_department INTEGER NOT NULL DEFAULT 0,
    require_cost_center INTEGER NOT NULL DEFAULT 0,
    require_project INTEGER NOT NULL DEFAULT 0,
    require_subledger INTEGER NOT NULL DEFAULT 0,
    require_tax INTEGER NOT NULL DEFAULT 0,
    effective_from TEXT NOT NULL,
    effective_to TEXT,
    is_active INTEGER NOT NULL DEFAULT 1,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS posting_audits (
    id TEXT PRIMARY KEY,
    company_id TEXT REFERENCES companies(id),
    source_module TEXT NOT NULL,
    source_table TEXT NOT NULL,
    source_document_id TEXT NOT NULL,
    source_document_number TEXT,
    reference TEXT,
    customer_or_vendor TEXT,
    triggering_event TEXT NOT NULL,
    posting_rule_id TEXT REFERENCES posting_rules(id),
    user_id TEXT REFERENCES users(id),
    approval_reference TEXT,
    posting_timestamp TEXT NOT NULL DEFAULT (datetime('now')),
    period_id TEXT REFERENCES accounting_periods(id),
    branch_id TEXT REFERENCES branches(id),
    legal_entity_id TEXT REFERENCES companies(id),
    department_id TEXT REFERENCES departments(id),
    cost_center_id TEXT REFERENCES cost_centers(id),
    project_id TEXT REFERENCES projects(id),
    tax_code TEXT,
    currency_code TEXT,
    narration TEXT,
    correlation_id TEXT,
    idempotency_key TEXT NOT NULL UNIQUE,
    reversal_of_audit_id TEXT REFERENCES posting_audits(id),
    rule_snapshot TEXT,
    journal_header_id TEXT REFERENCES journal_headers(id),
    status TEXT NOT NULL DEFAULT 'success',
    posted_by TEXT REFERENCES users(id),
    posted_at TEXT NOT NULL DEFAULT (datetime('now')),
    error_message TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_coa_company ON chart_of_accounts(company_id);
CREATE INDEX IF NOT EXISTS idx_coa_type ON chart_of_accounts(company_id, account_type);
CREATE INDEX IF NOT EXISTS idx_journal_headers_company ON journal_headers(company_id);
CREATE INDEX IF NOT EXISTS idx_journal_headers_status ON journal_headers(company_id, status);
CREATE INDEX IF NOT EXISTS idx_journal_lines_header ON journal_lines(journal_header_id);
CREATE INDEX IF NOT EXISTS idx_journal_lines_account ON journal_lines(account_id);
CREATE INDEX IF NOT EXISTS idx_posting_audit_source ON posting_audits(source_module, source_table, source_document_id);
CREATE INDEX IF NOT EXISTS idx_posting_audit_idempotency ON posting_audits(idempotency_key);
CREATE INDEX IF NOT EXISTS idx_posting_rules_module ON posting_rules(company_id, module, transaction_type);
CREATE INDEX IF NOT EXISTS idx_accounting_periods_company_date ON accounting_periods(company_id, start_date, end_date);
CREATE INDEX IF NOT EXISTS idx_posting_rules_lookup ON posting_rules(company_id, module, transaction_type, is_active);

-- 004_sales: Sales Module
CREATE TABLE IF NOT EXISTS customers (
    id TEXT PRIMARY KEY,
    company_id TEXT NOT NULL REFERENCES companies(id) ON DELETE CASCADE,
    code TEXT NOT NULL,
    name TEXT NOT NULL,
    email TEXT,
    phone TEXT,
    tax_id TEXT,
    customer_type TEXT NOT NULL DEFAULT 'B2B' CHECK (customer_type IN ('B2B', 'B2C', 'GOVERNMENT')),
    credit_limit REAL NOT NULL DEFAULT 0,
    payment_terms INTEGER NOT NULL DEFAULT 30,
    is_active INTEGER NOT NULL DEFAULT 1,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(company_id, code)
);

CREATE TABLE IF NOT EXISTS customer_addresses (
    id TEXT PRIMARY KEY,
    customer_id TEXT NOT NULL REFERENCES customers(id) ON DELETE CASCADE,
    line1 TEXT NOT NULL,
    line2 TEXT,
    city TEXT,
    state TEXT,
    country_code TEXT NOT NULL DEFAULT 'NG',
    postal_code TEXT,
    is_default INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE IF NOT EXISTS sales_invoices (
    id TEXT PRIMARY KEY,
    company_id TEXT NOT NULL REFERENCES companies(id) ON DELETE CASCADE,
    branch_id TEXT REFERENCES branches(id) ON DELETE SET NULL,
    customer_id TEXT NOT NULL REFERENCES customers(id),
    number TEXT NOT NULL,
    date TEXT NOT NULL,
    due_date TEXT,
    invoice_type TEXT NOT NULL DEFAULT 'STANDARD' CHECK (invoice_type IN ('STANDARD', 'CREDIT_NOTE', 'DEBIT_NOTE', 'PROFORMA')),
    status TEXT NOT NULL DEFAULT 'DRAFT' CHECK (status IN ('DRAFT', 'SUBMITTED', 'APPROVED', 'POSTED', 'PARTIALLY_PAID', 'PAID', 'CANCELLED', 'VOID')),
    currency_code TEXT NOT NULL DEFAULT 'NGN',
    exchange_rate REAL NOT NULL DEFAULT 1,
    taxable_amount REAL NOT NULL DEFAULT 0,
    tax_amount REAL NOT NULL DEFAULT 0,
    total_amount REAL NOT NULL DEFAULT 0,
    amount_paid REAL NOT NULL DEFAULT 0,
    narration TEXT,
    is_einvoice_eligible INTEGER NOT NULL DEFAULT 0,
    einvoice_irn TEXT,
    created_by TEXT NOT NULL REFERENCES users(id),
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(company_id, number)
);

CREATE TABLE IF NOT EXISTS sales_invoice_items (
    id TEXT PRIMARY KEY,
    sales_invoice_id TEXT NOT NULL REFERENCES sales_invoices(id) ON DELETE CASCADE,
    line_number INTEGER NOT NULL,
    product_id TEXT,
    sku TEXT,
    description TEXT NOT NULL,
    quantity REAL NOT NULL,
    unit_price REAL NOT NULL,
    discount_percent REAL NOT NULL DEFAULT 0,
    tax_rate REAL,
    taxable_amount REAL NOT NULL,
    tax_amount REAL NOT NULL DEFAULT 0,
    line_amount REAL NOT NULL,
    cost_center_id TEXT REFERENCES cost_centers(id) ON DELETE SET NULL,
    project_id TEXT REFERENCES projects(id) ON DELETE SET NULL
);

CREATE TABLE IF NOT EXISTS customer_receipts (
    id TEXT PRIMARY KEY,
    company_id TEXT NOT NULL REFERENCES companies(id) ON DELETE CASCADE,
    customer_id TEXT NOT NULL REFERENCES customers(id),
    number TEXT NOT NULL,
    date TEXT NOT NULL,
    amount REAL NOT NULL,
    payment_method TEXT NOT NULL CHECK (payment_method IN ('CASH', 'BANK_TRANSFER', 'CHEQUE', 'CARD', 'POS', 'MOBILE_MONEY')),
    bank_account_id TEXT,
    reference TEXT,
    narration TEXT,
    status TEXT NOT NULL DEFAULT 'DRAFT' CHECK (status IN ('DRAFT', 'SUBMITTED', 'APPROVED', 'POSTED', 'CANCELLED')),
    posted_to_gl INTEGER NOT NULL DEFAULT 0,
    gl_journal_id TEXT REFERENCES journal_headers(id),
    created_by TEXT NOT NULL REFERENCES users(id),
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(company_id, number)
);

CREATE TABLE IF NOT EXISTS proforma_invoices (
    id TEXT PRIMARY KEY,
    company_id TEXT NOT NULL REFERENCES companies(id) ON DELETE CASCADE,
    customer_id TEXT NOT NULL REFERENCES customers(id),
    number TEXT NOT NULL,
    date TEXT NOT NULL,
    valid_until TEXT,
    status TEXT NOT NULL DEFAULT 'DRAFT',
    total_amount REAL NOT NULL DEFAULT 0,
    converted_to_invoice INTEGER NOT NULL DEFAULT 0,
    sales_invoice_id TEXT REFERENCES sales_invoices(id),
    created_by TEXT NOT NULL REFERENCES users(id),
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(company_id, number)
);

CREATE INDEX IF NOT EXISTS idx_customers_company ON customers(company_id);
CREATE INDEX IF NOT EXISTS idx_sales_invoices_company ON sales_invoices(company_id);
CREATE INDEX IF NOT EXISTS idx_sales_invoices_customer ON sales_invoices(customer_id);
CREATE INDEX IF NOT EXISTS idx_sales_invoices_status ON sales_invoices(company_id, status);
CREATE INDEX IF NOT EXISTS idx_sales_invoice_items_invoice ON sales_invoice_items(sales_invoice_id);
CREATE INDEX IF NOT EXISTS idx_customer_receipts_company ON customer_receipts(company_id);

-- 005_purchases: Purchases Module
CREATE TABLE IF NOT EXISTS suppliers (
    id TEXT PRIMARY KEY,
    company_id TEXT NOT NULL REFERENCES companies(id) ON DELETE CASCADE,
    code TEXT NOT NULL,
    name TEXT NOT NULL,
    email TEXT,
    phone TEXT,
    tax_id TEXT,
    payment_terms INTEGER NOT NULL DEFAULT 30,
    is_active INTEGER NOT NULL DEFAULT 1,
    bank_name TEXT,
    bank_account_number TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(company_id, code)
);

CREATE TABLE IF NOT EXISTS supplier_addresses (
    id TEXT PRIMARY KEY,
    supplier_id TEXT NOT NULL REFERENCES suppliers(id) ON DELETE CASCADE,
    line1 TEXT NOT NULL,
    line2 TEXT,
    city TEXT,
    state TEXT,
    country_code TEXT NOT NULL DEFAULT 'NG',
    postal_code TEXT,
    is_default INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE IF NOT EXISTS purchase_bills (
    id TEXT PRIMARY KEY,
    company_id TEXT NOT NULL REFERENCES companies(id) ON DELETE CASCADE,
    branch_id TEXT REFERENCES branches(id) ON DELETE SET NULL,
    supplier_id TEXT NOT NULL REFERENCES suppliers(id),
    number TEXT NOT NULL,
    date TEXT NOT NULL,
    due_date TEXT,
    status TEXT NOT NULL DEFAULT 'DRAFT' CHECK (status IN ('DRAFT', 'SUBMITTED', 'APPROVED', 'POSTED', 'PARTIALLY_PAID', 'PAID', 'CANCELLED')),
    currency_code TEXT NOT NULL DEFAULT 'NGN',
    exchange_rate REAL NOT NULL DEFAULT 1,
    taxable_amount REAL NOT NULL DEFAULT 0,
    tax_amount REAL NOT NULL DEFAULT 0,
    total_amount REAL NOT NULL DEFAULT 0,
    amount_paid REAL NOT NULL DEFAULT 0,
    narration TEXT,
    created_by TEXT NOT NULL REFERENCES users(id),
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(company_id, number)
);

CREATE TABLE IF NOT EXISTS purchase_bill_items (
    id TEXT PRIMARY KEY,
    purchase_bill_id TEXT NOT NULL REFERENCES purchase_bills(id) ON DELETE CASCADE,
    line_number INTEGER NOT NULL,
    product_id TEXT,
    sku TEXT,
    description TEXT NOT NULL,
    quantity REAL NOT NULL,
    unit_price REAL NOT NULL,
    discount_percent REAL NOT NULL DEFAULT 0,
    tax_rate REAL,
    taxable_amount REAL NOT NULL,
    tax_amount REAL NOT NULL DEFAULT 0,
    line_amount REAL NOT NULL,
    cost_center_id TEXT REFERENCES cost_centers(id) ON DELETE SET NULL,
    project_id TEXT REFERENCES projects(id) ON DELETE SET NULL
);

CREATE TABLE IF NOT EXISTS supplier_payments (
    id TEXT PRIMARY KEY,
    company_id TEXT NOT NULL REFERENCES companies(id) ON DELETE CASCADE,
    supplier_id TEXT NOT NULL REFERENCES suppliers(id),
    purchase_bill_id TEXT REFERENCES purchase_bills(id),
    number TEXT NOT NULL,
    date TEXT NOT NULL,
    amount REAL NOT NULL,
    payment_method TEXT NOT NULL CHECK (payment_method IN ('CASH', 'BANK_TRANSFER', 'CHEQUE', 'CARD', 'POS', 'MOBILE_MONEY')),
    bank_account_id TEXT,
    reference TEXT,
    narration TEXT,
    status TEXT NOT NULL DEFAULT 'DRAFT' CHECK (status IN ('DRAFT', 'SUBMITTED', 'APPROVED', 'POSTED', 'CANCELLED')),
    posted_to_gl INTEGER NOT NULL DEFAULT 0,
    gl_journal_id TEXT REFERENCES journal_headers(id),
    created_by TEXT NOT NULL REFERENCES users(id),
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(company_id, number)
);

CREATE INDEX IF NOT EXISTS idx_suppliers_company ON suppliers(company_id);
CREATE INDEX IF NOT EXISTS idx_purchase_bills_company ON purchase_bills(company_id);
CREATE INDEX IF NOT EXISTS idx_purchase_bills_supplier ON purchase_bills(supplier_id);
CREATE INDEX IF NOT EXISTS idx_supplier_payments_company ON supplier_payments(company_id);

-- 006_inventory: Inventory Module
CREATE TABLE IF NOT EXISTS products (
    id TEXT PRIMARY KEY,
    company_id TEXT NOT NULL REFERENCES companies(id) ON DELETE CASCADE,
    sku TEXT NOT NULL,
    name TEXT NOT NULL,
    description TEXT,
    category TEXT,
    product_type TEXT NOT NULL DEFAULT 'FINISHED' CHECK (product_type IN ('FINISHED', 'RAW_MATERIAL', 'SEMI_FINISHED', 'SERVICE', 'CONSUMABLE')),
    unit_of_measure TEXT NOT NULL DEFAULT 'PCS',
    cost_price REAL NOT NULL DEFAULT 0,
    selling_price REAL NOT NULL DEFAULT 0,
    tax_code TEXT,
    is_active INTEGER NOT NULL DEFAULT 1,
    track_inventory INTEGER NOT NULL DEFAULT 1,
    reorder_point REAL NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(company_id, sku)
);

CREATE TABLE IF NOT EXISTS warehouses (
    id TEXT PRIMARY KEY,
    company_id TEXT NOT NULL REFERENCES companies(id) ON DELETE CASCADE,
    branch_id TEXT REFERENCES branches(id) ON DELETE SET NULL,
    code TEXT NOT NULL,
    name TEXT NOT NULL,
    location TEXT,
    is_active INTEGER NOT NULL DEFAULT 1,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(company_id, code)
);

CREATE TABLE IF NOT EXISTS inventory_stock_levels (
    id TEXT PRIMARY KEY,
    product_id TEXT NOT NULL REFERENCES products(id) ON DELETE CASCADE,
    warehouse_id TEXT NOT NULL REFERENCES warehouses(id) ON DELETE CASCADE,
    quantity_on_hand REAL NOT NULL DEFAULT 0,
    quantity_reserved REAL NOT NULL DEFAULT 0,
    quantity_available REAL NOT NULL DEFAULT 0,
    average_cost REAL NOT NULL DEFAULT 0,
    last_cost REAL NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(product_id, warehouse_id)
);

CREATE TABLE IF NOT EXISTS stock_movements (
    id TEXT PRIMARY KEY,
    company_id TEXT NOT NULL REFERENCES companies(id) ON DELETE CASCADE,
    product_id TEXT NOT NULL REFERENCES products(id),
    warehouse_id TEXT NOT NULL REFERENCES warehouses(id),
    movement_type TEXT NOT NULL CHECK (movement_type IN ('RECEIPT', 'ISSUE', 'TRANSFER', 'ADJUSTMENT', 'RETURN')),
    quantity REAL NOT NULL,
    unit_cost REAL NOT NULL DEFAULT 0,
    reference_type TEXT,
    reference_id TEXT,
    narration TEXT,
    date TEXT NOT NULL,
    created_by TEXT NOT NULL REFERENCES users(id),
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_products_company ON products(company_id);
CREATE INDEX IF NOT EXISTS idx_warehouses_company ON warehouses(company_id);
CREATE INDEX IF NOT EXISTS idx_stock_levels_product ON inventory_stock_levels(product_id);
CREATE INDEX IF NOT EXISTS idx_stock_levels_warehouse ON inventory_stock_levels(warehouse_id);
CREATE INDEX IF NOT EXISTS idx_stock_movements_product ON stock_movements(product_id);
CREATE INDEX IF NOT EXISTS idx_stock_movements_date ON stock_movements(date);

-- 007_tax: Nigerian Tax Module
CREATE TABLE IF NOT EXISTS tax_configs (
    id TEXT PRIMARY KEY,
    company_id TEXT NOT NULL REFERENCES companies(id) ON DELETE CASCADE,
    tax_type TEXT NOT NULL CHECK (tax_type IN ('VAT', 'WHT', 'CIT', 'EDU_TAX', 'CGT', 'STAMP_DUTY', 'PAYE')),
    name TEXT NOT NULL,
    rate REAL NOT NULL,
    is_active INTEGER NOT NULL DEFAULT 1,
    effective_from TEXT NOT NULL,
    effective_to TEXT,
    description TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(company_id, tax_type, name)
);

CREATE TABLE IF NOT EXISTS tax_transactions (
    id TEXT PRIMARY KEY,
    company_id TEXT NOT NULL REFERENCES companies(id) ON DELETE CASCADE,
    tax_config_id TEXT NOT NULL REFERENCES tax_configs(id),
    transaction_type TEXT NOT NULL CHECK (transaction_type IN ('OUTPUT', 'INPUT')),
    source_module TEXT NOT NULL,
    source_document_id TEXT NOT NULL,
    source_document_number TEXT,
    taxable_amount REAL NOT NULL,
    tax_amount REAL NOT NULL,
    tax_rate REAL NOT NULL,
    posting_date TEXT NOT NULL,
    is_reported INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS wht_rate_categories (
    id TEXT PRIMARY KEY,
    company_id TEXT NOT NULL REFERENCES companies(id) ON DELETE CASCADE,
    category_name TEXT NOT NULL,
    rate REAL NOT NULL,
    is_active INTEGER NOT NULL DEFAULT 1,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS capital_allowance_categories (
    id TEXT PRIMARY KEY,
    company_id TEXT NOT NULL REFERENCES companies(id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    initial_rate REAL NOT NULL,
    annual_rate REAL NOT NULL,
    is_active INTEGER NOT NULL DEFAULT 1,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_tax_configs_company ON tax_configs(company_id, tax_type);
CREATE INDEX IF NOT EXISTS idx_tax_transactions_company ON tax_transactions(company_id);
CREATE INDEX IF NOT EXISTS idx_tax_transactions_date ON tax_transactions(posting_date);
CREATE INDEX IF NOT EXISTS idx_tax_transactions_reported ON tax_transactions(company_id, is_reported);

-- 008_fixed_assets: Fixed Assets & Depreciation
CREATE TABLE IF NOT EXISTS asset_categories (
    id TEXT PRIMARY KEY,
    company_id TEXT NOT NULL REFERENCES companies(id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    depreciation_method TEXT NOT NULL DEFAULT 'STRAIGHT_LINE' CHECK (depreciation_method IN ('STRAIGHT_LINE', 'DECLINING_BALANCE', 'SUM_OF_YEARS', 'UNITS_OF_PRODUCTION')),
    useful_life_years INTEGER NOT NULL,
    residual_rate REAL NOT NULL DEFAULT 0,
    debit_account_id TEXT NOT NULL REFERENCES chart_of_accounts(id),
    credit_account_id TEXT NOT NULL REFERENCES chart_of_accounts(id),
    is_active INTEGER NOT NULL DEFAULT 1,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS fixed_assets (
    id TEXT PRIMARY KEY,
    company_id TEXT NOT NULL REFERENCES companies(id) ON DELETE CASCADE,
    branch_id TEXT REFERENCES branches(id) ON DELETE SET NULL,
    category_id TEXT NOT NULL REFERENCES asset_categories(id),
    code TEXT NOT NULL,
    name TEXT NOT NULL,
    description TEXT,
    acquisition_date TEXT NOT NULL,
    acquisition_cost REAL NOT NULL,
    accumulated_depreciation REAL NOT NULL DEFAULT 0,
    net_book_value REAL NOT NULL,
    residual_value REAL NOT NULL DEFAULT 0,
    useful_life_years INTEGER NOT NULL,
    depreciation_method TEXT NOT NULL DEFAULT 'STRAIGHT_LINE',
    status TEXT NOT NULL DEFAULT 'ACTIVE' CHECK (status IN ('ACTIVE', 'DISPOSED', 'WRITTEN_OFF', 'TRANSFERRED')),
    location TEXT,
    created_by TEXT NOT NULL REFERENCES users(id),
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(company_id, code)
);

CREATE TABLE IF NOT EXISTS depreciation_schedules (
    id TEXT PRIMARY KEY,
    asset_id TEXT NOT NULL REFERENCES fixed_assets(id) ON DELETE CASCADE,
    fiscal_year_id TEXT NOT NULL REFERENCES fiscal_years(id),
    period_id TEXT NOT NULL REFERENCES accounting_periods(id),
    depreciation_amount REAL NOT NULL,
    accumulated_depreciation REAL NOT NULL,
    net_book_value REAL NOT NULL,
    is_posted INTEGER NOT NULL DEFAULT 0,
    posted_journal_id TEXT REFERENCES journal_headers(id),
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS depreciation_runs (
    id TEXT PRIMARY KEY,
    company_id TEXT NOT NULL REFERENCES companies(id) ON DELETE CASCADE,
    fiscal_year_id TEXT NOT NULL REFERENCES fiscal_years(id),
    period_id TEXT NOT NULL REFERENCES accounting_periods(id),
    run_date TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'DRAFT' CHECK (status IN ('DRAFT', 'POSTED', 'REVERSED')),
    total_depreciation REAL NOT NULL DEFAULT 0,
    posted_by TEXT REFERENCES users(id),
    posted_at TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_asset_categories_company ON asset_categories(company_id);
CREATE INDEX IF NOT EXISTS idx_fixed_assets_company ON fixed_assets(company_id);
CREATE INDEX IF NOT EXISTS idx_fixed_assets_category ON fixed_assets(category_id);
CREATE INDEX IF NOT EXISTS idx_depreciation_schedules_asset ON depreciation_schedules(asset_id);

-- 009_loans: Loans & Treasury
CREATE TABLE IF NOT EXISTS loans (
    id TEXT PRIMARY KEY,
    company_id TEXT NOT NULL REFERENCES companies(id) ON DELETE CASCADE,
    lender_name TEXT NOT NULL,
    principal_amount REAL NOT NULL,
    interest_rate REAL NOT NULL,
    loan_type TEXT NOT NULL DEFAULT 'TERM' CHECK (loan_type IN ('TERM', 'OVERDRAFT', 'FACILITY', 'MORTGAGE')),
    start_date TEXT NOT NULL,
    maturity_date TEXT NOT NULL,
    payment_frequency TEXT NOT NULL DEFAULT 'MONTHLY' CHECK (payment_frequency IN ('WEEKLY', 'MONTHLY', 'QUARTERLY', 'SEMI_ANNUALLY', 'ANNUALLY')),
    status TEXT NOT NULL DEFAULT 'ACTIVE' CHECK (status IN ('ACTIVE', 'PAID_OFF', 'DEFAULTED', 'RESTRUCTURED')),
    outstanding_balance REAL NOT NULL,
    disbursed_amount REAL NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS loan_disbursements (
    id TEXT PRIMARY KEY,
    loan_id TEXT NOT NULL REFERENCES loans(id) ON DELETE CASCADE,
    amount REAL NOT NULL,
    disbursement_date TEXT NOT NULL,
    bank_account_id TEXT,
    reference TEXT,
    posted_to_gl INTEGER NOT NULL DEFAULT 0,
    gl_journal_id TEXT REFERENCES journal_headers(id),
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS loan_repayments (
    id TEXT PRIMARY KEY,
    loan_id TEXT NOT NULL REFERENCES loans(id) ON DELETE CASCADE,
    amount REAL NOT NULL,
    principal_portion REAL NOT NULL DEFAULT 0,
    interest_portion REAL NOT NULL DEFAULT 0,
    fee_portion REAL NOT NULL DEFAULT 0,
    payment_date TEXT NOT NULL,
    bank_account_id TEXT,
    reference TEXT,
    posted_to_gl INTEGER NOT NULL DEFAULT 0,
    gl_journal_id TEXT REFERENCES journal_headers(id),
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS loan_amortization_schedule (
    id TEXT PRIMARY KEY,
    loan_id TEXT NOT NULL REFERENCES loans(id) ON DELETE CASCADE,
    period_number INTEGER NOT NULL,
    payment_date TEXT NOT NULL,
    opening_balance REAL NOT NULL,
    payment_amount REAL NOT NULL,
    principal_portion REAL NOT NULL,
    interest_portion REAL NOT NULL,
    closing_balance REAL NOT NULL,
    is_paid INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_loans_company ON loans(company_id);
CREATE INDEX IF NOT EXISTS idx_loan_repayments_loan ON loan_repayments(loan_id);
CREATE INDEX IF NOT EXISTS idx_loan_amortization_loan ON loan_amortization_schedule(loan_id);

-- 010_einvoicing: NRS/FIRS E-Invoicing
CREATE TABLE IF NOT EXISTS einvoice_profiles (
    id TEXT PRIMARY KEY,
    company_id TEXT NOT NULL UNIQUE REFERENCES companies(id) ON DELETE CASCADE,
    tin TEXT NOT NULL,
    legal_name TEXT NOT NULL,
    trade_name TEXT,
    business_email TEXT,
    business_phone TEXT,
    country_code TEXT NOT NULL DEFAULT 'NG',
    state TEXT,
    city TEXT,
    address_line1 TEXT NOT NULL,
    address_line2 TEXT,
    postal_code TEXT,
    access_point_provider_name TEXT,
    access_point_provider_code TEXT,
    default_currency_code TEXT NOT NULL DEFAULT 'NGN',
    is_complete INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS einvoice_credentials (
    id TEXT PRIMARY KEY,
    company_id TEXT NOT NULL UNIQUE REFERENCES companies(id) ON DELETE CASCADE,
    api_key TEXT NOT NULL,
    api_secret TEXT NOT NULL,
    crypto_key TEXT,
    base_url TEXT NOT NULL DEFAULT 'https://einvoice.firs.gov.ng',
    environment TEXT NOT NULL DEFAULT 'SANDBOX' CHECK (environment IN ('SANDBOX', 'PRODUCTION')),
    is_active INTEGER NOT NULL DEFAULT 1,
    last_tested_at TEXT,
    api_key_nonce BLOB,
    api_key_tag BLOB,
    api_secret_nonce BLOB,
    api_secret_tag BLOB,
    client_secret_nonce BLOB,
    client_secret_tag BLOB,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS einvoice_documents (
    id TEXT PRIMARY KEY,
    company_id TEXT NOT NULL REFERENCES companies(id) ON DELETE CASCADE,
    sales_invoice_id TEXT NOT NULL REFERENCES sales_invoices(id),
    irn TEXT,
    status TEXT NOT NULL DEFAULT 'LOCAL_ONLY' CHECK (status IN (
        'LOCAL_ONLY', 'PENDING_VALIDATION', 'VALIDATED', 'PENDING_SIGNING',
        'SIGNED', 'PENDING_CONFIRMATION', 'CONFIRMED', 'DOWNLOADED',
        'UPDATED', 'REJECTED', 'ERROR', 'EXCHANGE_SENT', 'EXCHANGE_ACKNOWLEDGED'
    )),
    invoice_category TEXT CHECK (invoice_category IN ('B2B', 'B2C', 'SIMPLIFIED')),
    validation_result TEXT,
    signing_result TEXT,
    confirmation_result TEXT,
    download_data TEXT,
    firs_submitted_at TEXT,
    firs_confirmed_at TEXT,
    error_message TEXT,
    retry_count INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(company_id, sales_invoice_id)
);

CREATE TABLE IF NOT EXISTS einvoice_webhook_events (
    id TEXT PRIMARY KEY,
    company_id TEXT NOT NULL REFERENCES companies(id) ON DELETE CASCADE,
    event_type TEXT NOT NULL,
    irn TEXT,
    payload TEXT NOT NULL,
    processed INTEGER NOT NULL DEFAULT 0,
    processed_at TEXT,
    error_message TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS einvoice_audit_trail (
    id TEXT PRIMARY KEY,
    company_id TEXT NOT NULL REFERENCES companies(id) ON DELETE CASCADE,
    einvoice_document_id TEXT NOT NULL REFERENCES einvoice_documents(id),
    action TEXT NOT NULL,
    endpoint TEXT,
    request_payload TEXT,
    response_payload TEXT,
    status_code INTEGER,
    user_id TEXT REFERENCES users(id),
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_einvoice_profiles_company ON einvoice_profiles(company_id);
CREATE INDEX IF NOT EXISTS idx_einvoice_documents_company ON einvoice_documents(company_id);
CREATE INDEX IF NOT EXISTS idx_einvoice_documents_irn ON einvoice_documents(irn);
CREATE INDEX IF NOT EXISTS idx_einvoice_documents_status ON einvoice_documents(company_id, status);
CREATE INDEX IF NOT EXISTS idx_einvoice_audit_document ON einvoice_audit_trail(einvoice_document_id);
CREATE INDEX IF NOT EXISTS idx_einvoice_webhook_processed ON einvoice_webhook_events(company_id, processed);

-- 011_ai_intelligence: AI Intelligence Module
CREATE TABLE IF NOT EXISTS ai_analysis_results (
    id TEXT PRIMARY KEY,
    company_id TEXT NOT NULL REFERENCES companies(id) ON DELETE CASCADE,
    analysis_type TEXT NOT NULL CHECK (analysis_type IN (
        'anomaly_detection', 'cashflow_forecast', 'tax_optimization',
        'financial_health', 'spending_pattern', 'revenue_forecast'
    )),
    source_module TEXT NOT NULL,
    source_data TEXT NOT NULL,
    result_data TEXT NOT NULL,
    confidence_score REAL,
    model_version TEXT NOT NULL DEFAULT '1.0.0',
    status TEXT NOT NULL DEFAULT 'completed' CHECK (status IN ('pending', 'running', 'completed', 'failed')),
    error_message TEXT,
    triggered_by TEXT REFERENCES users(id),
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS ai_tax_computations (
    id TEXT PRIMARY KEY,
    company_id TEXT NOT NULL REFERENCES companies(id) ON DELETE CASCADE,
    tax_type TEXT NOT NULL,
    period_start TEXT NOT NULL,
    period_end TEXT NOT NULL,
    taxable_income REAL NOT NULL DEFAULT 0,
    computed_tax REAL NOT NULL DEFAULT 0,
    effective_rate REAL,
    savings_identified REAL NOT NULL DEFAULT 0,
    recommendations TEXT NOT NULL DEFAULT '[]',
    risk_flags TEXT NOT NULL DEFAULT '[]',
    computation_data TEXT NOT NULL,
    model_version TEXT NOT NULL DEFAULT '1.0.0',
    status TEXT NOT NULL DEFAULT 'draft' CHECK (status IN ('draft', 'reviewed', 'approved', 'rejected')),
    reviewed_by TEXT REFERENCES users(id),
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(company_id, tax_type, period_start, period_end)
);

CREATE TABLE IF NOT EXISTS ai_agent_logs (
    id TEXT PRIMARY KEY,
    company_id TEXT NOT NULL REFERENCES companies(id) ON DELETE CASCADE,
    agent_name TEXT NOT NULL,
    agent_type TEXT NOT NULL CHECK (agent_type IN (
        'tax_advisor', 'financial_analyst', 'compliance_checker',
        'anomaly_detector', 'forecast_agent', 'reconciliation_agent'
    )),
    task_description TEXT NOT NULL,
    input_data TEXT,
    output_data TEXT,
    status TEXT NOT NULL DEFAULT 'pending' CHECK (status IN ('pending', 'running', 'completed', 'failed', 'cancelled')),
    started_at TEXT,
    completed_at TEXT,
    duration_ms INTEGER,
    error_message TEXT,
    tokens_used INTEGER,
    model_version TEXT NOT NULL DEFAULT '1.0.0',
    triggered_by TEXT REFERENCES users(id),
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS industry_benchmarks (
    id TEXT PRIMARY KEY,
    industry_code TEXT NOT NULL,
    industry_name TEXT NOT NULL,
    metric_name TEXT NOT NULL,
    metric_value REAL NOT NULL,
    metric_unit TEXT,
    percentile_25 REAL,
    percentile_50 REAL,
    percentile_75 REAL,
    source TEXT NOT NULL,
    country_code TEXT NOT NULL DEFAULT 'NG',
    effective_year INTEGER NOT NULL,
    version INTEGER NOT NULL DEFAULT 1,
    is_active INTEGER NOT NULL DEFAULT 1,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(industry_code, metric_name, effective_year, version)
);

CREATE TABLE IF NOT EXISTS regulatory_updates (
    id TEXT PRIMARY KEY,
    regulation_type TEXT NOT NULL CHECK (regulation_type IN (
        'tax_law', 'accounting_standard', 'cbn_directive', 'firs_notice',
        'companies_act', 'ndpr', 'firs_einvoicing', 'pension_reform'
    )),
    title TEXT NOT NULL,
    description TEXT NOT NULL,
    authority TEXT NOT NULL,
    reference_number TEXT,
    effective_date TEXT NOT NULL,
    compliance_deadline TEXT,
    impact_areas TEXT NOT NULL DEFAULT '[]',
    required_actions TEXT NOT NULL DEFAULT '[]',
    source_url TEXT,
    is_critical INTEGER NOT NULL DEFAULT 0,
    is_read INTEGER NOT NULL DEFAULT 0,
    dismissed_by TEXT REFERENCES users(id),
    version INTEGER NOT NULL DEFAULT 1,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_ai_analysis_company ON ai_analysis_results(company_id, analysis_type);
CREATE INDEX IF NOT EXISTS idx_ai_analysis_status ON ai_analysis_results(company_id, status);
CREATE INDEX IF NOT EXISTS idx_ai_tax_computations_company ON ai_tax_computations(company_id, tax_type);
CREATE INDEX IF NOT EXISTS idx_ai_tax_computations_period ON ai_tax_computations(company_id, period_start, period_end);
CREATE INDEX IF NOT EXISTS idx_ai_agent_logs_company ON ai_agent_logs(company_id, agent_type);
CREATE INDEX IF NOT EXISTS idx_ai_agent_logs_status ON ai_agent_logs(company_id, status);
CREATE INDEX IF NOT EXISTS idx_industry_benchmarks_industry ON industry_benchmarks(industry_code, metric_name);
CREATE INDEX IF NOT EXISTS idx_regulatory_updates_type ON regulatory_updates(regulation_type, effective_date);
CREATE INDEX IF NOT EXISTS idx_regulatory_updates_critical ON regulatory_updates(is_critical, is_read);

-- 012_ocr_documents: OCR Document Processing
CREATE TABLE IF NOT EXISTS ocr_documents (
    id TEXT PRIMARY KEY,
    company_id TEXT NOT NULL REFERENCES companies(id) ON DELETE CASCADE,
    original_filename TEXT NOT NULL,
    file_path TEXT NOT NULL,
    file_size INTEGER NOT NULL,
    mime_type TEXT NOT NULL,
    document_type TEXT NOT NULL CHECK (document_type IN (
        'invoice', 'receipt', 'bank_statement', 'purchase_order',
        'delivery_note', 'tax_certificate', 'pay_slip', 'contract',
        'utility_bill', 'id_document', 'other'
    )),
    ocr_status TEXT NOT NULL DEFAULT 'pending' CHECK (ocr_status IN (
        'pending', 'processing', 'completed', 'failed', 'requires_review'
    )),
    ocr_engine TEXT NOT NULL DEFAULT 'tesseract' CHECK (ocr_engine IN ('tesseract', 'paddleocr', 'aws_textract', 'google_vision')),
    raw_text TEXT,
    confidence_score REAL,
    page_count INTEGER NOT NULL DEFAULT 1,
    language TEXT NOT NULL DEFAULT 'eng',
    is_verified INTEGER NOT NULL DEFAULT 0,
    verified_by TEXT REFERENCES users(id),
    verified_at TEXT,
    linked_document_id TEXT,
    linked_document_type TEXT,
    processing_started_at TEXT,
    processing_completed_at TEXT,
    processing_duration_ms INTEGER,
    error_message TEXT,
    uploaded_by TEXT NOT NULL REFERENCES users(id),
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS ocr_extraction_fields (
    id TEXT PRIMARY KEY,
    ocr_document_id TEXT NOT NULL REFERENCES ocr_documents(id) ON DELETE CASCADE,
    field_name TEXT NOT NULL,
    field_value TEXT NOT NULL,
    confidence_score REAL NOT NULL,
    bounding_box TEXT,
    page_number INTEGER NOT NULL DEFAULT 1,
    is_edited INTEGER NOT NULL DEFAULT 0,
    original_value TEXT,
    edited_by TEXT REFERENCES users(id),
    edited_at TEXT,
    field_type TEXT NOT NULL DEFAULT 'text' CHECK (field_type IN (
        'text', 'number', 'date', 'amount', 'email', 'phone', 'tin', 'percentage'
    )),
    is_verified INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(ocr_document_id, field_name, page_number)
);

CREATE INDEX IF NOT EXISTS idx_ocr_documents_company ON ocr_documents(company_id, document_type);
CREATE INDEX IF NOT EXISTS idx_ocr_documents_status ON ocr_documents(company_id, ocr_status);
CREATE INDEX IF NOT EXISTS idx_ocr_documents_uploaded ON ocr_documents(uploaded_by, created_at);
CREATE INDEX IF NOT EXISTS idx_ocr_extraction_document ON ocr_extraction_fields(ocr_document_id);
CREATE INDEX IF NOT EXISTS idx_ocr_extraction_field_name ON ocr_extraction_fields(ocr_document_id, field_name);

-- 013_payroll: Nigerian Payroll Module
CREATE TABLE IF NOT EXISTS employees (
    id TEXT PRIMARY KEY,
    company_id TEXT NOT NULL REFERENCES companies(id) ON DELETE CASCADE,
    branch_id TEXT REFERENCES branches(id),
    department_id TEXT REFERENCES departments(id),
    employee_number TEXT NOT NULL,
    first_name TEXT NOT NULL,
    last_name TEXT NOT NULL,
    email TEXT,
    phone TEXT,
    designation TEXT NOT NULL,
    grade_level TEXT,
    employment_type TEXT NOT NULL DEFAULT 'full_time' CHECK (employment_type IN (
        'full_time', 'part_time', 'contract', 'intern', 'consultant'
    )),
    salary_amount REAL NOT NULL DEFAULT 0,
    currency_code TEXT NOT NULL DEFAULT 'NGN',
    tax_identification_number TEXT,
    pension_provider TEXT,
    pension_number TEXT,
    nhf_number TEXT,
    bank_name TEXT,
    bank_account_number TEXT,
    bank_sort_code TEXT,
    date_of_birth TEXT,
    date_joined TEXT NOT NULL,
    date_terminated TEXT,
    is_active INTEGER NOT NULL DEFAULT 1,
    paye_state TEXT,
    tax_exemptions TEXT NOT NULL DEFAULT '[]',
    cost_center_id TEXT,
    created_by TEXT NOT NULL REFERENCES users(id),
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(company_id, employee_number)
);

CREATE TABLE IF NOT EXISTS payroll_runs (
    id TEXT PRIMARY KEY,
    company_id TEXT NOT NULL REFERENCES companies(id) ON DELETE CASCADE,
    branch_id TEXT REFERENCES branches(id),
    run_name TEXT NOT NULL,
    period_month INTEGER NOT NULL CHECK (period_month BETWEEN 1 AND 12),
    period_year INTEGER NOT NULL,
    status TEXT NOT NULL DEFAULT 'draft' CHECK (status IN (
        'draft', 'processing', 'completed', 'approved', 'posted', 'cancelled'
    )),
    total_gross REAL NOT NULL DEFAULT 0,
    total_deductions REAL NOT NULL DEFAULT 0,
    total_net REAL NOT NULL DEFAULT 0,
    total_paye REAL NOT NULL DEFAULT 0,
    total_pension REAL NOT NULL DEFAULT 0,
    total_nhf REAL NOT NULL DEFAULT 0,
    total_nsitf REAL NOT NULL DEFAULT 0,
    total_itf REAL NOT NULL DEFAULT 0,
    employee_count INTEGER NOT NULL DEFAULT 0,
    payment_date TEXT,
    approved_by TEXT REFERENCES users(id),
    approved_at TEXT,
    posted_by TEXT REFERENCES users(id),
    posted_at TEXT,
    journal_header_id TEXT,
    created_by TEXT NOT NULL REFERENCES users(id),
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(company_id, period_month, period_year, run_name)
);

CREATE TABLE IF NOT EXISTS payslips (
    id TEXT PRIMARY KEY,
    company_id TEXT NOT NULL REFERENCES companies(id) ON DELETE CASCADE,
    payroll_run_id TEXT NOT NULL REFERENCES payroll_runs(id) ON DELETE CASCADE,
    employee_id TEXT NOT NULL REFERENCES employees(id),
    employee_number TEXT NOT NULL,
    employee_name TEXT NOT NULL,
    period_month INTEGER NOT NULL,
    period_year INTEGER NOT NULL,
    basic_salary REAL NOT NULL DEFAULT 0,
    housing_allowance REAL NOT NULL DEFAULT 0,
    transport_allowance REAL NOT NULL DEFAULT 0,
    meal_allowance REAL NOT NULL DEFAULT 0,
    utility_allowance REAL NOT NULL DEFAULT 0,
    entertainment_allowance REAL NOT NULL DEFAULT 0,
    medical_allowance REAL NOT NULL DEFAULT 0,
    education_allowance REAL NOT NULL DEFAULT 0,
    other_allowances REAL NOT NULL DEFAULT 0,
    other_allowances_desc TEXT,
    total_gross REAL NOT NULL DEFAULT 0,
    paye_tax REAL NOT NULL DEFAULT 0,
    pension_employee REAL NOT NULL DEFAULT 0,
    nhf_employee REAL NOT NULL DEFAULT 0,
    nsitf_employee REAL NOT NULL DEFAULT 0,
    itf_employee REAL NOT NULL DEFAULT 0,
    loan_deduction REAL NOT NULL DEFAULT 0,
    salary_advance REAL NOT NULL DEFAULT 0,
    other_deductions REAL NOT NULL DEFAULT 0,
    other_deductions_desc TEXT,
    total_deductions REAL NOT NULL DEFAULT 0,
    net_pay REAL NOT NULL DEFAULT 0,
    employer_pension REAL NOT NULL DEFAULT 0,
    employer_nsitf REAL NOT NULL DEFAULT 0,
    employer_itf REAL NOT NULL DEFAULT 0,
    employer_nhf REAL NOT NULL DEFAULT 0,
    paye_details TEXT NOT NULL DEFAULT '{}',
    status TEXT NOT NULL DEFAULT 'draft' CHECK (status IN ('draft', 'approved', 'paid', 'cancelled')),
    paid_at TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(payroll_run_id, employee_id)
);

CREATE TABLE IF NOT EXISTS leave_balances (
    id TEXT PRIMARY KEY,
    company_id TEXT NOT NULL REFERENCES companies(id) ON DELETE CASCADE,
    employee_id TEXT NOT NULL REFERENCES employees(id) ON DELETE CASCADE,
    leave_type TEXT NOT NULL CHECK (leave_type IN (
        'annual', 'sick', 'maternity', 'paternity', 'compassionate', 'casual', 'study'
    )),
    year INTEGER NOT NULL,
    total_days REAL NOT NULL DEFAULT 0,
    days_taken REAL NOT NULL DEFAULT 0,
    days_remaining REAL NOT NULL DEFAULT 0,
    days_carried_forward REAL NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(employee_id, leave_type, year)
);

CREATE TABLE IF NOT EXISTS loan_deductions (
    id TEXT PRIMARY KEY,
    company_id TEXT NOT NULL REFERENCES companies(id) ON DELETE CASCADE,
    employee_id TEXT NOT NULL REFERENCES employees(id) ON DELETE CASCADE,
    loan_type TEXT NOT NULL CHECK (loan_type IN (
        'salary_advance', 'staff_loan', 'emergency_loan', 'car_loan', 'housing_loan'
    )),
    principal_amount REAL NOT NULL,
    interest_rate REAL NOT NULL DEFAULT 0,
    monthly_deduction REAL NOT NULL,
    outstanding_balance REAL NOT NULL,
    total_months INTEGER NOT NULL,
    remaining_months INTEGER NOT NULL,
    start_date TEXT NOT NULL,
    end_date TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'active' CHECK (status IN ('active', 'completed', 'defaulted', 'written_off')),
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_employees_company ON employees(company_id, is_active);
CREATE INDEX IF NOT EXISTS idx_employees_department ON employees(company_id, department_id);
CREATE INDEX IF NOT EXISTS idx_employees_number ON employees(company_id, employee_number);
CREATE INDEX IF NOT EXISTS idx_payroll_runs_company ON payroll_runs(company_id, period_year, period_month);
CREATE INDEX IF NOT EXISTS idx_payroll_runs_status ON payroll_runs(company_id, status);
CREATE INDEX IF NOT EXISTS idx_payslips_run ON payslips(payroll_run_id);
CREATE INDEX IF NOT EXISTS idx_payslips_employee ON payslips(employee_id, period_year, period_month);
CREATE INDEX IF NOT EXISTS idx_leave_balances_employee ON leave_balances(employee_id, year);
CREATE INDEX IF NOT EXISTS idx_loan_deductions_employee ON loan_deductions(employee_id, status);

-- 014_bi_crm_notifications: BI, CRM & Notifications
CREATE TABLE IF NOT EXISTS bi_dashboards (
    id TEXT PRIMARY KEY,
    company_id TEXT NOT NULL REFERENCES companies(id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    description TEXT,
    layout_config TEXT NOT NULL DEFAULT '{}',
    is_default INTEGER NOT NULL DEFAULT 0,
    is_shared INTEGER NOT NULL DEFAULT 0,
    owner_id TEXT NOT NULL REFERENCES users(id),
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS bi_widgets (
    id TEXT PRIMARY KEY,
    dashboard_id TEXT NOT NULL REFERENCES bi_dashboards(id) ON DELETE CASCADE,
    widget_type TEXT NOT NULL CHECK (widget_type IN (
        'kpi_card', 'line_chart', 'bar_chart', 'pie_chart', 'table',
        'gauge', 'heatmap', 'funnel', 'scatter', 'area_chart', 'treemap'
    )),
    title TEXT NOT NULL,
    dataset_id TEXT,
    query_id TEXT,
    config TEXT NOT NULL DEFAULT '{}',
    position_x INTEGER NOT NULL DEFAULT 0,
    position_y INTEGER NOT NULL DEFAULT 0,
    width INTEGER NOT NULL DEFAULT 4,
    height INTEGER NOT NULL DEFAULT 3,
    refresh_interval_seconds INTEGER NOT NULL DEFAULT 300,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS bi_datasets (
    id TEXT PRIMARY KEY,
    company_id TEXT NOT NULL REFERENCES companies(id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    source_type TEXT NOT NULL CHECK (source_type IN ('query', 'api', 'upload', 'realtime')),
    source_config TEXT NOT NULL DEFAULT '{}',
    schema_definition TEXT NOT NULL DEFAULT '{}',
    cache_duration_seconds INTEGER NOT NULL DEFAULT 3600,
    last_refreshed_at TEXT,
    row_count INTEGER,
    is_active INTEGER NOT NULL DEFAULT 1,
    created_by TEXT NOT NULL REFERENCES users(id),
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS bi_queries (
    id TEXT PRIMARY KEY,
    company_id TEXT NOT NULL REFERENCES companies(id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    description TEXT,
    query_type TEXT NOT NULL CHECK (query_type IN ('sql', 'builder', 'stored_procedure')),
    query_text TEXT NOT NULL,
    parameters TEXT NOT NULL DEFAULT '[]',
    schedule_cron TEXT,
    last_executed_at TEXT,
    avg_execution_ms INTEGER,
    is_active INTEGER NOT NULL DEFAULT 1,
    created_by TEXT NOT NULL REFERENCES users(id),
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS contacts (
    id TEXT PRIMARY KEY,
    company_id TEXT NOT NULL REFERENCES companies(id) ON DELETE CASCADE,
    contact_type TEXT NOT NULL CHECK (contact_type IN ('lead', 'prospect', 'customer', 'vendor', 'partner')),
    first_name TEXT,
    last_name TEXT,
    organization_name TEXT,
    email TEXT,
    phone TEXT,
    website TEXT,
    title TEXT,
    industry TEXT,
    lead_source TEXT CHECK (lead_source IN ('referral', 'website', 'cold_call', 'advertisement', 'social_media', 'event', 'other')),
    stage TEXT NOT NULL DEFAULT 'new' CHECK (stage IN (
        'new', 'contacted', 'qualified', 'proposal_sent', 'negotiation', 'won', 'lost', 'nurturing'
    )),
    estimated_value REAL,
    probability REAL NOT NULL DEFAULT 0,
    address TEXT,
    notes TEXT,
    assigned_to TEXT REFERENCES users(id),
    tags TEXT NOT NULL DEFAULT '[]',
    custom_fields TEXT NOT NULL DEFAULT '{}',
    last_contacted_at TEXT,
    created_by TEXT NOT NULL REFERENCES users(id),
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS deals (
    id TEXT PRIMARY KEY,
    company_id TEXT NOT NULL REFERENCES companies(id) ON DELETE CASCADE,
    contact_id TEXT NOT NULL REFERENCES contacts(id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    value REAL NOT NULL DEFAULT 0,
    currency_code TEXT NOT NULL DEFAULT 'NGN',
    probability REAL NOT NULL DEFAULT 0,
    stage TEXT NOT NULL DEFAULT 'prospecting' CHECK (stage IN (
        'prospecting', 'qualification', 'proposal', 'negotiation', 'closed_won', 'closed_lost'
    )),
    expected_close_date TEXT,
    actual_close_date TEXT,
    loss_reason TEXT,
    pipeline TEXT NOT NULL DEFAULT 'default',
    assigned_to TEXT REFERENCES users(id),
    products TEXT NOT NULL DEFAULT '[]',
    notes TEXT,
    custom_fields TEXT NOT NULL DEFAULT '{}',
    created_by TEXT NOT NULL REFERENCES users(id),
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS activities (
    id TEXT PRIMARY KEY,
    company_id TEXT NOT NULL REFERENCES companies(id) ON DELETE CASCADE,
    contact_id TEXT REFERENCES contacts(id) ON DELETE SET NULL,
    deal_id TEXT REFERENCES deals(id) ON DELETE SET NULL,
    activity_type TEXT NOT NULL CHECK (activity_type IN ('call', 'email', 'meeting', 'task', 'note')),
    subject TEXT NOT NULL,
    description TEXT,
    status TEXT NOT NULL DEFAULT 'pending' CHECK (status IN ('pending', 'completed', 'cancelled')),
    priority TEXT NOT NULL DEFAULT 'medium' CHECK (priority IN ('low', 'medium', 'high', 'urgent')),
    due_date TEXT,
    completed_at TEXT,
    duration_minutes INTEGER,
    assigned_to TEXT NOT NULL REFERENCES users(id),
    created_by TEXT NOT NULL REFERENCES users(id),
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS notifications (
    id TEXT PRIMARY KEY,
    company_id TEXT NOT NULL REFERENCES companies(id) ON DELETE CASCADE,
    user_id TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    notification_type TEXT NOT NULL CHECK (notification_type IN (
        'info', 'success', 'warning', 'error', 'approval_request',
        'task_assignment', 'system', 'compliance', 'deadline'
    )),
    category TEXT NOT NULL CHECK (category IN (
        'accounting', 'tax', 'sales', 'purchases', 'inventory',
        'payroll', 'einvoicing', 'system', 'crm', 'general'
    )),
    title TEXT NOT NULL,
    message TEXT NOT NULL,
    action_url TEXT,
    is_read INTEGER NOT NULL DEFAULT 0,
    read_at TEXT,
    is_dismissed INTEGER NOT NULL DEFAULT 0,
    dismissed_at TEXT,
    priority TEXT NOT NULL DEFAULT 'normal' CHECK (priority IN ('low', 'normal', 'high', 'critical')),
    expires_at TEXT,
    related_entity_type TEXT,
    related_entity_id TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS notification_preferences (
    id TEXT PRIMARY KEY,
    user_id TEXT NOT NULL UNIQUE REFERENCES users(id) ON DELETE CASCADE,
    email_enabled INTEGER NOT NULL DEFAULT 1,
    push_enabled INTEGER NOT NULL DEFAULT 1,
    in_app_enabled INTEGER NOT NULL DEFAULT 1,
    quiet_hours_start TEXT,
    quiet_hours_end TEXT,
    category_preferences TEXT NOT NULL DEFAULT '{}',
    digest_frequency TEXT NOT NULL DEFAULT 'immediate' CHECK (digest_frequency IN ('immediate', 'hourly', 'daily', 'weekly', 'disabled')),
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_bi_dashboards_company ON bi_dashboards(company_id, owner_id);
CREATE INDEX IF NOT EXISTS idx_bi_widgets_dashboard ON bi_widgets(dashboard_id);
CREATE INDEX IF NOT EXISTS idx_bi_datasets_company ON bi_datasets(company_id, is_active);
CREATE INDEX IF NOT EXISTS idx_bi_queries_company ON bi_queries(company_id, query_type);
CREATE INDEX IF NOT EXISTS idx_contacts_company ON contacts(company_id, contact_type);
CREATE INDEX IF NOT EXISTS idx_contacts_stage ON contacts(company_id, stage);
CREATE INDEX IF NOT EXISTS idx_contacts_assigned ON contacts(assigned_to);
CREATE INDEX IF NOT EXISTS idx_deals_company ON deals(company_id, stage);
CREATE INDEX IF NOT EXISTS idx_deals_pipeline ON deals(company_id, pipeline, stage);
CREATE INDEX IF NOT EXISTS idx_activities_contact ON activities(contact_id);
CREATE INDEX IF NOT EXISTS idx_activities_deal ON activities(deal_id);
CREATE INDEX IF NOT EXISTS idx_activities_assigned ON activities(assigned_to, status);
CREATE INDEX IF NOT EXISTS idx_notifications_user ON notifications(user_id, is_read);
CREATE INDEX IF NOT EXISTS idx_notifications_category ON notifications(company_id, category);
CREATE INDEX IF NOT EXISTS idx_notifications_priority ON notifications(user_id, priority, is_read);

-- 015_licensing: Licensing Module
-- license_tier enum replaced with TEXT + CHECK
CREATE TABLE IF NOT EXISTS license_keys (
    id TEXT PRIMARY KEY,
    key TEXT NOT NULL UNIQUE,
    tier TEXT NOT NULL CHECK (tier IN ('starter', 'professional', 'enterprise', 'government')),
    max_users INTEGER NOT NULL DEFAULT 5,
    max_companies INTEGER NOT NULL DEFAULT 1,
    features TEXT NOT NULL DEFAULT '[]',
    issued_at TEXT NOT NULL DEFAULT (datetime('now')),
    expires_at TEXT NOT NULL,
    is_active INTEGER NOT NULL DEFAULT 1,
    issued_to TEXT NOT NULL,
    signature TEXT NOT NULL,
    last_validated_at TEXT,
    validation_count INTEGER NOT NULL DEFAULT 0,
    revoked_at TEXT,
    revoked_by TEXT REFERENCES users(id),
    revoke_reason TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS feature_flags (
    id TEXT PRIMARY KEY,
    key TEXT NOT NULL UNIQUE,
    display_name TEXT NOT NULL,
    description TEXT,
    tier_required TEXT NOT NULL DEFAULT 'starter' CHECK (tier_required IN ('starter', 'professional', 'enterprise', 'government')),
    is_active INTEGER NOT NULL DEFAULT 1,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS subscription_records (
    id TEXT PRIMARY KEY,
    company_id TEXT NOT NULL REFERENCES companies(id) ON DELETE CASCADE,
    license_key_id TEXT NOT NULL REFERENCES license_keys(id),
    tier TEXT NOT NULL CHECK (tier IN ('starter', 'professional', 'enterprise', 'government')),
    status TEXT NOT NULL DEFAULT 'active' CHECK (status IN ('trial', 'active', 'past_due', 'cancelled', 'expired')),
    started_at TEXT NOT NULL DEFAULT (datetime('now')),
    current_period_start TEXT NOT NULL,
    current_period_end TEXT NOT NULL,
    cancel_at_period_end INTEGER NOT NULL DEFAULT 0,
    amount TEXT NOT NULL DEFAULT '{}',
    currency_code TEXT NOT NULL DEFAULT 'NGN',
    payment_method TEXT,
    billing_email TEXT,
    last_payment_at TEXT,
    next_payment_at TEXT,
    cancelled_at TEXT,
    cancellation_reason TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(company_id, license_key_id)
);

CREATE INDEX IF NOT EXISTS idx_license_keys_key ON license_keys(key);
CREATE INDEX IF NOT EXISTS idx_license_keys_tier ON license_keys(tier, is_active);
CREATE INDEX IF NOT EXISTS idx_license_keys_expires ON license_keys(expires_at);
CREATE INDEX IF NOT EXISTS idx_feature_flags_tier ON feature_flags(tier_required, is_active);
CREATE INDEX IF NOT EXISTS idx_subscriptions_company ON subscription_records(company_id, status);
CREATE INDEX IF NOT EXISTS idx_subscriptions_period ON subscription_records(current_period_start, current_period_end);

INSERT OR IGNORE INTO feature_flags (key, display_name, description, tier_required) VALUES
    ('accounting', 'Accounting', 'Chart of accounts, journals, posting', 'starter'),
    ('tax_vat', 'VAT', 'Value Added Tax management', 'starter'),
    ('tax_paye', 'PAYE', 'Pay-As-You-Earn tax', 'starter'),
    ('tax_wht', 'WHT', 'Withholding Tax management', 'starter'),
    ('einvoicing_basic', 'E-Invoicing Basic', 'Basic e-invoice generation', 'starter'),
    ('reports_basic', 'Basic Reports', 'Trial balance, P&L, Balance Sheet', 'starter'),
    ('tax_all', 'All Tax Types', 'VAT, WHT, PAYE, CIT, CGT, Stamp Duty, Edu Tax', 'professional'),
    ('einvoicing_full', 'E-Invoicing Full', 'FIRS NRS integration with submission', 'professional'),
    ('payroll', 'Payroll', 'Full Nigerian payroll with payslips', 'professional'),
    ('bi_basic', 'Basic BI', 'KPI dashboards and basic charts', 'professional'),
    ('crm_basic', 'Basic CRM', 'Contact management and deal tracking', 'professional'),
    ('reports_advanced', 'Advanced Reports', 'Cash flow, ratios, budget variance, tax schedules', 'professional'),
    ('payroll_loans', 'Loan Management', 'Employee loan deductions and amortization', 'enterprise'),
    ('bi_full', 'Full BI', 'Dashboard builder, query builder, datasets', 'enterprise'),
    ('crm_full', 'Full CRM', 'Pipeline, activities, contact stages', 'enterprise'),
    ('ai_agents', 'AI Intelligence', 'AI-powered tax advisory and anomaly detection', 'enterprise'),
    ('ocr', 'OCR Processing', 'Document scanning and field extraction', 'enterprise'),
    ('api_access', 'API Access', 'External API for integrations', 'enterprise'),
    ('custom_integrations', 'Custom Integrations', 'Custom third-party integrations', 'enterprise'),
    ('multi_company', 'Multi-Company', 'Consolidated multi-company reporting', 'enterprise'),
    ('on_premise', 'On-Premise', 'On-premise deployment option', 'government'),
    ('audit_trail_enhanced', 'Enhanced Audit', 'Immutable audit trail with integrity verification', 'government'),
    ('compliance_reports', 'Compliance Reports', 'NDPR, CBN, FIRS compliance reports', 'government'),
    ('custom_dashboards', 'Custom Dashboards', 'Agency-specific dashboard templates', 'government');
