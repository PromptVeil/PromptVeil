use super::*;
use std::io::Cursor;

fn create_test_block() -> DataBlock {
    DataBlock {
        header: BlockHeader {
            size: 100,
            checksum: [0; 32],
            compression_type: CompressionType::Julia,
        },
        compressed_data: vec![0; 100],
        encryption_metadata: EncryptionMetadata {
            version: 1,
            algorithm: "AES-256-GCM".to_string(),
            key_id: "test-key".to_string(),
            nonce: [0; 12],
        },
        encrypted_data: vec![0; 100],
    }
}

fn create_test_embedding() -> EmbeddingBlock {
    EmbeddingBlock {
        model: "gpt-4".to_string(),
        dimension: 1536,
        vectors: vec![Vector { data: vec![0.0; 1536] }],
        chunk_map: HashMap::new(),
    }
}

#[test]
fn test_basic_file_operations() {
    let mut file = PVeilFile::new();
    assert_eq!(file.partition_count(), 0);
    assert_eq!(file.header.version, 1);
    assert_eq!(&file.header.magic, b"PVEIL");
}

#[test]
fn test_partition_management() {
    let mut file = PVeilFile::new();
    
    // Add partition
    let mut partition = Partition::new();
    partition.add_data_block(create_test_block());
    file.add_partition(partition);
    
    assert_eq!(file.partition_count(), 1);
    assert_eq!(file.header.partition_count, 1);
    
    // Get partition
    let partition = file.get_partition(0).unwrap();
    assert_eq!(partition.data_blocks.len(), 1);
    assert_eq!(partition.metadata.message_count, 1);
}

#[test]
fn test_embedding_operations() {
    let mut file = PVeilFile::new();
    let mut partition = Partition::new();
    
    // Add embedding block
    let embedding = create_test_embedding();
    partition.add_embedding_block(embedding);
    file.add_partition(partition);
    
    let partition = file.get_partition(0).unwrap();
    assert_eq!(partition.embedding_blocks.len(), 1);
    assert_eq!(partition.embedding_blocks[0].dimension, 1536);
}

#[test]
fn test_serialization() {
    let mut file = PVeilFile::new();
    
    // Add data and embeddings
    let mut partition = Partition::new();
    partition.add_data_block(create_test_block());
    partition.add_embedding_block(create_test_embedding());
    file.add_partition(partition);
    
    // Serialize
    let mut buffer = Cursor::new(Vec::new());
    let mut writer = io::FileWriter::new(&mut buffer);
    writer.write_file(&file).unwrap();
    
    // Deserialize
    buffer.set_position(0);
    let mut reader = io::FileReader::new(buffer);
    let loaded = reader.read_file().unwrap();
    
    // Verify
    assert_eq!(loaded.partition_count(), 1);
    let partition = loaded.get_partition(0).unwrap();
    assert_eq!(partition.data_blocks.len(), 1);
    assert_eq!(partition.embedding_blocks.len(), 1);
    assert_eq!(partition.metadata.message_count, 1);
}

#[test]
fn test_schema_compatibility() {
    let mut file = PVeilFile::new();
    
    // Add fields to schema
    let fields = vec![
        Field {
            name: "text".to_string(),
            field_type: FieldType::String,
            nullable: false,
        },
        Field {
            name: "metadata".to_string(),
            field_type: FieldType::Struct(vec![
                Field {
                    name: "timestamp".to_string(),
                    field_type: FieldType::Integer,
                    nullable: false,
                },
                Field {
                    name: "tags".to_string(),
                    field_type: FieldType::Array(Box::new(FieldType::String)),
                    nullable: true,
                },
            ]),
            nullable: true,
        },
    ];
    
    file.schema.fields = fields;
    
    // Serialize and deserialize
    let mut buffer = Cursor::new(Vec::new());
    let mut writer = io::FileWriter::new(&mut buffer);
    writer.write_file(&file).unwrap();
    
    buffer.set_position(0);
    let mut reader = io::FileReader::new(buffer);
    let loaded = reader.read_file().unwrap();
    
    // Verify schema
    assert_eq!(loaded.schema.fields.len(), 2);
    if let FieldType::Struct(metadata_fields) = &loaded.schema.fields[1].field_type {
        assert_eq!(metadata_fields.len(), 2);
    } else {
        panic!("Expected struct field type");
    }
}

