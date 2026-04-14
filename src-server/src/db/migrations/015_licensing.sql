-- HAQLY ERP - Licensing Module
-- Author: Quadri Atharu

CREATE TYPE license_tier AS ENUM ('starter', 'professional', 'enterprise', 'government');

CREATE TABLE license_keys (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    key VARCHAR(100) NOT NULL UNIQUE,
    tier license_tier NOT NULL,
    max_users INT NOT NULL DEFAULT 5,
    max_companies INT NOT NULL DEFAULT 1,
    features JSONB NOT NULL DEFAULT '[]',
    issued_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    expires_at TIMESTAMPTZ NOT NULL,
    is_active BOOLEAN NOT NULL DEFAULT true,
    issued_to VARCHAR(255) NOT NULL,
    signature VARCHAR(255) NOT NULL,
    last_validated_at TIMESTAMPTZ,
    validation_count INT NOT NULL DEFAULT 0,
    revoked_at TIMESTAMPTZ,
    revoked_by UUID REFERENCES users(id),
    revoke_reason TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE feature_flags (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    key VARCHAR(100) NOT NULL UNIQUE,
    display_name VARCHAR(255) NOT NULL,
    description TEXT,
    tier_required license_tier NOT NULL DEFAULT 'starter',
    is_active BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE subscription_records (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    company_id UUID NOT NULL REFERENCES companies(id) ON DELETE CASCADE,
    license_key_id UUID NOT NULL REFERENCES license_keys(id),
    tier license_tier NOT NULL,
    status VARCHAR(20) NOT NULL DEFAULT 'active' CHECK (status IN ('trial', 'active', 'past_due', 'cancelled', 'expired')),
    started_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    current_period_start TIMESTAMPTZ NOT NULL,
    current_period_end TIMESTAMPTZ NOT NULL,
    cancel_at_period_end BOOLEAN NOT NULL DEFAULT false,
    amount JSONB NOT NULL DEFAULT '{}',
    currency_code VARCHAR(3) NOT NULL DEFAULT 'NGN',
    payment_method VARCHAR(50),
    billing_email VARCHAR(255),
    last_payment_at TIMESTAMPTZ,
    next_payment_at TIMESTAMPTZ,
    cancelled_at TIMESTAMPTZ,
    cancellation_reason TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE(company_id, license_key_id)
);

CREATE INDEX idx_license_keys_key ON license_keys(key);
CREATE INDEX idx_license_keys_tier ON license_keys(tier, is_active);
CREATE INDEX idx_license_keys_expires ON license_keys(expires_at);
CREATE INDEX idx_feature_flags_tier ON feature_flags(tier_required, is_active);
CREATE INDEX idx_subscriptions_company ON subscription_records(company_id, status);
CREATE INDEX idx_subscriptions_period ON subscription_records(current_period_start, current_period_end);

INSERT INTO feature_flags (key, display_name, description, tier_required) VALUES
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
