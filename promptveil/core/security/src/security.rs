use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Error as AesError, Nonce,
};
use rand::RngCore;
use thiserror::Error;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub struct EncryptedData {
    pub ciphertext: Vec<u8>,
    pub nonce: [u8; 12],
}

impl EncryptedData {
    pub fn new(data: &[u8], key: &[u8; 32]) -> Result<Self, SecurityError> {
        let cipher = Aes256Gcm::new(key.into());
        let mut nonce = [0u8; 12];
        rand::thread_rng().fill_bytes(&mut nonce);
        
        let ciphertext = cipher
            .encrypt(Nonce::from_slice(&nonce), data)
            .map_err(|e| SecurityError::EncryptionError(e))?;

        Ok(Self { ciphertext, nonce })
    }

    pub fn decrypt(&self, key: &[u8; 32]) -> Result<Vec<u8>, SecurityError> {
        let cipher = Aes256Gcm::new(key.into());
        
        cipher
            .decrypt(Nonce::from_slice(&self.nonce), self.ciphertext.as_ref())
            .map_err(|e| SecurityError::DecryptionError(Box::new(e)))
    }

    pub fn as_bytes(&self) -> Vec<u8> {
        bincode::serialize(&self).unwrap()
    }

    pub fn from_bytes(bytes: &[u8]) -> Self {
        bincode::deserialize(bytes).unwrap()
    }
}

#[derive(Debug, Error)]
pub enum SecurityError {
    #[error("Encryption error: {0}")]
    EncryptionError(#[from] AesError),
    #[error("Decryption error: {0}")]
    DecryptionError(#[from] Box<dyn std::error::Error>),
    #[error("Invalid key length")]
    InvalidKeyLength,
}

pub fn generate_key() -> Result<[u8; 32], SecurityError> {
    let mut key = [0u8; 32];
    rand::thread_rng().fill_bytes(&mut key);
    Ok(key)
}

pub fn encrypt(data: &[u8], key: &[u8; 32]) -> Result<Vec<u8>, SecurityError> {
    let cipher = Aes256Gcm::new_from_slice(key)
        .map_err(|_| SecurityError::InvalidKeyLength)?;

    let mut nonce = [0u8; 12];
    rand::thread_rng().fill_bytes(&mut nonce);
    let nonce = Nonce::from_slice(&nonce);

    let mut result = nonce.to_vec();
    let ciphertext = cipher.encrypt(nonce, data)
        .map_err(SecurityError::EncryptionError)?;
    result.extend_from_slice(&ciphertext);

    Ok(result)
}

pub fn decrypt(data: &[u8], key: &[u8; 32]) -> Result<Vec<u8>, SecurityError> {
    if data.len() < 12 {
        return Err(SecurityError::DecryptionError("Invalid data length".into()));
    }

    let cipher = Aes256Gcm::new_from_slice(key)
        .map_err(|_| SecurityError::InvalidKeyLength)?;

    let nonce = Nonce::from_slice(&data[..12]);
    let ciphertext = &data[12..];

    cipher.decrypt(nonce, ciphertext)
        .map_err(|e| SecurityError::DecryptionError(Box::new(e)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encryption_decryption() {
        let data = b"Hello, World!";
        let mut key = [0u8; 32];
        rand::thread_rng().fill_bytes(&mut key);

        let encrypted = encrypt(data, &key).unwrap();
        let decrypted = decrypt(&encrypted, &key).unwrap();

        assert_eq!(data, &decrypted[..]);
    }
} 