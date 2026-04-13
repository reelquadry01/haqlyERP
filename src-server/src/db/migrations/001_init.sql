-- HAQLY ERP - Database Initialization
-- Author: Quadri Atharu

CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
CREATE EXTENSION IF NOT EXISTS "pgcrypto";

-- Users & Authentication
CREATE TABLE users (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    email VARCHAR(255) NOT NULL UNIQUE,
    password_hash VARCHAR(255) NOT NULL,
    full_name VARCHAR(255) NOT NULL,
    is_active BOOLEAN NOT NULL DEFAULT true,
    mfa_secret VARCHAR(255),
    mfa_enabled BOOLEAN NOT NULL DEFAULT false,
    last_login TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE sessions (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    refresh_token VARCHAR(512) NOT NULL,
    expires_at TIMESTAMPTZ NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- RBAC
CREATE TABLE roles (
    id SERIAL PRIMARY KEY,
    name VARCHAR(100) NOT NULL UNIQUE,
    description TEXT
);

CREATE TABLE permissions (
    id SERIAL PRIMARY KEY,
    name VARCHAR(100) NOT NULL UNIQUE,
    description TEXT
);

CREATE TABLE role_permissions (
    role_id INT NOT NULL REFERENCES roles(id) ON DELETE CASCADE,
    permission_id INT NOT NULL REFERENCES permissions(id) ON DELETE CASCADE,
    PRIMARY KEY (role_id, permission_id)
);

CREATE TABLE user_roles (
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    role_id INT NOT NULL REFERENCES roles(id) ON DELETE CASCADE,
    PRIMARY KEY (user_id, role_id)
);

-- Audit
CREATE TABLE audit_logs (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id UUID REFERENCES users(id),
    action VARCHAR(100) NOT NULL,
    entity VARCHAR(100) NOT NULL,
    entity_id UUID,
    details JSONB,
    ip_address VARCHAR(45),
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_audit_logs_user ON audit_logs(user_id);
CREATE INDEX idx_audit_logs_entity ON audit_logs(entity, entity_id);
CREATE INDEX idx_audit_logs_created ON audit_logs(created_at);

-- Seed roles
INSERT INTO roles (name, description) VALUES
    ('SuperAdmin', 'Full system access'),
    ('Admin', 'Company administration'),
    ('Accountant', 'Accounting and finance operations'),
    ('Sales', 'Sales module access'),
    ('Purchaser', 'Purchase module access'),
    ('InventoryMgr', 'Inventory management'),
    ('HR', 'Human resources and payroll'),
    ('Treasury', 'Treasury and cash management'),
    ('Viewer', 'Read-only access');

-- Seed permissions
INSERT INTO permissions (name, description) VALUES
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
