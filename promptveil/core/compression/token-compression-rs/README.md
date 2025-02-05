# token-compression-rs

Rust bindings for TokenCompression.jl using Jlrs, providing an async interface for token sequence compression.

## Features

- Async interface using Tokio
- Single sequence compression
- Batch compression with configurable batch size
- GPU acceleration via TokenCompression.jl
- Automatic fallback to CPU when GPU is unavailable
- Thread-safe operations
- Proper error handling and propagation
- Local development integration with TokenCompression.jl

## Requirements

- Rust 1.77 or higher
- Julia 1.11
- Local copy of TokenCompression.jl
- Tokio runtime

## Installation

1. Clone the repository with submodules:
```bash
git clone --recursive https://github.com/yourusername/token-compression-rs.git
```

2. The build script will automatically:
   - Locate your local TokenCompression.jl
   - Configure Julia's development environment
   - Set up the package for local development

3. Add to your project's `Cargo.toml`:
```toml
[dependencies]
token-compression-rs = { path = "path/to/token-compression-rs" }
```

## Testing

The library includes a comprehensive test suite that mirrors the tests from TokenCompression.jl:

### Functional Tests
```bash
cargo test test_basic_compression
cargo test test_edge_cases
cargo test test_pattern_detection
```

- **Basic Compression**: Verifies core compression functionality
- **Edge Cases**: Tests empty sequences, single tokens, and non-repeating patterns
- **Pattern Detection**: Validates identification and compression of repeated patterns

### Performance Tests
```bash
cargo test test_sequential_vs_parallel
cargo test test_simd_performance
cargo test test_gpu_vs_cpu
```

- **Sequential vs Parallel**: Compares performance of different processing modes
- **SIMD Performance**: Validates SIMD optimizations through Julia
- **GPU vs CPU**: Tests GPU acceleration and CPU fallback

### Concurrency Tests
```bash
cargo test test_thread_safety
```

- **Thread Safety**: Verifies concurrent operations
- **Async Runtime**: Tests tokio integration
- **Resource Management**: Validates proper cleanup

### Memory Tests
```bash
cargo test test_memory_usage
```

- **Memory Usage**: Monitors memory consumption
- **Resource Leaks**: Checks for memory leaks
- **Platform-Specific**: Includes Linux-specific memory tracking

### Running All Tests
```bash
# Run all tests
cargo test

# Run with output
cargo test -- --nocapture

# Run specific test
cargo test test_name
```

### Test Coverage
- Functional parity with Julia implementation
- Performance benchmarking
- Error handling verification
- Concurrency safety
- Memory management
- Cross-platform compatibility

## Usage

### Single Sequence Compression

```rust
use token_compression_rs::TokenCompressor;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize compressor
    let compressor = TokenCompressor::new()?;

    // Compress token sequence
    let tokens = vec![1000, 2000, 3000, 4000];
    let compressed = compressor.compress_tokens(tokens).await?;
    println!("Compressed: {:?}", compressed);

    Ok(())
}
```

### Batch Compression

```rust
use token_compression_rs::TokenCompressor;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let compressor = TokenCompressor::new()?;

    // Prepare batch of token sequences
    let batch = vec![
        vec![1000, 2000, 3000],
        vec![4000, 5000, 6000],
    ];

    // Compress batch with batch size of 1000
    let compressed = compressor.compress_batch(batch, 1000).await?;
    println!("Compressed batch: {:?}", compressed);

    Ok(())
}
```

## Development Workflow

1. Make changes to TokenCompression.jl
2. Run the test suite: `cargo test`
3. Check performance tests: `cargo test test_performance -- --nocapture`
4. Verify memory usage: `cargo test test_memory_usage -- --nocapture`
5. Build and use in your project

## Performance Considerations

- Batch compression is more efficient for multiple sequences
- GPU acceleration is automatically used when available
- Optimal batch size depends on available GPU memory
- Token sequences are processed asynchronously
- Memory usage scales with batch size
- SIMD optimizations from local TokenCompression.jl

## Contributing

Contributions are welcome! Please ensure:
1. All tests pass: `cargo test`
2. Performance is maintained or improved
3. Memory usage is reasonable
4. Error handling is robust
5. Documentation is updated

## License

This project is licensed under the MIT License - see the LICENSE file for details. 