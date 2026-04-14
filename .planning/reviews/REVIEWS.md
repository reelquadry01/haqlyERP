---
project: HAQLY ERP
reviewer: OpenCode AI Reviewer
reviewed_at: 2026-04-14
commits_reviewed: [86f9860, 1cee96c, dc77e6c, f050b31, bec63b7]
files_reviewed: 40+
---

# Cross-AI Plan Review — HAQLY ERP Full Project

## 1. Summary

HAQLY ERP is an ambitious Nigeria-compliant, IFRS-compliant desktop ERP built with a 3-tier architecture (Tauri/Axum/Python). The project demonstrates impressive scope — 15 migrations, 6 AI agents, full Nigerian tax engine, e-invoicing integration, and comprehensive security documentation. However, the implementation has critical gaps between its well-designed documentation and actual code: schema-migration mismatches that would prevent compilation, security features described in docs but not implemented in code, pervasive use of `unwrap()` in financial code paths, and virtually zero test coverage on the Rust backend. The project is at an early-prototype stage, not production-ready. The most urgent issues are: (1) database schema vs. code model mismatches, (2) JWT using HS256 instead of RS256, (3) e-invoicing credentials stored unencrypted, (4) no database transactions in multi-step financial operations, and (5) missing RBAC enforcement middleware.

## 2. Architecture Review

The 3-tier Tauri/Axum/Python architecture is architecturally sound for a desktop ERP. Axum provides a robust Rust backend, Tauri offers native desktop integration, and the Python sidecar is a pragmatic choice for ML/OCR workloads.

### 2.1 Strengths

- **Clean boundary separation**: Frontend ↔ Axum ↔ Python, no cross-boundary shortcuts in the design
- **Double-entry posting engine** with idempotency keys and posting rules is well-architected (`posting_service.rs`)
- **Journal lifecycle** (Draft → Validated → Submitted → Approved → Posted → Reversed) follows proper accounting workflow
- **Nigerian tax engine** covers VAT, WHT, CIT, Education Tax, CGT with correct rates and brackets
- **Migration strategy**: 15 numbered migrations provide logical schema evolution
- **Security documentation** is comprehensive — security-plan.md and security-hardening.md are professional-grade
- **CI/CD pipeline**: Both CI and release workflows are configured across 4 platforms

### 2.2 Concerns

- **CRITICAL — Schema/code mismatch**: Code queries `accounts` table but migration 003 creates `chart_of_accounts`. Code inserts `company_id` into `users` but migration 001 has no such column. Code inserts `token` into `sessions` but migration 001 has no such column. Journal headers: code uses `entry_number` but migration defines `number`. These mismatches mean `cargo check` will fail with the actual database schema.
- **HIGH — No PostgreSQL bundling**: Desktop app requires users to independently install and configure PostgreSQL 16. The `SidecarManager` starts Axum and Python but not the database. This is a deployment blocker.
- **HIGH — Python sidecar requires local Python**: `python_engine::locate_python()` searches for system Python — no embedded Python or PyInstaller bundle strategy.
- **MEDIUM — `SidecarManager` uses `std::process::Command`** instead of Tauri's built-in sidecar mechanism (`tauri::api::process::Command::new_sidecar`). This bypasses Tauri's sidecar lifecycle management and security sandboxing.
- **MEDIUM — Sidecar startup on a `std::thread`**: `src-tauri/src/main.rs:109-116` spawns a `std::thread` with a blocking tokio runtime inside Tauri's setup, which is fragile and can deadlock.
- **MEDIUM — No graceful degradation**: If the Python AI sidecar fails to start, AI features silently break. No fallback or user notification.

### 2.3 Suggestions

1. **Resolve schema mismatches** — either rename migration tables to match code, or update code queries. Add `company_id` to `users` and `token` to `sessions` in a new migration.
2. **Bundle PostgreSQL** for desktop — use an embedded PostgreSQL (PG + initdb on first launch) or switch to SQLite for single-user desktop deployments.
3. **Use Tauri sidecar mechanism** — define `haqly-backend` and `haqly-ai-engine` in `tauri.conf.json` `externalBin` and use `tauri::api::process::Command::new_sidecar()`.
4. **Embed Python** via PyOxidizer or ship a standalone Python build; alternatively, rewrite AI engine as an Axum sub-module in Rust.
5. **Add startup health dashboard** — show sidecar status to the user on first launch with clear error messages.

