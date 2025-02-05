use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Key, Nonce,
};
use rand::RngCore;
use crate::SecurityError;

pub struct AesGcmProvider {
    cipher: Aes256Gcm,
}

impl AesGcmProvider {
    pub fn new() -> Result<Self, SecurityError> {
        let mut key = vec![0u8; 32];
        rand::thread_rng().fill_bytes(&mut key);
        
        let cipher_key = Key::<Aes256Gcm>::from_slice(&key);
        let cipher = Aes256Gcm::new(cipher_key);
        
        Ok(Self { cipher })
    }

    pub fn encrypt(&self, data: &[u8]) -> Result<Vec<u8>, SecurityError> {
        let mut nonce_bytes = vec![0u8; 12];
        rand::thread_rng().fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);

        let ciphertext = self.cipher
            .encrypt(nonce, data)
            .map_err(|e| SecurityError::EncryptionError(e.to_string()))?;

        // Combine nonce and ciphertext
        let mut result = nonce.to_vec();
        result.extend_from_slice(&ciphertext);
        Ok(result)
    }

    pub fn decrypt(&self, encrypted_data: &[u8]) -> Result<Vec<u8>, SecurityError> {
        if encrypted_data.len() < 12 {
            return Err(SecurityError::DecryptionError("Invalid encrypted data length".to_string()));
        }

        let (nonce_bytes, ciphertext) = encrypted_data.split_at(12);
        let nonce = Nonce::from_slice(nonce_bytes);

        self.cipher
            .decrypt(nonce, ciphertext)
            .map_err(|e| SecurityError::DecryptionError(e.to_string()))
    }

    pub fn update_key(&mut self, new_key: &[u8]) -> Result<(), SecurityError> {
        if new_key.len() != 32 {
            return Err(SecurityError::KeyManagementError("Invalid key length".to_string()));
        }
        
        let cipher_key = Key::<Aes256Gcm>::from_slice(new_key);
        self.cipher = Aes256Gcm::new(cipher_key);
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encryption_decryption() {
        let provider = AesGcmProvider::new().unwrap();
        let data = b"Hello, World!";
        
        let encrypted = provider.encrypt(data).unwrap();
        let decrypted = provider.decrypt(&encrypted).unwrap();
        
        assert_eq!(data.as_ref(), decrypted.as_slice());
    }

    #[test]
    fn test_key_update() {
        let mut provider = AesGcmProvider::new().unwrap();
        let data = b"Test data";
        
        // Encrypt with original key
        let encrypted = provider.encrypt(data).unwrap();
        
        // Update key
        let new_key = vec![1u8; 32];
        provider.update_key(&new_key).unwrap();
        
        // Try to decrypt with new key (should fail)
        assert!(provider.decrypt(&encrypted).is_err());
    }

    #[test]
    fn test_invalid_data() {
        let provider = AesGcmProvider::new().unwrap();
        let invalid_data = vec![0u8; 10]; // Too short for nonce
        
        assert!(provider.decrypt(&invalid_data).is_err());
    }

    #[test]
    fn test_large_data() {
        let provider = AesGcmProvider::new().unwrap();
        let large_data = vec![0u8; 1024 * 1024]; // 1MB
        
        let encrypted = provider.encrypt(&large_data).unwrap();
        let decrypted = provider.decrypt(&encrypted).unwrap();
        
        assert_eq!(large_data, decrypted);
    }
} 