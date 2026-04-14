-- HAQLY ERP - OCR Document Processing
-- Author: Quadri Atharu

CREATE TABLE ocr_documents (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    company_id UUID NOT NULL REFERENCES companies(id) ON DELETE CASCADE,
    original_filename VARCHAR(500) NOT NULL,
    file_path VARCHAR(1000) NOT NULL,
    file_size BIGINT NOT NULL,
    mime_type VARCHAR(100) NOT NULL,
    document_type VARCHAR(50) NOT NULL CHECK (document_type IN (
        'invoice', 'receipt', 'bank_statement', 'purchase_order',
        'delivery_note', 'tax_certificate', 'pay_slip', 'contract',
        'utility_bill', 'id_document', 'other'
    )),
    ocr_status VARCHAR(20) NOT NULL DEFAULT 'pending' CHECK (ocr_status IN (
        'pending', 'processing', 'completed', 'failed', 'requires_review'
    )),
    ocr_engine VARCHAR(30) NOT NULL DEFAULT 'tesseract' CHECK (ocr_engine IN ('tesseract', 'paddleocr', 'aws_textract', 'google_vision')),
    raw_text TEXT,
    confidence_score NUMERIC(5,2),
    page_count INT NOT NULL DEFAULT 1,
    language VARCHAR(10) NOT NULL DEFAULT 'eng',
    is_verified BOOLEAN NOT NULL DEFAULT false,
    verified_by UUID REFERENCES users(id),
    verified_at TIMESTAMPTZ,
    linked_document_id UUID,
    linked_document_type VARCHAR(50),
    processing_started_at TIMESTAMPTZ,
    processing_completed_at TIMESTAMPTZ,
    processing_duration_ms INT,
    error_message TEXT,
    uploaded_by UUID NOT NULL REFERENCES users(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE ocr_extraction_fields (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    ocr_document_id UUID NOT NULL REFERENCES ocr_documents(id) ON DELETE CASCADE,
    field_name VARCHAR(100) NOT NULL,
    field_value TEXT NOT NULL,
    confidence_score NUMERIC(5,2) NOT NULL,
    bounding_box JSONB,
    page_number INT NOT NULL DEFAULT 1,
    is_edited BOOLEAN NOT NULL DEFAULT false,
    original_value TEXT,
    edited_by UUID REFERENCES users(id),
    edited_at TIMESTAMPTZ,
    field_type VARCHAR(30) NOT NULL DEFAULT 'text' CHECK (field_type IN (
        'text', 'number', 'date', 'amount', 'email', 'phone', 'tin', 'percentage'
    )),
    is_verified BOOLEAN NOT NULL DEFAULT false,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE(ocr_document_id, field_name, page_number)
);

CREATE INDEX idx_ocr_documents_company ON ocr_documents(company_id, document_type);
CREATE INDEX idx_ocr_documents_status ON ocr_documents(company_id, ocr_status);
CREATE INDEX idx_ocr_documents_uploaded ON ocr_documents(uploaded_by, created_at);
CREATE INDEX idx_ocr_extraction_document ON ocr_extraction_fields(ocr_document_id);
CREATE INDEX idx_ocr_extraction_field_name ON ocr_extraction_fields(ocr_document_id, field_name);
CREATE INDEX idx_ocr_extraction_low_confidence ON ocr_extraction_fields(ocr_document_id) WHERE confidence_score < 70.0;
