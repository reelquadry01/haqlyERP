-- HAQLY ERP - Encrypted e-invoicing credentials & MFA recovery codes
-- Author: Quadri Atharu

ALTER TABLE einvoice_credentials
    ADD COLUMN IF NOT EXISTS api_key_nonce BYTEA,
    ADD COLUMN IF NOT EXISTS api_key_tag BYTEA,
    ADD COLUMN IF NOT EXISTS api_secret_nonce BYTEA,
    ADD COLUMN IF NOT EXISTS api_secret_tag BYTEA,
    ADD COLUMN IF NOT EXISTS client_secret_nonce BYTEA,
    ADD COLUMN IF NOT EXISTS client_secret_tag BYTEA;

ALTER TABLE users
    ADD COLUMN IF NOT EXISTS mfa_recovery_codes JSONB;
