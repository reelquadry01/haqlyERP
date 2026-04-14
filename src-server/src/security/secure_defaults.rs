// HAQLY ERP - Secure Defaults Audit & Enforcement
// Based on Trail of Bits insecure-defaults security practices
// Author: Quadri Atharu

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityFinding {
    pub category: String,
    pub severity: Severity,
    pub description: String,
    pub remediation: String,
    pub is_fixed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Severity {
    Critical,
    High,
    Medium,
    Low,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityAuditResult {
    pub findings: Vec<SecurityFinding>,
    pub critical_count: usize,
    pub high_count: usize,
    pub medium_count: usize,
    pub low_count: usize,
    pub overall_status: String,
}

pub struct SecureDefaultsChecker;

impl SecureDefaultsChecker {
    pub fn check_insecure_defaults(settings: &crate::config::settings::Settings) -> Vec<SecurityFinding> {
        let mut findings = Vec::new();

        if settings.jwt_secret == "change-me-in-production" || settings.jwt_secret.len() < 32 {
            findings.push(SecurityFinding {
                category: "Authentication".into(),
                severity: Severity::Critical,
                description: "JWT secret is default or too short (<32 chars)".into(),
                remediation: "Generate a strong random secret: openssl rand -hex 64".into(),
                is_fixed: false,
            });
        }

        if settings.database_url.contains("haqly:haqly@") && !settings.database_url.contains("localhost") {
            findings.push(SecurityFinding {
                category: "Database".into(),
                severity: Severity::Critical,
                description: "Database using default credentials in non-local environment".into(),
                remediation: "Use strong, unique database credentials for production".into(),
                is_fixed: false,
            });
        }

        if settings.cors_origins.iter().any(|o| o == "*") {
            findings.push(SecurityFinding {
                category: "API".into(),
                severity: Severity::High,
                description: "CORS allows all origins (*)".into(),
                remediation: "Restrict CORS to specific trusted origins".into(),
                is_fixed: false,
            });
        }

        if settings.jwt_expiration > 86400 {
            findings.push(SecurityFinding {
                category: "Authentication".into(),
                severity: Severity::Medium,
                description: "JWT expiration exceeds 24 hours".into(),
                remediation: "Set JWT expiration to 1-8 hours, use refresh tokens for longer sessions".into(),
                is_fixed: false,
            });
        }

        if settings.firs_api_key.is_empty() && settings.firs_environment == "PRODUCTION" {
            findings.push(SecurityFinding {
                category: "E-Invoicing".into(),
                severity: Severity::High,
                description: "FIRS API key not configured in production environment".into(),
                remediation: "Configure FIRS API credentials before enabling production e-invoicing".into(),
                is_fixed: false,
            });
        }

        findings.push(SecurityFinding {
            category: "Encryption".into(),
            severity: Severity::Medium,
            description: "Verify AES-256-GCM encryption is enabled for sensitive fields".into(),
            remediation: "Ensure encryption_service is used for PII and financial data at rest".into(),
            is_fixed: false,
        });

        findings.push(SecurityFinding {
            category: "Rate Limiting".into(),
            severity: Severity::Medium,
            description: "Verify rate limiting is configured on all public endpoints".into(),
            remediation: "Enable rate_limit middleware with appropriate thresholds per endpoint".into(),
            is_fixed: false,
        });

        findings.push(SecurityFinding {
            category: "Password Policy".into(),
            severity: Severity::High,
            description: "Verify argon2id password hashing with sufficient memory/time cost".into(),
            remediation: "Use argon2id with m=65536, t=3, p=4 as minimum parameters".into(),
            is_fixed: false,
        });

        findings
    }

    pub fn audit_full(settings: &crate::config::settings::Settings) -> SecurityAuditResult {
        let findings = Self::check_insecure_defaults(settings);
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
