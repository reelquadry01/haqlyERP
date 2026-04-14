// Author: Quadri Atharu
use std::env;
use std::str::FromStr;

pub const SERVER_PORT: &str = "SERVER_PORT";
pub const DATABASE_URL: &str = "DATABASE_URL";
pub const JWT_SECRET: &str = "JWT_SECRET";
pub const JWT_EXPIRATION: &str = "JWT_EXPIRATION";
pub const CORS_ORIGINS: &str = "CORS_ORIGINS";
pub const FIRS_BASE_URL: &str = "FIRS_BASE_URL";
pub const FIRS_API_KEY: &str = "FIRS_API_KEY";
pub const FIRS_API_SECRET: &str = "FIRS_API_SECRET";
pub const FIRS_ENVIRONMENT: &str = "FIRS_ENVIRONMENT";
pub const OLLAMA_BASE_URL: &str = "OLLAMA_BASE_URL";
pub const PYTHON_ENGINE_URL: &str = "PYTHON_ENGINE_URL";
pub const REDIS_URL: &str = "REDIS_URL";
pub const RUST_LOG: &str = "RUST_LOG";
pub const RSA_PRIVATE_KEY_PATH: &str = "RSA_PRIVATE_KEY_PATH";
pub const RSA_PUBLIC_KEY_PATH: &str = "RSA_PUBLIC_KEY_PATH";
pub const SMTP_HOST: &str = "SMTP_HOST";
pub const SMTP_PORT: &str = "SMTP_PORT";
pub const SMTP_USERNAME: &str = "SMTP_USERNAME";
pub const SMTP_PASSWORD: &str = "SMTP_PASSWORD";
pub const SMTP_FROM_EMAIL: &str = "SMTP_FROM_EMAIL";
pub const EMAIL_ENABLED: &str = "EMAIL_ENABLED";

pub fn env_or(key: &str, default: &str) -> String {
    env::var(key).unwrap_or_else(|_| default.to_string())
}

pub fn env_or_parse<T: FromStr>(key: &str, default: T) -> T {
    env::var(key)
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(default)
}