## 3. Security Review

The security documentation is excellent — arguably better than the implementation. The gap between documented security practices and actual code is the core concern.

### 3.1 Applied Fixes Assessment

| Fix | Status | Assessment |
|---|---|---|
| AES-256-GCM field encryption | **Partial** | `encryption_service.rs` implements encrypt/decrypt correctly, but it is never called from any service. E-invoicing credentials, bank account numbers, and PII fields are stored as plaintext. |
| Argon2id password hashing | **Implemented** | `auth_service.rs:264-271` uses argon2id with defaults. However, parameters are not explicitly set to OWASP recommendations (m=65536, t=3, p=4). Uses `Argon2::default()`. |
| Zeroize guard | **Implemented** | `zeroize_guard.rs` provides `SecretString` and `SecretBytes` with `ZeroizeOnDrop`. But these types are never used in `auth_service.rs` or `encryption_service.rs` — JWT secret and encryption keys are stored as plain `String`. |
| Security headers middleware | **Implemented** | `security.rs:70-108` sets HSTS, CSP, X-Frame-Options, X-Content-Type-Options correctly. |
| Request size limit | **Implemented** | `security.rs:110-131` enforces 10MB limit. |
| XSS/SQL injection detection | **Implemented but problematic** | Regex-based detection in `security.rs` only checks query strings, not request bodies. Pattern matching is a poor substitute for parameterized queries (which sqlx already provides). |
| Audit logging layer | **Implemented** | `AuditLayer` is applied in `main.rs:75`. |
| Secure defaults checker | **Implemented** | `secure_defaults.rs` and `audit_security.rs` check configuration at startup, but the audit results are never logged or enforced — the app starts regardless of findings. |

### 3.2 Remaining Vulnerabilities

1. **CRITICAL — JWT uses HS256 (symmetric)**: `auth.rs:32-34` uses `DecodingKey::from_secret()` and `EncodingKey::from_secret()`. The security-hardening.md explicitly recommends RS256. With HS256, any party with the JWT secret can forge tokens.
2. **CRITICAL — E-invoicing credentials stored unencrypted**: `einvoicing_service.rs:46` — `let encrypted_secret = req.client_secret.clone()` just copies the plaintext. The variable name is misleading. Migration `010_einvoicing.sql:29-30` stores `api_key` and `api_secret` as plain VARCHAR. FIRS client at `firs_client.rs:85-89` sends raw credentials in HTTP headers.
3. **CRITICAL — RBAC middleware not applied**: `main.rs:71-77` applies AuthLayer and AuditLayer but no RBAC/permission middleware. Any authenticated user can access any endpoint regardless of role.
4. **HIGH — Client-side token in localStorage**: `api.ts:17-18` stores JWT in `localStorage`, vulnerable to XSS. Security-hardening.md specifies httpOnly cookies.
5. **HIGH — CSP allows `unsafe-eval`**: `tauri.conf.json:31` CSP includes `script-src 'self' 'unsafe-inline' 'unsafe-eval'`. Security-hardening.md states "No `eval()`, `Function()`". This defeats the CSP purpose.
6. **HIGH — MFA implementation is incorrect**: `auth_service.rs:283-300` implements TOTP using a custom SHA1 hash instead of HMAC-SHA1. The function `hmac_sha1` at line 303-310 just computes SHA1(key || message) — that's not HMAC. This will produce wrong TOTP codes. Use the `totp-rs` crate instead.
7. **HIGH — Empty updater public key**: `tauri.conf.json:90` — `"pubkey": ""`. Unsigned updates will be accepted, enabling supply-chain attacks.
8. **HIGH — Recovery codes stored in plaintext**: `auth_service.rs:200-202` generates recovery codes but they're returned in the response without being hashed before storage.
9. **MEDIUM — Python error tracebacks exposed in API**: `base_agent.py:89` includes `traceback` in `ErrorResult.to_dict()`. This leaks internal paths and code structure to API consumers.
10. **MEDIUM — Redis URL but no Redis integration**: `docker-compose.yml` runs Redis, `settings.rs` loads `redis_url`, but no code uses Redis. Rate limiting and session revocation (both requiring Redis) are unimplemented.
11. **MEDIUM — `sessions` table has ON DELETE CASCADE**: Migration `001_init.sql:23` — `REFERENCES users(id) ON DELETE CASCADE`. Deleting a user cascades to delete sessions, destroying audit trail.

