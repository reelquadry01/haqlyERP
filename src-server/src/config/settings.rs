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

        Ok(Settings {
            database_url: env_or(DATABASE_URL, "postgres://localhost:5432/haqly_erp"),
            jwt_secret: env_or(JWT_SECRET, "change-me-in-production-haqly-erp-secret-key"),
            jwt_expiration: env_or_parse(JWT_EXPIRATION, 86400u64),
            server_port: env_or_parse(SERVER_PORT, 8080u16),
            cors_origins: env_or(CORS_ORIGINS, "http://localhost:3000")
                .split(',')
                .map(|s| s.trim().to_string())
                .collect(),
            firs_base_url: env_or(FIRS_BASE_URL, "https://einvoice.firs.gov.ng"),
            firs_api_key: env_or(FIRS_API_KEY, ""),
            firs_api_secret: env_or(FIRS_API_SECRET, ""),
            firs_environment: env_or(FIRS_ENVIRONMENT, "SANDBOX"),
            ollama_base_url: env_or(OLLAMA_BASE_URL, "http://localhost:11434"),
            python_engine_url: env_or(PYTHON_ENGINE_URL, "http://localhost:8000"),
            redis_url: env_or(REDIS_URL, "redis://localhost:6379"),
        })
    }
}
