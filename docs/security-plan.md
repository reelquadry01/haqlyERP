# HAQLY ERP — Comprehensive Security Plan

**Author**: Quadri Atharu
**Version**: 1.0.0
**Last Updated**: 2026-04-14

---

## 1. Authentication

### 1.1 JWT + Refresh Tokens
- Access tokens: JWT with RS256 signing, 15-minute expiry
- Refresh tokens: Opaque random 256-bit tokens, 7-day expiry, single-use rotation
- Token payload: `sub` (user UUID), `email`, `role`, `company_id`, `iat`, `exp`
- Refresh token stored in `httpOnly`, `Secure`, `SameSite=Strict` cookie
- Access token transmitted via `Authorization: Bearer` header

### 1.2 Multi-Factor Authentication (TOTP)
- TOTP-based MFA using RFC 6238 (30-second step, 6 digits)
- Secret generation via `totp-rs` crate during setup
- QR code generated for authenticator apps (Google Authenticator, Authy)
- 8 recovery codes (alphanumeric, 10 chars each), argon2id hashed in DB
- MFA enforced for SuperAdmin and Admin roles; optional for others
- MFA bypass: recovery codes only, no SMS fallback (security vs cost)

### 1.3 Session Management
- Concurrent session limit: 3 per user
- Session invalidation on password change
- Automatic session cleanup: expired sessions purged every 60 minutes
- "Sign out all devices" functionality for users
- Session fingerprint: IP + User-Agent hash stored, alert on mismatch

### 1.4 Brute-Force Protection
- Rate limiting: 5 failed login attempts per email per 15 minutes
- Account lockout: 30-minute lock after 5 failures
- Progressive delays: 1s, 2s, 4s, 8s, 16s between attempts
- IP-based rate limit: 20 login requests per IP per hour
- Login attempt logging with IP, timestamp, and user agent

---

## 2. Authorization

### 2.1 RBAC with 9 Roles
| Role | Level | Description |
|---|---|---|
| SuperAdmin | 0 | Full system access across all companies |
| Admin | 1 | Company-level full access |
| Accountant | 2 | Accounting, journals, financial reports |
| SalesManager | 2 | Sales module full access |
| PurchaseManager | 2 | Purchases module full access |
| InventoryManager | 2 | Inventory module full access |
| HRManager | 2 | Payroll and employee management |
| TreasuryManager | 2 | Loans, treasury, banking |
| Viewer | 3 | Read-only access to permitted modules |

### 2.2 Permissions (22+)
- `users:view`, `users:create`, `users:update`, `users:delete`
- `accounting:coa`, `accounting:journal`, `accounting:voucher`, `accounting:period`
- `sales:view`, `sales:create`, `sales:approve`
- `purchases:view`, `purchases:create`, `purchases:approve`
- `inventory:view`, `inventory:create`
- `tax:view`, `tax:compute`, `tax:file`
- `payroll:view`, `payroll:run`, `payroll:approve`
- `reports:view`, `reports:export`
- `einvoicing:manage`, `admin:roles`, `admin:settings`
- `bi:view`, `bi:create`, `crm:view`, `crm:create`

### 2.3 Segregation of Duties (SoD)
- Creator cannot approve own transactions (journal entries, payment vouchers, payroll runs)
- Payment voucher creator ≠ payment voucher approver
- Payroll run processor ≠ payroll run approver
- Tax computation creator ≠ tax computation filer
- SoD rules enforced at middleware level, not just UI

### 2.4 Permission Inheritance
- Roles inherit from lower-level roles (e.g., Admin inherits all Accountant permissions)
- Company-scoped: permissions apply only within user's assigned company
- Branch-scoped: optional branch restriction for multi-branch deployments
- SuperAdmin cross-company: explicit company_id switching required

---

## 3. Data Protection

### 3.1 Encryption at Rest
- **Algorithm**: AES-256-GCM for sensitive field-level encryption
- **Encrypted fields**: bank_account_number, tax_identification_number, api_secret, einvoice credentials, pension_number
- **Key management**: Master key encrypted with KEK (Key Encryption Key), stored in environment variable or OS keychain
- **Key rotation**: 90-day schedule, re-encryption of all encrypted fields on rotation
- **IV/Nonce**: 96-bit random nonce per encryption operation, stored alongside ciphertext

### 3.2 Password Hashing
- **Algorithm**: Argon2id (winner of Password Hashing Competition)
- **Parameters**: memory=65536 KiB, iterations=3, parallelism=4, output=32 bytes
- **Salt**: 16-byte random salt per password
- **Verification**: constant-time comparison to prevent timing attacks

### 3.3 TLS 1.3 in Transit
- All client-server communication over TLS 1.3
- Certificate pinning in Tauri desktop app
- HSTS header: `max-age=31536000; includeSubDomains; preload`
- No TLS 1.0, 1.1, or 1.2 fallback

### 3.4 Key Rotation Schedule
| Key Type | Rotation Period | Procedure |
|---|---|---|
| JWT signing key | 90 days | Key ID header, old key valid for 1 hour |
| AES field encryption | 90 days | Re-encrypt all encrypted fields |
| API keys | On demand | Generate new, revoke old |
| MFA recovery codes | On use | Auto-regenerate single-use codes |

---

## 4. API Security

### 4.1 CORS
- Strict allowlist: only `tauri://localhost`, `https://localhost:3001` (dev), production domain
- No wildcard (`*`) origins in production
- Credentials: not allowed cross-origin
- Methods: only GET, POST, PATCH, DELETE, OPTIONS
- Headers: only Authorization, Content-Type, Accept, X-Request-ID

