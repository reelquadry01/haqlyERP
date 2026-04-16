-- File storage and document attachments
-- Author: Quadri Atharu

CREATE TABLE IF NOT EXISTS document_attachments (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    company_id UUID NOT NULL REFERENCES companies(id) ON DELETE CASCADE,
    entity_type VARCHAR(30) NOT NULL CHECK (entity_type IN ('journal', 'invoice', 'bill', 'voucher', 'asset', 'employee', 'report', 'other')),
    entity_id UUID NOT NULL,
    file_name VARCHAR(500) NOT NULL,
    file_path VARCHAR(1000) NOT NULL,
    file_size BIGINT NOT NULL DEFAULT 0,
    mime_type VARCHAR(200) NOT NULL DEFAULT 'application/octet-stream',
    description TEXT,
    uploaded_by UUID REFERENCES users(id),
    is_deleted BOOLEAN NOT NULL DEFAULT FALSE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_doc_attachments_entity ON document_attachments(entity_type, entity_id);
CREATE INDEX idx_doc_attachments_company ON document_attachments(company_id);
CREATE INDEX idx_doc_attachments_deleted ON document_attachments(is_deleted);
