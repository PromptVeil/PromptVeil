use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};

mod error;
mod io_utils;
mod index;
#[cfg(test)]
mod tests;

pub use error::Error;
pub use index::{BlockIndex, BlockLocation, BlockType, QueryOptions};
pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PVeilFile {
    header: PVeilHeader,
    schema: PVeilSchema,
    partitions: Vec<Partition>,
    block_index: BlockIndex,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PVeilHeader {
    magic: [u8; 5],           // "PVEIL"
    version: u32,             // Format version
    flags: u32,               // File flags
    partition_count: u64,     // Number of partitions
    schema_offset: u64,       // Offset to schema
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PVeilSchema {
    version: u32,
    fields: Vec<Field>,
    compatibility: SchemaCompatibility,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Partition {
    metadata: PartitionMetadata,
    data_blocks: Vec<DataBlock>,
    embedding_blocks: Vec<EmbeddingBlock>,
    local_indices: PartitionIndices,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PartitionMetadata {
    date_range: (u64, u64),
    model_types: Vec<String>,
    size_bytes: u64,
    message_count: u32,
    stats: DataStats,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DataBlock {
    header: BlockHeader,
    compressed_data: Vec<u8>,
    encryption_metadata: EncryptionMetadata,
    encrypted_data: Vec<u8>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BlockHeader {
    size: u32,
    checksum: [u8; 32],
    compression_type: CompressionType,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EncryptionMetadata {
    version: u32,
    algorithm: String,
    key_id: String,
    nonce: [u8; 12],
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EmbeddingBlock {
    model: String,
    dimension: u32,
    vectors: Vec<Vector>,
    chunk_map: HashMap<u32, Range>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Vector {
    data: Vec<f32>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Range {
    start: u64,
    end: u64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Field {
    name: String,
    field_type: FieldType,
    nullable: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum FieldType {
    String,
    Integer,
    Float,
    Boolean,
    Array(Box<FieldType>),
    Struct(Vec<Field>),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum SchemaCompatibility {
    Backward,
    Forward,
    Full,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum CompressionType {
    None,
    Julia { level: u8, optimize_tokens: bool },
    Zstd { level: u8 },
}

impl Default for CompressionType {
    fn default() -> Self {
        CompressionType::Julia { level: 3, optimize_tokens: true }
    }
}

impl CompressionType {
    pub fn with_level(self, level: u8) -> Self {
        match self {
            Self::None => Self::None,
            Self::Julia { optimize_tokens, .. } => Self::Julia {
                level,
                optimize_tokens,
            },
            Self::Zstd { .. } => Self::Zstd { level: 3 },
        }
    }

    pub fn with_token_optimization(self, optimize: bool) -> Self {
        match self {
            Self::Julia { level, .. } => Self::Julia {
                level,
                optimize_tokens: optimize,
            },
            other => other,
        }
    }

    pub fn level(&self) -> Option<u8> {
        match self {
            Self::None => None,
            Self::Julia { level, .. } => Some(*level),
            Self::Zstd { level } => Some(*level),
        }
    }

    pub fn optimizes_tokens(&self) -> bool {
        matches!(self, Self::Julia { optimize_tokens: true, .. })
    }
}

impl PVeilFile {
    pub fn new() -> Self {
        Self {
            header: PVeilHeader {
                magic: *b"PVEIL",
                version: 1,
                flags: 0,
                partition_count: 0,
                schema_offset: 0,
            },
            schema: PVeilSchema {
                version: 1,
                fields: Vec::new(),
                compatibility: SchemaCompatibility::Full,
            },
            partitions: Vec::new(),
            block_index: BlockIndex::new(),
        }
    }

    pub fn save<P: AsRef<std::path::Path>>(&self, path: P) -> Result<()> {
        let file = File::create(path)?;
        let mut writer = io_utils::FileWriter::new(file);
        let data = bincode::serialize(self)?;
        writer.write(&data)?;
        Ok(())
    }

    pub fn load<P: AsRef<std::path::Path>>(path: P) -> Result<Self> {
        let file = File::open(path)?;
        let mut reader = io_utils::FileReader::new(file);
        let mut data = Vec::new();
        reader.read(&mut data)?;
        Ok(bincode::deserialize(&data)?)
    }

    pub fn add_partition(&mut self, partition: Partition) {
        self.partitions.push(partition);
        self.header.partition_count = self.partitions.len() as u64;
    }

    pub fn get_partition(&self, index: usize) -> Option<&Partition> {
        self.partitions.get(index)
    }

    pub fn get_partition_mut(&mut self, index: usize) -> Option<&mut Partition> {
        self.partitions.get_mut(index)
    }

    pub fn partition_count(&self) -> usize {
        self.partitions.len()
    }

    pub fn remove_partition(&mut self, index: usize) -> Option<Partition> {
        if index < self.partitions.len() {
            let partition = self.partitions.remove(index);
            self.header.partition_count = self.partitions.len() as u64;
            Some(partition)
        } else {
            None
        }
    }

    pub fn clear_partitions(&mut self) {
        self.partitions.clear();
        self.header.partition_count = 0;
    }

    pub fn merge_partitions(&mut self, start: usize, end: usize) -> Result<()> {
        if start >= end || end > self.partitions.len() {
            return Err(Error::InvalidPartition("Invalid partition range".into()));
        }

        let mut merged = Partition::new();
        let partitions = self.partitions.drain(start..end).collect::<Vec<_>>();
        
        // Merge logic here...
        for partition in partitions {
            for block in partition.data_blocks {
                merged.add_data_block(block);
            }
            for block in partition.embedding_blocks {
                merged.add_embedding_block(block);
            }
        }
        
        self.partitions.insert(start, merged);
        Ok(())
    }

    pub fn read_block<R: Read + Seek>(&mut self, reader: &mut R, block_id: u32) -> Result<Option<DataBlock>> {
        if let Some(location) = self.block_index.get_block_location(block_id) {
            reader.seek(SeekFrom::Start(location.offset))?;
            // Read block logic here...
            Ok(None)
        } else {
            Ok(None)
        }
    }

    pub fn read_embedding<R: Read + Seek>(&mut self, reader: &mut R, block_id: u32) -> Result<Option<EmbeddingBlock>> {
        if let Some(location) = self.block_index.get_block_location(block_id) {
            reader.seek(SeekFrom::Start(location.offset))?;
            // Read embedding logic here...
            Ok(None)
        } else {
            Ok(None)
        }
    }

    pub fn find_blocks_in_range(&mut self, start: u64, end: u64) -> Vec<u32> {
        self.block_index.find_blocks_in_range(start, end)
    }

    pub fn find_blocks_by_model(&mut self, model: &str) -> Vec<u32> {
        self.block_index.find_blocks_by_model(model)
    }

    pub fn read_blocks_in_range<R: Read + Seek>(&mut self, reader: &mut R, start: u64, end: u64) -> Result<Vec<DataBlock>> {
        let block_ids = self.find_blocks_in_range(start, end);
        let mut blocks = Vec::new();
        for id in block_ids {
            if let Some(block) = self.read_block(reader, id)? {
                blocks.push(block);
            }
        }
        Ok(blocks)
    }

    pub fn read_blocks_by_model<R: Read + Seek>(&mut self, reader: &mut R, model: &str) -> Result<Vec<DataBlock>> {
        let block_ids = self.find_blocks_by_model(model);
        let mut blocks = Vec::new();
        for id in block_ids {
            if let Some(block) = self.read_block(reader, id)? {
                blocks.push(block);
            }
        }
        Ok(blocks)
    }

    pub fn query_blocks<R: Read + Seek>(&mut self, reader: &mut R, options: &QueryOptions) -> Result<Vec<DataBlock>> {
        let block_ids = self.block_index.query_blocks(options);
        let mut blocks = Vec::new();
        for id in block_ids {
            if let Some(block) = self.read_block(reader, id)? {
                blocks.push(block);
            }
        }
        Ok(blocks)
    }

    pub fn query_embeddings<R: Read + Seek>(&mut self, reader: &mut R, options: &QueryOptions) -> Result<Vec<EmbeddingBlock>> {
        let block_ids = self.block_index.query_blocks(options);
        let mut blocks = Vec::new();
        for id in block_ids {
            if let Some(block) = self.read_embedding(reader, id)? {
                blocks.push(block);
            }
        }
        Ok(blocks)
    }

    pub fn add_block_tag(&mut self, block_id: u32, tag: String) {
        if let Some(block) = self.block_index.get_block_mut(block_id) {
            block.tags.insert(tag);
        }
    }

    pub fn remove_block_tag(&mut self, block_id: u32, tag: &str) {
        if let Some(block) = self.block_index.get_block_mut(block_id) {
            block.tags.remove(tag);
        }
    }

    pub fn get_block_tags(&self, block_id: u32) -> Option<&std::collections::HashSet<String>> {
        self.block_index.get_block(block_id).map(|block| &block.tags)
    }
}

impl Partition {
    pub fn new() -> Self {
        Self {
            metadata: PartitionMetadata {
                date_range: (0, 0),
                model_types: Vec::new(),
                size_bytes: 0,
                message_count: 0,
                stats: DataStats::default(),
            },
            data_blocks: Vec::new(),
            embedding_blocks: Vec::new(),
            local_indices: PartitionIndices::default(),
        }
    }

    pub fn add_data_block(&mut self, block: DataBlock) {
        self.metadata.size_bytes += block.header.size as u64;
        self.metadata.message_count += 1;
        self.data_blocks.push(block);
    }

    pub fn add_embedding_block(&mut self, block: EmbeddingBlock) {
        self.embedding_blocks.push(block);
    }

    pub fn update_stats(&mut self) {
        let mut stats = DataStats::default();
        
        for block in &self.data_blocks {
            stats.compressed_size += block.compressed_data.len() as u64;
            stats.original_size += block.header.size as u64;
        }
        
        if stats.original_size > 0 {
            stats.compression_ratio = stats.compressed_size as f32 / stats.original_size as f32;
        }
        
        self.metadata.stats = stats;
    }

    pub fn get_data_block(&self, index: usize) -> Option<&DataBlock> {
        self.data_blocks.get(index)
    }

    pub fn get_embedding_block(&self, index: usize) -> Option<&EmbeddingBlock> {
        self.embedding_blocks.get(index)
    }

    pub fn find_messages_by_timestamp(&self, start: u64, end: u64) -> Vec<u32> {
        let mut messages = Vec::new();
        for (&timestamp, indices) in &self.local_indices.timestamp_index {
            if timestamp >= start && timestamp <= end {
                messages.extend(indices);
            }
        }
        messages.sort_unstable();
        messages.dedup();
        messages
    }

    pub fn find_messages_by_model(&self, model: &str) -> Vec<u32> {
        self.local_indices.model_index
            .get(model)
            .cloned()
            .unwrap_or_default()
    }

    pub fn update_block_index(&self, index: &mut BlockIndex, base_offset: u64) -> u64 {
        let mut current_offset = base_offset;
        
        // Index data blocks
        for (i, block) in self.data_blocks.iter().enumerate() {
            let location = BlockLocation::new(
                current_offset,
                block.header.size,
                BlockType::Data,
            );
            index.add_block(i as u32, location);
            current_offset += bincode::serialized_size(block)
                .unwrap_or(0) as u64;
        }
        
        // Index embedding blocks
        for (i, block) in self.embedding_blocks.iter().enumerate() {
            let location = BlockLocation::new(
                current_offset,
                (block.dimension * block.vectors.len() as u32 * 4) as u32,
                BlockType::Embedding,
            );
            index.add_block((i + self.data_blocks.len()) as u32, location);
            current_offset += bincode::serialized_size(block)
                .unwrap_or(0) as u64;
        }
        
        current_offset
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct DataStats {
    pub total_tokens: u64,
    pub compressed_size: u64,
    pub original_size: u64,
    pub compression_ratio: f32,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct PartitionIndices {
    pub message_index: HashMap<String, Vec<u32>>,
    pub timestamp_index: HashMap<u64, Vec<u32>>,
    pub model_index: HashMap<String, Vec<u32>>,
}

impl DataBlock {
    pub fn new(data: Vec<u8>, compression: CompressionType) -> Self {
        Self {
            header: BlockHeader {
                size: data.len() as u32,
                checksum: [0; 32],
                compression_type: compression,
            },
            compressed_data: Vec::new(),
            encryption_metadata: EncryptionMetadata {
                version: 1,
                algorithm: "AES-256-GCM".to_string(),
                key_id: String::new(),
                nonce: [0; 12],
            },
            encrypted_data: Vec::new(),
        }
    }

    pub fn with_compression(mut self, compression: CompressionType) -> Self {
        self.header.compression_type = compression;
        self
    }

    pub fn compression_type(&self) -> CompressionType {
        self.header.compression_type
    }

    pub fn compression_level(&self) -> Option<u8> {
        self.header.compression_type.level()
    }

    pub fn set_compression_level(&mut self, level: u8) {
        self.header.compression_type = self.header.compression_type.with_level(level);
    }

    pub fn set_token_optimization(&mut self, optimize: bool) {
        self.header.compression_type = self.header.compression_type.with_token_optimization(optimize);
    }

    pub fn compression_ratio(&self) -> f32 {
        if self.header.size == 0 {
            return 0.0;
        }
        self.compressed_data.len() as f32 / self.header.size as f32
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::io_utils::{FileReader, FileWriter};
    use std::io::Cursor;

    #[test]
    fn test_file_operations() {
        let mut file = PVeilFile::new();
        
        // Add a partition
        let mut partition = Partition::new();
        let block = DataBlock {
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
        };
        partition.add_data_block(block);
        file.add_partition(partition);

        // Test serialization/deserialization
        let mut buffer = Cursor::new(Vec::new());
        let mut writer = FileWriter::new(&mut buffer);
        writer.write_file(&file).unwrap();

        buffer.set_position(0);
        let mut reader = FileReader::new(buffer);
        let loaded = reader.read_file().unwrap();

        assert_eq!(loaded.partition_count(), 1);
        assert_eq!(loaded.header.version, 1);
    }

    #[test]
    fn test_query_blocks() {
        let mut file = PVeilFile::new();
        
        // Add test data
        let mut partition = Partition::new();
        for i in 0..5 {
            let block = DataBlock::new(
                vec![0; 100],
                CompressionType::Julia { level: 3, optimize_tokens: true }
            );
            partition.add_data_block(block);
            
            // Add metadata
            partition.local_indices.timestamp_index.insert(1000 * (i as u64 + 1), vec![i as u32]);
            partition.local_indices.model_index.insert(
                format!("gpt-{}", i + 1),
                vec![i as u32]
            );
        }
        file.add_partition(partition);

        // Test queries
        let options = QueryOptions {
            start_time: Some(1000),
            end_time: Some(3000),
            model: Some("gpt-[12]".to_string()),
            block_type: None,
            tags: Some(vec!["important".to_string()]),
            limit: Some(2),
            offset: None,
        };

        let mut buffer = Cursor::new(Vec::new());
        let blocks = file.query_blocks(&mut buffer, &options).unwrap();
        assert!(!blocks.is_empty());
    }
} 