### 3.3 Security Suggestions

1. **Switch to RS256 JWT**: Generate RSA keypair at setup, store private key in OS keychain, distribute public key. See security-hardening.md section 2.1 for implementation.
2. **Encrypt e-invoicing credentials**: Call `encryption_service::encrypt_field()` in `einvoicing_service.rs:save_credentials()` before INSERT. Decrypt only at request time in `firs_client.rs`.
3. **Apply RBAC middleware**: Add a `RbacLayer` after `AuthLayer` in `main.rs`. Check `claims.role` against required permission on each route.
4. **Move tokens to secure storage**: Use Tauri's OS keychain integration (`keyring` crate) instead of localStorage.
5. **Fix CSP**: Remove `'unsafe-inline'` and `'unsafe-eval'` from script-src in `tauri.conf.json:31`.
6. **Fix MFA**: Replace custom TOTP with `totp-rs` crate (already available in Rust ecosystem).
7. **Set updater public key**: Generate Ed25519 keypair, bake public key into `tauri.conf.json`.
8. **Hash recovery codes**: Use argon2id to hash recovery codes before DB storage, verify against hash on use.
9. **Implement rate limiting**: Use `tower-governor` crate (referenced in security-hardening.md) on auth endpoints.
10. **Remove ON DELETE CASCADE from sessions**: Change to `ON DELETE SET NULL` or `ON DELETE RESTRICT` to preserve audit trail.

## 4. Completeness Review

### 4.1 Implemented vs Required

| Module | Backend | Frontend | Integration | Status |
|---|---|---|---|---|
| Auth (Login/Register/MFA) | 70% | Login page only | Partial | MFA broken, no RBAC |
| Companies | Migration only | None | None | Skeleton |
| Chart of Accounts | CRUD service | None | None | Basic |
| Journal Entries | Full lifecycle | None | None | Good |
| Posting Engine | Full + rules | N/A | N/A | Good |
| Sales | Migration only | None | None | Skeleton |
| Purchases | Migration only | None | None | Skeleton |
| Inventory | Migration only | None | None | Skeleton |
| Tax Engine | Computation only | None | None | No filing/returns |
| Fixed Assets | Migration only | None | None | Skeleton |
| Loans | Service skeleton | None | None | Partial |
| Payroll | Full compute + GL posting | None | None | Good |
| E-Invoicing | Submit/status flow | None | None | Partial |
| AI Engine | 6 agents defined | None | None | In-memory only |
| Reports | None | None | None | Not started |
| Administration | Users list only | Admin page | None | Minimal |

**Estimated completion: 25-30%** of a production ERP.

### 4.2 Missing Features

1. **Financial reports** — Trial balance, P&L, balance sheet, cash flow statement (documented but not implemented)
2. **Bank reconciliation** — No bank statement import, matching, or reconciliation module
3. **RBAC enforcement** — Roles defined in migration but no middleware checks them
4. **Multi-currency with FX** — Currency columns exist but no FX gain/loss computation
5. **Budget management** — Documented but absent
6. **Audit trail integrity** — Chain hash described in security-plan.md but not implemented
7. **Password reset flow** — `users_handler.rs:59-64` returns "not implemented"
8. **Email notifications** — `lettre` crate in Cargo.toml but no email service
9. **Data export** — CSV and Excel writers in Cargo.toml but no export endpoints
10. **Document/OCR pipeline** — Tesseract references in config but no OCR service

### 4.3 Priority Order

1. **Fix schema/code mismatches** (blocks all development)
2. **Implement RBAC middleware** (security blocker)
3. **Fix JWT to RS256** (security blocker)
4. **Encrypt sensitive fields at rest** (security blocker)
5. **Add database transactions to financial operations** (data integrity)
6. **Implement financial reports** (core value proposition)
7. **Build frontend for accounting/journal modules** (usability)
8. **Bundle PostgreSQL for desktop deployment** (deployment blocker)
9. **Implement password reset and email notifications** (compliance)
10. **Add integration tests** (quality assurance)

## 5. Code Quality Review