#[test]
fn test_invalid_magic() {
    let mut file = PVeilFile::new();
    file.header.magic = *b"INVALID";
    
    let mut buffer = Cursor::new(Vec::new());
    let mut writer = io::FileWriter::new(&mut buffer);
    writer.write_file(&file).unwrap();
    
    buffer.set_position(0);
    let mut reader = io::FileReader::new(buffer);
    assert!(matches!(reader.read_file().unwrap_err(), Error::InvalidMagic));
}

#[test]
fn test_partition_indices() {
    let mut file = PVeilFile::new();
    let mut partition = Partition::new();
    
    // Add data with indices
    partition.local_indices.message_index.insert(
        "msg1".to_string(),
        vec![0],
    );
    partition.local_indices.timestamp_index.insert(
        1234567890,
        vec![0],
    );
    partition.local_indices.model_index.insert(
        "gpt-4".to_string(),
        vec![0],
    );
    
    file.add_partition(partition);
    
    // Serialize and deserialize
    let mut buffer = Cursor::new(Vec::new());
    let mut writer = io::FileWriter::new(&mut buffer);
    writer.write_file(&file).unwrap();
    
    buffer.set_position(0);
    let mut reader = io::FileReader::new(buffer);
    let loaded = reader.read_file().unwrap();
    
    // Verify indices
    let partition = loaded.get_partition(0).unwrap();
    assert_eq!(partition.local_indices.message_index.len(), 1);
    assert_eq!(partition.local_indices.timestamp_index.len(), 1);
    assert_eq!(partition.local_indices.model_index.len(), 1);
}

#[test]
fn test_random_access() {
    let mut file = PVeilFile::new();
    let mut partition = Partition::new();
    
    // Add multiple blocks with different timestamps
    let mut block1 = create_test_block();
    let mut block2 = create_test_block();
    let mut block3 = create_test_block();
    
    // Set different data for each block
    block1.compressed_data = vec![1; 100];
    block2.compressed_data = vec![2; 100];
    block3.compressed_data = vec![3; 100];
    
    partition.add_data_block(block1);
    partition.add_data_block(block2);
    partition.add_data_block(block3);
    
    // Add timestamp ranges
    partition.local_indices.timestamp_index.insert(1000, vec![0]);
    partition.local_indices.timestamp_index.insert(2000, vec![1]);
    partition.local_indices.timestamp_index.insert(3000, vec![2]);
    
    file.add_partition(partition);
    
    // Serialize to buffer
    let mut buffer = Cursor::new(Vec::new());
    let mut writer = io::FileWriter::new(&mut buffer);
    writer.write_file(&file).unwrap();
    
    // Test random access
    buffer.set_position(0);
    let block = file.read_block(&mut buffer, 1).unwrap().unwrap();
    assert_eq!(block.compressed_data, vec![2; 100]);
    
    // Test range query
    let blocks = file.read_blocks_in_range(&mut buffer, 1500, 2500).unwrap();
    assert_eq!(blocks.len(), 1);
    assert_eq!(blocks[0].compressed_data, vec![2; 100]);
}

