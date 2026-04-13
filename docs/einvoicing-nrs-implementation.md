# HAQLY ERP — NRS/FIRS E-Invoicing Implementation Guide

**Author:** Quadri Atharu  
**Version:** 0.1.0  
**Date:** 2026-04-13

---

## 1. Overview

Nigeria's Federal Inland Revenue Service (FIRS) mandates electronic invoicing through the National Revenue System (NRS). HAQLY ERP integrates directly with the NRS API to generate Invoice Reference Numbers (IRNs), transmit invoices, and receive acknowledgements — ensuring every sales invoice is FIRS-compliant before it reaches the customer.

---

## 2. Regulatory Context

- **Finance Act 2021** mandates e-invoicing for businesses above ₦25M annual turnover.
- **FIRS E-Invoicing Regulations 2023** define the technical and procedural requirements.
- Every invoice must have a unique IRN issued by the NRS before transmission to the buyer.
- IRN format: `IRN-{BusinessID}-{YYYYMMDD}-{SerialNumber}`.
- Invoices must be transmitted within 48 hours of issuance.
- Credit notes and debit notes also require IRNs.

---

## 3. Architecture

```
Sales Invoice Created
        │
        ▼
┌──────────────────┐
│  HAQLY ERP       │
│  Sales Module    │
└──────┬───────────┘
       │ POST /api/v1/einvoicing/generate-irn
       ▼
┌──────────────────┐
│  Axum Backend    │──────► NRS API (https://api.nrs.ng/v1)
│  E-Invoicing     │◄────── IRN Response
│  Module          │
└──────┬───────────┘
       │ Update invoice with IRN + QR data
       ▼
┌──────────────────┐
│  PostgreSQL      │  einvoicing_records table
└──────────────────┘
       │
       ▼
┌──────────────────┐
│  Transmission    │──────► NRS API (batch or real-time)
│  Engine          │◄────── Acknowledgement
└──────────────────┘
```

---

## 4. NRS API Contract

### 4.1 Authentication

All NRS API calls require an API key passed in the `Authorization` header:

```
Authorization: Bearer {NRS_API_KEY}
```

API keys are issued by NRS after business registration. The key is stored in HAQLY's environment as `NRS_API_KEY`.

### 4.2 Base URL

| Environment | Base URL |
|---|---|
| Production | `https://api.nrs.ng/v1` |
| Sandbox | `https://sandbox.api.nrs.ng/v1` |

Controlled by `NRS_BASE_URL` environment variable.

### 4.3 Generate IRN

**POST** `/irn/generate`

Request:
```json
{
  "businessId": "BN123456789",
  "invoiceType": "STANDARD",
  "invoiceNumber": "INV-2026-001",
  "invoiceDate": "2026-04-13",
  "sellerTIN": "12345678-0001",
  "buyerTIN": "98765432-0001",
  "invoiceValue": 1500000.00,
  "vatAmount": 225000.00,
  "currency": "NGN"
}
```

Response (200):
```json
{
  "irn": "IRN-BN123456789-20260413-001",
  "qrCodeData": "IRN-BN123456789-20260413-001|INV-2026-001|2026-04-13|1500000.00|12345678-0001",
  "acknowledgementId": "ACK-20260413-ABC123",
  "generatedAt": "2026-04-13T10:30:00Z",
  "validUntil": "2026-04-15T23:59:59Z"
}
```

Error Response (400):
```json
{
  "errorCode": "INVALID_TIN",
  "message": "Seller TIN format is invalid",
  "details": {
    "field": "sellerTIN"
  }
}
```

### 4.4 Transmit Invoice

**POST** `/invoices/transmit`

Request:
```json
{
  "irn": "IRN-BN123456789-20260413-001",
  "invoiceType": "STANDARD",
  "invoiceNumber": "INV-2026-001",
  "invoiceDate": "2026-04-13",
  "seller": {
    "tin": "12345678-0001",
    "name": "HAQLY Technologies Ltd",
    "address": "12 Marina Street, Lagos",
    "email": "billing@haqly.com",
    "phone": "+234-812-345-6789"
  },
  "buyer": {
    "tin": "98765432-0001",
    "name": "Acme Corp Nigeria Ltd",
    "address": "45 Allen Avenue, Ikeja, Lagos",
    "email": "ap@acme.ng",
    "phone": "+234-809-876-5432"
  },
  "lineItems": [
    {
      "description": "ERP Software License - Annual",
      "quantity": 1,
      "unitPrice": 1500000.00,
      "totalAmount": 1500000.00,
      "vatRate": 7.5,
      "vatAmount": 112500.00,
      "whtRate": 5.0,
      "whtAmount": 75000.00
    }
  ],
  "totals": {
    "subtotal": 1500000.00,
    "vatTotal": 112500.00,
    "whtTotal": 75000.00,
    "grandTotal": 1612500.00,
    "amountDue": 1537500.00
  },
  "paymentTerms": "Net 30",
  "currency": "NGN"
}
```

Response (200):
```json
{
  "transmissionId": "TXN-20260413-XYZ789",
  "irn": "IRN-BN123456789-20260413-001",
  "status": "ACCEPTED",
  "timestamp": "2026-04-13T10:31:00Z",
  "validations": [
    { "rule": "TIN_FORMAT", "status": "PASS" },
    { "rule": "VAT_COMPUTATION", "status": "PASS" },
    { "rule": "DUPLICATE_CHECK", "status": "PASS" }
  ]
}
```

