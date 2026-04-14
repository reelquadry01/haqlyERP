// Author: Quadri Atharu
use aes_gcm::aead::{Aead, KeyInit, OsRng};
use aes_gcm::{Aes256Gcm, AeadCore, Nonce};
use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use argon2::password_hash::SaltString;
use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use rand::RngCore;

const AES_KEY_LEN: usize = 32;
const NONCE_LEN: usize = 12;

pub struct EncryptedData {
    pub ciphertext: String,
    pub nonce: String,
    pub tag: String,
}

pub fn encrypt_field(plaintext: &str, key: &[u8]) -> Result<EncryptedData, String> {
    if key.len() != AES_KEY_LEN {
        return Err("Encryption key must be 32 bytes".to_string());
    }

    let cipher = Aes256Gcm::new_from_slice(key)
        .map_err(|e| format!("Failed to create cipher: {}", e))?;

    let nonce = Aes256Gcm::generate_nonce(&mut OsRng);

    let ciphertext = cipher
        .encrypt(&nonce, plaintext.as_bytes())
        .map_err(|e| format!("Encryption failed: {}", e))?;

    let (encrypted_payload, tag_bytes) = ciphertext.split_at(ciphertext.len() - 16);

    Ok(EncryptedData {
        ciphertext: BASE64.encode(encrypted_payload),
        nonce: BASE64.encode(nonce.as_slice()),
        tag: BASE64.encode(tag_bytes),
    })
}

pub fn decrypt_field(ciphertext: &str, key: &[u8], nonce: &str, tag: &str) -> Result<String, String> {
    if key.len() != AES_KEY_LEN {
        return Err("Decryption key must be 32 bytes".to_string());
    }

    let cipher = Aes256Gcm::new_from_slice(key)
        .map_err(|e| format!("Failed to create cipher: {}", e))?;

    let ciphertext_bytes = BASE64
        .decode(ciphertext)
        .map_err(|e| format!("Failed to decode ciphertext: {}", e))?;

    let nonce_bytes = BASE64
        .decode(nonce)
        .map_err(|e| format!("Failed to decode nonce: {}", e))?;

    let tag_bytes = BASE64
        .decode(tag)
        .map_err(|e| format!("Failed to decode tag: {}", e))?;

    let mut combined = ciphertext_bytes;
    combined.extend_from_slice(&tag_bytes);

    let nonce = Nonce::from_slice(&nonce_bytes);

    let plaintext = cipher
        .decrypt(nonce, combined.as_ref())
        .map_err(|e| format!("Decryption failed: {}", e))?;

    String::from_utf8(plaintext).map_err(|e| format!("Failed to convert to string: {}", e))
}

pub fn hash_password(password: &str) -> Result<String, String> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();

    let hash = argon2
        .hash_password(password.as_bytes(), &salt)
        .map_err(|e| format!("Password hashing failed: {}", e))?;

    Ok(hash.to_string())
}

pub fn verify_password(password: &str, hash: &str) -> Result<bool, String> {
    let parsed_hash = PasswordHash::new(hash)
        .map_err(|e| format!("Failed to parse password hash: {}", e))?;

    let argon2 = Argon2::default();

    match argon2.verify_password(password.as_bytes(), &parsed_hash) {
        Ok(_) => Ok(true),
        Err(_) => Ok(false),
    }
}

pub fn generate_api_key() -> String {
    let mut key = [0u8; 32];
    OsRng.fill_bytes(&mut key);
    format!("haqly_{}", BASE64.encode(key))
}

pub fn rotate_encryption_key(
    old_key: &[u8],
    new_key: &[u8],
    encrypted_records: Vec<(String, String, String, String)>,
) -> Result<Vec<EncryptedData>, String> {
    if old_key.len() != AES_KEY_LEN || new_key.len() != AES_KEY_LEN {
        return Err("Both keys must be 32 bytes".to_string());
    }

    let mut re_encrypted = Vec::new();

    for (id, ciphertext, nonce, tag) in encrypted_records {
        let plaintext = decrypt_field(&ciphertext, old_key, &nonce, &tag)?;
        let new_encrypted = encrypt_field(&plaintext, new_key)?;
        re_encrypted.push(new_encrypted);
    }

    Ok(re_encrypted)
}

pub fn generate_encryption_key() -> [u8; AES_KEY_LEN] {
    let mut key = [0u8; AES_KEY_LEN];
    OsRng.fill_bytes(&mut key);
    key
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encrypt_decrypt_roundtrip() {
        let key = generate_encryption_key();
        let plaintext = "0123456789";
        let encrypted = encrypt_field(plaintext, &key).unwrap();
        let decrypted = decrypt_field(&encrypted.ciphertext, &key, &encrypted.nonce, &encrypted.tag).unwrap();
        assert_eq!(plaintext, decrypted);
    }

    #[test]
    fn test_password_hash_verify() {
        let password = "SecureP@ssw0rd";
        let hash = hash_password(password).unwrap();
        assert!(verify_password(password, &hash).unwrap());
        assert!(!verify_password("wrong", &hash).unwrap());
    }

    #[test]
    fn test_api_key_format() {
        let key = generate_api_key();
        assert!(key.starts_with("haqly_"));
        assert!(key.len() > 40);
    }
}
