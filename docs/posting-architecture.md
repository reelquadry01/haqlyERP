# HAQLY ERP — Posting Engine Architecture

**Author:** Quadri Atharu  
**Version:** 0.1.0  
**Date:** 2026-04-13

---

## 1. Purpose

The posting engine is the core financial integrity layer of HAQLY ERP. It transforms draft journal entries into immutable, balanced ledger records. Every financial transaction in the system — sales invoices, purchase bills, tax computations, loan disbursements, asset depreciation — ultimately flows through the posting engine as journal entry lines.

---

## 2. Core Principles

1. **Double-Entry Enforcement:** Every posting must produce balanced debit and credit lines. The engine rejects any entry where `SUM(debits) ≠ SUM(credits)`.
2. **Immutability After Posting:** Posted journal entries and their lines are frozen. No UPDATE or DELETE is permitted on posted records. Corrections require a reversing entry that references the original.
3. **Fiscal Period Gating:** Postings are accepted only into open fiscal periods. Once a period is closed, no postings can target it.
4. **Atomicity:** Each posting is a single database transaction. If any line fails, the entire posting rolls back.
5. **Idempotency:** Re-posting the same draft entry is a no-op if it was already posted. The engine checks the entry's status before proceeding.
6. **Audit Trail:** Every posting generates an audit log entry with the full before/after state.

---

## 3. Data Model

### 3.1 Journal Entry

```sql
CREATE TABLE journal_entries (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    company_id      UUID NOT NULL REFERENCES companies(id),
    entry_number    VARCHAR(50) NOT NULL,
    entry_date      DATE NOT NULL,
    fiscal_period_id UUID NOT NULL REFERENCES fiscal_periods(id),
    description     TEXT,
    reference_type  VARCHAR(50),  -- 'sales_invoice', 'purchase_bill', 'tax_return', etc.
    reference_id    UUID,         -- FK to the originating document
    status          VARCHAR(20) NOT NULL DEFAULT 'draft',
                        -- 'draft', 'submitted', 'approved', 'posted'
    total_debit     NUMERIC(18,2) NOT NULL DEFAULT 0,
    total_credit    NUMERIC(18,2) NOT NULL DEFAULT 0,
    posted_at       TIMESTAMPTZ,
    posted_by       UUID REFERENCES users(id),
    version         INTEGER NOT NULL DEFAULT 1,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    deleted_at      TIMESTAMPTZ,
    UNIQUE(company_id, entry_number)
);
```

### 3.2 Journal Entry Line

```sql
CREATE TABLE journal_entry_lines (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    journal_entry_id UUID NOT NULL REFERENCES journal_entries(id),
    line_number     INTEGER NOT NULL,
    account_id      UUID NOT NULL REFERENCES accounts(id),
    description     TEXT,
    debit           NUMERIC(18,2) NOT NULL DEFAULT 0,
    credit          NUMERIC(18,2) NOT NULL DEFAULT 0,
    cost_center_id  UUID REFERENCES cost_centers(id),
    tax_code        VARCHAR(20),
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE(journal_entry_id, line_number)
);
```

### 3.3 Account Balance (Materialized View Target)

```sql
CREATE TABLE account_balances (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    company_id      UUID NOT NULL REFERENCES companies(id),
    account_id      UUID NOT NULL REFERENCES accounts(id),
    fiscal_period_id UUID NOT NULL REFERENCES fiscal_periods(id),
    debit_total     NUMERIC(18,2) NOT NULL DEFAULT 0,
    credit_total    NUMERIC(18,2) NOT NULL DEFAULT 0,
    balance         NUMERIC(18,2) NOT NULL DEFAULT 0,
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE(company_id, account_id, fiscal_period_id)
);
```

---

## 4. Posting Workflow

```
Draft → Validate → Submit → Approve → Post → Update Balances → Audit Log
                                    ↓
                              (Rejection → back to Draft)
```

### 4.1 Draft

User creates a journal entry with one or more lines. Lines may have zero amounts initially. The entry is in `draft` status and can be freely edited.

### 4.2 Validate

Before submission, the engine validates:
- At least 2 lines exist.
- Every line has a non-zero debit OR credit (never both).
- `SUM(debits) = SUM(credits)` — the entry balances.
- All referenced accounts exist and are active.
- The `entry_date` falls within an open fiscal period.
- No duplicate account on the same line.

