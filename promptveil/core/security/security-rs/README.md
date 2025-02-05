# Security-RS

A Rust implementation of the security layer for PromptVeil, providing high-performance encryption and secure key management.

## Features

- Hardware-accelerated AES-GCM encryption
- Secure key management with automatic rotation
- Memory protection for sensitive data
- Asynchronous API for cryptographic operations
- Comprehensive security testing

## Installation

To use this module, add it as a dependency in your `Cargo.toml`:

```toml
[dependencies]
security-rs = { path = "path/to/security-rs" }
```

## Usage

### Basic Example

```rust
use security_rs::{SecurityManager, AesGcmProvider};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize the encryption provider
    let provider = AesGcmProvider::new()?;
    let manager = SecurityManager::new(Box::new(provider));

    // Encrypt data
    let data = b"Sensitive data";
    let encrypted = manager.encrypt(data).await?;

    // Decrypt data
    let decrypted = manager.decrypt(&encrypted).await?;
    assert_eq!(data.as_ref(), decrypted.as_slice());

    Ok(())
}
```

### Key Management

```rust
use security_rs::{KeyManager, SecureKey};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let key_manager = KeyManager::new();
    
    // Add a new key
    let key_data = vec![0u8; 32];
    let key = SecureKey::new(key_data, "key1".to_string());
    key_manager.add_key(key).await?;
    
    // Rotate to a new key
    let new_key_data = vec![1u8; 32];
    let new_key = SecureKey::new(new_key_data, "key2".to_string());
    key_manager.rotate_active_key(new_key).await?;
    
    Ok(())
}
```

### Memory Protection

```rust
use security_rs::memory::{SecureMemory, MemoryGuard};

fn handle_sensitive_data() {
    // Allocate secure memory
    let mut secure_mem = SecureMemory::new(1024).unwrap();
    
    // Write sensitive data
    let sensitive_data = b"Very sensitive data";
    secure_mem.write(sensitive_data).unwrap();
    
    // Data is automatically cleared when secure_mem is dropped
}
```

## Security

This module implements several security measures:

- Use of AES-GCM for authenticated encryption
- Secure key generation using system entropy
- Automatic memory clearing for sensitive data
- Key rotation to limit exposure
- Protection against memory leaks

## Development

To build and test the module:

```bash
cargo build --release
cargo test
```

## Contributing

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/MyFeature`)
3. Commit your changes (`git commit -am 'Add new feature'`)
4. Push to the branch (`git push origin feature/MyFeature`)
5. Create a Pull Request

## License

This project is licensed under the MIT License - see the LICENSE file for details. 