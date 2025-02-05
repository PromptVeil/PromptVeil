# Security-RS API Documentation

This document describes the public API of the security-rs crate, which provides cryptographic and security functionality for the PromptVeil system.

## Core Components

### SecurityManager

The `SecurityManager` is the main entry point for encryption and decryption operations.

```rust
pub struct SecurityManager {
    // internal fields omitted
}

impl SecurityManager {
    // Creates a new SecurityManager instance
    pub fn new() -> Result<Self, SecurityError>

    // Encrypts the provided data
    pub async fn encrypt(&self, data: &[u8]) -> Result<Vec<u8>, SecurityError>

    // Decrypts the provided data
    pub async fn decrypt(&self, data: &[u8]) -> Result<Vec<u8>, SecurityError>

    // Rotates encryption keys
    pub async fn rotate_keys(&mut self) -> Result<(), SecurityError>
}
```

### Error Handling

The crate uses a custom error type for all operations:

```rust
pub enum SecurityError {
    EncryptionError(String),
    DecryptionError(String),
    KeyManagementError(String),
    MemoryError(String),
    IoError(std::io::Error),
}
```

### Re-exported Types

The following types are re-exported from internal modules:

- `AesGcmProvider`: Provider for AES-GCM encryption operations
- `KeyManager`: Manages cryptographic keys
- `SecureKey`: Represents a secure cryptographic key
- `MemoryGuard`: Provides secure memory operations
- `SecureMemory`: Interface for secure memory handling

## Usage Examples

### Basic Encryption/Decryption

```rust
// Create a new security manager
let security_manager = SecurityManager::new()?;

// Encrypt data
let data = b"sensitive data";
let encrypted = security_manager.encrypt(data).await?;

// Decrypt data
let decrypted = security_manager.decrypt(&encrypted).await?;
assert_eq!(data, &decrypted[..]);
```

### Key Rotation

```rust
// Create a mutable security manager
let mut security_manager = SecurityManager::new()?;

// Rotate encryption keys
security_manager.rotate_keys().await?;
```

## Security Considerations

1. The `SecurityManager` automatically handles secure cleanup of sensitive resources when dropped
2. All cryptographic operations are performed using industry-standard algorithms (AES-GCM)
3. Keys are managed securely in memory with protection against memory dumps
4. All operations are designed to be thread-safe and async-compatible

## Error Handling Examples

```rust
match security_manager.encrypt(data).await {
    Ok(encrypted) => {
        // Handle successful encryption
    },
    Err(SecurityError::EncryptionError(msg)) => {
        // Handle encryption-specific error
    },
    Err(SecurityError::IoError(e)) => {
        // Handle I/O error
    },
    Err(e) => {
        // Handle other error types
    }
}
``` 