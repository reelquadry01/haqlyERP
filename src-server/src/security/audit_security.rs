// Author: Quadri Atharu

use super::secure_defaults::{SecurityAuditResult, SecurityFinding, Severity};

pub struct AuditSecurityChecker;

impl AuditSecurityChecker {
    pub fn check_auth_security(settings: &crate::config::settings::Settings) -> Vec<SecurityFinding> {
        let mut findings = Vec::new();

        findings.push(SecurityFinding {
            category: "Auth::JWT".into(),
            severity: Severity::Low,
            description: "JWT uses RS256 asymmetric signing (key rotation via kid header supported)".into(),
            remediation: "Rotate RSA keypairs periodically; archive old keys for token validation during transition".into(),
            is_fixed: true,
        });

        if settings.jwt_expiration > 28800 {
            findings.push(SecurityFinding {
                category: "Auth::Session".into(),
                severity: Severity::High,
                description: format!("JWT expiration of {} seconds exceeds recommended 8-hour maximum", settings.jwt_expiration),
                remediation: "Set JWT expiration to 1-8 hours (3600-28800 seconds) and implement refresh token rotation".into(),
                is_fixed: false,
            });
        }

        findings.push(SecurityFinding {
            category: "Auth::PasswordHashing".into(),
            severity: Severity::High,
            description: "Verify argon2id password hashing parameters meet OWASP recommendations".into(),
            remediation: "Use argon2id with memory cost >= 65536 KiB, time cost >= 3 iterations, parallelism >= 4 lanes".into(),
            is_fixed: false,
        });

        findings.push(SecurityFinding {
            category: "Auth::SessionManagement".into(),
            severity: Severity::Medium,
            description: "Verify server-side session revocation is implemented for logout and password change".into(),
            remediation: "Implement a session store (Redis-backed) that allows immediate token revocation on logout or credential change".into(),
            is_fixed: false,
        });

        findings.push(SecurityFinding {
            category: "Auth::MFA".into(),
            severity: Severity::High,
            description: "Multi-factor authentication status must be verified for admin and financial roles".into(),
            remediation: "Enforce TOTP-based MFA for all users with admin, accountant, or auditor roles".into(),
            is_fixed: false,
        });

        findings.push(SecurityFinding {
            category: "Auth::TokenStorage".into(),
            severity: Severity::Medium,
            description: "Verify JWT tokens are stored securely on the client side (httpOnly cookies or secure storage)".into(),
            remediation: "Use httpOnly, Secure, SameSite=Strict cookies for token storage; never store tokens in localStorage".into(),
            is_fixed: false,
        });

        findings
    }

    pub fn check_authorization_security(_settings: &crate::config::settings::Settings) -> Vec<SecurityFinding> {
        let mut findings = Vec::new();

        findings.push(SecurityFinding {
            category: "Authorization::RBAC".into(),
            severity: Severity::Critical,
            description: "Verify Role-Based Access Control module is enabled and enforced on all routes".into(),
            remediation: "Ensure the rbac middleware is applied globally and every route has an associated permission requirement".into(),
            is_fixed: false,
        });

        findings.push(SecurityFinding {
            category: "Authorization::RoleHierarchy".into(),
            severity: Severity::High,
            description: "Verify role hierarchy prevents privilege escalation (e.g., viewer cannot acquire admin permissions)".into(),
            remediation: "Implement a strict role hierarchy: SuperAdmin > Admin > Accountant > Auditor > Viewer with no upward permission inheritance".into(),
            is_fixed: false,
        });

        findings.push(SecurityFinding {
            category: "Authorization::PermissionGranularity".into(),
            severity: Severity::Medium,
            description: "Verify permissions are granular (resource+action pairs) rather than coarse role checks".into(),
            remediation: "Define permissions as resource:action pairs (e.g., journal:post, account:read, report:export) and check both role and specific permission".into(),
            is_fixed: false,
        });

        findings.push(SecurityFinding {
            category: "Authorization::AdminControls".into(),
            severity: Severity::High,
            description: "Verify admin-only endpoints require re-authentication for destructive operations (user deletion, role changes)".into(),
            remediation: "Implement step-up authentication for admin operations: require password confirmation before user deletion, role modification, or system configuration changes".into(),
            is_fixed: false,
        });

        findings.push(SecurityFinding {
            category: "Authorization::DataIsolation".into(),
            severity: Severity::High,
            description: "Verify multi-tenant data isolation ensures users can only access their organization's data".into(),
            remediation: "Enforce organization_id filtering on every database query at the repository layer; never rely on client-side filtering alone".into(),
            is_fixed: false,
        });

        findings
    }

