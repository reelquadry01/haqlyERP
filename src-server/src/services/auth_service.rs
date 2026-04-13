// Author: Quadri Atharu
use anyhow::{anyhow, Context, Result};
use argon2::password_hash::SaltString;
use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use base64::Engine;
use chrono::Utc;
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use rand::rngs::OsRng;
use sqlx::PgPool;
use uuid::Uuid;

use crate::dtos::auth_dto::{
    AuthResponse, LoginRequest, MfaSetupResponse, MfaVerifyRequest, RefreshRequest, RegisterRequest,
};
use crate::models::user::{Session, User};

#[derive(Clone)]
pub struct AuthService {
    pub pool: PgPool,
    pub jwt_secret: String,
    pub jwt_expiration: u64,
}

impl AuthService {
    pub fn new(pool: PgPool, jwt_secret: String, jwt_expiration: u64) -> Self {
        Self {
            pool,
            jwt_secret,
            jwt_expiration,
        }
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

        let access_token = self.generate_jwt(&user, &self.jwt_secret, self.jwt_expiration)?;
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

        let access_token = self.generate_jwt(&user, &self.jwt_secret, self.jwt_expiration)?;
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

        let access_token = self.generate_jwt(&user, &self.jwt_secret, self.jwt_expiration)?;
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
        let secret = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(rand::random::<[u8; 20]>());
        let recovery_codes: Vec<String> = (0..8)
            .map(|_| Uuid::now_v7().to_string().replace("-", "").chars().take(8).collect())
            .collect();

        sqlx::query("UPDATE users SET mfa_secret = $1, mfa_enabled = true WHERE id = $2")
            .bind(&secret)
            .bind(user_id)
            .execute(&self.pool)
            .await?;

        let qr_code_url = format!(
            "otpauth://totp/HAQLY-ERP:user-{}?secret={}&issuer=HAQLY-ERP",
            user_id, secret
        );

        Ok(MfaSetupResponse {
            secret,
            qr_code_url,
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
        let time_counter = Utc::now().timestamp() / 30;
        let expected = self.generate_totp_code(&secret, time_counter);
        let prev_expected = self.generate_totp_code(&secret, time_counter - 1);

        if code == expected || code == prev_expected {
            Ok(true)
        } else {
            Ok(false)
        }
    }

    pub fn generate_jwt(&self, user: &User, secret: &str, expiration: u64) -> Result<String> {
        let now = Utc::now();
        let claims = serde_json::json!({
            "sub": user.id.to_string(),
            "email": user.email,
            "company_id": user.company_id.to_string(),
            "iat": now.timestamp(),
            "exp": now.timestamp() + expiration as i64,
        });

        let token = encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(secret.as_bytes()),
        )
        .context("Failed to encode JWT")?;

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

    fn generate_totp_code(&self, secret: &str, time_counter: i64) -> String {
        let key = base64::engine::general_purpose::URL_SAFE_NO_PAD
            .decode(secret)
            .unwrap_or_default();
        let counter_bytes = time_counter.to_be_bytes();
        let mut hmac_input = [0u8; 8];
        hmac_input.copy_from_slice(&counter_bytes);

        use aes_gcm::aead::generic_array::GenericArray;
        let mut mac = [0u8; 20];
        let _ = hmac_sha1(&key, &hmac_input, &mut mac);
        let offset = (mac[19] & 0xf) as usize;
        let code = ((mac[offset] as u32 & 0x7f) << 24)
            | ((mac[offset + 1] as u32) << 16)
            | ((mac[offset + 2] as u32) << 8)
            | (mac[offset + 3] as u32);
        format!("{:06}", code % 1_000_000)
    }
}

fn hmac_sha1(key: &[u8], message: &[u8], output: &mut [u8; 20]) -> Result<(), ()> {
    use std::io::Write;
    let mut hasher = openssl::sha::Sha1::new();
    let _ = hasher.write(key);
    let _ = hasher.write(message);
    let result = hasher.finish();
    output.copy_from_slice(&result);
    Ok(())
}
