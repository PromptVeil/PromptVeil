# PromptVeil Compression System

## Overview

PromptVeil's compression system is built on three layers:

1. **TokenCompression.jl**: Core compression engine ([GitHub](https://github.com/pmatheusvinhas/TokenCompression.jl))
2. **PromptVeilCore.jl**: Julia/Rust bridge with SIMD optimizations
3. **Rust Layer**: High-level compression API

## Token Compression

### Core Features

- **BPE-based Compression**: Byte Pair Encoding optimized for LLM tokens
- **GPU Acceleration**: Automatic for sequences >1000 tokens
- **Parallel Processing**: Efficient batch operations
- **Pattern Learning**: Adaptive compression based on token patterns

### Optimization Techniques

1. **Token-Level Optimization**:
   - Pattern detection in token sequences
   - Frequency-based pair merging
   - Vocabulary optimization

2. **SIMD Acceleration**:
   - 4-element vector operations
   - Aligned memory access
   - Automatic CPU/GPU selection

3. **Batch Processing**:
   - Parallel compression of multiple sequences
   - GPU-accelerated batch operations
   - Memory-efficient processing

## Architecture Components

### 1. TokenCompression.jl

Core compression engine providing:

```julia
# Token optimization with GPU support
function optimize_tokens(tokens::Vector{UInt32}, pattern::TokenPattern)
    if has_gpu() && length(tokens) > MIN_PARALLEL_SIZE
        # GPU-accelerated optimization
    else
        # CPU-based optimization
    end
end

# Batch compression operations
function compress_batch(tokens::Matrix{UInt32})
    # Parallel compression with GPU support
end

function decompress_batch(tokens::Matrix{UInt32})
    # Parallel decompression with GPU support
end
```

### 2. PromptVeilCore.jl

Julia/Rust bridge with additional optimizations:

```julia
# SIMD-optimized token processing
function optimize_tokens_simd(tokens::Vector{UInt32})
    # Apply TokenCompression's optimize_tokens
    compressed = TokenCompression.optimize_tokens(tokens)
    
    # SIMD optimizations for 4-element blocks
    result = apply_simd_optimizations(compressed)
    return result
end

# FFI functions for Rust integration
Base.@ccallable function julia_optimize_tokens(ptr::Ptr{UInt32}, len::Int64)
    # Safe FFI wrapper
end
```

### 3. Rust Layer

High-level compression API:

```rust
// Token compression with error handling
pub fn compress_tokens(tokens: &[u32]) -> io::Result<Vec<u8>> {
    // Optimize tokens via Julia
    let optimized = JuliaInterface::optimize_tokens(tokens);
    
    // Convert to bytes
    Ok(optimized.iter()
        .flat_map(|&token| token.to_le_bytes().to_vec())
        .collect())
}

// Batch operations
pub fn compress_batch(tokens: &[u32], rows: usize, cols: usize) -> io::Result<Vec<u32>> {
    Ok(JuliaInterface::compress_batch(tokens, rows, cols))
}
```

## Performance Characteristics

### Compression Ratios

- **Average Ratio**: 25-50% size reduction
- **Best Case**: Up to 75% for repetitive sequences
- **Worst Case**: 10-15% for highly random sequences

### Performance Metrics

1. **Single Sequence**:
   - Small (<1k tokens): ~1ms on CPU
   - Medium (1k-10k): ~5ms with SIMD
   - Large (>10k): ~10ms with GPU

2. **Batch Processing**:
   - Small batches: Linear scaling
   - Large batches: Sub-linear with GPU
   - Memory usage: O(n) with streaming support

### Hardware Acceleration

1. **GPU Support**:
   - CUDA acceleration for large sequences
   - Automatic fallback to CPU
   - Configurable threshold (default: 1000 tokens)

2. **SIMD Operations**:
   - 4-wide vector operations
   - Automatic alignment
   - Platform-specific optimizations

## Integration Guidelines

### Python Usage

```python
from promptveil import ConversationStore

# Configure compression
store = ConversationStore(compression_config={
    'gpu_enabled': True,
    'batch_size': 1000,
    'optimization_level': 'high'
})

# Automatic compression
conversation.add_message("user", "Hello!")
store.add_conversation(conversation)
```

### Custom Compression Settings

```python
from promptveil.compression import CompressionConfig

config = CompressionConfig(
    min_gpu_tokens=1000,    # GPU threshold
    batch_size=5000,        # Batch processing size
    simd_enabled=True,      # SIMD optimization
    pattern_learning=True   # Adaptive compression
)
```

## Error Handling

### Common Issues

1. **GPU Errors**:
   - Device not available
   - Insufficient memory
   - Driver issues

2. **Memory Errors**:
   - Batch size too large
   - Resource exhaustion
   - Allocation failures

3. **Data Errors**:
   - Invalid token sequences
   - Corruption during compression
   - Decompression failures

### Recovery Strategies

```python
try:
    compressed = store.compress_conversation(conv)
except GPUError:
    # Fallback to CPU
    config.gpu_enabled = False
    compressed = store.compress_conversation(conv)
except MemoryError:
    # Reduce batch size
    config.batch_size //= 2
    compressed = store.compress_conversation(conv)
```

## Best Practices

1. **Resource Management**:
   - Monitor GPU memory usage
   - Use appropriate batch sizes
   - Clean up resources properly

2. **Performance Optimization**:
   - Enable GPU for large datasets
   - Use batch operations when possible
   - Configure compression levels appropriately

3. **Error Handling**:
   - Implement proper fallbacks
   - Monitor compression ratios
   - Log compression statistics

4. **Memory Management**:
   - Stream large sequences
   - Use appropriate buffer sizes
   - Clean up temporary data 