# HAQLY ERP — OCR Pipeline Architecture

**Author:** Quadri Atharu  
**Version:** 0.1.0  
**Date:** 2026-04-13

---

## 1. Overview

The OCR pipeline transforms uploaded document images and PDFs into structured financial data ready for journal entry creation. It combines traditional OCR extraction with LLM-based reasoning to achieve high accuracy on Nigerian financial documents (invoices, receipts, bank statements, purchase orders).

---

## 2. Pipeline Stages

```
File Intake → Preprocessing → Extraction → Classification → LLM Reasoning → Structured Output → Review → Approval → Journal Entry
```

### 2.1 File Intake

**Endpoint:** `POST /api/v1/documents/upload`

Accepted formats:
- PDF (single and multi-page)
- JPEG, PNG, TIFF, BMP
- HEIC (Apple devices)

The upload handler:
1. Validates file type and size (max 20MB per file).
2. Generates a unique document ID.
3. Stores the file at `~/.haqly/uploads/{company_id}/{year}/{month}/{doc_id}.{ext}`.
4. Creates a `documents` record with status `uploaded`.
5. Triggers the pipeline asynchronously.

```sql
CREATE TABLE documents (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    company_id      UUID NOT NULL REFERENCES companies(id),
    filename        VARCHAR(255) NOT NULL,
    original_name   VARCHAR(255) NOT NULL,
    mime_type       VARCHAR(100) NOT NULL,
    file_size       BIGINT NOT NULL,
    file_path       TEXT NOT NULL,
    status          VARCHAR(30) NOT NULL DEFAULT 'uploaded',
                       -- 'uploaded', 'preprocessing', 'extracting', 'classifying',
                       -- 'reasoning', 'structured', 'reviewing', 'approved', 'rejected', 'error'
    pipeline_step   VARCHAR(30),
    pipeline_error  TEXT,
    uploaded_by     UUID REFERENCES users(id),
    reviewed_by     UUID REFERENCES users(id),
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);
```

### 2.2 Preprocessing

Goal: Normalize the document for optimal OCR extraction.

Steps:
1. **PDF rasterization** — Convert PDF pages to 300 DPI PNG images using `pdf2image` (poppler).
2. **Deskewing** — Detect and correct page rotation using `skimage`.
3. **Binarization** — Convert to grayscale with adaptive thresholding for text clarity.
4. **Denoising** — Remove scanner artifacts using `opencv` fastNlMeansDenoising.
5. **Region detection** — Identify header, body, and footer regions for structured extraction.

Output: Preprocessed images stored at `~/.haqly/uploads/{company_id}/preprocessed/{doc_id}_page_{n}.png`.

The document status updates to `preprocessing` → `extracting`.

### 2.3 Extraction

Goal: Convert preprocessed images into raw text with bounding box coordinates.

Two extraction engines run in parallel:

**Engine A — Tesseract OCR:**
- Language: English + custom Nigerian financial dictionary.
- PSM mode: 6 (assume uniform block of text).
- Output: Text + confidence scores per word.
- Good for: Structured forms, typed documents.

**Engine B — PaddleOCR:**
- Model: PP-OCRv4 (English + multilingual).
- Output: Text + bounding boxes + confidence.
- Good for: Handwritten notes, complex layouts, receipts.

**Ensemble Strategy:**
- For each text region, take the output with higher confidence.
- Where both engines agree (Levenshtein distance < 3), confidence is boosted.
- Where they disagree, both results are preserved for LLM adjudication.

Output: Raw text + per-word confidence + bounding box metadata stored as JSONB.

```sql
CREATE TABLE ocr_results (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    document_id     UUID NOT NULL REFERENCES documents(id),
    page_number     INTEGER NOT NULL DEFAULT 1,
    engine          VARCHAR(20) NOT NULL,
                       -- 'tesseract', 'paddleocr'
    raw_text        TEXT NOT NULL,
    word_data       JSONB,
                       -- [{text, confidence, bbox: {x1,y1,x2,y2}}, ...]
    avg_confidence  REAL,
    processing_time_ms INTEGER,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE(document_id, page_number, engine)
);
```

### 2.4 Classification

Goal: Determine the document type to select the appropriate extraction schema.

**Classification Model:** Fine-tuned DistilBERT on Nigerian financial documents.

Supported document types:
| Type | Code | Key Fields |
|---|---|---|
| Sales Invoice | `sales_invoice` | Invoice number, date, seller, buyer, line items, totals, VAT |
| Purchase Bill | `purchase_bill` | Bill number, date, vendor, line items, totals, VAT, WHT |
| Receipt | `receipt` | Receipt number, date, amount, payer, reference |
| Bank Statement | `bank_statement` | Account number, period, transactions (date, desc, amount, balance) |
| Purchase Order | `purchase_order` | PO number, date, vendor, items, quantities |
| Delivery Note | `delivery_note` | DN number, date, items, quantities, recipient |
| Credit Note | `credit_note` | CN number, date, reference invoice, amounts |

Output: Document type + confidence score stored on the `documents` record.

```sql
ALTER TABLE documents ADD COLUMN doc_type VARCHAR(30);
ALTER TABLE documents ADD COLUMN doc_type_confidence REAL;
```

### 2.5 LLM Reasoning

Goal: Extract structured financial data from raw OCR text using LLM intelligence.

**Engine:** OpenAI GPT-4o-mini (or local Llama 3 via Ollama for offline mode).

**Prompt Architecture:**

