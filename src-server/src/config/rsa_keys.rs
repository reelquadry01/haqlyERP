// Author: Quadri Atharu
use anyhow::{Context, Result};
use rsa::pkcs8::{EncodePrivateKey, EncodePublicKey};
use std::path::Path;
use uuid::Uuid;

#[derive(Debug, Default)]
pub struct RsaKeypair {
    pub private_pem: Vec<u8>,
    pub public_pem: Vec<u8>,
    pub kid: String,
}

pub fn ensure_rsa_keypair(private_key_path: &str, public_key_path: &str) -> Result<RsaKeypair> {
    let priv_path = Path::new(private_key_path);
    let pub_path = Path::new(public_key_path);
    let kid_file = format!("{}.kid", private_key_path);

    if priv_path.exists() && pub_path.exists() && Path::new(&kid_file).exists() {
        let private_pem = std::fs::read(private_key_path)
            .with_context(|| format!("Failed to read RSA private key from {}", private_key_path))?;
        let public_pem = std::fs::read(public_key_path)
            .with_context(|| format!("Failed to read RSA public key from {}", public_key_path))?;
        let kid = std::fs::read_to_string(&kid_file)
            .with_context(|| format!("Failed to read key ID from {}", kid_file))?;
        tracing::info!("Loaded existing RSA keypair (kid={})", kid);
        return Ok(RsaKeypair {
            private_pem,
            public_pem,
            kid: kid.trim().to_string(),
        });
    }

    let mut rng = rand::rngs::OsRng;
    let private_key = rsa::RsaPrivateKey::new(&mut rng, 2048)
        .with_context(|| "Failed to generate RSA-2048 private key")?;
    let public_key = rsa::RsaPublicKey::from(&private_key);

    let private_pem_str = private_key
        .to_pkcs8_pem(rsa::pkcs8::LineEnding::LF)
        .with_context(|| "Failed to encode private key to PKCS8 PEM")?;
    let public_pem_str = public_key
        .to_public_key_pem(rsa::pkcs8::LineEnding::LF)
        .with_context(|| "Failed to encode public key to PKCS8 PEM")?;

    if let Some(parent) = priv_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    if let Some(parent) = pub_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let kid = Uuid::now_v7().to_string();

    std::fs::write(private_key_path, private_pem_str.as_bytes())
        .with_context(|| format!("Failed to write private key to {}", private_key_path))?;
    std::fs::write(public_key_path, public_pem_str.as_bytes())
        .with_context(|| format!("Failed to write public key to {}", public_key_path))?;
    std::fs::write(&kid_file, &kid)
        .with_context(|| format!("Failed to write kid to {}", kid_file))?;

    tracing::info!("Generated new RSA-2048 keypair (kid={}) at {} and {}", kid, private_key_path, public_key_path);

    Ok(RsaKeypair {
        private_pem: (*private_pem_str).as_bytes().to_vec(),
        public_pem: public_pem_str.into_bytes(),
        kid,
    })
}
