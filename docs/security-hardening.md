# HAQLY ERP — Security Hardening Guide

**Author**: Quadri Atharu
**Version**: 1.0.0
**Last Updated**: 2026-04-14

---

## 1. Architecture Security Overview

### 1.1 Tauri Desktop Security Model

HAQLY ERP runs as a Tauri v2 desktop application with three security boundaries:

```
┌─────────────────────────────────────────────────┐
│  Renderer Process (WebView / Next.js 15)        │
│  - Sandboxed, no direct filesystem access       │
│  - Communicates only via IPC allowlist           │
├─────────────────────────────────────────────────┤
│  Rust Backend (Core Process)                     │
│  - Owns all system access                       │
│  - Validates every IPC command                  │
│  - Owns database connection pool                │
├─────────────────────────────────────────────────┤
│  Python Sidecar (AI / OCR / Tax Engine)         │
│  - Isolated subprocess, no network egress       │
│  - Communicates via stdout/stdin JSON protocol   │
│  - Chrooted working directory, minimal perms    │
└─────────────────────────────────────────────────┘
```

**Key principles:**
- **Principle of least privilege**: The renderer never touches the filesystem, network, or OS APIs directly.
- **Dual validation**: Every IPC command is validated on both the renderer (TypeScript) and the Rust backend.
- **Sidecar isolation**: The Python sidecar runs with a restricted PATH, no outbound network, and a read-only filesystem except its designated output directory.
- **No shell escape**: IPC commands never pass through a shell; all arguments are structured and typed.

### 1.2 Rust Backend Security

- All SQL queries use **sqlx** compile-time checked parameterized queries — zero raw SQL strings.
- Sensitive configuration (DB URL, JWT secret, AES keys) loaded from environment variables or OS keychain — never hardcoded.
- Panic handlers wrap all command handlers to prevent renderer crash propagation.
- `unsafe` code is banned outside explicitly reviewed modules.
- All integer arithmetic on financial values uses `rust_decimal::Decimal` — no floating-point.

### 1.3 Python Sidecar Isolation

```json
{
  "sidecar": {
    "command": "python3",
    "args": ["-u", "sidecar/main.py"],
    "env": {
      "PYTHONPATH": "/app/sidecar",
      "PYTHONDONTWRITEBYTECODE": "1",
      "PYTHONNOUSERSITE": "1"
    },
    "sandbox": {
      "filesystem": ["read=/app/sidecar", "write=/app/sidecar/output"],
      "network": "deny-all",
      "capabilities": ["ipc"]
    }
  }
}
```

- The sidecar cannot make outbound HTTP requests — all external API calls (FIRS, etc.) are proxied through the Rust backend.
- Filesystem access is restricted to its working directory.
- No `os.system()`, `subprocess.call()`, or `eval()` permitted — enforced via AST linting at build time.

---

## 2. Authentication & Session Security

### 2.1 JWT Best Practices

Always use **asymmetric signing** (RS256 or ES256) over symmetric (HS256):

```toml
# Cargo.toml
jsonwebtoken = "9"
```

```rust
use jsonwebtoken::{encode, decode, EncodingKey, DecodingKey, Algorithm, Header, Validation};

const JWT_ALGORITHM: Algorithm = Algorithm::RS256;

pub fn create_token(claims: &TokenClaims, private_key: &[u8]) -> Result<String, AuthError> {
    let header = Header::new(JWT_ALGORITHM);
    encode(&header, claims, &EncodingKey::from_rsa_pem(private_key)?)
        .map_err(AuthError::TokenCreation)
}

pub fn verify_token(token: &str, public_key: &[u8]) -> Result<TokenClaims, AuthError> {
    let mut validation = Validation::new(JWT_ALGORITHM);
    validation.set_required_spec_claims(&["exp", "iat", "sub"]);
    validation.leeway = 30;
    decode::<TokenClaims>(token, &DecodingKey::from_rsa_pem(public_key)?, &validation)
        .map(|data| data.claims)
        .map_err(AuthError::TokenVerification)
}
```

**Token configuration:**

| Property | Access Token | Refresh Token |
|---|---|---|
| Type | JWT (RS256) | Opaque 256-bit random |
| Expiry | 15 minutes | 7 days |
| Storage | Memory (frontend) | httpOnly Secure cookie |
| Rotation | Key ID header for key rotation | Single-use, rotated on refresh |
| Revocation | Short expiry + blocklist | Deleted from DB |

### 2.2 Token Rotation

