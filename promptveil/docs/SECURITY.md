# Security in PromptVeil

This document details the security features and implementation in PromptVeil.

## Overview

PromptVeil implements multiple layers of security to protect conversation data:

1. Encryption at rest using AES-GCM
2. Hardware-accelerated encryption operations
3. Secure key management with automatic rotation
4. Memory security practices

## Encryption

### AES-GCM Implementation

- Uses AES-GCM (Galois/Counter Mode) for authenticated encryption
- 256-bit keys for maximum security
- Unique nonce generation for each encryption operation
- Authentication tags to verify data integrity

### Key Management

- Automatic key generation using secure random number generation
- Key rotation policies based on time and usage
- Secure key storage with memory protection
- Support for user-provided keys

## Implementation Details

### Rust Layer (`security.rs`)

The core security functionality is implemented in Rust for performance and memory safety:

```rust
// Example security manager configuration
struct SecurityConfig {
    key_rotation_interval: Duration,
    max_key_age: Duration,
    hardware_acceleration: bool
}
```

### Python Interface (`security.py`)

High-level Python interface with error handling:

```python
def encrypt(data: Union[bytes, str], key: Union[str, bytes]) -> bytes:
    """Encrypt data using AES-GCM."""
    
def decrypt(data: Union[bytes, BinaryIO], key: Union[str, bytes]) -> bytes:
    """Decrypt data using AES-GCM."""
```

## File Format Security

The `.pveil` file format includes:

1. Format version and metadata
2. Encrypted payload with authentication
3. Key identifiers for rotation support

## Best Practices

### Memory Security

- Secure memory wiping after key usage
- Protection against memory dumps
- Minimal key lifetime in memory

### Error Handling

- Secure error messages that don't leak sensitive information
- Automatic cleanup on errors
- Validation of all cryptographic operations

## Integration with Other Components

### Compression Integration

- Encryption happens after compression
- Compressed data is treated as opaque bytes
- Authentication covers the entire compressed payload

### Search Integration

- Index data is encrypted at rest
- Search operations work on decrypted data in memory
- Secure cleanup after search operations

## Security Considerations

### Known Limitations

1. No forward secrecy (would require session keys)
2. Keys must be managed by the application
3. Memory security depends on OS protections

### Recommendations

1. Use hardware security modules when available
2. Implement secure key backup procedures
3. Regular security audits of the codebase

## Future Improvements

1. Support for hardware security modules
2. Forward secrecy for conversation sessions
3. Enhanced memory protection mechanisms
4. Additional encryption algorithms 