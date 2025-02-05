# Token-Compression-RS API Documentation

This document describes the public API of the token-compression-rs crate, which provides token sequence compression functionality for the PromptVeil system using Julia-based compression algorithms.

## Core Components

### TokenCompressor

The `TokenCompressor` is the main entry point for token compression operations.

```rust
pub struct TokenCompressor {
    // internal fields omitted
}

impl TokenCompressor {
    // Creates a new TokenCompressor instance
    pub fn new() -> Result<Self>

    // Compresses a single token sequence
    pub async fn compress_tokens(&self, tokens: Vec<u32>) -> Result<Vec<u32>>

    // Compresses multiple token sequences in batch
    pub async fn compress_batch(&self, tokens: Vec<Vec<u32>>, batch_size: usize) -> Result<Vec<Vec<u32>>>
}
```

### Error Handling

```rust
pub enum CompressionError {
    InitError(String),      // Julia environment initialization errors
    JuliaError(String),     // Julia runtime errors
    AsyncError(String),     // Async task errors
    InvalidTokens(String),  // Token validation errors
}

pub type Result<T> = std::result::Result<T, CompressionError>;
```

## Usage Examples

### Single Sequence Compression

```rust
// Create a new compressor
let compressor = TokenCompressor::new()?;

// Compress a single token sequence
let tokens = vec![1, 2, 3, 4, 5];
let compressed = compressor.compress_tokens(tokens).await?;
```

### Batch Compression

```rust
// Create token sequences
let sequences = vec![
    vec![1, 2, 3],
    vec![4, 5, 6],
    vec![7, 8, 9],
];

// Compress in batch
let batch_size = 32;
let compressed_sequences = compressor.compress_batch(sequences, batch_size).await?;
```

## Technical Details

### Julia Integration

The crate uses the TokenCompression.jl Julia package for the actual compression algorithms. The integration is handled through the `jlrs` crate, which provides Rust bindings for Julia.

### Initialization Process

When creating a new `TokenCompressor`, the following steps occur:

1. Julia runtime is initialized
2. Local TokenCompression.jl package is loaded
3. Required Julia functions are made available

### Memory Management

- Token sequences are automatically padded for batch processing
- Zero padding is used for sequences of different lengths
- Memory is automatically cleaned up when `TokenCompressor` is dropped

## Best Practices

1. Reuse `TokenCompressor` instances when possible
2. Use batch compression for multiple sequences
3. Choose appropriate batch sizes based on your memory constraints
4. Handle compression errors appropriately in your application
5. Ensure token sequences are valid before compression
6. Consider the overhead of Julia runtime initialization

## Performance Considerations

1. First compression operation may be slower due to Julia JIT compilation
2. Batch processing is more efficient for multiple sequences
3. Memory usage scales with the size of token sequences
4. Consider the trade-off between compression ratio and speed

## Error Handling Examples

```rust
match compressor.compress_tokens(tokens).await {
    Ok(compressed) => {
        // Use compressed tokens
    },
    Err(CompressionError::JuliaError(msg)) => {
        // Handle Julia-specific errors
    },
    Err(CompressionError::InvalidTokens(msg)) => {
        // Handle invalid token sequences
    },
    Err(e) => {
        // Handle other error types
    }
}
``` 