### 5.1 Rust Code Quality

**`unwrap()` in production code (57 instances found):**

- `payroll_service.rs:173-178,237-252` — `BigDecimal::from_f64(...).unwrap()` in 16 places. These can panic on NaN/Infinity. Use `unwrap_or(BigDecimal::from(0))` or proper error propagation.
- `posting_service.rs:166` — `exact.into_iter().next().unwrap().clone()` on a filtered iterator that was just checked for `len() == 1`. Safe but fragile — use `first().ok_or_else(...)?` instead.
- `accounting_service.rs:113,115` — `NaiveDate::from_ymd_opt(...).unwrap()` for month boundaries. Safe for valid months but should use `ok_or_else`.
- `performance_monitor.rs:90,107,115,147` — `self.inner.write().unwrap()` and `self.inner.read().unwrap()` on `std::sync::RwLock`. These can panic if a previous holder panicked. Use `tokio::sync::RwLock` instead.
- `payment_vouchers_service.rs:180-181` — `voucher.payee_account.unwrap()` and `voucher.payee_bank_code.unwrap()` — will panic if these optional fields are None.
- `license_service.rs:185` — `chars.chars().nth(idx).unwrap()` — can panic if index out of bounds.

**No database transactions in critical financial paths:**

- `posting_service.rs:61-103` — Inserts journal header and multiple journal lines without a transaction. A failure between lines 82-103 leaves a partially created journal entry with inconsistent data.
- `journals_service.rs:43-82` — Same issue: header + line inserts without transaction.
- `payroll_service.rs:258-280` — Payslip inserts for each employee without transaction. A failure midway leaves a partial payroll run.
- `payroll_service.rs:326-530` — GL posting creates journal header and 5-7 journal lines without transaction. If any line insert fails, the journal is inconsistent.

**N+1 query patterns:**

- `journals_service.rs:441-449` — `list_journals()` fetches all headers, then makes a separate query for each header's lines. Should use a JOIN or batch query.
- `posting_service.rs:83-103` — Journal lines inserted one-by-one in a loop. Should use `query_builder` batch insert.

**Concurrency-unsafe entry number generation:**

- `journals_service.rs:593-602` and `posting_service.rs:386-394` — `generate_entry_number()` uses `COUNT(*) + 1`. Two concurrent requests will generate the same entry number. Use a PostgreSQL sequence or `SERIAL` column instead.

**Schema name mismatches:**

- Code: `accounts` → Migration: `chart_of_accounts`
- Code: `users.company_id` → Migration: no `company_id` column in `users`
- Code: `sessions.token` → Migration: no `token` column in `sessions`
- Code: `journal_headers.entry_number` → Migration: `journal_headers.number`
- Code: `journal_headers.narration` → Migration: `journal_headers.narration` (matches)

### 5.2 Python Code Quality

- `journal_engine.py:46-47` — In-memory `_entries` dict. Not connected to any database. Journal entries are lost on restart. The `database_url` in `config.py` is never used by `JournalEngine`.
- `journal_engine.py:283-287` — `_get_wht_rate()` returns hardcoded 0.05 for all amounts. Ignores the rich WHT rate table defined in `config.py:wht_rates`.
- `api/main.py:76` — CORS `allow_methods=["*"]` and `allow_headers=["*"]`. Should be restricted to specific methods and headers as documented in security-plan.md.
- `config.py:24` — Hardcoded database password `haqly:haqly_secret` in default value. Should fail-secure with no default.
- `base_agent.py:89` — `traceback` included in `ErrorResult.to_dict()` API response. Leaks internal code structure. Should only log server-side.
- `vat.py` — Uses `round(float, 2)` for monetary calculations. Floating-point rounding errors will accumulate. Should use `decimal.Decimal` for all tax computations.

### 5.3 TypeScript Code Quality

- `api.ts:17-18` — JWT stored in `localStorage.removeItem("haqly_token")` and `localStorage.removeItem("haqly_company")`. Vulnerable to XSS. Use Tauri secure storage plugin.
- `api.ts:1` — `BASE_URL` hardcoded to `http://localhost:8100/api/v1`. Should be configurable for different environments.
- `administration/page.tsx:59` — Empty `catch {}` block silently swallows all API errors. Should at minimum log or display error state.
- `administration/page.tsx:71-88` — Extensive inline styles (200+ characters of inline CSS per column). Should use CSS classes or design tokens.
- `page.tsx:7` — Client-side redirect via `window.location.replace("/dashboard")`. Should use Next.js `redirect()` for SSR compatibility.
- No TypeScript error types defined — all API responses are `Response` objects requiring manual `.json()` parsing. Define typed response interfaces.
- No loading states, no error boundaries, no retry logic in any frontend component.