```rust
pub async fn rotate_refresh_token(old_token: &str) -> Result<(String, String), AuthError> {
    let stored = refresh_tokens::find_by_token(old_token).await?;
    if stored.is_used {
        consume_family(&stored.family_id).await?;
        return Err(AuthError::RefreshTokenReuse);
    }
    refresh_tokens::mark_used(old_token).await?;
    let new_refresh = generate_secure_token(32);
    let new_access = create_access_token(&stored.user_id)?;
    refresh_tokens::create(new_refresh.clone(), stored.user_id, stored.family_id).await?;
    Ok((new_access, new_refresh))
}
```

**Refresh token reuse detection**: If a previously-used refresh token is presented, the entire token family is consumed — this indicates token theft. The user is forced to re-authenticate.

### 2.3 Session Invalidation

Sessions are invalidated in these scenarios:
- Password change → all refresh tokens for user deleted
- MFA enable/disable → all sessions except current revoked
- Admin "force logout" → targeted user sessions cleared
- Role change → access tokens invalidated (short expiry handles this within 15 min)
- Security alert (IP/UA mismatch) → affected sessions revoked

---

## 3. Authorization & RBAC

### 3.1 Role Hierarchy

```
SuperAdmin (0)
  └── Admin (1)
        ├── Accountant (2)
        ├── SalesManager (2)
        ├── PurchaseManager (2)
        ├── InventoryManager (2)
        ├── HRManager (2)
        └── TreasuryManager (2)
              └── Viewer (3)
```

Level 0–1 roles span all modules within their company. Level 2 roles are module-scoped. Level 3 (Viewer) is read-only.

### 3.2 Permission Granularity

Each permission follows the pattern `module:action`:

| Module | Permissions |
|---|---|
| users | `view`, `create`, `update`, `deactivate` |
| accounting | `coa`, `journal`, `voucher`, `period`, `reconcile` |
| sales | `view`, `create`, `approve`, `cancel` |
| purchases | `view`, `create`, `approve`, `cancel` |
| inventory | `view`, `create`, `adjust`, `transfer` |
| tax | `view`, `compute`, `file`, `override` |
| payroll | `view`, `run`, `approve`, `payslip` |
| reports | `view`, `export`, `schedule` |
| einvoicing | `manage`, `submit`, `cancel` |
| admin | `roles`, `settings`, `audit` |
| bi | `view`, `create`, `export` |
| crm | `view`, `create`, `edit`, `delete` |

### 3.3 Segregation of Duties (SoD)

Enforced at the middleware level — not just UI hiding:

```rust
pub fn check_sod(actor_role: &Role, action: &str, record: &Record) -> Result<(), AuthError> {
    if let Some(creator_id) = record.created_by {
        if creator_id == actor_role.user_id {
            match action {
                "approve" | "file" | "post" => {
                    return Err(AuthError::SegregationOfDuties {
                        reason: format!("Creator cannot {} their own record", action),
                    });
                }
                _ => {}
            }
        }
    }
    Ok(())
}
```

**SoD rules enforced:**

| Creator Action | Blocked Approver Action |
|---|---|
| Create journal entry | Approve same journal entry |
| Create payment voucher | Approve same voucher |
| Run payroll | Approve same payroll run |
| Compute tax | File same tax return |
| Create e-invoice | Cancel same e-invoice |

### 3.4 Admin Access Controls

- SuperAdmin requires MFA (TOTP) for every session.
- Admin role changes require two-admin approval (four-eyes principle).
- SuperAdmin cross-company access requires explicit `company_id` scope in the JWT claims.
- Admin session timeout: 30 minutes of inactivity (vs 2 hours for regular users).

---

## 4. Data Protection

### 4.1 AES-256-GCM Encryption at Rest

```rust
use aes_gcm::{Aes256Gcm, KeyInit, Nonce};
use aes_gcm::aead::Aead;
use rand::RngCore;

const NONCE_LEN: usize = 12;

pub fn encrypt_field(plaintext: &[u8], key: &[u8; 32]) -> Result<Vec<u8>, CryptoError> {
    let cipher = Aes256Gcm::new_from_slice(key)?;
    let mut nonce_bytes = [0u8; NONCE_LEN];
    rand::thread_rng().fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);
    let ciphertext = cipher.encrypt(nonce, plaintext)?;
    let mut output = Vec::with_capacity(NONCE_LEN + ciphertext.len());
    output.extend_from_slice(&nonce_bytes);
    output.extend_from_slice(&ciphertext);
    Ok(output)
}

pub fn decrypt_field(mut data: &[u8], key: &[u8; 32]) -> Result<Vec<u8>, CryptoError> {
    if data.len() < NONCE_LEN + 16 {
        return Err(CryptoError::InvalidCiphertext);
    }
    let nonce_bytes = &data[..NONCE_LEN];
    data = &data[NONCE_LEN..];
    let cipher = Aes256Gcm::new_from_slice(key)?;
    let nonce = Nonce::from_slice(nonce_bytes);
    cipher.decrypt(nonce, data).map_err(CryptoError::Decryption)
}
```

