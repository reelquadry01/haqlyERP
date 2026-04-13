-- HAQLY ERP - NRS/FIRS E-Invoicing
-- Author: Quadri Atharu

CREATE TABLE einvoice_profiles (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    company_id UUID NOT NULL UNIQUE REFERENCES companies(id) ON DELETE CASCADE,
    tin VARCHAR(50) NOT NULL,
    legal_name VARCHAR(255) NOT NULL,
    trade_name VARCHAR(255),
    business_email VARCHAR(255),
    business_phone VARCHAR(50),
    country_code VARCHAR(3) NOT NULL DEFAULT 'NG',
    state VARCHAR(100),
    city VARCHAR(100),
    address_line1 VARCHAR(255) NOT NULL,
    address_line2 VARCHAR(255),
    postal_code VARCHAR(20),
    access_point_provider_name VARCHAR(255),
    access_point_provider_code VARCHAR(50),
    default_currency_code VARCHAR(3) NOT NULL DEFAULT 'NGN',
    is_complete BOOLEAN NOT NULL DEFAULT false,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE einvoice_credentials (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    company_id UUID NOT NULL UNIQUE REFERENCES companies(id) ON DELETE CASCADE,
    api_key VARCHAR(255) NOT NULL,
    api_secret VARCHAR(255) NOT NULL,
    crypto_key VARCHAR(512),
    base_url VARCHAR(255) NOT NULL DEFAULT 'https://einvoice.firs.gov.ng',
    environment VARCHAR(20) NOT NULL DEFAULT 'SANDBOX' CHECK (environment IN ('SANDBOX', 'PRODUCTION')),
    is_active BOOLEAN NOT NULL DEFAULT true,
    last_tested_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE einvoice_documents (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    company_id UUID NOT NULL REFERENCES companies(id) ON DELETE CASCADE,
    sales_invoice_id UUID NOT NULL REFERENCES sales_invoices(id),
    irn VARCHAR(100),
    status VARCHAR(30) NOT NULL DEFAULT 'LOCAL_ONLY' CHECK (status IN (
        'LOCAL_ONLY', 'PENDING_VALIDATION', 'VALIDATED', 'PENDING_SIGNING',
        'SIGNED', 'PENDING_CONFIRMATION', 'CONFIRMED', 'DOWNLOADED',
        'UPDATED', 'REJECTED', 'ERROR', 'EXCHANGE_SENT', 'EXCHANGE_ACKNOWLEDGED'
    )),
    invoice_category VARCHAR(10) CHECK (invoice_category IN ('B2B', 'B2C', 'SIMPLIFIED')),
    validation_result JSONB,
    signing_result JSONB,
    confirmation_result JSONB,
    download_data JSONB,
    firs_submitted_at TIMESTAMPTZ,
    firs_confirmed_at TIMESTAMPTZ,
    error_message TEXT,
    retry_count INT NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE(company_id, sales_invoice_id)
);

CREATE TABLE einvoice_webhook_events (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    company_id UUID NOT NULL REFERENCES companies(id) ON DELETE CASCADE,
    event_type VARCHAR(100) NOT NULL,
    irn VARCHAR(100),
    payload JSONB NOT NULL,
    processed BOOLEAN NOT NULL DEFAULT false,
    processed_at TIMESTAMPTZ,
    error_message TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE einvoice_audit_trail (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    company_id UUID NOT NULL REFERENCES companies(id) ON DELETE CASCADE,
    einvoice_document_id UUID NOT NULL REFERENCES einvoice_documents(id),
    action VARCHAR(50) NOT NULL,
    endpoint VARCHAR(255),
    request_payload JSONB,
    response_payload JSONB,
    status_code INT,
    user_id UUID REFERENCES users(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_einvoice_profiles_company ON einvoice_profiles(company_id);
CREATE INDEX idx_einvoice_documents_company ON einvoice_documents(company_id);
CREATE INDEX idx_einvoice_documents_irn ON einvoice_documents(irn);
CREATE INDEX idx_einvoice_documents_status ON einvoice_documents(company_id, status);
CREATE INDEX idx_einvoice_audit_document ON einvoice_audit_trail(einvoice_document_id);
CREATE INDEX idx_einvoice_webhook_processed ON einvoice_webhook_events(company_id, processed);
