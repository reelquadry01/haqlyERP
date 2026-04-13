# HAQLY ERP — Architecture Document

**Author:** Quadri Atharu  
**Version:** 0.1.0  
**Date:** 2026-04-13

---

## 1. Overview

HAQLY ERP is a desktop-first enterprise resource planning application built for Nigerian businesses. It combines a Rust-based backend server, a Tauri desktop shell, a Next.js 15 web frontend, and a Python AI sidecar into a single deployable unit. The system handles full-cycle accounting, tax compliance, e-invoicing (NRS/FIRS), document intelligence, and AI-powered analysis.

---

## 2. Tech Stack

| Layer | Technology | Purpose |
|---|---|---|
| Desktop Shell | Tauri 2 (Rust) | Native window, IPC, auto-update, sidecar management |
| Backend Server | Axum (Rust) | REST API, business logic, PostgreSQL access |
| Database | PostgreSQL 16 | Persistent storage, ACID transactions, JSONB columns |
| Migrations | sqlx + custom runner | Schema versioning, seed data |
| Frontend | Next.js 15 (React 19) | SPA with static export for Tauri webview |
| Styling | CSS custom properties | Dark theme, no Tailwind dependency |
| AI Sidecar | Python 3.12 + FastAPI | OCR pipeline, LLM agents, NLP classification |
| E-Invoicing | NRS/FIRS SDK | Nigerian e-invoicing IRN generation and transmission |
| Auth | JWT + MFA (TOTP) | Token-based auth with multi-factor support |
| CI/CD | GitHub Actions | Build, test, release across platforms |

---

## 3. Module Map

| Module | API Prefix | Description |
|---|---|---|
| Auth | `/api/v1/auth` | Login, MFA, session management, RBAC |
| Companies | `/api/v1/companies` | Multi-company context, settings |
| Accounting | `/api/v1/accounting` | Chart of accounts, fiscal periods |
| Journal Entries | `/api/v1/journal-entries` | Double-entry posting engine |
| Sales | `/api/v1/sales` | Invoices, receipts, credit notes |
| Purchases | `/api/v1/purchases` | Bills, payments, debit notes |
| Inventory | `/api/v1/inventory` | Stock tracking, warehouses, movements |
| Tax | `/api/v1/tax` | VAT, WHT, CIT computations, returns |
| Fixed Assets | `/api/v1/fixed-assets` | Depreciation, disposal, revaluation |
| Loans | `/api/v1/loans` | Loan amortization, interest schedules |
| Reports | `/api/v1/reports` | Financial statements, trial balance |
| Admin | `/api/v1/admin` | User management, roles, audit log |
| E-Invoicing | `/api/v1/einvoicing` | IRN generation, FIRS transmission |
| AI Engine | `/api/v1/ai` | Document analysis, classification, insights |
| Documents | `/api/v1/documents` | File storage, OCR pipeline trigger |

---

## 4. Boundary Rules

### 4.1 API Boundary

- The Next.js frontend communicates **only** with the Axum backend at `http://localhost:8100/api/v1`.
- The Axum backend communicates with PostgreSQL via SQLx connection pool.
- The Axum backend communicates with the Python AI sidecar at `http://localhost:8200/api/v1`.
- The Tauri shell communicates with the Axum backend for system-level operations (file dialogs, printing).
- No frontend code directly accesses the database or the AI sidecar.

### 4.2 Module Boundary

- Each module owns its own service layer, repository layer, and DTOs.
- Cross-module access must go through a module's public service interface, not its repository.
- No circular dependencies between modules. Dependency direction: Auth → Accounting → Journal → (Sales, Purchases, etc.).
- Shared utilities (date formatting, currency, number-to-words) live in `src-server/shared/`.

### 4.3 Data Boundary

- All database mutations happen inside the Axum server. No external writes.
- The Python sidecar is read-only from the main database; it writes only to its own `ai_results` and `ai_datasets` tables.
- File uploads are stored on the local filesystem under `~/.haqly/uploads/` with database references.

---

## 5. Data Integrity Principles

