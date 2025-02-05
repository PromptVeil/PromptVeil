use thiserror::Error;
use tracing::debug;
use rand::RngCore;

mod encryption;
mod key_management;
mod memory;

#[derive(Debug, Error)]
pub enum SecurityError {
    #[error("Encryption failed: {0}")]
    EncryptionError(String),
    
    #[error("Decryption failed: {0}")]
    DecryptionError(String),
    
    #[error("Key management error: {0}")]
    KeyManagementError(String),
    
    #[error("Memory error: {0}")]
    MemoryError(String),
    
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

pub struct SecurityManager {
    encryption_provider: encryption::AesGcmProvider,
}

impl SecurityManager {
    pub fn new() -> Result<Self, SecurityError> {
        let encryption_provider = encryption::AesGcmProvider::new()?;
        Ok(Self { encryption_provider })
    }

    pub async fn encrypt(&self, data: &[u8]) -> Result<Vec<u8>, SecurityError> {
        self.encryption_provider.encrypt(data)
    }

    pub async fn decrypt(&self, data: &[u8]) -> Result<Vec<u8>, SecurityError> {
        self.encryption_provider.decrypt(data)
    }

    pub async fn rotate_keys(&mut self) -> Result<(), SecurityError> {
        let mut new_key = vec![0u8; 32];
        rand::thread_rng().fill_bytes(&mut new_key);
        self.encryption_provider.update_key(&new_key)
    }
}

impl Drop for SecurityManager {
    fn drop(&mut self) {
        debug!("Cleaning up security manager resources");
    }
}

// Re-export types
pub use encryption::AesGcmProvider;
pub use key_management::{KeyManager, SecureKey};
pub use memory::{MemoryGuard, SecureMemory}; 