    pub fn check_data_protection(_settings: &crate::config::settings::Settings) -> Vec<SecurityFinding> {
        let mut findings = Vec::new();

        findings.push(SecurityFinding {
            category: "DataProtection::EncryptionAtRest".into(),
            severity: Severity::Critical,
            description: "Verify AES-256-GCM encryption is applied to all sensitive data at rest".into(),
            remediation: "Enable the encryption_service with AES-256-GCM for all PII fields, financial amounts, and credentials stored in the database".into(),
            is_fixed: false,
        });

        findings.push(SecurityFinding {
            category: "DataProtection::PIIFields".into(),
            severity: Severity::High,
            description: "Verify PII fields (names, emails, phone numbers, tax IDs, bank details) are encrypted at the column level".into(),
            remediation: "Apply column-level encryption via encryption_service.encrypt() for all PII fields before database insertion; decrypt on read with access logging".into(),
            is_fixed: false,
        });

        findings.push(SecurityFinding {
            category: "DataProtection::DatabaseConnection".into(),
            severity: Severity::High,
            description: "Verify database connections use SSL/TLS encryption (sslmode=verify-full)".into(),
            remediation: "Set sslmode=verify-full in the database URL and provide the CA certificate; reject connections that cannot be verified".into(),
            is_fixed: false,
        });

        findings.push(SecurityFinding {
            category: "DataProtection::BackupEncryption".into(),
            severity: Severity::Medium,
            description: "Verify database backups are encrypted with AES-256 before storage or transfer".into(),
            remediation: "Configure pg_dump with --encrypt option or pipe backups through AES-256 encryption before writing to storage".into(),
            is_fixed: false,
        });

        findings.push(SecurityFinding {
            category: "DataProtection::KeyManagement".into(),
            severity: Severity::High,
            description: "Verify encryption keys are rotated on a defined schedule and stored in a secure key management system".into(),
            remediation: "Implement key rotation every 90 days; store master keys in HSM or vault (HashiCorp Vault / AWS KMS); never embed keys in source code".into(),
            is_fixed: false,
        });

        findings
    }

    pub fn check_api_security(settings: &crate::config::settings::Settings) -> Vec<SecurityFinding> {
        let mut findings = Vec::new();

        findings.push(SecurityFinding {
            category: "API::RateLimiting".into(),
            severity: Severity::High,
            description: "Verify rate limiting is configured and enforced on all public API endpoints".into(),
            remediation: "Enable the rate_limit middleware with per-endpoint thresholds: auth endpoints 5 req/min, general API 60 req/min, bulk operations 10 req/min".into(),
            is_fixed: false,
        });

        findings.push(SecurityFinding {
            category: "API::InputValidation".into(),
            severity: Severity::High,
            description: "Verify all request inputs are validated with serde deserialization and explicit constraints".into(),
            remediation: "Use #[derive(Deserialize)] with #[serde(rename_all)] and custom validators; reject unexpected fields with serde(deny_unknown_fields)".into(),
            is_fixed: false,
        });

        findings.push(SecurityFinding {
            category: "API::SQLInjection".into(),
            severity: Severity::Critical,
            description: "Verify all database queries use parameterized statements via sqlx (no string interpolation in SQL)".into(),
            remediation: "Use sqlx::query!() and sqlx::query_as!() macros exclusively; never format SQL strings with user input".into(),
            is_fixed: false,
        });

        findings.push(SecurityFinding {
            category: "API::XSS".into(),
            severity: Severity::Medium,
            description: "Verify output encoding prevents XSS in all API responses that may be rendered as HTML".into(),
            remediation: "Sanitize all user-provided text stored in the database; encode HTML entities on output; set Content-Type headers explicitly".into(),
            is_fixed: false,
        });

        if settings.cors_origins.iter().any(|o| o == "*") {
            findings.push(SecurityFinding {
                category: "API::CORS".into(),
                severity: Severity::High,
                description: "CORS configuration allows all origins (*)".into(),
                remediation: "Restrict CORS origins to the specific frontend domain(s) and Tauri scheme; remove wildcard origin".into(),
                is_fixed: false,
            });
        } else {
            findings.push(SecurityFinding {
                category: "API::CORS".into(),
                severity: Severity::Low,
                description: "CORS origins are restricted (good); verify they match production frontend URLs".into(),
                remediation: "Ensure CORS origins list exactly matches the deployed frontend URL and Tauri custom protocol".into(),
                is_fixed: true,
            });
        }

        findings.push(SecurityFinding {
            category: "API::RequestSizeLimit".into(),
            severity: Severity::Medium,
            description: "Verify request body size limits are configured to prevent denial-of-service via large payloads".into(),
            remediation: "Set axum body size limit to 1MB for general endpoints, 10MB for file upload endpoints; reject oversized requests with 413 status".into(),
            is_fixed: false,
        });

        findings
    }

