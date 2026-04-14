-- HAQLY ERP - Schema Reconciliation
-- Author: Quadri Atharu
-- Fixes critical mismatches between Rust code and SQL migrations

-- 1. Rename journal_headers.number to entry_number (code uses entry_number)
ALTER TABLE journal_headers RENAME COLUMN number TO entry_number;

-- 2. Rename journal_headers.reference_id to reference (code uses reference)
ALTER TABLE journal_headers RENAME COLUMN reference_id TO reference;

-- 3. Add missing columns to journal_headers
ALTER TABLE journal_headers ADD COLUMN IF NOT EXISTS journal_type VARCHAR(50);
ALTER TABLE journal_headers ADD COLUMN IF NOT EXISTS source_document_number VARCHAR(100);

-- 4. Add missing columns to journal_lines
ALTER TABLE journal_lines ADD COLUMN IF NOT EXISTS currency_code VARCHAR(3) NOT NULL DEFAULT 'NGN';
ALTER TABLE journal_lines ADD COLUMN IF NOT EXISTS exchange_rate NUMERIC(18,6) NOT NULL DEFAULT 1;
ALTER TABLE journal_lines ADD COLUMN IF NOT EXISTS department_id UUID REFERENCES departments(id) ON DELETE SET NULL;

-- 5. Add missing columns to users
ALTER TABLE users ADD COLUMN IF NOT EXISTS company_id UUID REFERENCES companies(id);
ALTER TABLE users ADD COLUMN IF NOT EXISTS phone VARCHAR(50);
ALTER TABLE users ADD COLUMN IF NOT EXISTS avatar_url VARCHAR(500);

-- 6. Rename users.last_login to last_login_at (code uses last_login_at)
ALTER TABLE users RENAME COLUMN last_login TO last_login_at;

-- 7. Add missing columns to sessions
ALTER TABLE sessions ADD COLUMN IF NOT EXISTS token TEXT NOT NULL DEFAULT '';
ALTER TABLE sessions ADD COLUMN IF NOT EXISTS ip_address VARCHAR(45);
ALTER TABLE sessions ADD COLUMN IF NOT EXISTS user_agent TEXT;

-- Make token unique after adding with default
CREATE UNIQUE INDEX IF NOT EXISTS idx_sessions_token ON sessions(token);

-- 8. Rename posting_audit to posting_audits (code uses posting_audits)
ALTER TABLE posting_audit RENAME TO posting_audits;

-- Add missing columns to posting_audits (code references these)
ALTER TABLE posting_audits ADD COLUMN IF NOT EXISTS company_id UUID REFERENCES companies(id);
ALTER TABLE posting_audits ADD COLUMN IF NOT EXISTS journal_header_id UUID REFERENCES journal_headers(id);
ALTER TABLE posting_audits ADD COLUMN IF NOT EXISTS status VARCHAR(20) NOT NULL DEFAULT 'success';
ALTER TABLE posting_audits ADD COLUMN IF NOT EXISTS posted_by UUID REFERENCES users(id);
ALTER TABLE posting_audits ADD COLUMN IF NOT EXISTS posted_at TIMESTAMPTZ NOT NULL DEFAULT now();
ALTER TABLE posting_audits ADD COLUMN IF NOT EXISTS error_message TEXT;

-- 9. Add missing columns to accounting_periods
ALTER TABLE accounting_periods ADD COLUMN IF NOT EXISTS company_id UUID REFERENCES companies(id);
ALTER TABLE accounting_periods ADD COLUMN IF NOT EXISTS updated_at TIMESTAMPTZ NOT NULL DEFAULT now();

-- 10. Add missing columns to user_roles
ALTER TABLE user_roles ADD COLUMN IF NOT EXISTS id UUID DEFAULT uuid_generate_v4();
ALTER TABLE user_roles ADD COLUMN IF NOT EXISTS assigned_at TIMESTAMPTZ NOT NULL DEFAULT now();

-- 11. Fix status value casing (migrations use UPPERCASE, code uses lowercase)
ALTER TABLE journal_headers DROP CONSTRAINT IF EXISTS journal_headers_status_check;
ALTER TABLE journal_headers ADD CONSTRAINT journal_headers_status_check CHECK (status IN ('draft', 'validated', 'submitted', 'approved', 'posted', 'reversed', 'cancelled'));
ALTER TABLE journal_headers ALTER COLUMN status SET DEFAULT 'draft';

ALTER TABLE accounting_periods DROP CONSTRAINT IF EXISTS accounting_periods_status_check;
ALTER TABLE accounting_periods ADD CONSTRAINT accounting_periods_status_check CHECK (status IN ('open', 'closed', 'locked'));
ALTER TABLE accounting_periods ALTER COLUMN status SET DEFAULT 'open';

-- 12. Drop old unique constraint that referenced 'number' and recreate for 'entry_number'
ALTER TABLE journal_headers DROP CONSTRAINT IF EXISTS journal_headers_company_id_number_key;
CREATE UNIQUE INDEX IF NOT EXISTS journal_headers_company_id_entry_number_key ON journal_headers(company_id, entry_number);

-- 13. Performance indexes
CREATE INDEX IF NOT EXISTS idx_accounting_periods_company_date ON accounting_periods(company_id, start_date, end_date);
CREATE INDEX IF NOT EXISTS idx_posting_rules_lookup ON posting_rules(company_id, module, transaction_type, is_active);
CREATE INDEX IF NOT EXISTS idx_journal_headers_status ON journal_headers(company_id, status) WHERE status = 'draft';
CREATE INDEX IF NOT EXISTS idx_sessions_token ON sessions(token);
