use criterion::{black_box, criterion_group, criterion_main, Criterion};
use format_rs::{FormatManager, FormatProvider, Message, MessageMetadata, FormatError};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct BenchData {
    string_field: String,
    number_field: i64,
    array_field: Vec<i32>,
    optional_field: Option<String>,
}

struct BenchFormatProvider;

#[async_trait]
impl FormatProvider for BenchFormatProvider {
    type Input = Message<BenchData>;
    type Output = Message<BenchData>;

    async fn serialize(&self, data: &Self::Input) -> Result<Vec<u8>, FormatError> {
        serde_json::to_vec(data)
            .map_err(|e| FormatError::SerializationError(e.to_string()))
    }

    async fn deserialize(&self, data: &[u8]) -> Result<Self::Output, FormatError> {
        serde_json::from_slice(data)
            .map_err(|e| FormatError::DeserializationError(e.to_string()))
    }

    async fn validate_schema(&self, data: &[u8]) -> Result<bool, FormatError> {
        serde_json::from_slice::<Message<BenchData>>(data)
            .map(|_| true)
            .map_err(|e| FormatError::SchemaError(e.to_string()))
    }
}

fn create_test_data(size: usize) -> Message<BenchData> {
    Message {
        metadata: MessageMetadata {
            version: "1.0".to_string(),
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs() as i64,
            format_type: "benchmark".to_string(),
        },
        content: BenchData {
            string_field: "benchmark".repeat(size),
            number_field: 12345678,
            array_field: (0..size as i32).collect(),
            optional_field: Some("optional data".repeat(size/10)),
        },
    }
}

fn bench_serialization(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let manager = FormatManager::new(BenchFormatProvider);

    let mut group = c.benchmark_group("serialization");
    for size in [10, 100, 1000, 10000].iter() {
        group.bench_function(format!("serialize_{}", size), |b| {
            let data = create_test_data(*size);
            b.iter(|| {
                rt.block_on(async {
                    black_box(manager.serialize(black_box(&data)).await.unwrap())
                })
            });
        });
    }
    group.finish();
}

fn bench_deserialization(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let manager = FormatManager::new(BenchFormatProvider);

    let mut group = c.benchmark_group("deserialization");
    for size in [10, 100, 1000, 10000].iter() {
        group.bench_function(format!("deserialize_{}", size), |b| {
            let data = create_test_data(*size);
            let serialized = rt.block_on(async {
                manager.serialize(&data).await.unwrap()
            });
            b.iter(|| {
                rt.block_on(async {
                    black_box(manager.deserialize(black_box(&serialized)).await.unwrap())
                })
            });
        });
    }
    group.finish();
}

fn bench_validation(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let manager = FormatManager::new(BenchFormatProvider);

    let mut group = c.benchmark_group("validation");
    for size in [10, 100, 1000, 10000].iter() {
        group.bench_function(format!("validate_{}", size), |b| {
            let data = create_test_data(*size);
            let serialized = rt.block_on(async {
                manager.serialize(&data).await.unwrap()
            });
            b.iter(|| {
                rt.block_on(async {
                    black_box(manager.validate(black_box(&serialized)).await.unwrap())
                })
            });
        });
    }
    group.finish();
}

criterion_group!(benches, bench_serialization, bench_deserialization, bench_validation);
criterion_main!(benches); 