    pub fn check_desktop_security(_settings: &crate::config::settings::Settings) -> Vec<SecurityFinding> {
        let mut findings = Vec::new();

        findings.push(SecurityFinding {
            category: "Desktop::CSP".into(),
            severity: Severity::High,
            description: "Verify Tauri Content Security Policy restricts script sources, connect-src, and object-src".into(),
            remediation: "Set CSP to: script-src 'self'; connect-src 'self' https://einvoice.firs.gov.ng; object-src 'none'; style-src 'self' 'unsafe-inline'; upgrade-insecure-requests".into(),
            is_fixed: false,
        });

        findings.push(SecurityFinding {
            category: "Desktop::IPC".into(),
            severity: Severity::Critical,
            description: "Verify Tauri IPC allowlist restricts commands to only those required by the application".into(),
            remediation: "Configure tauri.conf.json allowlist to deny all by default; explicitly enable only required IPC commands (shell.execute: false, fs.readDir: true with scope)".into(),
            is_fixed: false,
        });

        findings.push(SecurityFinding {
            category: "Desktop::SidecarIsolation".into(),
            severity: Severity::Medium,
            description: "Verify sidecar processes (Python engine, Ollama) run with minimal OS privileges and are isolated from the main process".into(),
            remediation: "Run sidecar processes under a restricted user account; use OS-level sandboxing (AppArmor/seccomp on Linux); communicate only via localhost HTTP".into(),
            is_fixed: false,
        });

        findings.push(SecurityFinding {
            category: "Desktop::LocalStorage".into(),
            severity: Severity::High,
            description: "Verify local storage uses encryption for any persisted sensitive data (tokens, credentials, keys)".into(),
            remediation: "Use the encryption_service to encrypt all sensitive data before writing to local storage or tauri::fs; never store plaintext secrets on disk".into(),
            is_fixed: false,
        });

        findings.push(SecurityFinding {
            category: "Desktop::UpdateVerification".into(),
            severity: Severity::Critical,
            description: "Verify application updates are verified with cryptographic signatures before installation".into(),
            remediation: "Enable Tauri updater with endpoint signature verification; use Ed25519 signing keys; reject updates that fail signature validation".into(),
            is_fixed: false,
        });

        findings
    }

    pub fn audit_all(settings: &crate::config::settings::Settings) -> SecurityAuditResult {
        let mut findings = Vec::new();
        findings.extend(Self::check_auth_security(settings));
        findings.extend(Self::check_authorization_security(settings));
        findings.extend(Self::check_data_protection(settings));
        findings.extend(Self::check_api_security(settings));
        findings.extend(Self::check_desktop_security(settings));

        let critical_count = findings.iter().filter(|f| f.severity == Severity::Critical).count();
        let high_count = findings.iter().filter(|f| f.severity == Severity::High).count();
        let medium_count = findings.iter().filter(|f| f.severity == Severity::Medium).count();
        let low_count = findings.iter().filter(|f| f.severity == Severity::Low).count();

        let overall_status = if critical_count > 0 {
            "CRITICAL - Immediate action required".into()
        } else if high_count > 0 {
            "WARNING - Security issues need attention".into()
        } else if medium_count > 0 {
            "MODERATE - Some improvements recommended".into()
        } else {
            "GOOD - No significant security issues detected".into()
        };

        SecurityAuditResult {
            findings,
            critical_count,
            high_count,
            medium_count,
            low_count,
            overall_status,
        }
    }
}
