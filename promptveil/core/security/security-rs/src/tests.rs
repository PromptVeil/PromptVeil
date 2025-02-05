#[cfg(test)]
mod tests {
    use super::*;
    use crate::encryption::AesGcmProvider;
    use crate::key_management::{KeyManager, SecureKey};
    use crate::memory::{MemoryGuard, SecureMemory};
    use std::time::Duration;

    // Encryption Tests
    #[tokio::test]
    async fn test_encryption_decryption() {
        let provider = AesGcmProvider::new().unwrap();
        let manager = SecurityManager::new(Box::new(provider));

        let data = b"Hello, World!";
        let encrypted = manager.encrypt(data).await.unwrap();
        let decrypted = manager.decrypt(&encrypted).await.unwrap();

        assert_eq!(data.as_ref(), decrypted.as_slice());
    }

    #[tokio::test]
    async fn test_encryption_large_data() {
        let provider = AesGcmProvider::new().unwrap();
        let manager = SecurityManager::new(Box::new(provider));

        let large_data = vec![0u8; 1024 * 1024]; // 1MB
        let encrypted = manager.encrypt(&large_data).await.unwrap();
        let decrypted = manager.decrypt(&encrypted).await.unwrap();

        assert_eq!(large_data, decrypted);
    }

    #[tokio::test]
    async fn test_encryption_empty_data() {
        let provider = AesGcmProvider::new().unwrap();
        let manager = SecurityManager::new(Box::new(provider));

        let data = b"";
        let encrypted = manager.encrypt(data).await.unwrap();
        let decrypted = manager.decrypt(&encrypted).await.unwrap();

        assert_eq!(data.as_ref(), decrypted.as_slice());
    }

    #[tokio::test]
    async fn test_encryption_invalid_data() {
        let provider = AesGcmProvider::new().unwrap();
        let manager = SecurityManager::new(Box::new(provider));

        let invalid_encrypted = vec![0u8; 32];
        let result = manager.decrypt(&invalid_encrypted).await;
        assert!(result.is_err());
    }

    // Key Management Tests
    #[tokio::test]
    async fn test_key_rotation() {
        let provider = AesGcmProvider::new().unwrap();
        let mut manager = SecurityManager::new(Box::new(provider));

        let data = b"Test data";
        let encrypted = manager.encrypt(data).await.unwrap();
        
        // Rotate the keys
        manager.rotate_keys().await.unwrap();
        
        // Verify we can still decrypt with the new key
        let decrypted = manager.decrypt(&encrypted).await.unwrap();
        assert_eq!(data.as_ref(), decrypted.as_slice());
    }

    #[tokio::test]
    async fn test_key_rotation_multiple() {
        let provider = AesGcmProvider::new().unwrap();
        let mut manager = SecurityManager::new(Box::new(provider));

        let data = b"Test data";
        let mut encrypted_data = Vec::new();

        // Encrypt data with multiple keys
        for _ in 0..5 {
            let encrypted = manager.encrypt(data).await.unwrap();
            encrypted_data.push(encrypted);
            manager.rotate_keys().await.unwrap();
        }

        // Verify all encrypted data can still be decrypted
        for encrypted in encrypted_data {
            let decrypted = manager.decrypt(&encrypted).await.unwrap();
            assert_eq!(data.as_ref(), decrypted.as_slice());
        }
    }

    #[tokio::test]
    async fn test_key_expiration() {
        let key_manager = KeyManager::new();
        let key_data = vec![0u8; 32];
        let mut key = SecureKey::new(key_data, "expiring_key".to_string());
        key.set_expiration(Duration::from_secs(1));
        
        key_manager.add_key(key).await.unwrap();
        
        // Wait for key to expire
        tokio::time::sleep(Duration::from_secs(2)).await;
        
        let active_key = key_manager.get_active_key().await.unwrap();
        assert!(active_key.is_none());
    }

    // Memory Security Tests
    #[test]
    fn test_secure_memory() {
        let mut secure_mem = SecureMemory::new(1024).unwrap();
        let test_data = b"Sensitive data";
        
        secure_mem.write(test_data).unwrap();
        let read_data = secure_mem.read(test_data.len()).unwrap();
        
        assert_eq!(test_data.as_ref(), read_data.as_slice());
        
        secure_mem.clear();
        let cleared_data = secure_mem.read(test_data.len()).unwrap();
        assert!(cleared_data.iter().all(|&x| x == 0));
    }

    #[test]
    fn test_secure_memory_overflow() {
        let mut secure_mem = SecureMemory::new(10).unwrap();
        let test_data = b"Data larger than buffer";
        
        let result = secure_mem.write(test_data);
        assert!(result.is_err());
    }

    #[test]
    fn test_memory_guard() {
        let sensitive_data = vec![1, 2, 3, 4, 5];
        let guard = MemoryGuard::new(sensitive_data.clone());
        
        assert_eq!(guard.get(), &sensitive_data);
        
        // Data will be zeroed automatically when guard is dropped
        drop(guard);
    }

    #[test]
    fn test_memory_guard_drop() {
        let sensitive_data = vec![1, 2, 3, 4, 5];
        let ptr = sensitive_data.as_ptr();
        let guard = MemoryGuard::new(sensitive_data);
        
        // Keep a reference to verify memory is cleared
        let ptr_copy = ptr;
        
        drop(guard);
        
        unsafe {
            // Verify memory is zeroed
            for i in 0..5 {
                assert_eq!(*ptr_copy.add(i), 0);
            }
        }
    }

    // Integration Tests
    #[tokio::test]
    async fn test_end_to_end_encryption() {
        let provider = AesGcmProvider::new().unwrap();
        let mut manager = SecurityManager::new(Box::new(provider));
        let mut secure_mem = SecureMemory::new(1024).unwrap();

        // Test data flow: memory -> encryption -> decryption -> memory
        let original_data = b"Sensitive information";
        secure_mem.write(original_data).unwrap();
        
        let data_to_encrypt = secure_mem.read(original_data.len()).unwrap();
        let encrypted = manager.encrypt(&data_to_encrypt).await.unwrap();
        
        // Clear original data
        secure_mem.clear();
        
        // Decrypt and store in secure memory
        let decrypted = manager.decrypt(&encrypted).await.unwrap();
        secure_mem.write(&decrypted).unwrap();
        
        let final_data = secure_mem.read(original_data.len()).unwrap();
        assert_eq!(original_data.as_ref(), final_data.as_slice());
    }

    #[tokio::test]
    async fn test_concurrent_operations() {
        let provider = AesGcmProvider::new().unwrap();
        let manager = SecurityManager::new(Box::new(provider));
        let data = b"Test data";

        let mut handles = Vec::new();
        
        // Spawn multiple encryption/decryption tasks
        for _ in 0..10 {
            let manager_clone = manager.clone();
            let data_clone = data.to_vec();
            
            handles.push(tokio::spawn(async move {
                let encrypted = manager_clone.encrypt(&data_clone).await.unwrap();
                let decrypted = manager_clone.decrypt(&encrypted).await.unwrap();
                assert_eq!(data_clone, decrypted);
            }));
        }

        // Wait for all tasks to complete
        for handle in handles {
            handle.await.unwrap();
        }
    }
} 