1. **Double-Entry Integrity:** Every journal entry must balance (debits = credits) before posting. The posting engine enforces this at the database transaction level.
2. **Immutability of Posted Records:** Once a journal entry is posted, its lines cannot be modified. Corrections require a reversing entry.
3. **Fiscal Period Locking:** Closed fiscal periods reject any posting attempt. Period closing is an idempotent, auditable operation.
4. **Optimistic Concurrency:** All update operations use a `version` column. Concurrent edits are rejected with a 409 Conflict.
5. **Audit Trail:** Every mutation inserts a row into `audit_log` with user ID, timestamp, previous value (JSON diff), and action type.
6. **Cascade Safety:** No ON DELETE CASCADE on business tables. Deletion is always soft (`deleted_at` column) and restricted by business rules.
7. **Currency Precision:** All monetary values stored as `NUMERIC(18,2)`. Computed values use `NUMERIC(18,4)` internally, rounded to 2 decimals for display.

---

## 6. Role-Based Access Control (RBAC)

### 6.1 Role Hierarchy

```
SuperAdmin
  └── Admin
       └── Manager
            └── Accountant
                 └── Clerk
```

### 6.2 Permission Model

Permissions are strings in the format `{module}:{action}`:

| Module | Permissions |
|---|---|
| dashboard | `dashboard:view` |
| accounting | `accounting:view`, `accounting:create`, `accounting:edit`, `accounting:delete` |
| journal | `journal:view`, `journal:create`, `journal:edit`, `journal:delete`, `journal:post`, `journal:reverse` |
| sales | `sales:view`, `sales:create`, `sales:edit`, `sales:delete`, `sales:approve` |
| purchases | `purchases:view`, `purchases:create`, `purchases:edit`, `purchases:delete`, `purchases:approve` |
| inventory | `inventory:view`, `inventory:create`, `inventory:edit`, `inventory:adjust` |
| tax | `tax:view`, `tax:compute`, `tax:file` |
| assets | `assets:view`, `assets:create`, `assets:depreciate`, `assets:dispose` |
| loans | `loans:view`, `loans:create`, `loans:amortize` |
| reports | `reports:view`, `reports:export` |
| admin | `admin:view`, `admin:users`, `admin:roles`, `admin:settings`, `admin:audit` |
| einvoicing | `einvoicing:view`, `einvoicing:create`, `einvoicing:transmit` |
| ai | `ai:view`, `ai:analyze`, `ai:configure` |
| documents | `documents:view`, `documents:upload`, `documents:ocr` |

### 6.3 Enforcement

- Backend: Middleware extracts JWT claims, loads user permissions from cache, and checks each route handler.
- Frontend: Navigation items are filtered by `requiredPermission` against the user's permission set. API calls that return 403 display an inline error.

---

## 7. Folder Layout

```
haqlyERP-desktop/
├── src-tauri/                    # Tauri desktop shell (Rust)
│   ├── src/
│   │   ├── main.rs               # Tauri app entry
│   │   ├── commands.rs           # Tauri IPC commands
│   │   └── sidecar.rs            # Sidecar lifecycle management
│   ├── Cargo.toml
│   └── tauri.conf.json
│
├── src-server/                   # Axum backend (Rust)
│   ├── src/
│   │   ├── main.rs               # Server startup, router setup
│   │   ├── config.rs             # Environment configuration
│   │   ├── db.rs                  # Connection pool, migrations
│   │   ├── middleware/
│   │   │   ├── auth.rs           # JWT verification middleware
│   │   │   ├── rbac.rs           # Permission check middleware
│   │   │   └── company.rs        # Company context middleware
│   │   ├── modules/
│   │   │   ├── auth/             # Authentication module
│   │   │   ├── companies/        # Multi-company management
│   │   │   ├── accounting/       # Chart of accounts, periods
│   │   │   ├── journal/          # Journal entries & posting engine
│   │   │   ├── sales/            # Sales invoices & receipts
│   │   │   ├── purchases/        # Purchase bills & payments
│   │   │   ├── inventory/        # Stock & warehouse management
│   │   │   ├── tax/              # VAT, WHT, CIT computation
│   │   │   ├── fixed_assets/    # Asset depreciation & disposal
│   │   │   ├── loans/            # Loan management & amortization
│   │   │   ├── reports/          # Financial statement generation
│   │   │   ├── admin/            # User/role management, audit
│   │   │   ├── einvoicing/       # NRS/FIRS e-invoicing
│   │   │   ├── ai/              # AI engine proxy endpoints
│   │   │   └── documents/       # Document storage & OCR trigger
│   │   └── shared/
│   │       ├── error.rs          # Unified error types
│   │       ├── types.rs          # Common type aliases
│   │       ├── currency.rs       # Naira formatting, number-to-words
│   │       └── dates.rs         # Date utilities, fiscal calendar
│   ├── migrations/                # PostgreSQL schema migrations
│   └── Cargo.toml
│
├── src-web/                      # Next.js 15 frontend
│   ├── app/
│   │   ├── layout.tsx            # Root layout
│   │   ├── page.tsx              # Redirect to /dashboard
│   │   ├── globals.css           # Dark theme, CSS variables
│   │   ├── dashboard/            # Dashboard page
│   │   └── login/                # Login page
│   ├── components/
│   │   ├── workspace-shell.tsx   # Main layout shell
│   │   ├── login-screen.tsx      # Login form component
│   │   ├── app-splash.tsx        # Loading splash screen
│   │   └── ui/                   # Reusable UI components
│   │       ├── kpi-card.tsx
│   │       ├── data-table.tsx
│   │       ├── brand-lockup.tsx
│   │       ├── status-badge.tsx
│   │       ├── approval-stepper.tsx
│   │       ├── empty-state.tsx
│   │       └── action-menu.tsx
│   ├── lib/
│   │   ├── api.ts                # API client utilities
│   │   ├── session.ts            # Token & company session
│   │   └── navigation.ts         # Nav items & permissions
│   ├── next.config.ts
│   ├── tsconfig.json
│   └── package.json
│
├── src-ai/                       # Python AI sidecar
│   ├── main.py                   # FastAPI entry point
│   ├── agents/                   # Agent definitions
│   ├── ocr/                      # OCR pipeline
│   ├── llm/                      # LLM integration
│   ├── classification/          # Document classification
│   ├── datasets/                 # Training data management
│   ├── requirements.txt
│   └── setup.py
│
├── docs/                         # Project documentation
├── scripts/                      # Utility scripts
├── .github/workflows/            # CI/CD pipelines
├── Cargo.toml                    # Workspace Cargo config
├── docker-compose.yml            # Docker services
├── .env.example                  # Environment template
└── README.md
```

