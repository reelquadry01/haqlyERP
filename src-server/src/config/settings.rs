// Author: Quadri Atharu
use crate::config::env::*;
use crate::config::rsa_keys::{self, RsaKeypair};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    pub database_url: String,
    pub jwt_expiration: u64,
    pub rsa_private_key_path: String,
    pub rsa_public_key_path: String,
    #[serde(skip)]
    pub rsa_keypair: Arc<RsaKeypair>,
    pub server_port: u16,
    pub cors_origins: Vec<String>,
    pub firs_base_url: String,
    pub firs_api_key: String,
    pub firs_api_secret: String,
    pub firs_environment: String,
    pub ollama_base_url: String,
    pub python_engine_url: String,
    pub redis_url: String,
    pub smtp_host: String,
    pub smtp_port: u16,
    pub smtp_username: String,
    pub smtp_password: String,
    pub smtp_from_email: String,
    pub email_enabled: bool,
}

impl Settings {
    pub fn load() -> anyhow::Result<Self> {
        dotenvy::dotenv().ok();

        let is_dev = std::env::var("HAQLY_ENV").unwrap_or_default() == "development";

        let rsa_private_key_path = env_or(
            RSA_PRIVATE_KEY_PATH,
            if cfg!(windows) {
                let app_data = std::env::var("APPDATA").unwrap_or_else(|_| ".".to_string());
                format!("{}\\haqly-erp\\keys\\private.pem", app_data)
            } else {
                let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
                format!("{}/.haqly-erp/keys/private.pem", home)
            },
        );

        let rsa_public_key_path = env_or(
            RSA_PUBLIC_KEY_PATH,
            if cfg!(windows) {
                let app_data = std::env::var("APPDATA").unwrap_or_else(|_| ".".to_string());
                format!("{}\\haqly-erp\\keys\\public.pem", app_data)
            } else {
                let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
                format!("{}/.haqly-erp/keys/public.pem", home)
            },
        );

        let rsa_keypair = rsa_keys::ensure_rsa_keypair(&rsa_private_key_path, &rsa_public_key_path)?;

        Ok(Settings {
            database_url: std::env::var(DATABASE_URL).unwrap_or_else(|_| {
                if is_dev {
                    "postgres://localhost:5432/haqly_erp".to_string()
                } else {
                    eprintln!("FATAL: DATABASE_URL environment variable must be set in production");
                    std::process::exit(1);
                }
            }),
            jwt_expiration: env_or_parse(JWT_EXPIRATION, 86400u64),
            rsa_private_key_path,
            rsa_public_key_path,
            rsa_keypair: Arc::new(rsa_keypair),
            server_port: env_or_parse(SERVER_PORT, 8080u16),
            cors_origins: env_or(CORS_ORIGINS, "http://localhost:3000")
                .split(',')
                .map(|s| s.trim().to_string())
                .collect(),
            firs_base_url: env_or(FIRS_BASE_URL, "https://einvoice.firs.gov.ng"),
            firs_api_key: std::env::var(FIRS_API_KEY).unwrap_or_else(|_| {
                if is_dev {
                    String::new()
                } else {
                    eprintln!("FATAL: FIRS_API_KEY environment variable must be set in production");
                    std::process::exit(1);
                }
            }),
            firs_api_secret: std::env::var(FIRS_API_SECRET).unwrap_or_else(|_| {
                if is_dev {
                    String::new()
                } else {
                    eprintln!("FATAL: FIRS_API_SECRET environment variable must be set in production");
                    std::process::exit(1);
                }
            }),
            firs_environment: env_or(FIRS_ENVIRONMENT, "SANDBOX"),
            ollama_base_url: env_or(OLLAMA_BASE_URL, "http://localhost:11434"),
            python_engine_url: env_or(PYTHON_ENGINE_URL, "http://localhost:8000"),
            redis_url: env_or(REDIS_URL, "redis://localhost:6379"),
            smtp_host: env_or(SMTP_HOST, "smtp.example.com"),
            smtp_port: env_or_parse(SMTP_PORT, 587u16),
            smtp_username: std::env::var(SMTP_USERNAME).unwrap_or_default(),
            smtp_password: std::env::var(SMTP_PASSWORD).unwrap_or_default(),
            smtp_from_email: env_or(SMTP_FROM_EMAIL, "noreply@haqly-erp.com"),
            email_enabled: if is_dev {
                env_or_parse(EMAIL_ENABLED, false)
            } else {
                env_or_parse(EMAIL_ENABLED, true)
            },
        })
    }
}
