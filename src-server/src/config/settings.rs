// Author: Quadri Atharu
use crate::config::env::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    pub database_url: String,
    pub jwt_secret: String,
    pub jwt_expiration: u64,
    pub server_port: u16,
    pub cors_origins: Vec<String>,
    pub firs_base_url: String,
    pub firs_api_key: String,
    pub firs_api_secret: String,
    pub firs_environment: String,
    pub ollama_base_url: String,
    pub python_engine_url: String,
    pub redis_url: String,
}

impl Settings {
    pub fn load() -> anyhow::Result<Self> {
        dotenvy::dotenv().ok();

        let is_dev = std::env::var("HAQLY_ENV").unwrap_or_default() == "development";

        let fail_secure = |key: &str, label: &str| -> String {
            if is_dev {
                format!("dev-only-{}", label)
            } else {
                eprintln!("FATAL: {} environment variable must be set in production", key);
                std::process::exit(1);
            }
        };

        Ok(Settings {
            database_url: std::env::var(DATABASE_URL).unwrap_or_else(|_| {
                if is_dev {
                    "postgres://localhost:5432/haqly_erp".to_string()
                } else {
                    eprintln!("FATAL: DATABASE_URL environment variable must be set in production");
                    std::process::exit(1);
                }
            }),
            jwt_secret: std::env::var(JWT_SECRET).unwrap_or_else(|_| {
                if is_dev {
                    "dev-only-secret-change-in-production".to_string()
                } else {
                    eprintln!("FATAL: JWT_SECRET environment variable must be set in production");
                    std::process::exit(1);
                }
            }),
            jwt_expiration: env_or_parse(JWT_EXPIRATION, 86400u64),
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
        })
    }
}
