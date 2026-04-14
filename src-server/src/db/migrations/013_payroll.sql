-- HAQLY ERP - Nigerian Payroll Module
-- Author: Quadri Atharu

CREATE TABLE employees (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    company_id UUID NOT NULL REFERENCES companies(id) ON DELETE CASCADE,
    branch_id UUID REFERENCES branches(id),
    department_id UUID REFERENCES departments(id),
    employee_number VARCHAR(50) NOT NULL,
    first_name VARCHAR(100) NOT NULL,
    last_name VARCHAR(100) NOT NULL,
    email VARCHAR(255),
    phone VARCHAR(50),
    designation VARCHAR(100) NOT NULL,
    grade_level VARCHAR(20),
    employment_type VARCHAR(20) NOT NULL DEFAULT 'full_time' CHECK (employment_type IN (
        'full_time', 'part_time', 'contract', 'intern', 'consultant'
    )),
    salary_amount NUMERIC(18,2) NOT NULL DEFAULT 0,
    currency_code VARCHAR(3) NOT NULL DEFAULT 'NGN',
    tax_identification_number VARCHAR(50),
    pension_provider VARCHAR(100),
    pension_number VARCHAR(50),
    nhf_number VARCHAR(50),
    bank_name VARCHAR(100),
    bank_account_number VARCHAR(50),
    bank_sort_code VARCHAR(20),
    date_of_birth DATE,
    date_joined DATE NOT NULL,
    date_terminated DATE,
    is_active BOOLEAN NOT NULL DEFAULT true,
    paye_state VARCHAR(50),
    tax_exemptions JSONB NOT NULL DEFAULT '[]',
    cost_center_id UUID,
    created_by UUID NOT NULL REFERENCES users(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE(company_id, employee_number)
);

CREATE TABLE payroll_runs (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    company_id UUID NOT NULL REFERENCES companies(id) ON DELETE CASCADE,
    branch_id UUID REFERENCES branches(id),
    run_name VARCHAR(100) NOT NULL,
    period_month INT NOT NULL CHECK (period_month BETWEEN 1 AND 12),
    period_year INT NOT NULL,
    status VARCHAR(20) NOT NULL DEFAULT 'draft' CHECK (status IN (
        'draft', 'processing', 'completed', 'approved', 'posted', 'cancelled'
    )),
    total_gross NUMERIC(18,2) NOT NULL DEFAULT 0,
    total_deductions NUMERIC(18,2) NOT NULL DEFAULT 0,
    total_net NUMERIC(18,2) NOT NULL DEFAULT 0,
    total_paye NUMERIC(18,2) NOT NULL DEFAULT 0,
    total_pension NUMERIC(18,2) NOT NULL DEFAULT 0,
    total_nhf NUMERIC(18,2) NOT NULL DEFAULT 0,
    total_nsitf NUMERIC(18,2) NOT NULL DEFAULT 0,
    total_itf NUMERIC(18,2) NOT NULL DEFAULT 0,
    employee_count INT NOT NULL DEFAULT 0,
    payment_date DATE,
    approved_by UUID REFERENCES users(id),
    approved_at TIMESTAMPTZ,
    posted_by UUID REFERENCES users(id),
    posted_at TIMESTAMPTZ,
    journal_header_id UUID,
    created_by UUID NOT NULL REFERENCES users(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE(company_id, period_month, period_year, run_name)
);

CREATE TABLE payslips (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    company_id UUID NOT NULL REFERENCES companies(id) ON DELETE CASCADE,
    payroll_run_id UUID NOT NULL REFERENCES payroll_runs(id) ON DELETE CASCADE,
    employee_id UUID NOT NULL REFERENCES employees(id),
    employee_number VARCHAR(50) NOT NULL,
    employee_name VARCHAR(200) NOT NULL,
    period_month INT NOT NULL,
    period_year INT NOT NULL,
    basic_salary NUMERIC(18,2) NOT NULL DEFAULT 0,
    housing_allowance NUMERIC(18,2) NOT NULL DEFAULT 0,
    transport_allowance NUMERIC(18,2) NOT NULL DEFAULT 0,
    meal_allowance NUMERIC(18,2) NOT NULL DEFAULT 0,
    utility_allowance NUMERIC(18,2) NOT NULL DEFAULT 0,
    entertainment_allowance NUMERIC(18,2) NOT NULL DEFAULT 0,
    medical_allowance NUMERIC(18,2) NOT NULL DEFAULT 0,
    education_allowance NUMERIC(18,2) NOT NULL DEFAULT 0,
    other_allowances NUMERIC(18,2) NOT NULL DEFAULT 0,
    other_allowances_desc TEXT,
    total_gross NUMERIC(18,2) NOT NULL DEFAULT 0,
    paye_tax NUMERIC(18,2) NOT NULL DEFAULT 0,
    pension_employee NUMERIC(18,2) NOT NULL DEFAULT 0,
    nhf_employee NUMERIC(18,2) NOT NULL DEFAULT 0,
    nsitf_employee NUMERIC(18,2) NOT NULL DEFAULT 0,
    itf_employee NUMERIC(18,2) NOT NULL DEFAULT 0,
    loan_deduction NUMERIC(18,2) NOT NULL DEFAULT 0,
    salary_advance NUMERIC(18,2) NOT NULL DEFAULT 0,
    other_deductions NUMERIC(18,2) NOT NULL DEFAULT 0,
    other_deductions_desc TEXT,
    total_deductions NUMERIC(18,2) NOT NULL DEFAULT 0,
    net_pay NUMERIC(18,2) NOT NULL DEFAULT 0,
    employer_pension NUMERIC(18,2) NOT NULL DEFAULT 0,
    employer_nsitf NUMERIC(18,2) NOT NULL DEFAULT 0,
    employer_itf NUMERIC(18,2) NOT NULL DEFAULT 0,
    employer_nhf NUMERIC(18,2) NOT NULL DEFAULT 0,
    paye_details JSONB NOT NULL DEFAULT '{}',
    status VARCHAR(20) NOT NULL DEFAULT 'draft' CHECK (status IN ('draft', 'approved', 'paid', 'cancelled')),
    paid_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE(payroll_run_id, employee_id)
);

CREATE TABLE leave_balances (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    company_id UUID NOT NULL REFERENCES companies(id) ON DELETE CASCADE,
    employee_id UUID NOT NULL REFERENCES employees(id) ON DELETE CASCADE,
    leave_type VARCHAR(30) NOT NULL CHECK (leave_type IN (
        'annual', 'sick', 'maternity', 'paternity', 'compassionate', 'casual', 'study'
    )),
    year INT NOT NULL,
    total_days NUMERIC(5,1) NOT NULL DEFAULT 0,
    days_taken NUMERIC(5,1) NOT NULL DEFAULT 0,
    days_remaining NUMERIC(5,1) NOT NULL DEFAULT 0,
    days_carried_forward NUMERIC(5,1) NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE(employee_id, leave_type, year)
);

CREATE TABLE loan_deductions (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    company_id UUID NOT NULL REFERENCES companies(id) ON DELETE CASCADE,
    employee_id UUID NOT NULL REFERENCES employees(id) ON DELETE CASCADE,
    loan_type VARCHAR(30) NOT NULL CHECK (loan_type IN (
        'salary_advance', 'staff_loan', 'emergency_loan', 'car_loan', 'housing_loan'
    )),
    principal_amount NUMERIC(18,2) NOT NULL,
    interest_rate NUMERIC(5,2) NOT NULL DEFAULT 0,
    monthly_deduction NUMERIC(18,2) NOT NULL,
    outstanding_balance NUMERIC(18,2) NOT NULL,
    total_months INT NOT NULL,
    remaining_months INT NOT NULL,
    start_date DATE NOT NULL,
    end_date DATE NOT NULL,
    status VARCHAR(20) NOT NULL DEFAULT 'active' CHECK (status IN ('active', 'completed', 'defaulted', 'written_off')),
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_employees_company ON employees(company_id, is_active);
CREATE INDEX idx_employees_department ON employees(company_id, department_id);
CREATE INDEX idx_employees_number ON employees(company_id, employee_number);
CREATE INDEX idx_payroll_runs_company ON payroll_runs(company_id, period_year, period_month);
CREATE INDEX idx_payroll_runs_status ON payroll_runs(company_id, status);
CREATE INDEX idx_payslips_run ON payslips(payroll_run_id);
CREATE INDEX idx_payslips_employee ON payslips(employee_id, period_year, period_month);
CREATE INDEX idx_leave_balances_employee ON leave_balances(employee_id, year);
CREATE INDEX idx_loan_deductions_employee ON loan_deductions(employee_id, status);