**Encrypted fields (PII/sensitive):**

| Field | Table | Reason |
|---|---|---|
| bank_account_number | bank_accounts | Financial PII |
| tax_identification_number | companies | Government ID |
| api_secret | integrations | Credential |
| einvoice_credentials | einvoice_config | FIRS credential |
| pension_number | employees | Government ID |
| salary_amount | employees | Confidential compensation |

### 4.2 TLS 1.3 in Transit

```nginx
server {
    listen 443 ssl http2;
    ssl_protocols TLSv1.3;
    ssl_prefer_server_ciphers off;
    ssl_certificate /etc/ssl/haqly/fullchain.pem;
    ssl_certificate_key /etc/ssl/haqly/privkey.pem;
    add_header Strict-Transport-Security "max-age=31536000; includeSubDomains; preload" always;
    add_header X-Content-Type-Options "nosniff" always;
    add_header X-Frame-Options "DENY" always;
}
```

- No TLS 1.0, 1.1, or 1.2 fallback in production.
- Certificate pinning in the Tauri desktop app via `tauri.conf.json` `security.dangerousDisableAssetCscModification: false`.

### 4.3 PII Field-Level Encryption

All PII fields are encrypted before database insertion and decrypted on read. The encryption layer is transparent:

```rust
// Transparent field encryption via sqlx custom type
impl sqlx::Encode<'_, sqlx::Postgres> for EncryptedString {
    fn encode_by_ref(&self, buf: &mut Vec<u8>) -> sqlx::encode::IsNull {
        let encrypted = encrypt_field(self.0.as_bytes(), &get_field_key()).unwrap();
        buf.extend_from_slice(&encrypted);
        sqlx::encode::IsNull::No
    }
}
```

### 4.4 Database Connection Security

```rust
let pool = sqlx::postgres::PgPoolOptions::new()
    .max_connections(20)
    .min_connections(5)
    .acquire_timeout(Duration::from_secs(5))
    .idle_timeout(Duration::from_secs(600))
    .connect(&database_url) // Must use sslmode=verify-full
    .await?;
```

Required connection string parameters:
- `sslmode=verify-full` — full certificate verification
- `sslrootcert=/etc/ssl/haqly/ca.pem` — CA certificate path
- No plaintext connections — enforced at the pool level

---

## 5. API Security

### 5.1 Rate Limiting Configuration

```rust
use tower::ServiceBuilder;
use tower_governor::{GovernorConfig, GovernorRateLimitMiddleware};

let auth_limiter = GovernorConfig::default()
    .per_second(1)
    .per_minute(5)
    .burst_size(5);

let read_limiter = GovernorConfig::default()
    .per_second(10)
    .per_minute(100);

let write_limiter = GovernorConfig::default()
    .per_second(2)
    .per_minute(30);

let delete_limiter = GovernorConfig::default()
    .per_second(1)
    .per_minute(10);
```

| Endpoint Group | Rate Limit | Burst | Window |
|---|---|---|---|
| `POST /auth/login` | 5 req | 5 | 15 min |
| `POST /auth/refresh` | 20 req | 10 | 1 min |
| `POST /auth/password-reset` | 3 req | 3 | 60 min |
| `GET /api/*` | 100 req | 30 | 1 min |
| `POST /api/*` | 30 req | 10 | 1 min |
| `PATCH /api/*` | 30 req | 10 | 1 min |
| `DELETE /api/*` | 10 req | 5 | 1 min |
| `POST /api/reports/export` | 5 req | 3 | 5 min |
| `POST /api/einvoicing/*` | 20 req | 10 | 1 min |

### 5.2 Input Validation Rules

```rust
use validator::{Validate, ValidationError};

#[derive(Debug, Validate, Deserialize)]
pub struct CreateJournalEntry {
    #[validate(length(min = 1, max = 255))]
    pub description: String,
    #[validate(length(min = 1, max = 20))]
    pub entry_number: String,
    #[validate(length(min = 1, max = 50))]
    pub period: String,
    #[validate(range(min = 0.01, max = 1_000_000_000))]
    pub total_debit: Decimal,
    #[validate(range(min = 0.01, max = 1_000_000_000))]
    pub total_credit: Decimal,
    #[validate(length(min = 1, max = 200))]
    pub lines: Vec<JournalLineInput>,
}
```

