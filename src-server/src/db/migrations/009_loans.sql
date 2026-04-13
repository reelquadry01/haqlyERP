-- HAQLY ERP - Loans & Treasury
-- Author: Quadri Atharu

CREATE TABLE loans (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    company_id UUID NOT NULL REFERENCES companies(id) ON DELETE CASCADE,
    lender_name VARCHAR(255) NOT NULL,
    principal_amount NUMERIC(18,2) NOT NULL,
    interest_rate NUMERIC(8,4) NOT NULL,
    loan_type VARCHAR(20) NOT NULL DEFAULT 'TERM' CHECK (loan_type IN ('TERM', 'OVERDRAFT', 'FACILITY', 'MORTGAGE')),
    start_date DATE NOT NULL,
    maturity_date DATE NOT NULL,
    payment_frequency VARCHAR(20) NOT NULL DEFAULT 'MONTHLY' CHECK (payment_frequency IN ('WEEKLY', 'MONTHLY', 'QUARTERLY', 'SEMI_ANNUALLY', 'ANNUALLY')),
    status VARCHAR(20) NOT NULL DEFAULT 'ACTIVE' CHECK (status IN ('ACTIVE', 'PAID_OFF', 'DEFAULTED', 'RESTRUCTURED')),
    outstanding_balance NUMERIC(18,2) NOT NULL,
    disbursed_amount NUMERIC(18,2) NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE loan_disbursements (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    loan_id UUID NOT NULL REFERENCES loans(id) ON DELETE CASCADE,
    amount NUMERIC(18,2) NOT NULL,
    disbursement_date DATE NOT NULL,
    bank_account_id UUID,
    reference VARCHAR(100),
    posted_to_gl BOOLEAN NOT NULL DEFAULT false,
    gl_journal_id UUID REFERENCES journal_headers(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE loan_repayments (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    loan_id UUID NOT NULL REFERENCES loans(id) ON DELETE CASCADE,
    amount NUMERIC(18,2) NOT NULL,
    principal_portion NUMERIC(18,2) NOT NULL DEFAULT 0,
    interest_portion NUMERIC(18,2) NOT NULL DEFAULT 0,
    fee_portion NUMERIC(18,2) NOT NULL DEFAULT 0,
    payment_date DATE NOT NULL,
    bank_account_id UUID,
    reference VARCHAR(100),
    posted_to_gl BOOLEAN NOT NULL DEFAULT false,
    gl_journal_id UUID REFERENCES journal_headers(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE loan_amortization_schedule (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    loan_id UUID NOT NULL REFERENCES loans(id) ON DELETE CASCADE,
    period_number INT NOT NULL,
    payment_date DATE NOT NULL,
    opening_balance NUMERIC(18,2) NOT NULL,
    payment_amount NUMERIC(18,2) NOT NULL,
    principal_portion NUMERIC(18,2) NOT NULL,
    interest_portion NUMERIC(18,2) NOT NULL,
    closing_balance NUMERIC(18,2) NOT NULL,
    is_paid BOOLEAN NOT NULL DEFAULT false,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_loans_company ON loans(company_id);
CREATE INDEX idx_loan_repayments_loan ON loan_repayments(loan_id);
CREATE INDEX idx_loan_amortization_loan ON loan_amortization_schedule(loan_id);