## 6. Performance Review

### 6.1 Backend Performance

- **N+1 queries** in `journals_service.rs:list_journals()` — For 50 journals, this makes 51 SQL queries (1 for headers + 50 for lines). Should use a single JOIN query.
- **No connection pool configuration** — `main.rs:32` calls `create_pool()` but no visible max/min connections, idle timeout, or acquire timeout. Default sqlx pool may be too small for concurrent use.
- **COUNT(*) for entry numbers** — `journals_service.rs:594` runs `SELECT COUNT(*) FROM journal_headers WHERE company_id = $1` on every entry creation. This is O(n) and non-transactional. Use a PostgreSQL sequence.
- **Account balance updates** — `journals_service.rs:282-307` updates `accounts.balance` row-by-row in a loop without a transaction. For a journal with 20 lines, this makes 20 UPDATE queries. Should batch or use a CTE.
- **Regex-based security scanning** — `security.rs:11-37` compiles 20 regex patterns on every request via `Lazy`. While `Lazy` avoids recompilation, scanning every request body against 20 regexes adds latency. This should be optional and only applied to untrusted input endpoints.

### 6.2 Frontend Performance

- **No static export** — `tauri.conf.json:10` references `frontendDist: "../src-web/out"` but Next.js must be configured with `output: 'export'` in `next.config.ts` for Tauri compatibility. If this isn't set, the dev server approach is used which adds latency.
- **No data caching** — Frontend fetches fresh data on every page load. No SWR, React Query, or client-side cache. Each navigation re-fetches all data.
- **Inline styles** — 200+ char inline style strings in JSX are not cached by the browser and increase bundle size. Use CSS modules or Tailwind.

### 6.3 Database Performance

- **Missing indexes** — `accounting_periods` has no index on `(company_id, start_date, end_date)` despite being queried by `find_open_period()`. `posting_rules` needs an index on `(company_id, module, transaction_type, is_active)`.
- **No partial indexes** — `journal_headers` could benefit from a partial index `WHERE status = 'draft'` for the draft listing query.
- **Balance column on accounts** — Storing `balance NUMERIC(18,2)` directly on the `chart_of_accounts` table is a denormalization that can become inconsistent. Every posting must correctly update it. Consider computing balance from journal lines (with materialized view for performance).
- **Query optimizer generates raw SQL** — `query_optimization.rs:24-46` builds SQL strings via `format!()`, bypassing sqlx's compile-time query checking. The `cursor_paginate` function is also vulnerable to SQL injection if `table` or `sort_col` come from user input.

## 7. Testing Review

### 7.1 Rust Tests

**Coverage: ~5%** — Only 3 modules have `#[cfg(test)]` blocks:

| Module | Tests | What's Covered |
|---|---|---|
| `encryption_service.rs` | 3 | Encrypt/decrypt roundtrip, password hash, API key format |
| `compression.rs` | 4 | Gzip compress, no-compress for small body, unsupported encoding, should_compress |
| `performance_monitor.rs` | 4 | Record/retrieve, slow query threshold, percentile, circular buffer |

**Missing tests:**
- `auth_service.rs` — No tests for login, register, MFA, token refresh
- `journals_service.rs` — No tests for journal lifecycle (create, validate, approve, post, reverse)
- `posting_service.rs` — No tests for posting rule resolution, idempotency, period validation
- `tax_service.rs` — No tests for VAT/WHT/CIT computation
- `payroll_service.rs` — No tests for PAYE computation, allowance calculation
- `einvoicing_service.rs` — No tests for submit, confirm, download flow
- All middleware — No tests for AuthLayer, security headers, request size limits

**CI concern**: `ci.yml:60-64` runs `cargo test` with `DATABASE_URL=postgresql://test:test@localhost:5432/haqly_test` but no PostgreSQL service is started. Tests requiring a database will fail.

### 7.2 Python Tests

