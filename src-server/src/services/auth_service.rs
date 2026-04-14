// Author: Quadri Atharu
use anyhow::{anyhow, Context, Result};
use argon2::password_hash::SaltString;
use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use base64::Engine;
use chrono::Utc;
use jsonwebtoken::{encode, Algorithm, EncodingKey, Header};
use rand::rngs::OsRng;
use sha2::{Digest, Sha256};
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

#[derive(Debug, Clone, sqlx::FromRow)]
struct PasswordResetToken {
    id: Uuid,
    user_id: Uuid,
    token_hash: String,
    expires_at: chrono::NaiveDateTime,
    used_at: Option<chrono::NaiveDateTime>,
    created_at: chrono::NaiveDateTime,
}

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

    pub async fn request_password_reset(&self, email: &str) -> Result<String> {
        let user = sqlx::query_as::<_, User>(
            "SELECT * FROM users WHERE email = $1 AND is_active = true",
        )
        .bind(email)
        .fetch_optional(&self.pool)
        .await?
        .ok_or_else(|| anyhow!("No active user found with this email"))?;

        let token_raw = Uuid::now_v7().to_string()
            + &Uuid::now_v7().to_string().replace("-", "");
        let token_hash = hex::encode(Sha256::digest(token_raw.as_bytes()));
        let expires_at = Utc::now().naive_utc() + chrono::Duration::hours(1);

        let id = Uuid::now_v7();
        sqlx::query(
            r#"INSERT INTO password_reset_tokens (id, user_id, token_hash, expires_at, created_at)
               VALUES ($1, $2, $3, $4, NOW())"#,
        )
        .bind(id)
        .bind(user.id)
        .bind(&token_hash)
        .bind(expires_at)
        .execute(&self.pool)
        .await?;

        tracing::info!("Password reset token created for user: {}", user.id);
        Ok(token_raw)
    }

    pub async fn verify_password_reset(&self, token: &str, new_password: &str) -> Result<Uuid> {
        let token_hash = hex::encode(Sha256::digest(token.as_bytes()));

        let reset_token = sqlx::query_as::<_, PasswordResetToken>(
            "SELECT * FROM password_reset_tokens WHERE token_hash = $1",
        )
        .bind(&token_hash)
        .fetch_optional(&self.pool)
        .await?
        .ok_or_else(|| anyhow!("Invalid or expired reset token"))?;

        if reset_token.used_at.is_some() {
            return Err(anyhow!("Token has already been used"));
        }

        let now = Utc::now().naive_utc();
        if reset_token.expires_at < now {
            return Err(anyhow!("Token has expired"));
        }

        let new_password_hash = self.hash_password(new_password)?;

        sqlx::query("UPDATE users SET password_hash = $1, updated_at = NOW() WHERE id = $2")
            .bind(&new_password_hash)
            .bind(reset_token.user_id)
            .execute(&self.pool)
            .await?;

        sqlx::query("UPDATE password_reset_tokens SET used_at = NOW() WHERE id = $1")
            .bind(reset_token.id)
            .execute(&self.pool)
            .await?;

        tracing::info!("Password reset successful for user: {}", reset_token.user_id);
        Ok(reset_token.user_id)
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

#[cfg(test)]
mod tests {
    use argon2::password_hash::SaltString;
    use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
    use jsonwebtoken::{decode, Algorithm, DecodingKey, Validation};
    use rand::rngs::OsRng;
    use std::sync::Arc;
    use totp_rs::{Algorithm as TotpAlgorithm, Secret, TOTP};
    use uuid::Uuid;

    use crate::config::rsa_keys::RsaKeypair;
    use crate::middleware::auth::Claims;
    use crate::models::user::User;
    use crate::services::auth_service::AuthService;

    fn mock_pool() -> sqlx::PgPool {
        sqlx::postgres::PgPoolOptions::new()
            .connect_lazy("postgres://test:test@localhost/test")
            .expect("mock pool")
    }

    fn test_rsa_keypair() -> Arc<RsaKeypair> {
        let mut rng = OsRng;
        let private_key = rsa::RsaPrivateKey::new(&mut rng, 2048).unwrap();
        let public_key = rsa::RsaPublicKey::from(&private_key);
        let private_pem = private_key.to_pkcs8_pem(rsa::pkcs8::LineEnding::LF).unwrap();
        let public_pem = public_key.to_public_key_pem(rsa::pkcs8::LineEnding::LF).unwrap();
        Arc::new(RsaKeypair {
            private_pem: private_pem.as_bytes().to_vec(),
            public_pem: public_pem.as_bytes().to_vec(),
            kid: Uuid::now_v7().to_string(),
        })
    }

    fn test_auth_service() -> AuthService {
        AuthService::new(mock_pool(), test_rsa_keypair(), 3600)
    }

    fn test_user() -> User {
        User {
            id: Uuid::now_v7(),
            company_id: Uuid::now_v7(),
            email: "test@haqly.com".to_string(),
            password_hash: String::new(),
            full_name: "Test User".to_string(),
            phone: None,
            avatar_url: None,
            is_active: true,
            mfa_enabled: false,
            mfa_secret: None,
            mfa_recovery_codes: None,
            last_login_at: None,
            created_at: chrono::Utc::now().naive_utc(),
            updated_at: chrono::Utc::now().naive_utc(),
        }
    }

    #[test]
    fn test_password_hash_verify() {
        let svc = test_auth_service();
        let password = "Niger1an$ecureP@ss!";
        let hash = svc.hash_password(password).unwrap();

        assert!(svc.verify_password(password, &hash));
        assert!(!svc.verify_password("wrong_password", &hash));
    }

    #[test]
    fn test_jwt_generate_validate() {
        let svc = test_auth_service();
        let user = test_user();
        let role = "Admin";

        let token = svc.generate_jwt(&user, role, 3600).unwrap();

        let decoding_key = DecodingKey::from_rsa_pem(&svc.rsa_keypair.public_pem).unwrap();
        let validation = Validation::new(Algorithm::RS256);
        let data = decode::<Claims>(&token, &decoding_key, &validation).unwrap();

        assert_eq!(data.claims.sub, user.id);
        assert_eq!(data.claims.role, "Admin");
        assert_eq!(data.claims.company_id, user.company_id);
        assert!(data.claims.exp > data.claims.iat);
    }

    #[test]
    fn test_mfa_totp_generation() {
        let secret_raw = Secret::generate_secret();
        let secret_bytes = secret_raw.to_bytes().unwrap();

        let totp = TOTP::new(
            TotpAlgorithm::SHA1,
            6,
            1,
            30,
            secret_bytes,
            "HAQLY-ERP".to_string(),
            "user@haqly.com".to_string(),
        ).unwrap();

        let code = totp.generate_current().unwrap();
        assert_eq!(code.len(), 6);
        assert!(code.chars().all(|c| c.is_ascii_digit()));
    }

    #[test]
    fn test_recovery_code_hashing() {
        let svc = test_auth_service();
        let recovery_codes: Vec<String> = (0..8)
            .map(|_| Uuid::now_v7().to_string().replace("-", "").chars().take(8).collect())
            .collect();

        assert_eq!(recovery_codes.len(), 8);

        let hashed: Vec<String> = recovery_codes.iter()
            .filter_map(|code| svc.hash_password(code).ok())
            .collect();

        assert_eq!(hashed.len(), 8);

        for (code, hash) in recovery_codes.iter().zip(hashed.iter()) {
            assert!(svc.verify_password(code, hash));
            assert!(!svc.verify_password("wrong", hash));
        }

        assert_ne!(recovery_codes[0], hashed[0]);
    }
}