### 4.3 Submit

Entry status changes to `submitted`. It can no longer be edited by the original creator. It enters the approval queue.

### 4.4 Approve

A user with `journal:post` permission reviews and approves. Status changes to `approved`. In single-approval configurations, submission and approval can be combined.

### 4.5 Post

The engine executes the following in a single database transaction:

1. **Status check:** Confirm entry is `approved`.
2. **Balance re-verification:** Recompute `SUM(debits) = SUM(credits)`.
3. **Fiscal period check:** Confirm the target period is still open.
4. **Lock the entry:** Set `status = 'posted'`, `posted_at = now()`, `posted_by = current_user`.
5. **Update account balances:** For each line:
   ```sql
   INSERT INTO account_balances (company_id, account_id, fiscal_period_id, debit_total, credit_total, balance)
   VALUES ($1, $2, $3, $debit, $credit, $debit - $credit)
   ON CONFLICT (company_id, account_id, fiscal_period_id)
   DO UPDATE SET
       debit_total = account_balances.debit_total + EXCLUDED.debit_total,
       credit_total = account_balances.credit_total + EXCLUDED.credit_total,
       balance = account_balances.balance + EXCLUDED.balance,
       updated_at = now();
   ```
6. **Audit log:** Insert audit record with full entry snapshot.

### 4.6 Reversal

A posted entry can be reversed:
1. Create a new journal entry referencing the original via `reversal_of` column.
2. Copy all lines, swapping debits and credits.
3. The reversal goes through the full draft → submit → approve → post cycle.
4. The original entry's `reversed_by` field is set to the reversal entry ID.

---

## 5. Posting from Source Documents

Each ERP module has a posting adapter that converts its domain documents into journal entries:

| Source Module | Debit Accounts | Credit Accounts | Trigger |
|---|---|---|---|
| Sales Invoice | Accounts Receivable | Revenue, VAT Output | Invoice approval |
| Sales Receipt | Cash/Bank | Accounts Receivable | Receipt recording |
| Purchase Bill | Expense/VAT Input | Accounts Payable | Bill approval |
| Purchase Payment | Accounts Payable | Cash/Bank | Payment recording |
| Tax Return | VAT Payable | Cash/Bank | Return filing |
| Asset Depreciation | Depreciation Expense | Accumulated Depreciation | Period close |
| Loan Disbursement | Cash/Bank | Loan Payable | Loan approval |
| Loan Payment | Loan Payable, Interest Expense | Cash/Bank | Payment schedule |

Each adapter:
1. Creates the journal entry header with `reference_type` and `reference_id`.
2. Generates lines based on the document's financial breakdown.
3. Submits the entry to the posting engine.
4. Returns the journal entry ID for traceability.

---

## 6. Fiscal Period Management

```sql
CREATE TABLE fiscal_periods (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    company_id      UUID NOT NULL REFERENCES companies(id),
    name            VARCHAR(100) NOT NULL,
    start_date      DATE NOT NULL,
    end_date        DATE NOT NULL,
    status          VARCHAR(20) NOT NULL DEFAULT 'open',
                        -- 'open', 'closed', 'locked'
    closed_at       TIMESTAMPTZ,
    closed_by       UUID REFERENCES users(id),
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE(company_id, name)
);
```

**Closing a period:**
1. Verify all entries in the period are posted (no drafts/submitted/approved remaining).
2. Set period status to `closed`.
3. Generate closing journal entries (income summary → retained earnings).
4. Record the closing in the audit log.

---

## 7. Concurrency Control

- **Optimistic locking** on journal entries via the `version` column.
- **Advisory locks** on fiscal periods during posting to prevent race conditions on balance updates.
- **SERIALIZABLE isolation level** for the posting transaction when high contention is detected.

---

## 8. Error Handling

| Error | Code | Response |
|---|---|---|
| Entry does not balance | `UNBALANCED_ENTRY` | 400 — Return debit/credit totals |
| Fiscal period closed | `PERIOD_CLOSED` | 409 — Return period details |
| Entry already posted | `ALREADY_POSTED` | 409 — Idempotent, return existing entry |
| Account inactive | `ACCOUNT_INACTIVE` | 400 — Return account ID |
| Permission denied | `FORBIDDEN` | 403 — Return required permission |
| Concurrent edit conflict | `VERSION_CONFLICT` | 409 — Return current version |
