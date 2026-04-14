-- HAQLY ERP - AI Intelligence Module
-- Author: Quadri Atharu

CREATE TABLE ai_analysis_results (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    company_id UUID NOT NULL REFERENCES companies(id) ON DELETE CASCADE,
    analysis_type VARCHAR(50) NOT NULL CHECK (analysis_type IN (
        'anomaly_detection', 'cashflow_forecast', 'tax_optimization',
        'financial_health', 'spending_pattern', 'revenue_forecast'
    )),
    source_module VARCHAR(50) NOT NULL,
    source_data JSONB NOT NULL,
    result_data JSONB NOT NULL,
    confidence_score NUMERIC(5,2),
    model_version VARCHAR(50) NOT NULL DEFAULT '1.0.0',
    status VARCHAR(20) NOT NULL DEFAULT 'completed' CHECK (status IN ('pending', 'running', 'completed', 'failed')),
    error_message TEXT,
    triggered_by UUID REFERENCES users(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE ai_tax_computations (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    company_id UUID NOT NULL REFERENCES companies(id) ON DELETE CASCADE,
    tax_type VARCHAR(20) NOT NULL,
    period_start DATE NOT NULL,
    period_end DATE NOT NULL,
    taxable_income NUMERIC(18,2) NOT NULL DEFAULT 0,
    computed_tax NUMERIC(18,2) NOT NULL DEFAULT 0,
    effective_rate NUMERIC(5,2),
    savings_identified NUMERIC(18,2) NOT NULL DEFAULT 0,
    recommendations JSONB NOT NULL DEFAULT '[]',
    risk_flags JSONB NOT NULL DEFAULT '[]',
    computation_data JSONB NOT NULL,
    model_version VARCHAR(50) NOT NULL DEFAULT '1.0.0',
    status VARCHAR(20) NOT NULL DEFAULT 'draft' CHECK (status IN ('draft', 'reviewed', 'approved', 'rejected')),
    reviewed_by UUID REFERENCES users(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE(company_id, tax_type, period_start, period_end)
);

CREATE TABLE ai_agent_logs (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    company_id UUID NOT NULL REFERENCES companies(id) ON DELETE CASCADE,
    agent_name VARCHAR(100) NOT NULL,
    agent_type VARCHAR(50) NOT NULL CHECK (agent_type IN (
        'tax_advisor', 'financial_analyst', 'compliance_checker',
        'anomaly_detector', 'forecast_agent', 'reconciliation_agent'
    )),
    task_description TEXT NOT NULL,
    input_data JSONB,
    output_data JSONB,
    status VARCHAR(20) NOT NULL DEFAULT 'pending' CHECK (status IN ('pending', 'running', 'completed', 'failed', 'cancelled')),
    started_at TIMESTAMPTZ,
    completed_at TIMESTAMPTZ,
    duration_ms INT,
    error_message TEXT,
    tokens_used INT,
    model_version VARCHAR(50) NOT NULL DEFAULT '1.0.0',
    triggered_by UUID REFERENCES users(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE industry_benchmarks (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    industry_code VARCHAR(20) NOT NULL,
    industry_name VARCHAR(255) NOT NULL,
    metric_name VARCHAR(100) NOT NULL,
    metric_value NUMERIC(18,4) NOT NULL,
    metric_unit VARCHAR(50),
    percentile_25 NUMERIC(18,4),
    percentile_50 NUMERIC(18,4),
    percentile_75 NUMERIC(18,4),
    source VARCHAR(100) NOT NULL,
    country_code VARCHAR(3) NOT NULL DEFAULT 'NG',
    effective_year INT NOT NULL,
    version INT NOT NULL DEFAULT 1,
    is_active BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE(industry_code, metric_name, effective_year, version)
);

CREATE TABLE regulatory_updates (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    regulation_type VARCHAR(50) NOT NULL CHECK (regulation_type IN (
        'tax_law', 'accounting_standard', 'cbn_directive', 'firs_notice',
        'companies_act', 'ndpr', 'firs_einvoicing', 'pension_reform'
    )),
    title VARCHAR(500) NOT NULL,
    description TEXT NOT NULL,
    authority VARCHAR(100) NOT NULL,
    reference_number VARCHAR(100),
    effective_date DATE NOT NULL,
    compliance_deadline DATE,
    impact_areas JSONB NOT NULL DEFAULT '[]',
    required_actions JSONB NOT NULL DEFAULT '[]',
    source_url VARCHAR(500),
    is_critical BOOLEAN NOT NULL DEFAULT false,
    is_read BOOLEAN NOT NULL DEFAULT false,
    dismissed_by UUID REFERENCES users(id),
    version INT NOT NULL DEFAULT 1,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_ai_analysis_company ON ai_analysis_results(company_id, analysis_type);
CREATE INDEX idx_ai_analysis_status ON ai_analysis_results(company_id, status);
CREATE INDEX idx_ai_tax_computations_company ON ai_tax_computations(company_id, tax_type);
CREATE INDEX idx_ai_tax_computations_period ON ai_tax_computations(company_id, period_start, period_end);
CREATE INDEX idx_ai_agent_logs_company ON ai_agent_logs(company_id, agent_type);
CREATE INDEX idx_ai_agent_logs_status ON ai_agent_logs(company_id, status);
CREATE INDEX idx_industry_benchmarks_industry ON industry_benchmarks(industry_code, metric_name);
CREATE INDEX idx_regulatory_updates_type ON regulatory_updates(regulation_type, effective_date);
CREATE INDEX idx_regulatory_updates_critical ON regulatory_updates(is_critical, is_read);