### 4.5 Check Invoice Status

**GET** `/invoices/status/{irn}`

Response (200):
```json
{
  "irn": "IRN-BN123456789-20260413-001",
  "status": "ACCEPTED",
  "transmittedAt": "2026-04-13T10:31:00Z",
  "acknowledgedAt": "2026-04-13T10:32:15Z",
  "buyerNotifiedAt": "2026-04-13T10:33:00Z"
}
```

### 4.6 Cancel IRN

**POST** `/irn/cancel`

Request:
```json
{
  "irn": "IRN-BN123456789-20260413-001",
  "reason": "ISSUED_IN_ERROR",
  "remarks": "Invoice was created with incorrect amounts"
}
```

Response (200):
```json
{
  "irn": "IRN-BN123456789-20260413-001",
  "status": "CANCELLED",
  "cancelledAt": "2026-04-13T11:00:00Z"
}
```

---

## 5. HAQLY Internal API

### 5.1 Generate IRN for Invoice

**POST** `/api/v1/einvoicing/generate-irn`

Request:
```json
{
  "invoiceId": "uuid-of-sales-invoice"
}
```

Response (200):
```json
{
  "irn": "IRN-BN123456789-20260413-001",
  "qrCodeBase64": "data:image/png;base64,...",
  "acknowledgementId": "ACK-20260413-ABC123",
  "status": "irn_generated"
}
```

### 5.2 Transmit Invoice to NRS

**POST** `/api/v1/einvoicing/transmit`

Request:
```json
{
  "invoiceId": "uuid-of-sales-invoice"
}
```

Response (200):
```json
{
  "transmissionId": "TXN-20260413-XYZ789",
  "irn": "IRN-BN123456789-20260413-001",
  "status": "transmitted",
  "nrsStatus": "ACCEPTED"
}
```

### 5.3 Get E-Invoicing Status

**GET** `/api/v1/einvoicing/status/{invoiceId}`

Response (200):
```json
{
  "invoiceId": "uuid",
  "invoiceNumber": "INV-2026-001",
  "irn": "IRN-BN123456789-20260413-001",
  "status": "transmitted",
  "nrsStatus": "ACCEPTED",
  "generatedAt": "2026-04-13T10:30:00Z",
  "transmittedAt": "2026-04-13T10:31:00Z",
  "acknowledgedAt": "2026-04-13T10:32:15Z"
}
```

---

## 6. Database Schema

```sql
CREATE TABLE einvoicing_records (
    id                  UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    company_id          UUID NOT NULL REFERENCES companies(id),
    invoice_id          UUID NOT NULL REFERENCES sales_invoices(id),
    invoice_number      VARCHAR(50) NOT NULL,
    irn                 VARCHAR(100),
    qr_code_data        TEXT,
    acknowledgement_id  VARCHAR(100),
    status              VARCHAR(30) NOT NULL DEFAULT 'pending',
                           -- 'pending', 'irn_generated', 'transmitted', 'accepted',
                           -- 'rejected', 'cancelled', 'error'
    nrs_status          VARCHAR(30),
    nrs_error_code      VARCHAR(50),
    nrs_error_message   TEXT,
    transmission_id     VARCHAR(100),
    transmitted_at      TIMESTAMPTZ,
    acknowledged_at     TIMESTAMPTZ,
    retry_count         INTEGER NOT NULL DEFAULT 0,
    last_retry_at       TIMESTAMPTZ,
    request_payload     JSONB,
    response_payload    JSONB,
    created_at          TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at          TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE(company_id, invoice_id),
    UNIQUE(company_id, irn)
);
```

---

## 7. QR Code Generation

Every e-invoice must display a QR code containing:
- IRN
- Invoice number
- Invoice date
- Total amount
- Seller TIN

The QR code is generated using the `qrcode` npm package on the frontend and also stored as base64 PNG on the backend. The QR code is embedded in the PDF output of the invoice.

---

## 8. Transmission Strategy

### 8.1 Real-Time (Default)

When a sales invoice is approved, the system immediately:
1. Generates an IRN via NRS API.
2. Transmits the full invoice payload.
3. Updates the invoice with the IRN and transmission status.
4. If the NRS API is unreachable, falls back to queued transmission.

### 8.2 Queued (Fallback)

If real-time transmission fails, the record enters a retry queue:
- Retry schedule: 1min, 5min, 15min, 1hr, 6hrs (exponential backoff).
- Maximum retries: 10.
- After max retries, the record status becomes `error` and an admin notification is generated.

### 8.3 Batch

For high-volume businesses, batch transmission is available:
- Invoices are collected in a queue.
- A cron job runs every 15 minutes to transmit all pending records.
- Batch endpoint: `POST /api/v1/einvoicing/batch-transmit`

---

## 9. Compliance Checklist

- [ ] IRN generated for every sales invoice before customer delivery
- [ ] QR code displayed on invoice PDF
- [ ] Invoice transmitted within 48 hours of issuance
- [ ] Credit notes and debit notes also have IRNs
- [ ] Cancelled IRNs reported to NRS with reason
- [ ] Seller and buyer TINs validated against FIRS database
- [ ] VAT computation matches NRS validation rules
- [ ] Retention of e-invoicing records for minimum 6 years
- [ ] Audit trail for all IRN generation and transmission events
