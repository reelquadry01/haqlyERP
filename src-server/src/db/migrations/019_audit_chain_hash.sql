-- Author: Quadri Atharu

ALTER TABLE audit_logs
    ADD COLUMN previous_hash TEXT,
    ADD COLUMN entry_hash TEXT NOT NULL DEFAULT '',
    ADD COLUMN chain_valid BOOLEAN DEFAULT true;

CREATE TABLE audit_chain_verification (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    verified_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    last_valid_entry_id UUID,
    broken_at_entry_id UUID,
    is_intact BOOLEAN NOT NULL,
    total_entries INTEGER NOT NULL DEFAULT 0,
    verified_by TEXT NOT NULL DEFAULT 'system'
);

CREATE INDEX idx_audit_logs_entry_hash ON audit_logs(entry_hash);
CREATE INDEX idx_audit_chain_verification_verified ON audit_chain_verification(verified_at DESC);