---

## 8. Deployment Shape

### 8.1 Desktop (Primary)

The Tauri application bundles:
1. The Axum server binary (started as a sidecar)
2. The Next.js static export (served by Axum or loaded by Tauri webview)
3. The Python AI engine (optional sidecar, started on demand)

On launch:
1. Tauri starts the Axum sidecar on port 8100.
2. Tauri opens a webview pointing at `http://localhost:8100` or loads the static export directly.
3. If AI features are enabled, Tauri starts the Python sidecar on port 8200.

### 8.2 Docker (Development/Server)

```yaml
services:
  postgres:
    image: postgres:16
    ports: ["5432:5432"]
  api:
    build: ./src-server
    ports: ["8100:8100"]
    depends_on: [postgres]
  web:
    build: ./src-web
    ports: ["3001:3001"]
  ai:
    build: ./src-ai
    ports: ["8200:8200"]
```

### 8.3 Platform Targets

| Platform | Tauri Target | Installer |
|---|---|---|
| Windows x64 | `x86_64-pc-windows-msvc` | NSIS (.exe) / MSI |
| macOS (Apple Silicon) | `aarch64-apple-darwin` | DMG |
| macOS (Intel) | `x86_64-apple-darwin` | DMG |
| Linux x64 | `x86_64-unknown-linux-gnu` | AppImage / .deb |

---

## 9. Security Model

- All API routes require JWT except `/api/v1/auth/login` and `/api/v1/auth/mfa/verify`.
- JWTs expire after 8 hours. Refresh tokens are stored as HTTP-only cookies in the web context.
- Passwords are hashed with Argon2id.
- MFA uses TOTP (RFC 6238) with 6-digit codes and 30-second step.
- Company context is enforced at the middleware level — all data queries include `company_id` filtering.
- The audit log is append-only and cannot be truncated by any role except SuperAdmin with explicit confirmation.

---

## 10. Configuration

All configuration is loaded from environment variables or a `.env` file:

| Variable | Default | Description |
|---|---|---|
| `DATABASE_URL` | — | PostgreSQL connection string |
| `JWT_SECRET` | — | Signing key for JWT tokens |
| `JWT_EXPIRY_HOURS` | `8` | Token expiration time |
| `AI_SIDECAR_URL` | `http://localhost:8200` | Python AI engine URL |
| `UPLOAD_DIR` | `~/.haqly/uploads` | File upload directory |
| `NRS_API_KEY` | — | NRS/FIRS API key |
| `NRS_BASE_URL` | `https://einvoice.nrs.ng` | NRS API base URL |
| `LOG_LEVEL` | `info` | Logging verbosity |