#[test]
fn test_embedding_access() {
    let mut file = PVeilFile::new();
    let mut partition = Partition::new();
    
    // Add data blocks and embeddings
    partition.add_data_block(create_test_block());
    
    let mut embedding1 = create_test_embedding();
    let mut embedding2 = create_test_embedding();
    
    // Set different models
    embedding1.model = "gpt-3.5".to_string();
    embedding2.model = "gpt-4".to_string();
    
    partition.add_embedding_block(embedding1);
    partition.add_embedding_block(embedding2);
    
    // Add model indices
    partition.local_indices.model_index.insert(
        "gpt-3.5".to_string(),
        vec![1],
    );
    partition.local_indices.model_index.insert(
        "gpt-4".to_string(),
        vec![2],
    );
    
    file.add_partition(partition);
    
    // Serialize to buffer
    let mut buffer = Cursor::new(Vec::new());
    let mut writer = io::FileWriter::new(&mut buffer);
    writer.write_file(&file).unwrap();
    
    // Test random access to embeddings
    buffer.set_position(0);
    let embedding = file.read_embedding(&mut buffer, 2).unwrap().unwrap();
    assert_eq!(embedding.model, "gpt-4");
    
    // Test model query
    let blocks = file.read_blocks_by_model(&mut buffer, "gpt-3.5").unwrap();
    assert_eq!(blocks.len(), 1);
}

#[test]
fn test_block_index_updates() {
    let mut file = PVeilFile::new();
    let mut partition = Partition::new();
    
    // Add blocks
    partition.add_data_block(create_test_block());
    partition.add_embedding_block(create_test_embedding());
    
    // Update block index
    let mut index = BlockIndex::new();
    partition.update_block_index(&mut index, 0);
    
    assert_eq!(index.block_count(), 2);
    assert!(index.get_block_location(0).is_some());
    assert!(index.get_block_location(1).is_some());
    
    // Verify block types
    assert_eq!(index.get_block_location(0).unwrap().block_type, BlockType::Data);
    assert_eq!(index.get_block_location(1).unwrap().block_type, BlockType::Embedding);
}

#[test]
fn test_invalid_block_access() {
    let mut file = PVeilFile::new();
    let mut partition = Partition::new();
    
    partition.add_data_block(create_test_block());
    partition.add_embedding_block(create_test_embedding());
    file.add_partition(partition);
    
    let mut buffer = Cursor::new(Vec::new());
    let mut writer = io::FileWriter::new(&mut buffer);
    writer.write_file(&file).unwrap();
    
    buffer.set_position(0);
    
    // Try to read embedding as data block
    assert!(matches!(
        file.read_block(&mut buffer, 1).unwrap_err(),
        Error::InvalidBlock(_)
    ));
    
    // Try to read data block as embedding
    assert!(matches!(
        file.read_embedding(&mut buffer, 0).unwrap_err(),
        Error::InvalidBlock(_)
    ));
    
    // Try to read non-existent block
    assert!(file.read_block(&mut buffer, 999).unwrap().is_none());
}

#[test]
fn test_complex_queries() {
    let mut file = PVeilFile::new();
    let mut partition = Partition::new();
    
    // Add blocks with different timestamps and models
    for i in 0..5 {
        let mut block = create_test_block();
        block.compressed_data = vec![i as u8; 100];
        partition.add_data_block(block);
        
        partition.local_indices.timestamp_index.insert(1000 * (i + 1), vec![i]);
        partition.local_indices.model_index.insert(
            format!("gpt-{}", i + 1),
            vec![i],
        );
    }
    
    file.add_partition(partition);
    
    // Add tags to blocks
    file.add_block_tag(0, "important".to_string());
    file.add_block_tag(1, "important".to_string());
    file.add_block_tag(2, "archived".to_string());
    
    // Serialize to buffer
    let mut buffer = Cursor::new(Vec::new());
    let mut writer = io::FileWriter::new(&mut buffer);
    writer.write_file(&file).unwrap();
    buffer.set_position(0);
    
    // Test multi-criteria query
    let options = QueryOptions {
        time_range: Some(1000..3000),
        model_pattern: Some("gpt-[12]".to_string()),
        tags: Some(vec!["important".to_string()]),
        limit: Some(2),
        offset: None,
    };
    
    let blocks = file.query_blocks(&mut buffer, &options).unwrap();
    assert_eq!(blocks.len(), 1);
    assert_eq!(blocks[0].compressed_data, vec![1; 100]);
    
    // Test pattern matching
    let options = QueryOptions {
        time_range: None,
        model_pattern: Some("gpt-[345]".to_string()),
        tags: None,
        limit: None,
        offset: None,
    };
    
    let blocks = file.query_blocks(&mut buffer, &options).unwrap();
    assert_eq!(blocks.len(), 3);
    
    // Test pagination
    let options = QueryOptions {
        time_range: None,
        model_pattern: None,
        tags: None,
        limit: Some(2),
        offset: Some(2),
    };
    
    let blocks = file.query_blocks(&mut buffer, &options).unwrap();
    assert_eq!(blocks.len(), 2);
    assert_eq!(blocks[0].compressed_data, vec![2; 100]);
    assert_eq!(blocks[1].compressed_data, vec![3; 100]);
}

