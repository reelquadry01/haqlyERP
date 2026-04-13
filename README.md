# HAQLY ERP

**Nigeria-Compliant | IFRS-Compliant | CFA-Level Intelligence | NRS E-Invoicing | Desktop-First**

A production-grade Enterprise Resource Planning desktop application built with Rust (Tauri + Axum), featuring Smart AI Accounting Intelligence, NRS/FIRS E-Invoicing compliance, and Nigerian Finance Intelligence.

## Author

**Quadri Atharu**

## Architecture

```
┌──────────────────────────────────────────────────────────┐
│                 TAURI DESKTOP SHELL (Rust)                │
│  ┌──────────┐  ┌──────────────┐  ┌───────────────────┐  │
│  │ Next.js  │  │ Rust Backend │  │ Python AI Engine  │  │
│  │ WebView  │  │ (Axum +SQLx) │  │ (FastAPI Sidecar) │  │
│  │  (UI)    │  │ (Biz Logic)  │  │ (OCR/Tax/IFRS/)   │  │
│  └─────┬────┘  └──────┬───────┘  └────────┬──────────┘  │
│        └───────────────┴──────────────────┘              │
│                    │                                     │
│              ┌─────▼─────┐                                │
│              │ PostgreSQL │                                │
│              └───────────┘                                │
│  ┌────────────────────────────────────────────────────┐  │
│  │  OCR Pipeline (Tesseract + Ollama LLM)            │  │
│  │  NRS E-Invoicing (FIRS client + orchestration)    │  │
│  └────────────────────────────────────────────────────┘  │
└──────────────────────────────────────────────────────────┘
```

## Tech Stack

| Layer | Technology |
|-------|-----------|
| Desktop Shell | Tauri v2 (Rust) |
| Backend | Axum + SQLx (Rust) |
| Database | PostgreSQL (local/remote) |
| Frontend | Next.js 15 + TypeScript |
| AI Engine | Python FastAPI sidecar |
| OCR | Tesseract + Ollama LLM |
| E-Invoicing | NRS/FIRS API integration |

## Features

- **Full Accounting Lifecycle** — Transaction recognition → Journal → Ledger → Trial Balance → Financial Statements → Closing
- **NRS E-Invoicing** — Validate → Sign → Confirm → Download → Report via FIRS API
- **OCR Document Intelligence** — Scan receipts/invoices → LLM extraction → auto-suggest account postings → approval workflow
- **Nigerian Tax Engine** — VAT (7.5%), WHT (5-10%), CIT (20-30%), Education Tax (2%), CGT (10%), Stamp Duties
- **IFRS Compliance** — IFRS 9, IFRS 15, IFRS 16, IAS 2, IAS 16, IAS 12, IAS 37
- **15 Industry Profiles** — Oil & Gas, Manufacturing, Banking, Insurance, Retail, Telecom, Agriculture, Construction, Logistics, Healthcare, Education, Government, NGO, Technology, Automotive
- **CFA-Level Financial Analysis** — Liquidity, Profitability, Leverage, Efficiency ratios + trend + peer comparison
- **Valuation Engine** — NPV, IRR, WACC, DCF
- **Risk Management** — Credit, Liquidity, Market risk dashboards
- **Multi-Company** — Consolidation, intercompany elimination, minority interest
- **Foreign Currency** — Multi-currency with FX gain/loss recognition
- **Treasury Management** — Cash position, bank reconciliation, loan management
- **Budgeting & Forecasting** — Annual budgets, rolling forecasts, variance analysis
- **Audit Intelligence** — Trail generation, sampling, exception detection, working papers
- **6 AI Agents** — Journal, Ledger, Tax, Reporting, Finance, Audit

## Getting Started

### Prerequisites

- Rust (1.75+)
- Node.js (20+)
- PostgreSQL (15+)
- Python (3.11+)
- Tauri CLI v2
- Ollama (for local LLM)

### Development

```bash
# Start PostgreSQL
docker-compose up -d postgres redis

# Run Rust backend
cd src-server && cargo run

# Run Next.js frontend
cd src-web && npm install && npm run dev

# Run Python AI engine
cd src-ai && pip install -e . && uvicorn naija_finance_accounting_intelligence_engine.api.main:app --port 8200

# Run Tauri desktop app
cd src-tauri && cargo tauri dev
```

### Build Desktop App

```bash
cd src-tauri && cargo tauri build
```

## Project Structure

```
haqlyERP/
├── src-tauri/         # Tauri desktop shell (Rust)
├── src-server/        # Axum backend (Rust)
├── src-web/           # Next.js frontend
├── src-ai/            # Python AI intelligence engine
├── docs/              # Architecture & implementation docs
├── scripts/           # Seed, migration, setup scripts
└── .github/           # CI/CD workflows
```

## License

Proprietary — All rights reserved.