**General validation rules:**
- All string inputs bounded: names ≤ 255, text fields ≤ 5000, codes ≤ 20
- Numeric amounts: 0.01 to 1,000,000,000 (one billion naira cap)
- Tax rates: 0.0 to 100.0
- Percentages: 0.0 to 100.0
- No HTML in any text field — stripped server-side before storage
- Unicode normalization (NFC) applied to all string inputs

### 5.3 SQL Injection Prevention

All database queries use **sqlx** compile-time checked parameterized queries:

```rust
// CORRECT: Parameterized query
sqlx::query_as!(
    Account,
    "SELECT * FROM accounts WHERE code = $1 AND company_id = $2",
    code, company_id
)
.fetch_optional(&pool)
.await?;

// FORBIDDEN: String interpolation
// format!("SELECT * FROM accounts WHERE code = '{}'", code)
```

The codebase includes a CI lint that rejects any `.query(` or `.execute(` call containing string interpolation (`format!`, `concat!`, or string `+` operators).

### 5.4 CORS Policy

```rust
use tower_http::cors::{CorsLayer, Any};

let cors = CorsLayer::new()
    .allow_origin([
        "tauri://localhost".parse()?,
        "https://tauri.localhost".parse()?,
        "https://localhost:3001".parse()?, // Dev only
    ])
    .allow_methods([Method::GET, Method::POST, Method::PATCH, Method::DELETE, Method::OPTIONS])
    .allow_headers([
        header::AUTHORIZATION,
        header::CONTENT_TYPE,
        header::ACCEPT,
        HeaderName::from_static("x-request-id"),
    ])
    .max_age(Duration::from_secs(3600));
```

- No wildcard (`*`) origins in production — CI rejects this.
- `Access-Control-Allow-Credentials: false` for cross-origin requests.
- Preflight caching: 1 hour.

### 5.5 Request Size Limits

```rust
use axum::extract::DefaultBodyLimit;

let app = Router::new()
    .route("/api/einvoicing/upload", post(handle_einvoice_upload))
    .layer(DefaultBodyLimit::max(5 * 1024 * 1024)) // 5 MB for uploads
    .route("/api/*", any(handle_api))
    .layer(DefaultBodyLimit::max(1 * 1024 * 1024)); // 1 MB for general API
```

| Route | Max Body Size |
|---|---|
| General API | 1 MB |
| File upload (OCR, e-invoice) | 5 MB |
| Batch import | 10 MB |

### 5.6 CSRF Protection

Since HAQLY ERP is a Tauri desktop app (not a browser-to-server web app), traditional CSRF is less relevant. However, for the web deployment variant:

- `SameSite=Strict` on all cookies
- `Origin` header validation on all state-changing requests
- Double-submit cookie pattern for non-Tauri deployments
- Custom `X-Request-ID` header required (proves JS-origin, not form-origin)

---

## 6. Desktop Security (Tauri)

### 6.1 Tauri CSP Configuration

```json
{
  "security": {
    "csp": "default-src 'self'; script-src 'self'; style-src 'self' 'unsafe-inline'; img-src 'self' asset: https://data:; font-src 'self'; connect-src 'self' https://api.haqly.com wss://api.haqly.com; frame-src 'none'; object-src 'none'; base-uri 'self'; form-action 'self'"
  }
}
```

- No `eval()`, `Function()`, or inline script execution.
- `unsafe-inline` only for `style-src` (required by Tailwind CSS).
- `connect-src` restricted to the API backend and websocket endpoint.
- `frame-src 'none'` — no iframe embedding.
- `object-src 'none'` — no Flash/Java plugins.

### 6.2 IPC Allowlist

```json
{
  "ipc": {
    "allowlist": {
      "invoke": [
        "auth:login",
        "auth:logout",
        "auth:refresh",
        "accounting:get_accounts",
        "accounting:create_journal",
        "accounting:post_journal",
        "tax:compute",
        "tax:file",
        "payroll:run",
        "payroll:approve",
        "einvoicing:submit",
        "einvoicing:cancel",
        "admin:get_users",
        "admin:create_user",
        "admin:update_role",
        "file:open_dialog",
        "file:save_report"
      ],
      "listen": ["auth:session_expired", "update:available"]
    }
  }
}
```

- Every IPC command is explicitly allowed; all others are denied by default.
- No `shell:open`, `shell:execute`, or arbitrary command invocation.
- `file:*` commands restricted to specific directories via scope.

### 6.3 Sidecar Process Isolation

The Python sidecar is launched with:

