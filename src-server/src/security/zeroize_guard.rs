// Author: Quadri Atharu
use zeroize::{Zeroize, ZeroizeOnDrop};

#[derive(ZeroizeOnDrop)]
pub struct SecretString(String);

impl SecretString {
    pub fn new(s: String) -> Self {
        Self(s)
    }
    pub fn expose(&self) -> &str {
        &self.0
    }
}

#[derive(ZeroizeOnDrop)]
pub struct SecretBytes(Vec<u8>);

impl SecretBytes {
    pub fn new(b: Vec<u8>) -> Self {
        Self(b)
    }
    pub fn expose(&self) -> &[u8] {
        &self.0
    }
}
