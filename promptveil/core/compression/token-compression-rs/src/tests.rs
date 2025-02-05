#![cfg(test)]

use super::*;
use std::collections::HashSet;
use std::time::Instant;

// Helper function to generate test sequences
fn generate_test_sequence(n: usize) -> Vec<u32> {
    (1..=n as u32).collect()
}

// Helper function to get current memory usage (platform specific)
#[cfg(target_os = "linux")]
fn get_memory_usage() -> usize {
    use std::fs::File;
    use std::io::Read;
    
    let mut status = String::new();
    File::open("/proc/self/status")
        .unwrap()
        .read_to_string(&mut status)
        .unwrap();
    
    for line in status.lines() {
        if line.starts_with("VmRSS:") {
            return line
                .split_whitespace()
                .nth(1)
                .unwrap()
                .parse::<usize>()
                .unwrap() * 1024;
        }
    }
    0
}

#[cfg(not(target_os = "linux"))]
fn get_memory_usage() -> usize {
    // Placeholder for other platforms
    // In production, you'd want to implement this for each supported platform
    0
}

// Initialize TokenCompressor outside of Tokio runtime
fn init_compressor() -> TokenCompressor {
    TokenCompressor::new().expect("Failed to initialize TokenCompressor")
}

async fn test_basic_compression(compressor: &TokenCompressor) -> Result<()> {
    // Test data
    let tokens = vec![1000, 2000, 3000, 4000, 1000, 2000, 3000, 4000];
    
    // Test compression
    let compressed = compressor.compress_tokens(tokens.clone()).await?;
    assert!(compressed.len() < tokens.len());
    assert!(HashSet::<u32>::from_iter(compressed.iter().cloned()).len() <= HashSet::<u32>::from_iter(tokens.iter().cloned()).len());
    Ok(())
}

async fn test_edge_cases(compressor: &TokenCompressor) -> Result<()> {
    // Empty sequence should error
    let result = compressor.compress_tokens(vec![]).await;
    assert!(result.is_err());
    
    // Single token should return unchanged
    let single = vec![1000];
    let compressed = compressor.compress_tokens(single.clone()).await?;
    assert_eq!(compressed, single);
    
    // No repeating patterns should return similar length
    let unique_tokens = vec![1000, 2000, 3000, 4000];
    let compressed = compressor.compress_tokens(unique_tokens.clone()).await?;
    assert_eq!(compressed.len(), unique_tokens.len());  // Changed to exact match for unique tokens
    Ok(())
}

async fn test_pattern_detection(compressor: &TokenCompressor) -> Result<()> {
    // Create sequence with known patterns
    let pattern1 = vec![1000, 2000];
    let pattern2 = vec![3000, 4000];
    let mut tokens = Vec::new();
    
    // Add patterns multiple times to ensure they are detected
    for _ in 0..5 {
        tokens.extend_from_slice(&pattern1);
    }
    for _ in 0..5 {
        tokens.extend_from_slice(&pattern2);
    }
    
    // First compression to train on patterns
    let compressed = compressor.compress_tokens(tokens.clone()).await?;
    assert!(compressed.len() < tokens.len());
    
    // Test compression of new sequence with same patterns
    let mut new_tokens = Vec::new();
    // Add more repetitions to ensure pattern detection
    for _ in 0..3 {
        new_tokens.extend_from_slice(&pattern1);
        new_tokens.extend_from_slice(&pattern2);
    }
    
    let compressed_new = compressor.compress_tokens(new_tokens.clone()).await?;
    assert!(compressed_new.len() < new_tokens.len());
    Ok(())
}

async fn test_batch_processing(compressor: &TokenCompressor) -> Result<()> {
    // Test with various batch sizes
    for batch_size in [10, 100] {
        let mut batch = Vec::new();
        for _ in 0..batch_size {
            batch.push(vec![1000, 2000, 3000, 4000, 1000, 2000, 3000, 4000]);
        }
        
        let compressed = compressor.compress_batch(batch.clone(), batch_size).await?;
        assert_eq!(compressed.len(), batch_size);
        
        // Verify each sequence is properly compressed
        for compressed_seq in compressed {
            assert!(compressed_seq.len() < batch[0].len());
        }
    }
    Ok(())
}

