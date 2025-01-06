use criterion::{black_box, criterion_group, criterion_main, Criterion};
use std::time::Duration;
use crate::{BlockIndex, BlockLocation, BlockType, QueryOptions};

fn setup_test_index(size: usize) -> BlockIndex {
    let mut index = BlockIndex::new();
    
    // Add blocks with timestamps
    for i in 0..size {
        let location = BlockLocation {
            offset: i as u64 * 1000,
            size: 1000,
            block_type: BlockType::Data,
        };
        index.add_block(i as u32, location);
        
        // Add timestamp ranges
        let timestamp = (i as u64) * 60; // One minute intervals
        index.add_timestamp_range(timestamp, timestamp + 60, vec![i as u32]);
        
        // Add model blocks (distribute across 5 models)
        let model = format!("gpt-{}", i % 5);
        index.add_model_blocks(model, vec![i as u32]);
        
        // Add tags (distribute across 10 tags)
        let tag = format!("tag-{}", i % 10);
        index.add_tag(tag, i as u32);
    }
    
    index
}

fn bench_range_queries(c: &mut Criterion) {
    let mut group = c.benchmark_group("range_queries");
    group.measurement_time(Duration::from_secs(10));
    
    for size in [1000, 10000, 100000].iter() {
        let mut index = setup_test_index(*size);
        
        group.bench_function(format!("range_query_{}", size), |b| {
            b.iter(|| {
                // Query random ranges
                for i in 0..100 {
                    let start = black_box(i * 60);
                    let end = black_box((i + 10) * 60);
                    index.find_blocks_in_range(start, end);
                }
            });
        });
        
        // Test cache effectiveness
        group.bench_function(format!("cached_range_query_{}", size), |b| {
            b.iter(|| {
                // Query same range multiple times
                for _ in 0..100 {
                    let start = black_box(0);
                    let end = black_box(600);
                    index.find_blocks_in_range(start, end);
                }
            });
        });
    }
    
    group.finish();
}

fn bench_model_queries(c: &mut Criterion) {
    let mut group = c.benchmark_group("model_queries");
    group.measurement_time(Duration::from_secs(10));
    
    for size in [1000, 10000, 100000].iter() {
        let mut index = setup_test_index(*size);
        
        group.bench_function(format!("model_query_{}", size), |b| {
            b.iter(|| {
                // Query each model type
                for i in 0..5 {
                    let model = black_box(format!("gpt-{}", i));
                    index.find_blocks_by_model(&model);
                }
            });
        });
    }
    
    group.finish();
}

fn bench_complex_queries(c: &mut Criterion) {
    let mut group = c.benchmark_group("complex_queries");
    group.measurement_time(Duration::from_secs(10));
    
    for size in [1000, 10000, 100000].iter() {
        let mut index = setup_test_index(*size);
        
        group.bench_function(format!("complex_query_{}", size), |b| {
            b.iter(|| {
                let options = QueryOptions {
                    time_range: Some(0..600),
                    model_pattern: Some("gpt-[12]".to_string()),
                    tags: Some(vec!["tag-1".to_string(), "tag-2".to_string()]),
                    limit: Some(100),
                    offset: None,
                };
                index.query_blocks(&options);
            });
        });
    }
    
    group.finish();
}

fn bench_tag_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("tag_operations");
    group.measurement_time(Duration::from_secs(10));
    
    for size in [1000, 10000, 100000].iter() {
        let mut index = setup_test_index(*size);
        
        group.bench_function(format!("tag_add_remove_{}", size), |b| {
            b.iter(|| {
                // Add and remove tags
                for i in 0..100 {
                    let tag = black_box(format!("benchmark-tag-{}", i));
                    let block_id = black_box(i as u32);
                    index.add_tag(tag.clone(), block_id);
                    index.remove_tag(&tag, block_id);
                }
            });
        });
        
        group.bench_function(format!("tag_query_{}", size), |b| {
            b.iter(|| {
                // Query tags for random blocks
                for i in 0..100 {
                    let block_id = black_box(i as u32);
                    index.get_tags(block_id);
                }
            });
        });
    }
    
    group.finish();
}

criterion_group!(
    benches,
    bench_range_queries,
    bench_model_queries,
    bench_complex_queries,
    bench_tag_operations
);
criterion_main!(benches); 