### 4.2 Request Signing (Future)
- HMAC-SHA256 signature of request body + timestamp + path
- Header: `X-Request-Signature: t=<timestamp>, v1=<hmac>`
- 5-minute timestamp tolerance to prevent replay attacks
- Signed with per-session secret exchanged during auth

### 4.3 Input Validation
- `validator` crate for all incoming request DTOs
- Type-safe parsing: no stringly-typed fields where enums exist
- Length limits: all strings bounded (max 255 for names, 5000 for text fields)
- Numeric ranges: amounts 0..1_000_000_000, rates 0..100, percentages 0..100
- SQL injection prevention: parameterized queries exclusively via `sqlx` (no string concatenation)

### 4.4 Rate Limiting per Endpoint
| Endpoint Group | Rate Limit | Window |
|---|---|---|
| Login | 5 requests | 15 min |
| Password reset | 3 requests | 60 min |
| API read (GET) | 100 requests | 1 min |
| API write (POST/PATCH) | 30 requests | 1 min |
| API delete | 10 requests | 1 min |
| Report export | 5 requests | 5 min |

---

## 5. Audit Trail

### 5.1 Comprehensive Logging
- Every write operation (INSERT, UPDATE, DELETE) logged to `audit_log` table
- Captured data: `user_id`, `action`, `table_name`, `record_id`, `old_values` (JSONB), `new_values` (JSONB), `ip_address`, `user_agent`, `timestamp`
- Read operations: not logged by default (configurable for sensitive data)

### 5.2 Immutable Audit Logs
- Audit table: INSERT-only, no UPDATE or DELETE permissions
- Database trigger prevents modification of audit rows
- Application-level: no service method exposes audit log mutation

### 5.3 Audit Log Integrity Verification
- Chain hash: each audit entry includes SHA-256 hash of previous entry
- Tamper detection: verify chain integrity on demand
- Daily integrity check scheduled task
- Alert on chain break: security incident triggered

---

## 6. Desktop Security (Tauri)

### 6.1 Content Security Policy
- CSP enforcement via Tauri configuration
- `default-src 'self'`; `script-src 'self'`; `style-src 'self' 'unsafe-inline'`
- No `eval()`, no `Function()`, no inline event handlers
- `connect-src` limited to API backend and whitelisted external services

### 6.2 Secure IPC
- All Tauri IPC commands validated on Rust side
- No arbitrary command execution from renderer
- Command allowlist in `tauri.conf.json`
- Input sanitization on both renderer and Rust side

### 6.3 Local Credential Encryption
- Sensitive local storage encrypted with OS keychain integration
- `keyring` crate for credential storage (Windows Credential Manager, macOS Keychain, Linux Secret Service)
- No plaintext credentials in localStorage or files

### 6.4 Auto-Update Security
- Update signature verification (Ed25519)
- `tauri.conf.json` `updater.pubkey` set to HAQLY's public key
- No unsigned updates applied
- User confirmation before applying updates

---

## 7. Nigerian Compliance

### 7.1 NDPR (Nigeria Data Protection Regulation) Alignment
- Data processing lawful basis documented per data category
- Data subject rights: access, rectification, erasure, portability
- Data retention periods defined per data type
- Privacy notice displayed at registration
- Data Protection Impact Assessment (DPIA) for AI features
- Designated Data Protection Officer role

### 7.2 CBN Data Security Standards
- Financial data classification: Confidential, Internal, Public
- Encryption at rest for all financial data fields
- Access logging for financial data access
- Incident reporting to CBN within 24 hours
- Annual penetration testing requirement

### 7.3 FIRS Data Handling Requirements
- E-invoicing credentials stored encrypted
- Tax computation data retained for 6 years (FIRS requirement)
- IRN (Invoice Reference Number) immutability enforced
- Tax data export controls (no bulk export without authorization)

---

## 8. Penetration Testing Checklist

- [ ] Authentication bypass attempts
- [ ] JWT token forgery and manipulation
- [ ] SQL injection on all input vectors
- [ ] XSS in all user-rendered content
- [ ] CSRF on state-changing endpoints
- [ ] IDOR (Insecure Direct Object Reference) on company-scoped data
- [ ] Privilege escalation from lower roles to higher
- [ ] Rate limiting bypass
- [ ] API endpoint enumeration
- [ ] Encryption key extraction attempts
- [ ] Audit log tampering
- [ ] Tauri IPC command injection
- [ ] Local storage data extraction
- [ ] Auto-update chain compromise
- [ ] Session fixation and hijacking

---

## 9. Vulnerability Disclosure Policy

- Responsible disclosure: security@haqly.com
- 90-day disclosure deadline after vendor notification
- No legal action against good-faith researchers
- CVE assignment through MITRE
- Public acknowledgment for reporters (opt-in)

---

## 10. Incident Response Plan

### Severity Levels
| Level | Description | Response Time |
|---|---|---|
| P1 - Critical | Active data breach, system compromise | 15 minutes |
| P2 - High | Vulnerability being exploited, auth bypass | 1 hour |
| P3 - Medium | Vulnerability confirmed, no active exploit | 24 hours |
| P4 - Low | Security weakness, no immediate risk | 7 days |

### Response Steps
1. **Detect**: Automated monitoring, user reports, security scans
2. **Contain**: Isolate affected systems, revoke compromised credentials
3. **Analyze**: Determine scope, root cause, affected data
4. **Remediate**: Apply fix, rotate keys, patch vulnerability
5. **Report**: NDPR notification (72 hours for data breach), CBN notification if financial data
6. **Review**: Post-mortem, update security controls, prevent recurrence
