# HAQLY ERP — Agent Development Guide

## Quick Start
- Rust backend: `cd src-server && cargo run`
- Python AI engine: `cd src-ai && uvicorn naija_finance_accounting_intelligence_engine.api.main:app --port 8200`
- Next.js frontend: `cd src-web && npm run dev`
- Tauri desktop: `cd src-tauri && cargo tauri dev`

## Testing
- Rust unit tests: `cd src-server && cargo test`
- Rust integration tests: `cd src-server && cargo test -- --ignored` (requires PostgreSQL)
- Python tests: `cd src-ai && pytest -v`

## Linting
- Rust: `cargo clippy -- -D warnings && cargo fmt --all -- --check`
- Python: `cd src-ai && ruff check .`

## Important Conventions
- Table name is `chart_of_accounts` NOT `accounts`
- Column is `entry_number` NOT `number`
- Status values are lowercase: `draft`, `posted`, `approved` NOT `DRAFT`, `POSTED`
- Author comment: `// Author: Quadri Atharu` at top of every Rust file, `# Author: Quadri Atharu` for Python

## Security
- JWT uses RS256 (RSA-2048), keys auto-generated on first run
- Secrets are fail-secure: app exits if env vars missing in production
- E-invoicing credentials encrypted at rest with AES-256-GCM
- Audit trail uses SHA-256 chain hashing for tamper-evidence
- RBAC enforced via RbacLayer middleware

## Architecture
- Axum backend on port 8100, Python AI on 8200, Next.js on 3001
- Database: PostgreSQL primary, SQLite fallback for desktop
- All financial operations use database transactions
- LRU caching for accounts/posting rules/tax configs
