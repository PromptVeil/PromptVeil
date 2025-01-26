use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Error as AesError, Nonce,
};
use rand::RngCore;
use thiserror::Error;
use std::error::Error as StdError;
use bincode;

#[derive(Error, Debug)]
pub enum SecurityError {
    #[error("Encryption failed: {0}")]
    EncryptionError(String),
    
    #[error("Decryption failed: {0}")]
    DecryptionError(String),
    
    #[error("Invalid key length")]
    InvalidKeyLength,
    
    #[error("Invalid data format")]
    InvalidDataFormat,
    
    #[error("Serialization error: {0}")]
    SerializationError(String),
}

impl From<AesError> for SecurityError {
    fn from(err: AesError) -> Self {
        SecurityError::EncryptionError(err.to_string())
    }
}

impl From<bincode::Error> for SecurityError {
    fn from(err: bincode::Error) -> Self {
        SecurityError::SerializationError(err.to_string())
    }
}

pub fn encrypt(data: &[u8], key: &[u8; 32]) -> Result<Vec<u8>, SecurityError> {
    let cipher = Aes256Gcm::new_from_slice(key)
        .map_err(|_| SecurityError::InvalidKeyLength)?;

    let mut nonce = [0u8; 12];
    rand::thread_rng().fill_bytes(&mut nonce);
    let nonce = Nonce::from_slice(&nonce);

    let ciphertext = cipher
        .encrypt(nonce, data)
        .map_err(|e| SecurityError::EncryptionError(e.to_string()))?;

    let mut result = nonce.to_vec();
    result.extend_from_slice(&ciphertext);
    Ok(result)
}

pub fn decrypt(data: &[u8], key: &[u8; 32]) -> Result<Vec<u8>, SecurityError> {
    if data.len() < 12 {
        return Err(SecurityError::InvalidDataFormat);
    }

    let (nonce, ciphertext) = data.split_at(12);
    let cipher = Aes256Gcm::new_from_slice(key)
        .map_err(|_| SecurityError::InvalidKeyLength)?;

    cipher
        .decrypt(Nonce::from_slice(nonce), ciphertext)
        .map_err(|e| SecurityError::DecryptionError(e.to_string()))
}

pub fn generate_key() -> Result<[u8; 32], SecurityError> {
    let mut key = [0u8; 32];
    rand::thread_rng().fill_bytes(&mut key);
    Ok(key)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encryption_decryption() {
        let key = generate_key().unwrap();
        let data = b"Hello, World!";
        
        let encrypted = encrypt(data, &key).unwrap();
        let decrypted = decrypt(&encrypted, &key).unwrap();
        
        assert_eq!(data, &decrypted[..]);
    }
} 