**Coverage: ~40%** — 9 test files covering:

| Test File | Coverage |
|---|---|
| `test_tax_computation.py` | Good — VAT, WHT, CIT, Education Tax, CGT |
| `test_journal_accuracy.py` | Exists |
| `test_financial_statements.py` | Exists |
| `test_ledger_accuracy.py` | Exists |
| `test_agents.py` | Exists |
| `test_ifrs_compliance.py` | Exists |
| `test_valuation.py` | Exists |
| `test_risk.py` | Exists |
| `test_industry_profiles.py` | Exists |

**Missing**: No API endpoint tests, no integration tests with the Rust backend, no test for OCR pipeline.

### 7.3 Frontend Tests

**Coverage: 0%** — No test files found. No Jest, Vitest, or React Testing Library configuration.

### 7.4 Integration Tests

**None exist.** Critical gaps:
- No end-to-end test for journal creation → approval → posting → GL update
- No test for tax computation → journal entry creation
- No test for e-invoicing submit → FIRS API (even mocked)
- No test for payroll run → payslip generation → GL posting
- No test for multi-company data isolation
- No test for RBAC permission enforcement

## 8. Deployment Readiness

### 8.1 Build Readiness

**Cannot compile as-is** due to schema mismatches. The code references tables and columns that don't exist in the migration schema. Specifically:
- `accounts` vs `chart_of_accounts`
- `users.company_id` not in migration
- `sessions.token` not in migration
- `journal_headers.entry_number` vs `journal_headers.number`

Even after fixing these, `cargo check` would need a running PostgreSQL with the schema applied for sqlx compile-time verification.

### 8.2 Configuration Readiness

- `.env.example` has default credentials `haqly:haqly` for PostgreSQL — acceptable for dev but must be clearly documented as unsafe for production.
- `settings.rs` has a fail-secure pattern that `process::exit(1)` on missing production variables — good.
- **No `.env` validation** beyond presence checks. No validation that `DATABASE_URL` is a valid PostgreSQL URL, that `JWT_SECRET` is sufficiently long, or that `FIRS_API_KEY` matches expected format.
- **Python config has hardcoded DB password** `haqly:haqly_secret` as default — this will be used if env var is not set, silently connecting with default credentials.

### 8.3 Distribution Readiness

- **No PostgreSQL bundling** — Users must install PostgreSQL independently. This is a hard blocker for desktop distribution.
- **No Python bundling** — AI sidecar requires Python 3.11+ installed. No embedded interpreter.
- **Empty updater public key** — `tauri.conf.json:90` `"pubkey": ""` means auto-updates will either fail or accept unsigned updates.
- **No Windows code signing** — `tauri.conf.json:55` `"certificateThumbprint": null`. Unsigned Windows builds trigger SmartScreen warnings.
- **No MSI/NSIS installer configuration** — Bundle section targets "all" but no installer customization.
- **`externalBin` references non-existent paths** — `tauri.conf.json:79` references `binaries/haqly-backend` and `binaries/haqly-ai-engine` but these don't exist yet.
- **Tauri `$schema` URL is wrong** — `tauri.conf.json:2` references `nicegui/nicegui` instead of Tauri's schema. This may cause IDE validation issues but probably not runtime errors.

## 9. Top 10 Specific Improvements

