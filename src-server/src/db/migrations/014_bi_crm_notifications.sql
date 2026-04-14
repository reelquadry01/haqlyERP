-- HAQLY ERP - BI, CRM & Notifications Module
-- Author: Quadri Atharu

-- BI Tables
CREATE TABLE bi_dashboards (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    company_id UUID NOT NULL REFERENCES companies(id) ON DELETE CASCADE,
    name VARCHAR(100) NOT NULL,
    description TEXT,
    layout_config JSONB NOT NULL DEFAULT '{}',
    is_default BOOLEAN NOT NULL DEFAULT false,
    is_shared BOOLEAN NOT NULL DEFAULT false,
    owner_id UUID NOT NULL REFERENCES users(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE bi_widgets (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    dashboard_id UUID NOT NULL REFERENCES bi_dashboards(id) ON DELETE CASCADE,
    widget_type VARCHAR(30) NOT NULL CHECK (widget_type IN (
        'kpi_card', 'line_chart', 'bar_chart', 'pie_chart', 'table',
        'gauge', 'heatmap', 'funnel', 'scatter', 'area_chart', 'treemap'
    )),
    title VARCHAR(100) NOT NULL,
    dataset_id UUID,
    query_id UUID,
    config JSONB NOT NULL DEFAULT '{}',
    position_x INT NOT NULL DEFAULT 0,
    position_y INT NOT NULL DEFAULT 0,
    width INT NOT NULL DEFAULT 4,
    height INT NOT NULL DEFAULT 3,
    refresh_interval_seconds INT NOT NULL DEFAULT 300,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE bi_datasets (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    company_id UUID NOT NULL REFERENCES companies(id) ON DELETE CASCADE,
    name VARCHAR(100) NOT NULL,
    source_type VARCHAR(20) NOT NULL CHECK (source_type IN ('query', 'api', 'upload', 'realtime')),
    source_config JSONB NOT NULL DEFAULT '{}',
    schema_definition JSONB NOT NULL DEFAULT '{}',
    cache_duration_seconds INT NOT NULL DEFAULT 3600,
    last_refreshed_at TIMESTAMPTZ,
    row_count INT,
    is_active BOOLEAN NOT NULL DEFAULT true,
    created_by UUID NOT NULL REFERENCES users(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE bi_queries (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    company_id UUID NOT NULL REFERENCES companies(id) ON DELETE CASCADE,
    name VARCHAR(100) NOT NULL,
    description TEXT,
    query_type VARCHAR(20) NOT NULL CHECK (query_type IN ('sql', 'builder', 'stored_procedure')),
    query_text TEXT NOT NULL,
    parameters JSONB NOT NULL DEFAULT '[]',
    schedule_cron VARCHAR(100),
    last_executed_at TIMESTAMPTZ,
    avg_execution_ms INT,
    is_active BOOLEAN NOT NULL DEFAULT true,
    created_by UUID NOT NULL REFERENCES users(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- CRM Tables
CREATE TABLE contacts (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    company_id UUID NOT NULL REFERENCES companies(id) ON DELETE CASCADE,
    contact_type VARCHAR(20) NOT NULL CHECK (contact_type IN ('lead', 'prospect', 'customer', 'vendor', 'partner')),
    first_name VARCHAR(100),
    last_name VARCHAR(100),
    organization_name VARCHAR(255),
    email VARCHAR(255),
    phone VARCHAR(50),
    website VARCHAR(500),
    title VARCHAR(100),
    industry VARCHAR(100),
    lead_source VARCHAR(50) CHECK (lead_source IN ('referral', 'website', 'cold_call', 'advertisement', 'social_media', 'event', 'other')),
    stage VARCHAR(30) NOT NULL DEFAULT 'new' CHECK (stage IN (
        'new', 'contacted', 'qualified', 'proposal_sent', 'negotiation', 'won', 'lost', 'nurturing'
    )),
    estimated_value NUMERIC(18,2),
    probability NUMERIC(5,2) NOT NULL DEFAULT 0,
    address JSONB,
    notes TEXT,
    assigned_to UUID REFERENCES users(id),
    tags JSONB NOT NULL DEFAULT '[]',
    custom_fields JSONB NOT NULL DEFAULT '{}',
    last_contacted_at TIMESTAMPTZ,
    created_by UUID NOT NULL REFERENCES users(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE deals (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    company_id UUID NOT NULL REFERENCES companies(id) ON DELETE CASCADE,
    contact_id UUID NOT NULL REFERENCES contacts(id) ON DELETE CASCADE,
    name VARCHAR(200) NOT NULL,
    value NUMERIC(18,2) NOT NULL DEFAULT 0,
    currency_code VARCHAR(3) NOT NULL DEFAULT 'NGN',
    probability NUMERIC(5,2) NOT NULL DEFAULT 0,
    stage VARCHAR(30) NOT NULL DEFAULT 'prospecting' CHECK (stage IN (
        'prospecting', 'qualification', 'proposal', 'negotiation', 'closed_won', 'closed_lost'
    )),
    expected_close_date DATE,
    actual_close_date DATE,
    loss_reason TEXT,
    pipeline VARCHAR(50) NOT NULL DEFAULT 'default',
    assigned_to UUID REFERENCES users(id),
    products JSONB NOT NULL DEFAULT '[]',
    notes TEXT,
    custom_fields JSONB NOT NULL DEFAULT '{}',
    created_by UUID NOT NULL REFERENCES users(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE activities (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    company_id UUID NOT NULL REFERENCES companies(id) ON DELETE CASCADE,
    contact_id UUID REFERENCES contacts(id) ON DELETE SET NULL,
    deal_id UUID REFERENCES deals(id) ON DELETE SET NULL,
    activity_type VARCHAR(20) NOT NULL CHECK (activity_type IN ('call', 'email', 'meeting', 'task', 'note')),
    subject VARCHAR(200) NOT NULL,
    description TEXT,
    status VARCHAR(20) NOT NULL DEFAULT 'pending' CHECK (status IN ('pending', 'completed', 'cancelled')),
    priority VARCHAR(10) NOT NULL DEFAULT 'medium' CHECK (priority IN ('low', 'medium', 'high', 'urgent')),
    due_date DATE,
    completed_at TIMESTAMPTZ,
    duration_minutes INT,
    assigned_to UUID NOT NULL REFERENCES users(id),
    created_by UUID NOT NULL REFERENCES users(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Notifications
CREATE TABLE notifications (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    company_id UUID NOT NULL REFERENCES companies(id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    notification_type VARCHAR(30) NOT NULL CHECK (notification_type IN (
        'info', 'success', 'warning', 'error', 'approval_request',
        'task_assignment', 'system', 'compliance', 'deadline'
    )),
    category VARCHAR(30) NOT NULL CHECK (category IN (
        'accounting', 'tax', 'sales', 'purchases', 'inventory',
        'payroll', 'einvoicing', 'system', 'crm', 'general'
    )),
    title VARCHAR(200) NOT NULL,
    message TEXT NOT NULL,
    action_url VARCHAR(500),
    is_read BOOLEAN NOT NULL DEFAULT false,
    read_at TIMESTAMPTZ,
    is_dismissed BOOLEAN NOT NULL DEFAULT false,
    dismissed_at TIMESTAMPTZ,
    priority VARCHAR(10) NOT NULL DEFAULT 'normal' CHECK (priority IN ('low', 'normal', 'high', 'critical')),
    expires_at TIMESTAMPTZ,
    related_entity_type VARCHAR(50),
    related_entity_id UUID,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE notification_preferences (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id UUID NOT NULL UNIQUE REFERENCES users(id) ON DELETE CASCADE,
    email_enabled BOOLEAN NOT NULL DEFAULT true,
    push_enabled BOOLEAN NOT NULL DEFAULT true,
    in_app_enabled BOOLEAN NOT NULL DEFAULT true,
    quiet_hours_start TIME,
    quiet_hours_end TIME,
    category_preferences JSONB NOT NULL DEFAULT '{}',
    digest_frequency VARCHAR(20) NOT NULL DEFAULT 'immediate' CHECK (digest_frequency IN ('immediate', 'hourly', 'daily', 'weekly', 'disabled')),
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_bi_dashboards_company ON bi_dashboards(company_id, owner_id);
CREATE INDEX idx_bi_widgets_dashboard ON bi_widgets(dashboard_id);
CREATE INDEX idx_bi_datasets_company ON bi_datasets(company_id, is_active);
CREATE INDEX idx_bi_queries_company ON bi_queries(company_id, query_type);
CREATE INDEX idx_contacts_company ON contacts(company_id, contact_type);
CREATE INDEX idx_contacts_stage ON contacts(company_id, stage);
CREATE INDEX idx_contacts_assigned ON contacts(assigned_to);
CREATE INDEX idx_deals_company ON deals(company_id, stage);
CREATE INDEX idx_deals_pipeline ON deals(company_id, pipeline, stage);
CREATE INDEX idx_activities_contact ON activities(contact_id);
CREATE INDEX idx_activities_deal ON activities(deal_id);
CREATE INDEX idx_activities_assigned ON activities(assigned_to, status);
CREATE INDEX idx_notifications_user ON notifications(user_id, is_read);
CREATE INDEX idx_notifications_category ON notifications(company_id, category);
CREATE INDEX idx_notifications_priority ON notifications(user_id, priority, is_read);
