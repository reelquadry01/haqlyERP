// Author: Quadri Atharu
use anyhow::{anyhow, Context, Result};
use argon2::password_hash::SaltString;
use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use base64::Engine;
use chrono::Utc;
use jsonwebtoken::{encode, Algorithm, EncodingKey, Header};
use rand::rngs::OsRng;
use sqlx::PgPool;
use std::sync::Arc;
use totp_rs::{Algorithm as TotpAlgorithm, Secret, TOTP};
use uuid::Uuid;

use crate::config::rsa_keys::RsaKeypair;
use crate::dtos::auth_dto::{
    AuthResponse, LoginRequest, MfaSetupResponse, MfaVerifyRequest, RefreshRequest, RegisterRequest,
};
use crate::models::user::{Session, User};
use crate::middleware::auth::Claims;

#[derive(Clone)]
pub struct AuthService {
    pub pool: PgPool,
    pub rsa_keypair: Arc<RsaKeypair>,
    pub jwt_expiration: u64,
}

impl AuthService {
    pub fn new(pool: PgPool, rsa_keypair: Arc<RsaKeypair>, jwt_expiration: u64) -> Self {
        Self {
            pool,
            rsa_keypair,
            jwt_expiration,
        }
    }

    async fn get_user_role(&self, user_id: Uuid) -> String {
        let role_name: Option<String> = sqlx::query_scalar(
            r#"SELECT r.name FROM user_roles ur JOIN roles r ON ur.role_id = r.id WHERE ur.user_id = $1 LIMIT 1"#,
        )
        .bind(user_id)
        .fetch_optional(&self.pool)
        .await
        .ok()
        .flatten();
        role_name.unwrap_or_else(|| "Viewer".to_string())
    }

    pub async fn register(&self, req: RegisterRequest) -> Result<AuthResponse> {
        let existing = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM users WHERE email = $1 AND company_id = $2",
        )
        .bind(&req.email)
        .bind(req.company_id)
        .fetch_one(&self.pool)
        .await?;

        if existing > 0 {
            return Err(anyhow!("User with this email already exists"));
        }

        let password_hash = self.hash_password(&req.password)?;
        let user_id = Uuid::now_v7();

        sqlx::query(
            r#"INSERT INTO users (id, company_id, email, password_hash, full_name, phone, is_active, mfa_enabled, created_at, updated_at)
               VALUES ($1, $2, $3, $4, $5, $6, true, false, NOW(), NOW())"#,
        )
        .bind(user_id)
        .bind(req.company_id)
        .bind(&req.email)
        .bind(&password_hash)
        .bind(&req.full_name)
        .bind(&req.phone)
        .execute(&self.pool)
        .await?;

        let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = $1")
            .bind(user_id)
            .fetch_one(&self.pool)
            .await?;

        let role = self.get_user_role(user.id).await;
        let access_token = self.generate_jwt(&user, &role, self.jwt_expiration)?;
        let refresh_token = Uuid::now_v7().to_string();
        let expires_in = self.jwt_expiration;

        let session_id = Uuid::now_v7();
        let expires_at = Utc::now().naive_utc() + chrono::Duration::seconds(self.jwt_expiration as i64);
        sqlx::query(
            r#"INSERT INTO sessions (id, user_id, token, refresh_token, expires_at, created_at)
               VALUES ($1, $2, $3, $4, $5, NOW())"#,
        )
        .bind(session_id)
        .bind(user_id)
        .bind(&access_token)
        .bind(&refresh_token)
        .bind(expires_at)
        .execute(&self.pool)
        .await?;