| Priority | Issue | File:Line | Fix |
|---|---|---|---|
| 1 | **Schema/code mismatches** | `003_accounting.sql` vs `accounting_service.rs`, `posting_service.rs` | Rename migration tables to match code or vice versa. Add `company_id` to `users`, `token` to `sessions`. Create migration 016 to reconcile. |
| 2 | **JWT uses HS256** | `auth.rs:32-41`, `auth_service.rs:244-262` | Switch to RS256 with RSA keypair. Store private key in OS keychain. Update `Claims` struct to include `kid` header. |
| 3 | **E-invoicing credentials unencrypted** | `einvoicing_service.rs:46`, `010_einvoicing.sql:29-30` | Call `encryption_service::encrypt_field()` before INSERT. Change migration columns to `api_key_encrypted`, `api_key_nonce`, `api_key_tag`. |
| 4 | **No RBAC enforcement** | `main.rs:71-77` | Add `RbacLayer` middleware after `AuthLayer`. Load user permissions from `role_permissions` table. Check required permission per route. |
| 5 | **No transactions in financial operations** | `posting_service.rs:61-103`, `journals_service.rs:43-82`, `payroll_service.rs:258-280` | Wrap multi-step database operations in `sqlx::Transaction`. Use `pool.begin().await?` and `tx.commit().await?`. |
| 6 | **MFA uses broken HMAC** | `auth_service.rs:283-310` | Replace custom SHA1-TOTP with `totp-rs` crate. Remove `hmac_sha1` function and `openssl` dependency. |
| 7 | **CSP allows unsafe-eval** | `tauri.conf.json:31` | Change `script-src 'self' 'unsafe-inline' 'unsafe-eval'` to `script-src 'self'`. Add nonce-based inline script support if needed. |
| 8 | **Token in localStorage** | `api.ts:17-18` | Move to Tauri secure storage via IPC command. Use `tauri-plugin-store` with encryption or OS keychain. |
| 9 | **unwrap() in payroll** | `payroll_service.rs:173-178,237-252` (16 instances) | Replace `BigDecimal::from_f64(X).unwrap()` with `BigDecimal::from_f64(X).unwrap_or_else(|| BigDecimal::from(0))` or use pre-computed `BigDecimal` constants. |
| 10 | **No PostgreSQL bundling** | Architecture-wide | Add embedded PostgreSQL strategy: bundle `pg_ctl`, `initdb` on first launch, or switch to `rusqlite`/`libsql` for single-user desktop mode with PostgreSQL as server-mode option. |

## 10. Risk Assessment

### 10.1 Risk Matrix

| Risk | Probability | Impact | Severity |
|---|---|---|---|
| Schema mismatches prevent compilation | **Certain** | Blocks all development | **P0 — Critical** |
| JWT forgery via HS256 shared secret | **High** | Full system compromise | **P0 — Critical** |
| E-invoice credential leak from DB | **Medium** | FIRS compliance violation, financial fraud | **P0 — Critical** |
| Unauthorized access via missing RBAC | **High** | Data breach across all companies | **P1 — High** |
| Financial data corruption (no transactions) | **Medium** | Inconsistent ledgers, audit failure | **P1 — High** |
| PAYE computation error (from_f64 unwrap) | **Medium** | Incorrect tax deductions, PENCOM violation | **P1 — High** |
| Desktop deployment fails (no bundled DB) | **Certain** | Product unusable | **P1 — High** |
| XSS via CSP unsafe-eval | **Medium** | Token theft, session hijack | **P2 — Medium** |
| Concurrent entry number collision | **Medium** | Duplicate journal numbers | **P2 — Medium** |
| N+1 queries degrade performance at scale | **High** | Slow API responses >2s | **P2 — Medium** |
| No test coverage for financial logic | **Certain** | Regression in tax/Gl computations | **P2 — Medium** |
| Python sidecar startup failure | **Medium** | AI features unavailable | **P3 — Low** |
| Auto-update accepts unsigned packages | **Low** | Supply chain attack | **P3 — Low** |

### 10.2 Overall Risk Level

**HIGH**

The project has excellent documentation and an ambitious, well-considered architecture. However, the implementation has critical security gaps (HS256 JWT, unencrypted credentials, no RBAC), data integrity risks (no database transactions in financial operations), and a fundamental deployment blocker (no bundled PostgreSQL). The schema mismatches between migrations and code mean the application cannot compile against its own database schema today.

**The project is at a late-alpha / early-beta stage.** To reach production readiness:

1. **Immediate (1-2 weeks)**: Fix schema mismatches, switch to RS256, encrypt sensitive fields, add RBAC middleware, add database transactions to all financial operations.
2. **Short-term (2-4 weeks)**: Fix MFA, fix CSP, implement rate limiting, add Rust unit tests for all services, bundle PostgreSQL.
3. **Medium-term (1-2 months)**: Build frontend for core modules, implement financial reports, add integration tests, configure code signing and auto-update.
4. **Before launch**: Penetration test, NDPR compliance audit, FIRS e-invoicing certification, load testing with realistic data volumes.

The foundation is strong — the double-entry accounting model, the Nigerian tax engine, and the posting rules engine are well-designed. With focused effort on the critical items above, this project has the potential to become a viable Nigerian ERP desktop application.
