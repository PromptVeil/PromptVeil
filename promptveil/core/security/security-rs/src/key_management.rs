use std::sync::Arc;
use tokio::sync::RwLock;
use zeroize::Zeroize;

use crate::SecurityError;

#[derive(Clone)]
pub struct KeyMetadata {
    pub created_at: std::time::SystemTime,
    pub key_id: String,
    pub is_active: bool,
}

pub struct SecureKey {
    key_data: Vec<u8>,
    metadata: KeyMetadata,
}

impl SecureKey {
    pub fn new(key_data: Vec<u8>, key_id: String) -> Self {
        Self {
            key_data,
            metadata: KeyMetadata {
                created_at: std::time::SystemTime::now(),
                key_id,
                is_active: true,
            },
        }
    }

    pub fn get_data(&self) -> &[u8] {
        &self.key_data
    }

    pub fn get_metadata(&self) -> &KeyMetadata {
        &self.metadata
    }
}

impl Drop for SecureKey {
    fn drop(&mut self) {
        self.key_data.zeroize();
    }
}

pub struct KeyManager {
    keys: Arc<RwLock<Vec<SecureKey>>>,
}

impl KeyManager {
    pub fn new() -> Self {
        Self {
            keys: Arc::new(RwLock::new(Vec::new())),
        }
    }

    pub async fn add_key(&self, key: SecureKey) -> Result<(), SecurityError> {
        let mut keys = self.keys.write().await;
        keys.push(key);
        Ok(())
    }

    pub async fn get_active_key(&self) -> Result<Option<SecureKey>, SecurityError> {
        let keys = self.keys.read().await;
        Ok(keys.iter()
            .find(|k| k.metadata.is_active)
            .map(|k| SecureKey::new(k.key_data.clone(), k.metadata.key_id.clone())))
    }

    pub async fn rotate_active_key(&self, new_key: SecureKey) -> Result<(), SecurityError> {
        let mut keys = self.keys.write().await;
        
        // Deactivate all existing keys
        for key in keys.iter_mut() {
            key.metadata.is_active = false;
        }
        
        // Add the new key as active
        keys.push(new_key);
        
        Ok(())
    }

    pub async fn cleanup(&self) -> Result<(), SecurityError> {
        let mut keys = self.keys.write().await;
        keys.clear();
        Ok(())
    }
}

impl Default for KeyManager {
    fn default() -> Self {
        Self::new()
    }
} 