```rust
let mut cmd = Command::new_sidecar("haqly-sidecar")?;
cmd.env_clear()
   .env("PYTHONPATH", "/app/sidecar")
   .env("PYTHONDONTWRITEBYTECODE", "1")
   .env("PYTHONNOUSERSITE", "1")
   .env("SIDECAR_KEY", &sidecar_shared_secret) // HMAC key for IPC messages
   .stdout(Stdio::piped())
   .stderr(Stdio::piped())
   .stdin(Stdio::piped());
```

- `env_clear()` removes all inherited environment variables.
- Sidecar communication is HMAC-signed with a per-boot shared secret.
- Sidecar stdout/stdin use a strict JSON-RPC protocol — malformed messages are dropped.
- Sidecar process is killed if it exceeds 2 GB memory or 60 seconds of unresponsiveness.

### 6.4 Local Storage Encryption

```rust
use keyring::Entry;

pub fn store_secret(key: &str, value: &str) -> Result<(), SecurityError> {
    let entry = Entry::new("com.haqly.erp", key)?;
    entry.set_password(value)?;
    Ok(())
}

pub fn retrieve_secret(key: &str) -> Result<String, SecurityError> {
    let entry = Entry::new("com.haqly.erp", key)?;
    Ok(entry.get_password()?)
}
```

Sensitive data stored in the OS keychain:
- `haqly_db_url` — Database connection string
- `haqly_jwt_private_key` — JWT RS256 private key
- `haqly_field_encryption_key` — AES-256 field encryption key
- `haqly_sidecar_secret` — Sidecar IPC HMAC key

**No secrets in localStorage, IndexedDB, or flat files.**

### 6.5 Auto-Update Signature Verification

```json
{
  "updater": {
    "active": true,
    "endpoints": ["https://releases.haqly.com/updates/{{target}}/{{arch}}/{{current_version}}"],
    "dialog": true,
    "pubkey": "HAQLY_ED25519_PUBLIC_KEY_BASE64"
  }
}
```

- All updates must be signed with HAQLY's Ed25519 private key.
- The public key is baked into `tauri.conf.json` at build time.
- Unsigned or tampered updates are rejected — no bypass.
- User confirmation dialog before applying any update.

### 6.6 Clipboard Protection

- Sensitive data (passwords, tokens, TOTP secrets) is never written to the clipboard.
- When a user copies a financial figure, only the numeric value is placed on the clipboard — no surrounding context.
- Clipboard contents are cleared 30 seconds after copy via Tauri's clipboard API.

---

## 7. E-Invoicing Security

### 7.1 FIRS API Credential Management

```rust
pub struct FirsCredentials {
    pub api_key: EncryptedString,       // AES-256-GCM encrypted in DB
    pub client_id: EncryptedString,     // AES-256-GCM encrypted in DB
    pub client_secret: EncryptedString, // AES-256-GCM encrypted in DB
    pub certificate_pem: EncryptedString,
}

impl FirsCredentials {
    pub fn decrypt_for_request(&self, key: &[u8; 32]) -> Result<DecryptedFirsCreds, CryptoError> {
        Ok(DecryptedFirsCreds {
            api_key: self.api_key.decrypt(key)?,
            client_id: self.client_id.decrypt(key)?,
            client_secret: self.client_secret.decrypt(key)?,
            certificate: self.certificate_pem.decrypt(key)?,
        })
    }
}
```

- FIRS credentials are decrypted **only** immediately before an API call.
- Decrypted credentials live in memory for < 1 second and are zeroed after use.
- Credential rotation: FIRS API keys rotated quarterly; old keys are revoked 24 hours after new key deployment.

### 7.2 Certificate Pinning

```rust
use reqwest::Certificate;

pub fn build_firs_client(ca_cert_pem: &[u8]) -> Result<reqwest::Client, HttpError> {
    let ca_cert = Certificate::from_pem(ca_cert_pem)?;
    let client = reqwest::Client::builder()
        .add_root_certificate(ca_cert)
        .tls_built_in_root_certs(false) // Disable default CA store
        .min_tls_version(reqwest::tls::Version::TLS_1_3)
        .connect_timeout(Duration::from_secs(10))
        .timeout(Duration::from_secs(30))
        .build()?;
    Ok(client)
}
```

- Default CA store is disabled for FIRS API calls.
- Only the FIRS-specific CA certificate is trusted.
- Prevents MITM attacks even if a system-level CA is compromised.

### 7.3 Request Signing

Every e-invoice submission is signed with HMAC-SHA256:

