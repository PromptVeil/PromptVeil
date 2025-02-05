# PromptVeil Compression System

## Overview

PromptVeil's compression system is built on two main layers:

1. **TokenCompression.jl**: Core compression engine with native SIMD optimizations ([GitHub](https://github.com/pmatheusvinhas/TokenCompression.jl))
2. **Rust Layer**: High-level compression API using Jlrs for Julia integration

## Token Compression

### Core Features

- **BPE-based Compression**: Byte Pair Encoding optimized for LLM tokens
- **Native SIMD Optimization**: Julia's built-in SIMD for high performance
- **GPU Acceleration**: Automatic for sequences >1000 tokens with CPU fallback
- **Parallel Processing**: Multi-threaded operations with ThreadsX
- **Pattern Learning**: Adaptive compression based on token patterns
- **Memory Efficient**: Smart padding and variable sequence handling
- **Fault Tolerant**: Robust GPU fallback mechanisms

### Optimization Techniques

1. **Token-Level Optimization**:
   - Pattern detection in token sequences
   - Frequency-based pair merging
   - Vocabulary optimization
   - Atomic operations for GPU counting

2. **Native SIMD Acceleration**:
   - Julia's built-in SIMD vectorization
   - Optimized sequential operations
   - Zero-allocation merge operations
   - Automatic CPU feature detection

3. **Parallel Processing**:
   - ThreadsX-powered CPU parallelization
   - Batch-size aware GPU operations
   - Automatic workload distribution
   - Memory-efficient batch processing

4. **Memory Management**:
   - Smart sequence padding
   - GPU memory optimization
   - Variable length sequence handling
   - Efficient matrix operations

## Architecture Components

### 1. TokenCompression.jl

Core compression engine providing:

```julia
# Parallel frequency counting with GPU support
function parallel_countmap(tokens::Vector{UInt32})
    if length(tokens) < MIN_PARALLEL_SIZE
        # Sequential processing for small sequences
    elseif has_gpu()
        # GPU-accelerated counting with atomic operations
        CUDA.@atomic d_counts[token] += 1
    else
        # ThreadsX-powered CPU parallel counting
        ThreadsX.map(process_batch, batches)
    end
end

# Batch compression with memory management
function compress_batch(tokens::Matrix{UInt32})
    if has_gpu()
        # Process in smaller batches to avoid memory issues
        batch_size = 1000
        for batch in chunks(tokens, batch_size)
            # GPU-accelerated processing with memory management
        end
    else
        # CPU parallel processing with ThreadsX
    end
end

# Token optimization with automatic acceleration
function optimize_tokens(tokens::Vector{UInt32}, pattern::TokenPattern)
    if has_gpu() && length(tokens) > MIN_PARALLEL_SIZE
        # GPU-accelerated optimization
    else
        # SIMD-optimized CPU processing
    end
end
```

### 2. Rust Layer with Jlrs

High-level compression API using Jlrs for Julia integration:

```rust
use jlrs::prelude::*;

// Token compression with Jlrs integration
pub fn compress_tokens(tokens: &[u32]) -> io::Result<Vec<u8>> {
    let julia = Runtime::init().unwrap();
    let mut frame = StackFrame::new();
    
    // Call Julia functions via Jlrs
    let result = julia.scope(|mut frame| {
        let tokens = frame.create_vector(tokens)?;
        let result = frame.call::<_, JlrsResult<Vec<u8>>>(
            "optimize_tokens",
            (tokens,)
        )?;
        Ok(result)
    })?;
    
    Ok(result)
}

// Batch operations with memory management
pub fn compress_batch(tokens: &[u32], rows: usize, cols: usize) -> io::Result<Vec<u32>> {
    let julia = Runtime::init().unwrap();
    let mut frame = StackFrame::new();
    
    // Configure batch size for optimal memory usage
    let batch_size = determine_optimal_batch_size(rows, cols);
    
    // Process in batches with proper memory management
    julia.scope(|mut frame| {
        // Batch processing implementation
    })
}
```

## Performance Characteristics

### Memory Usage

- **GPU Mode**: 
  - Batch size limited to 1000 sequences
  - Automatic memory management
  - Atomic operations for counting
  
- **CPU Mode**:
  - ThreadsX parallel processing
  - Efficient sequence padding
  - Variable length handling

### Error Handling

- **GPU Fallback**:
  - Automatic fallback to CPU on GPU errors
  - Warning messages for debugging
  - Seamless recovery from GPU memory issues

- **Memory Management**:
  - Smart padding for variable sequences
  - Efficient matrix operations
  - Prevention of GPU memory leaks

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
   - Optimal batch size: 1000 sequences
   - Linear scaling up to GPU memory limit
   - Automatic CPU fallback for large batches

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

## Unique Approach to Token Compression

While traditional token compression methods focus on converting text to tokens, PromptVeil takes a novel approach:

### Traditional Token Compression
- Primary focus on text-to-token conversion
- Fixed vocabulary and patterns
- General-purpose compression
- ~4 bytes per token average

### PromptVeil's Advanced Compression
- Specialized for already-tokenized LLM conversations
- Adaptive pattern learning from conversation structures
- Hardware-accelerated compression (GPU/SIMD)
- Achieves 25-75% additional size reduction
- Optimized for dialogue patterns and conversation flow

Key advantages of our approach:
1. **Conversation-Aware**: Learns and optimizes for dialogue patterns
2. **Post-Tokenization**: Works with any tokenizer's output
3. **Hardware Acceleration**: Leverages GPU/SIMD for performance
4. **Adaptive Learning**: Improves compression based on conversation patterns 