```
System: You are a Nigerian financial document parser. Extract structured data from the OCR text.
Return JSON matching the schema for the document type. Handle common OCR errors:
- "N" or "NGN" → Nigerian Naira
- "V.A.T" or "VAT" → Value Added Tax
- "W.H.T" or "WHT" → Withholding Tax
- TIN format: 8 digits + hyphen + 4 digits (e.g., 12345678-0001)

Context: Document type: {doc_type}
Schema: {json_schema}

OCR Text:
{raw_text}

Return only valid JSON.
```

**Fallback Strategy:**
1. Attempt extraction with GPT-4o-mini.
2. If the response fails JSON validation, retry with explicit error feedback.
3. If 3 retries fail, fall back to rule-based extraction (regex patterns for known document templates).
4. Mark confidence as `low` for rule-based extractions.

Output: Structured JSON matching the document type schema.

```sql
CREATE TABLE extraction_results (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    document_id     UUID NOT NULL REFERENCES documents(id),
    doc_type        VARCHAR(30) NOT NULL,
    extracted_data  JSONB NOT NULL,
    confidence      VARCHAR(10) NOT NULL,
                       -- 'high', 'medium', 'low'
    extraction_method VARCHAR(20) NOT NULL,
                       -- 'llm', 'rule_based', 'manual'
    llm_model       VARCHAR(50),
    llm_tokens_used INTEGER,
    processing_time_ms INTEGER,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE(document_id)
);
```

### 2.6 Structured Output

The LLM output is validated against a JSON Schema for the detected document type. Invalid outputs trigger re-extraction.

Example schema for `sales_invoice`:
```json
{
  "type": "object",
  "required": ["invoiceNumber", "invoiceDate", "seller", "lineItems", "totals"],
  "properties": {
    "invoiceNumber": { "type": "string" },
    "invoiceDate": { "type": "string", "format": "date" },
    "seller": {
      "type": "object",
      "required": ["name", "tin"],
      "properties": {
        "name": { "type": "string" },
        "tin": { "type": "string", "pattern": "^\\d{8}-\\d{4}$" },
        "address": { "type": "string" }
      }
    },
    "buyer": {
      "type": "object",
      "properties": {
        "name": { "type": "string" },
        "tin": { "type": "string" },
        "address": { "type": "string" }
      }
    },
    "lineItems": {
      "type": "array",
      "items": {
        "type": "object",
        "required": ["description", "quantity", "unitPrice", "totalAmount"],
        "properties": {
          "description": { "type": "string" },
          "quantity": { "type": "number" },
          "unitPrice": { "type": "number" },
          "totalAmount": { "type": "number" },
          "vatRate": { "type": "number" },
          "vatAmount": { "type": "number" }
        }
      }
    },
    "totals": {
      "type": "object",
      "required": ["subtotal", "grandTotal"],
      "properties": {
        "subtotal": { "type": "number" },
        "vatTotal": { "type": "number" },
        "whtTotal": { "type": "number" },
        "grandTotal": { "type": "number" }
      }
    }
  }
}
```

### 2.7 Review

The structured output is presented to an accountant for review in the HAQLY frontend.

**Review UI Features:**
- Side-by-side view: Original document image + extracted data.
- Highlighted regions showing which OCR text maps to each field.
- Editable fields for corrections.
- Confidence indicators (green/yellow/red) per field.
- "Approve" and "Reject" buttons.

Document status transitions: `structured` → `reviewing`.

### 2.8 Approval

After review and any corrections, the accountant approves the extraction.

- Document status: `reviewing` → `approved`.
- The approved data is ready for journal entry creation.

### 2.9 Journal Entry Creation

**Endpoint:** `POST /api/v1/documents/{documentId}/create-journal-entry`

The system:
1. Reads the approved extraction result.
2. Maps the document type to the appropriate posting adapter.
3. Generates a draft journal entry with the extracted amounts.
4. Links the journal entry to the source document via `reference_type` and `reference_id`.
5. The journal entry follows the standard draft → submit → approve → post workflow.

---

## 3. Python Sidecar Implementation

The OCR pipeline runs as a Python FastAPI sidecar at `http://localhost:8200`.

### 3.1 API Endpoints

| Method | Path | Description |
|---|---|---|
| POST | `/api/v1/ocr/process` | Start pipeline for a document |
| GET | `/api/v1/ocr/status/{document_id}` | Get pipeline status |
| POST | `/api/v1/ocr/reprocess/{document_id}` | Retry from a specific step |
| GET | `/api/v1/ocr/result/{document_id}` | Get extraction result |
| POST | `/api/v1/classify` | Classify document type from text |
| POST | `/api/v1/extract` | Run LLM extraction on text |

### 3.2 Process Request

```json
{
  "documentId": "uuid",
  "filePath": "/path/to/document.pdf",
  "companyId": "uuid",
  "options": {
    "engines": ["tesseract", "paddleocr"],
    "llmModel": "gpt-4o-mini",
    "skipSteps": [],
    "maxRetries": 3
  }
}
```

---

## 4. Error Handling

| Error | Handling |
|---|---|
| File too large | Reject at upload, return 413 |
| Unsupported format | Reject at upload, return 415 |
| OCR engine failure | Retry with alternate engine, mark as `error` if both fail |
| LLM API timeout | Retry up to 3 times with exponential backoff |
| Invalid LLM JSON output | Retry with error feedback, fall back to rule-based |
| Schema validation failure | Mark confidence as `low`, require manual review |
| Disk space full | Queue document, alert admin |

---

## 5. Performance Targets

| Metric | Target |
|---|---|
| Single-page invoice processing | < 15 seconds |
| Multi-page bank statement (10 pages) | < 60 seconds |
| Classification accuracy | > 95% |
| Field extraction accuracy (high confidence) | > 90% |
| End-to-end (upload to structured output) | < 30 seconds (single page) |