```rust
pub fn sign_einvoice_request(body: &[u8], timestamp: i64, secret: &[u8]) -> String {
    let mut message = Vec::new();
    message.extend_from_slice(body);
    message.extend_from_slice(timestamp.to_string().as_bytes());
    let sig = hmac_sha256(secret, &message);
    format!("t={},v1={}", timestamp, hex::encode(sig))
}

pub fn verify_request_timestamp(timestamp: i64) -> Result<(), SecurityError> {
    let now = chrono::Utc::now().timestamp();
    if (now - timestamp).abs() > 300 {
        return Err(SecurityError::StaleTimestamp);
    }
    Ok(())
}
```

- 5-minute timestamp tolerance prevents replay attacks.
- HMAC secret is per-company and rotated every 90 days.

### 7.4 Audit Logging

All e-invoicing operations are immutably logged:

```sql
CREATE TABLE einvoice_audit_log (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    company_id UUID NOT NULL,
    user_id UUID NOT NULL,
    action VARCHAR(50) NOT NULL,       -- submit, cancel, query, irn_validate
    irn VARCHAR(100),                  -- Invoice Reference Number
    firs_request_hash VARCHAR(64),     -- SHA-256 of request body
    firs_response_hash VARCHAR(64),    -- SHA-256 of response body
    status VARCHAR(20) NOT NULL,      -- success, failure, timeout
    ip_address INET,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- INSERT-only, no UPDATE or DELETE
REVOKE UPDATE, DELETE ON einvoice_audit_log FROM PUBLIC;
CREATE RULE prevent_update AS ON UPDATE TO einvoice_audit_log DO INSTEAD NOTHING;
CREATE RULE prevent_delete AS ON DELETE TO einvoice_audit_log DO INSTEAD NOTHING;
```

---

## 8. Secure Defaults Checklist

The `SecureDefaultsChecker` validates the following at application startup. Any finding blocks the application from serving requests.

| # | Check | Severity | How to Fix |
|---|---|---|---|
| 1 | JWT signing algorithm is RS256 or ES256 | Critical | Set `JWT_ALGORITHM=RS256` and provide RSA/EC keypair |
| 2 | JWT secret is not a default/test value | Critical | Generate a new keypair: `openssl genrsa -out private.pem 2048` |
| 3 | Database connection uses `sslmode=verify-full` | Critical | Append `sslmode=verify-full&sslrootcert=/path/to/ca.pem` to DB URL |
| 4 | AES field encryption key is 32 bytes | Critical | Generate: `openssl rand -hex 32` and set `FIELD_ENCRYPTION_KEY` |
| 5 | CORS does not allow wildcard origins | High | Remove any `*` from `CORS_ALLOWED_ORIGINS` |
| 6 | Tauri IPC allowlist is not empty | High | Define all required commands in `tauri.conf.json` `ipc.allowlist` |
| 7 | CSP header does not allow `eval` or `unsafe-inline` (except styles) | High | Review `security.csp` in Tauri config; remove `unsafe-eval` |
| 8 | Refresh token rotation is enabled | High | Set `REFRESH_TOKEN_ROTATION=true` |
| 9 | Rate limiting is enabled on auth endpoints | High | Configure `GovernorConfig` for `/auth/*` routes |
| 10 | Auto-update public key is set | Medium | Set `updater.pubkey` in `tauri.conf.json` |
| 11 | Sidecar environment variables are cleared | Medium | Ensure `env_clear()` is called before sidecar launch |
| 12 | FIRS certificate pinning is enabled | Medium | Provide FIRS CA cert via `FIRS_CA_CERT_PATH` env var |
| 13 | MFA is enforced for SuperAdmin and Admin | Medium | Set `MFA_ENFORCED_ROLES=SuperAdmin,Admin` |
| 14 | Audit log table is INSERT-only | Medium | Run `REVOKE UPDATE, DELETE ON audit_log FROM PUBLIC` |
| 15 | Request body size limits are set | Low | Configure `DefaultBodyLimit` on API routes |
| 16 | HSTS header is present | Low | Add `Strict-Transport-Security` header in reverse proxy config |
| 17 | Clipboard auto-clear is configured | Low | Set `CLIPBOARD_CLEAR_SECONDS=30` |

### Startup Check Implementation

```rust
pub struct SecureDefaultsChecker {
    findings: Vec<SecurityFinding>,
}

impl SecureDefaultsChecker {
    pub async fn run_checks(&mut self) -> Result<(), SecurityCheckError> {
        self.check_jwt_algorithm()?;
        self.check_jwt_secret_not_default()?;
        self.check_db_ssl_mode().await?;
        self.check_field_encryption_key()?;
        self.check_cors_config()?;
        self.check_ipc_allowlist()?;
        self.check_csp_config()?;
        self.check_refresh_rotation()?;
        self.check_rate_limiting().await?;
        self.check_update_pubkey()?;
        self.check_sidecar_isolation()?;
        self.check_firs_cert_pinning()?;
        self.check_mfa_enforcement().await?;
        self.check_audit_log_immutability().await?;
        self.check_body_limits()?;
        self.check_hsts_header()?;
        self.check_clipboard_clear()?;

        let criticals = self.findings.iter().filter(|f| f.severity == "critical").count();
        if criticals > 0 {
            return Err(SecurityCheckError::CriticalFindings(criticals));
        }
        Ok(())
    }
}
```

