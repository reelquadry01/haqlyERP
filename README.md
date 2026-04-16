[![Rust](https://img.shields.io/badge/Rust-1.77%2B-dea584?logo=rust&logoColor=white)](https://www.rust-lang.org/)
[![TypeScript](https://img.shields.io/badge/TypeScript-5.x-3178c6?logo=typescript&logoColor=white)](https://www.typescriptlang.org/)
[![Python](https://img.shields.io/badge/Python-3.11%2B-3776ab?logo=python&logoColor=white)](https://www.python.org/)
[![Tauri](https://img.shields.io/badge/Tauri-v2-ffc131?logo=tauri&logoColor=black)](https://v2.tauri.app/)
[![License](https://img.shields.io/badge/License-Proprietary-red)](./LICENSE)

# HAQLY ERP — Full Finance Department in Software Form

**Author:** Quadri Atharu

Nigeria's first desktop-native ERP with AI-powered accounting intelligence, NRS e-invoicing, and full compliance with the **Nigeria Tax Reform Acts 2025**.

## Architecture

```
  ┌──────────────────── TAURI v2 DESKTOP ────────────────────┐
  │                                                          │
  │  ┌──────────┐   ┌──────────────┐   ┌─────────────────┐  │
  │  │ Next.js  │   │  Axum + SQLx │   │ Python FastAPI  │  │
  │  │ 15 (UI)  │   │  (Biz Logic) │   │  (AI Engine)    │  │
  │  └────┬─────┘   └──────┬───────┘   └───────┬─────────┘  │
  │       │                │                    │            │
  │       └────────────────┼────────────────────┘            │
  │                        │                    │             │
  │                  ┌─────▼─────┐        ┌─────▼─────┐      │
  │                  │ PostgreSQL │        │  Ollama    │      │
  │                  │ / SQLite  │        │  (LLM)     │      │
  │                  └───────────┘        └───────────┘      │
  └──────────────────────────────────────────────────────────┘
```

## Tech Stack

| Layer | Technology |
|-------|-----------|
| Desktop Shell | Tauri v2 (Rust) |
| Backend | Axum + SQLx (Rust) |
| Database | PostgreSQL 15+ / SQLite |
| Frontend | Next.js 15 + TypeScript |
| AI Engine | Python FastAPI + Ollama |
| E-Invoicing | NRS/FIRS API |

## Nigeria Tax Reform Acts 2025 Compliance

HAQLY ERP is the only desktop ERP with built-in compliance for the **Nigeria Tax Reform Acts 2025**:

| Tax | Rates / Rules |
|-----|--------------|
| **PAYE** | 0% / 15% / 20% / 25% / 30% / 35% (progressive bands) |
| **CIT** | Small (≤₦50M) = 0%, Medium (₦50M–₦250M) = 15%, Large (>₦250M) = 25% |
| **VAT** | 7.5% (registration threshold ₦50M) |
| **WHT** | 5% / 10% + 5% for individual recipients |
| **Education Tax** | 1% |
| **CGT** | Progressive 10% / 15% / 20% |
| **FIRS → NRS** | Full rename & integration with Nigeria Revenue Service |

## Features

- **Chart of Accounts** — Multi-level IFRS-aligned hierarchy
- **Journal Entries** — Double-entry with AI-assisted classification
- **VAT / WHT / CIT / PAYE** — Auto-calculation, filing templates, e-invoicing
- **E-Invoicing (NRS)** — Validate → Sign → Confirm → Download → Report
- **OCR Document Intelligence** — Scan → LLM extract → auto-post → approval
- **Payroll** — PAYE, pension, NHF, NSITF deductions
- **Fixed Assets** — Depreciation (straight-line, reducing balance, units-of-production)
- **CRM** — Customer & vendor lifecycle management
- **Business Intelligence** — CFA-level ratios, trend analysis, peer comparison
- **6 AI Agents** — Journal, Ledger, Tax, Reporting, Finance, Audit
- **IFRS Compliance** — IFRS 9, 15, 16; IAS 2, 12, 16, 37
- **Multi-Company** — Consolidation, intercompany elimination, minority interest
- **Licensing** — Hardware-bound activation with online/offline validation

## Industry Profiles

Oil & Gas · Manufacturing · Banking · Insurance · Retail · Telecom · Agriculture · Construction · Logistics · Healthcare · Education · Government · NGO · Technology · Automotive

## AI Agents

| Agent | Function |
|-------|----------|
| **Journal Agent** | Auto-classifies transactions, suggests account postings |
| **Ledger Agent** | Reconciles entries, detects imbalances |
| **Tax Agent** | Computes PAYE/CIT/VAT/WHT/CGT per 2025 reform acts |
| **Reporting Agent** | Generates IFRS financial statements |
| **Finance Agent** | CFA-level analysis, DCF, NPV, IRR, WACC |
| **Audit Agent** | Trail generation, sampling, exception detection |

## Prerequisites

| Requirement | Version |
|-------------|---------|
| Rust | 1.77+ |
| Node.js | 18+ |
| Python | 3.11+ |
| PostgreSQL | 15+ (or SQLite) |
| VS Build Tools | Required on Windows |
| Ollama | Latest (for local LLM) |

## Quick Start

```bash
# Start infrastructure
docker-compose up -d postgres redis

# Start backend
cd src-server && cargo run

# Start frontend
cd src-web && npm install && npm run dev

# Start AI engine
cd src-ai && pip install -e . && uvicorn naija_finance_accounting_intelligence_engine.api.main:app --port 8200

# Launch desktop app
cd src-tauri && cargo tauri dev
```

## Build for Production

```bash
cargo tauri build
```

## Project Structure

```
haqlyERP/
├── src-tauri/      # Tauri desktop shell (Rust)
├── src-server/     # Axum backend (Rust)
├── src-web/        # Next.js 15 frontend
├── src-ai/         # Python AI engine
├── docs/           # Architecture & API docs
├── scripts/        # Seed, migration, setup
└── .github/        # CI/CD workflows
```

## Security

| Measure | Implementation |
|---------|---------------|
| Authentication | RS256 JWT with rotation |
| Encryption | AES-256-GCM at rest |
| Authorization | Role-Based Access Control (RBAC) |
| Audit Trail | SHA-256 chained, tamper-evident |
| Key Management | Zeroize on drop |
| API Protection | Rate limiting + request signing |

## License

Proprietary — All rights reserved.

## Repository

[https://github.com/reelquadry01/haqlyERP](https://github.com/reelquadry01/haqlyERP)