#[test]
fn test_tag_operations() {
    let mut file = PVeilFile::new();
    let mut partition = Partition::new();
    
    // Add a block
    let block = create_test_block();
    partition.add_data_block(block);
    file.add_partition(partition);
    
    // Test tag operations
    file.add_block_tag(0, "test".to_string());
    file.add_block_tag(0, "important".to_string());
    
    let tags = file.get_block_tags(0);
    assert_eq!(tags.len(), 2);
    assert!(tags.contains(&"test".to_string()));
    assert!(tags.contains(&"important".to_string()));
    
    file.remove_block_tag(0, "test");
    let tags = file.get_block_tags(0);
    assert_eq!(tags.len(), 1);
    assert!(tags.contains(&"important".to_string()));
}

#[test]
fn test_empty_query() {
    let mut file = PVeilFile::new();
    let partition = Partition::new();
    file.add_partition(partition);
    
    let mut buffer = Cursor::new(Vec::new());
    let mut writer = io::FileWriter::new(&mut buffer);
    writer.write_file(&file).unwrap();
    buffer.set_position(0);
    
    // Test query with no criteria
    let options = QueryOptions {
        time_range: None,
        model_pattern: None,
        tags: None,
        limit: None,
        offset: None,
    };
    
    let blocks = file.query_blocks(&mut buffer, &options).unwrap();
    assert!(blocks.is_empty());
}

#[test]
fn test_compression_configuration() {
    // Test default compression
    let block = DataBlock::new(vec![0; 100], CompressionType::default());
    assert!(matches!(
        block.compression_type(),
        CompressionType::Julia { level: 3, optimize_tokens: true }
    ));

    // Test compression level modification
    let mut block = DataBlock::new(vec![0; 100], CompressionType::Julia {
        level: 1,
        optimize_tokens: true,
    });
    
    assert_eq!(block.compression_level(), Some(1));
    block.set_compression_level(5);
    assert_eq!(block.compression_level(), Some(5));
    assert!(block.compression_type().optimizes_tokens());

    // Test token optimization configuration
    block.set_token_optimization(false);
    assert!(!block.compression_type().optimizes_tokens());
    
    // Test Snappy compression
    let block = DataBlock::new(vec![0; 100], CompressionType::Snappy { level: 4 });
    assert_eq!(block.compression_level(), Some(4));
    assert!(!block.compression_type().optimizes_tokens());

    // Test no compression
    let block = DataBlock::new(vec![0; 100], CompressionType::None);
    assert_eq!(block.compression_level(), None);
    assert!(!block.compression_type().optimizes_tokens());
}

#[test]
fn test_compression_ratio() {
    let mut block = DataBlock::new(vec![0; 100], CompressionType::default());
    assert_eq!(block.compression_ratio(), 0.0);

    block.compressed_data = vec![0; 50];
    assert_eq!(block.compression_ratio(), 0.5);

    let mut block = DataBlock::new(vec![], CompressionType::default());
    assert_eq!(block.compression_ratio(), 0.0);
} 