---

## 9. Deployment Hardening

### 9.1 Environment Variable Management

```bash
# /etc/haqly/env (owned by root:haqly, mode 0400)
DATABASE_URL=postgres://haqly:xxx@db:5432/haqly?sslmode=verify-full&sslrootcert=/etc/ssl/haqly/ca.pem
JWT_PRIVATE_KEY_PATH=/etc/haqly/keys/jwt_private.pem
JWT_PUBLIC_KEY_PATH=/etc/haqly/keys/jwt_public.pem
FIELD_ENCRYPTION_KEY=<32-byte hex>
CORS_ALLOWED_ORIGINS=https://app.haqly.com,tauri://localhost
REDIS_URL=rediss://:xxx@redis:6380/0
FIRS_API_BASE_URL=https://api.firs.gov.ng
FIRS_CA_CERT_PATH=/etc/haqly/certs/firs_ca.pem
MFA_ENFORCED_ROLES=SuperAdmin,Admin
REFRESH_TOKEN_ROTATION=true
CLIPBOARD_CLEAR_SECONDS=30
LOG_LEVEL=info
```

- All env files owned by `root:haqly` with mode `0400`.
- No env files in the git repository — `.env*` in `.gitignore`.
- Production secrets injected via CI/CD pipeline (GitHub Actions secrets or Vault).
- Staging uses separate secrets from production.

### 9.2 Secret Rotation

| Secret | Rotation Period | Rotation Procedure |
|---|---|---|
| JWT RSA keypair | 90 days | Generate new keypair, deploy public key, keep old private key for 1 hour overlap |
| AES field encryption key | 90 days | Re-encrypt all encrypted fields with new key in a migration |
| FIRS API key | Quarterly | Generate new key in FIRS portal, update DB, revoke old key after 24 hours |
| Database password | 90 days | Alter user password, update env file, restart app |
| Redis password | 90 days | Update Redis config, update env file, restart app |
| Sidecar HMAC secret | Every boot | Generated randomly at startup, shared to sidecar via stdin |

### 9.3 Database Hardening

```sql
-- Create dedicated application role with minimal privileges
CREATE ROLE haqly_app LOGIN PASSWORD 'xxx';
GRANT SELECT, INSERT, UPDATE, DELETE ON ALL TABLES IN SCHEMA public TO haqly_app;
REVOKE UPDATE, DELETE ON audit_log FROM haqly_app;
REVOKE UPDATE, DELETE ON einvoice_audit_log FROM haqly_app;

-- Row-level security for multi-tenancy
ALTER TABLE accounts ENABLE ROW LEVEL SECURITY;
CREATE POLICY company_isolation ON accounts
    USING (company_id = current_setting('app.company_id')::UUID);

-- Connection pooling
-- PgBouncer in front of PostgreSQL
-- pgbouncer.ini: pool_mode = transaction, max_client_conn = 200, default_pool_size = 20
```

Additional hardening:
- `listen_addresses = '127.0.0.1'` in `postgresql.conf` (no remote connections)
- `log_connections = on`, `log_disconnections = on`
- `log_statement = 'mod'` (log all DML)
- `shared_preload_libraries = 'pgaudit'`
- Password authentication only via SCRAM-SHA-256 (`password_encryption = scram-sha-256`)

### 9.4 Docker Security

```dockerfile
FROM rust:1.82-slim AS builder
WORKDIR /app
COPY . .
RUN cargo build --release --locked

FROM gcr.io/distroless/cc-debian12
COPY --from=builder /app/target/release/haqly-erp /usr/local/bin/haqly-erp
USER nonroot:nonroot
ENTRYPOINT ["haqly-erp"]
```