async fn test_sequential_vs_parallel(compressor: &TokenCompressor) -> Result<()> {
    // Generate large test sequence
    let mut tokens = Vec::new();
    for i in 0..100_000 {
        if i % 2 == 0 {
            tokens.extend_from_slice(&[1000, 2000]);
        } else {
            tokens.extend_from_slice(&[3000, 4000]);
        }
    }
    
    // Compare sequential vs parallel processing
    let sequential_start = Instant::now();
    let _ = compressor.compress_tokens(tokens[0..1000].to_vec()).await?;
    let sequential_time = sequential_start.elapsed();
    
    let parallel_start = Instant::now();
    let _ = compressor.compress_tokens(tokens.clone()).await?;
    let parallel_time = parallel_start.elapsed();
    
    // Parallel should be relatively efficient for large sequences
    assert!(parallel_time.as_secs_f32() < sequential_time.as_secs_f32() * (tokens.len() as f32 / 1000.0));
    Ok(())
}

async fn test_simd_performance(compressor: &TokenCompressor) -> Result<()> {
    // Test SIMD performance on sequential operations with 10k tokens
    let tokens = generate_test_sequence(10000);
    
    // Measure performance of compression
    let start = Instant::now();
    let compressed = compressor.compress_tokens(tokens.clone()).await?;
    let duration = start.elapsed();
    
    // Test that execution time is reasonable (less than 1ms for 10k tokens)
    // Converting to nanoseconds for comparison with Julia's 1e6 (1ms) threshold
    assert!(duration.as_nanos() < 1_000_000, 
        "SIMD performance test failed: {:?} for 10k tokens (expected < 1ms)", duration);
    
    // Test compression ratio
    assert!(compressed.len() < tokens.len());
    
    // Log performance info
    println!("SIMD Performance:");
    println!("  Time: {:?} for 10k tokens", duration);
    println!("  Throughput: {:.2} tokens/ms", 
        (tokens.len() as f64) / (duration.as_nanos() as f64 / 1_000_000.0));
    
    Ok(())
}

async fn test_gpu_vs_cpu(compressor: &TokenCompressor) -> Result<()> {
    // Generate large sequence for GPU test
    let tokens = generate_test_sequence(100000);
    
    // First compression might use GPU if available
    let gpu_start = Instant::now();
    let compressed_gpu = compressor.compress_tokens(tokens.clone()).await?;
    let gpu_time = gpu_start.elapsed();
    
    // Force CPU compression by using small sequence
    let small_tokens = tokens[0..1000].to_vec();
    let cpu_start = Instant::now();
    let compressed_cpu = compressor.compress_tokens(small_tokens).await?;
    let cpu_time = cpu_start.elapsed();
    
    // Results should be consistent regardless of GPU/CPU
    assert!(compressed_gpu.len() < tokens.len());
    assert!(compressed_cpu.len() < 1000);
    
    // Log performance comparison
    println!("GPU time (100k tokens): {:?}", gpu_time);
    println!("CPU time (1k tokens): {:?}", cpu_time);
    Ok(())
}

async fn test_memory_usage(compressor: &TokenCompressor) -> Result<()> {
    // Test with increasingly large sequences
    for size in [1000, 10000, 100000] {
        let tokens = generate_test_sequence(size);
        
        // Measure memory before
        let before = get_memory_usage();
        
        // Perform compression
        let compressed = compressor.compress_tokens(tokens).await?;
        
        // Measure memory after
        let after = get_memory_usage();
        
        // Memory usage should be reasonable (less than 10x input size)
        let memory_ratio = (after - before) as f64 / (size * std::mem::size_of::<u32>()) as f64;
        assert!(memory_ratio < 10.0);
        
        // Verify compression worked
        assert!(compressed.len() < size);
    }
    Ok(())
}

#[test]
fn test_all() {
    // Initialize Julia outside of async runtime
    let compressor = init_compressor();
    
    // Configure and run Tokio runtime
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();

    rt.block_on(async {
        // Run all tests sequentially
        test_basic_compression(&compressor).await.expect("basic compression test failed");
        test_edge_cases(&compressor).await.expect("edge cases test failed");
        test_pattern_detection(&compressor).await.expect("pattern detection test failed");
        test_batch_processing(&compressor).await.expect("batch processing test failed");
        test_sequential_vs_parallel(&compressor).await.expect("sequential vs parallel test failed");
        test_simd_performance(&compressor).await.expect("SIMD performance test failed");
        test_gpu_vs_cpu(&compressor).await.expect("GPU vs CPU test failed");
        test_memory_usage(&compressor).await.expect("memory usage test failed");
    });
} 