use token_compression_rs::TokenCompressor;
use std::time::Instant;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize the compressor
    let compressor = TokenCompressor::new()?;

    // Example token sequence with repeating patterns
    let tokens = vec![1000, 2000, 3000, 4000, 1000, 2000, 3000, 4000];
    println!("Original tokens: {:?}", tokens);
    println!("Original length: {}", tokens.len());

    // Measure compression time
    let start = Instant::now();
    let compressed = compressor.compress_tokens(tokens.clone()).await?;
    let duration = start.elapsed();

    println!("Compressed tokens: {:?}", compressed);
    println!("Compressed length: {}", compressed.len());
    println!("Compression ratio: {:.2}%", 
        (1.0 - (compressed.len() as f64 / tokens.len() as f64)) * 100.0);
    println!("Compression time: {:?}\n", duration);

    // Example batch compression with performance metrics
    let batch = vec![
        vec![1000, 2000, 3000, 1000, 2000, 3000], // Sequence with patterns
        vec![4000, 5000, 6000, 4000, 5000, 6000], // Another sequence with patterns
        vec![7000, 8000, 9000, 7000, 8000, 9000], // Third sequence with patterns
    ];
    println!("Original batch: {:?}", batch);
    println!("Original batch total tokens: {}", 
        batch.iter().map(|seq| seq.len()).sum::<usize>());

    // Measure batch compression time
    let batch_start = Instant::now();
    let compressed_batch = compressor.compress_batch(batch.clone(), 1000).await?;
    let batch_duration = batch_start.elapsed();

    let total_compressed = compressed_batch.iter().map(|seq| seq.len()).sum::<usize>();
    println!("Compressed batch: {:?}", compressed_batch);
    println!("Compressed batch total tokens: {}", total_compressed);
    println!("Batch compression ratio: {:.2}%",
        (1.0 - (total_compressed as f64 / batch.iter().map(|seq| seq.len()).sum::<usize>() as f64)) * 100.0);
    println!("Batch compression time: {:?}", batch_duration);

    Ok(())
}