```yaml
# docker-compose.yml security hardening
services:
  app:
    security_opt:
      - no-new-privileges:true
      - seccomp:seccomp-profile.json
    cap_drop:
      - ALL
    cap_add:
      - NET_BIND_SERVICE
    read_only: true
    tmpfs:
      - /tmp:size=100M
    mem_limit: 512m
    cpus: 1.0
    pids_limit: 100

  db:
    security_opt:
      - no-new-privileges:true
    cap_drop:
      - ALL
    cap_add:
      - CHOWN
      - SETGID
      - SETUID
      - DAC_OVERRIDE
      - FOWNER
    volumes:
      - pgdata:/var/lib/postgresql/data
    environment:
      POSTGRES_INITDB_ARGS: "--auth-host=scram-sha-256"

  redis:
    security_opt:
      - no-new-privileges:true
    cap_drop:
      - ALL
    command: redis-server --requirepass ${REDIS_PASSWORD} --tls-port 6380 --port 0 --tls-cert-file /etc/ssl/redis/cert.pem --tls-key-file /etc/ssl/redis/key.pem
```

---

## 10. Incident Response

### 10.1 Security Event Logging

All security-relevant events are logged to a dedicated `security_events` table and forwarded to a SIEM:

```sql
CREATE TABLE security_events (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    event_type VARCHAR(50) NOT NULL,
    severity VARCHAR(10) NOT NULL,     -- critical, high, medium, low
    user_id UUID,
    ip_address INET,
    user_agent TEXT,
    details JSONB NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
```

**Events logged:**

| Event Type | Severity | Description |
|---|---|---|
| `auth.login_success` | low | Successful login |
| `auth.login_failure` | medium | Failed login attempt |
| `auth.account_locked` | high | Account lockout triggered |
| `auth.mfa_bypass_attempt` | critical | MFA bypass attempted |
| `auth.token_reuse` | critical | Refresh token reuse detected |
| `auth.session_expired` | low | Session naturally expired |
| `auth.privilege_escalation` | critical | Privilege escalation attempt |
| `data.encryption_key_access` | high | Encryption key accessed |
| `data.bulk_export` | medium | Bulk data export triggered |
| `einvoice.submission_failure` | medium | E-invoice submission failed |
| `einvoice.certificate_mismatch` | high | FIRS certificate mismatch |
| `admin.role_change` | high | User role changed |
| `admin.mfa_toggle` | high | MFA enabled/disabled |
| `system.sod_violation` | high | Segregation of duties violated |
| `system.rate_limit_exceeded` | medium | Rate limit exceeded |
| `system.csp_violation` | high | Content Security Policy violation |
| `system.integrity_check_failed` | critical | Audit log chain integrity failure |

### 10.2 Breach Notification

Nigerian regulatory requirements:

| Regulation | Notification Deadline | Authority |
|---|---|---|
| NDPR | 72 hours | NITDA (National Information Technology Development Agency) |
| CBN Consumer Protection | 24 hours | Central Bank of Nigeria (for financial data) |
| FIRS | 48 hours | Federal Inland Revenue Service (for tax data) |

**Notification procedure:**
1. Security team confirms breach scope and affected data categories
2. Legal counsel reviews notification requirements
3. Regulatory notifications filed within applicable deadlines
4. Affected users notified via email with: what data was exposed, what actions to take, what HAQLY is doing to remediate
5. Public disclosure after regulatory notifications, within 5 business days

### 10.3 Forensic Procedures

**Evidence preservation:**

```bash
# 1. Snapshot affected database
pg_dump -Fc --serializable-deferrable haqly > /forensics/db_snapshot_$(date +%Y%m%d%H%M%S).dump

# 2. Capture application logs
cp -r /var/log/haqly/ /forensics/logs_$(date +%Y%m%d%H%M%S)/

# 3. Dump security events table
psql -c "COPY security_events TO '/forensics/security_events.csv' WITH CSV HEADER" haqly

# 4. Preserve audit log chain
psql -c "COPY audit_log TO '/forensics/audit_log.csv' WITH CSV HEADER" haqly

# 5. Memory dump of running process (if applicable)
gcore -o /forensics/haqly_core $(pgrep haqly-erp)
```

**Forensic analysis checklist:**
- [ ] Identify attack vector (SQL injection, credential theft, insider, etc.)
- [ ] Determine first and last access timestamps
- [ ] Enumerate all records accessed by attacker sessions
- [ ] Verify audit log chain integrity (detect tampering)
- [ ] Check for persistence mechanisms (new accounts, API keys, scheduled tasks)
- [ ] Review all role/permission changes during breach window
- [ ] Analyze network logs for data exfiltration indicators
- [ ] Document findings in incident report with timeline
- [ ] Rotate all potentially compromised secrets
- [ ] Patch the vulnerability that enabled the breach
- [ ] Update security controls to prevent recurrence
- [ ] File regulatory notifications within required deadlines

**Post-incident review:**
- Conduct blameless post-mortem within 5 business days
- Produce written incident report: timeline, root cause, impact, remediation
- Update this security hardening guide with lessons learned
- Schedule follow-up penetration test focused on the breached vector