        Ok(AuthResponse {
            user_id: user.id,
            email: user.email,
            full_name: user.full_name,
            access_token,
            refresh_token,
            expires_in,
            company_id: user.company_id,
            mfa_enabled: user.mfa_enabled,
        })
    }

    pub async fn login(&self, req: LoginRequest) -> Result<AuthResponse> {
        let user = sqlx::query_as::<_, User>(
            "SELECT * FROM users WHERE email = $1 AND is_active = true",
        )
        .bind(&req.email)
        .fetch_optional(&self.pool)
        .await?
        .ok_or_else(|| anyhow!("Invalid email or password"))?;

        if !self.verify_password(&req.password, &user.password_hash) {
            return Err(anyhow!("Invalid email or password"));
        }

        if user.mfa_enabled {
            return Ok(AuthResponse {
                user_id: user.id,
                email: user.email.clone(),
                full_name: user.full_name.clone(),
                access_token: String::new(),
                refresh_token: String::new(),
                expires_in: 0,
                company_id: user.company_id,
                mfa_enabled: true,
            });
        }

        sqlx::query("UPDATE users SET last_login_at = NOW() WHERE id = $1")
            .bind(user.id)
            .execute(&self.pool)
            .await?;

        let role = self.get_user_role(user.id).await;
        let access_token = self.generate_jwt(&user, &role, self.jwt_expiration)?;
        let refresh_token = Uuid::now_v7().to_string();
        let expires_in = self.jwt_expiration;

        let session_id = Uuid::now_v7();
        let expires_at = Utc::now().naive_utc() + chrono::Duration::seconds(self.jwt_expiration as i64);
        sqlx::query(
            r#"INSERT INTO sessions (id, user_id, token, refresh_token, expires_at, created_at)
               VALUES ($1, $2, $3, $4, $5, NOW())"#,
        )
        .bind(session_id)
        .bind(user.id)
        .bind(&access_token)
        .bind(&refresh_token)
        .bind(expires_at)
        .execute(&self.pool)
        .await?;

        Ok(AuthResponse {
            user_id: user.id,
            email: user.email,
            full_name: user.full_name,
            access_token,
            refresh_token,
            expires_in,
            company_id: user.company_id,
            mfa_enabled: false,
        })
    }

    pub async fn refresh_token(&self, refresh_token_str: &str) -> Result<AuthResponse> {
        let session = sqlx::query_as::<_, Session>(
            "SELECT * FROM sessions WHERE refresh_token = $1",
        )
        .bind(refresh_token_str)
        .fetch_optional(&self.pool)
        .await?
        .ok_or_else(|| anyhow!("Invalid refresh token"))?;

        let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = $1")
            .bind(session.user_id)
            .fetch_one(&self.pool)
            .await?;

        let role = self.get_user_role(user.id).await;
        let access_token = self.generate_jwt(&user, &role, self.jwt_expiration)?;
        let new_refresh_token = Uuid::now_v7().to_string();
        let expires_at = Utc::now().naive_utc() + chrono::Duration::seconds(self.jwt_expiration as i64);

        sqlx::query(
            "UPDATE sessions SET token = $1, refresh_token = $2, expires_at = $3 WHERE id = $4",
        )
        .bind(&access_token)
        .bind(&new_refresh_token)
        .bind(expires_at)
        .bind(session.id)
        .execute(&self.pool)
        .await?;

        Ok(AuthResponse {
            user_id: user.id,
            email: user.email,
            full_name: user.full_name,
            access_token,
            refresh_token: new_refresh_token,
            expires_in: self.jwt_expiration,
            company_id: user.company_id,
            mfa_enabled: user.mfa_enabled,
        })
    }

    pub async fn setup_mfa(&self, user_id: Uuid) -> Result<MfaSetupResponse> {
        let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = $1")
            .bind(user_id)
            .fetch_one(&self.pool)
            .await?;

        let secret_raw = Secret::generate_secret();
        let secret_encoded = secret_raw.to_encoded();
        let secret_bytes = secret_raw.to_bytes()
            .map_err(|e| anyhow!("Invalid TOTP secret: {}", e))?;

        let totp = TOTP::new(
            TotpAlgorithm::SHA1,
            6,
            1,
            30,
            secret_bytes,
            "HAQLY-ERP".to_string(),
            user.email.clone(),
        ).map_err(|e| anyhow!("Failed to create TOTP: {}", e))?;

        let recovery_codes: Vec<String> = (0..8)
            .map(|_| Uuid::now_v7().to_string().replace("-", "").chars().take(8).collect())
            .collect();
        let hashed_recovery_codes: Vec<String> = recovery_codes.iter()
            .filter_map(|code| self.hash_password(code).ok())
            .collect();
        let recovery_json = serde_json::to_value(&hashed_recovery_codes)
            .unwrap_or(serde_json::Value::Null);

        sqlx::query("UPDATE users SET mfa_secret = $1, mfa_enabled = true, mfa_recovery_codes = $2 WHERE id = $3")
            .bind(secret_encoded.to_string())
            .bind(&recovery_json)
            .bind(user_id)
            .execute(&self.pool)
            .await?;

        Ok(MfaSetupResponse {
            secret: secret_encoded.to_string(),
            qr_code_url: totp.get_url(),
            recovery_codes,
        })
    }

    pub async fn verify_mfa(&self, user_id: Uuid, code: &str) -> Result<bool> {
        let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = $1")
            .bind(user_id)
            .fetch_one(&self.pool)
            .await?;

        if !user.mfa_enabled {
            return Err(anyhow!("MFA not enabled for this user"));
        }

        let secret = user.mfa_secret.ok_or_else(|| anyhow!("MFA secret not found"))?;
        let totp = build_totp(&secret, &user.email)?;

        match totp.check_current(code) {
            Ok(valid) => Ok(valid),
            Err(_) => Err(anyhow!("Time-based verification failed")),
        }
    }

    pub fn generate_jwt(&self, user: &User, role: &str, expiration: u64) -> Result<String> {
        let now = Utc::now();
        let claims = Claims {
            sub: user.id,
            email: user.email.clone(),
            role: role.to_string(),
            company_id: user.company_id,
            iat: now.timestamp() as usize,
            exp: (now.timestamp() + expiration as i64) as usize,
        };

        let mut header = Header::new(Algorithm::RS256);
        header.kid = Some(self.rsa_keypair.kid.clone());

        let encoding_key = EncodingKey::from_rsa_pem(&self.rsa_keypair.private_pem)
            .map_err(|e| anyhow!("Failed to create RSA encoding key: {}", e))?;

        let token = encode(&header, &claims, &encoding_key)
            .context("Failed to encode JWT with RS256")?;

        Ok(token)
    }

    pub fn hash_password(&self, password: &str) -> Result<String> {
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();
        let hash = argon2
            .hash_password(password.as_bytes(), &salt)
            .map_err(|e| anyhow!("Failed to hash password: {}", e))?;
        Ok(hash.to_string())
    }

    pub fn verify_password(&self, password: &str, hash: &str) -> bool {
        let parsed_hash = match PasswordHash::new(hash) {
            Ok(h) => h,
            Err(_) => return false,
        };
        Argon2::default()
            .verify_password(password.as_bytes(), &parsed_hash)
            .is_ok()
    }

}

fn build_totp(secret: &str, account_name: &str) -> Result<TOTP> {
    let secret_bytes = Secret::Encoded(secret.to_string()).to_bytes()
        .map_err(|e| anyhow!("Invalid TOTP secret: {}", e))?;
    let totp = TOTP::new(
        TotpAlgorithm::SHA1,
        6,
        1,
        30,
        secret_bytes,
        "HAQLY-ERP".to_string(),
        account_name.to_string(),
    ).map_err(|e| anyhow!("Failed to create TOTP: {}", e))?;
    Ok(totp)
}
