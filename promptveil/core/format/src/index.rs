use std::collections::HashMap;
use std::collections::HashSet;
use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BlockIndex {
    blocks: HashMap<u32, BlockLocation>,
    model_index: HashMap<String, Vec<u32>>,
    timestamp_index: HashMap<u64, Vec<u32>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BlockLocation {
    pub offset: u64,
    pub size: u32,
    pub block_type: BlockType,
    pub tags: HashSet<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq)]
pub enum BlockType {
    Data,
    Embedding,
}

#[derive(Debug, Clone)]
pub struct QueryOptions {
    pub start_time: Option<u64>,
    pub end_time: Option<u64>,
    pub model: Option<String>,
    pub block_type: Option<BlockType>,
    pub tags: Option<Vec<String>>,
    pub limit: Option<usize>,
    pub offset: Option<usize>,
}

impl QueryOptions {
    pub fn new() -> Self {
        Self {
            start_time: None,
            end_time: None,
            model: None,
            block_type: None,
            tags: None,
            limit: None,
            offset: None,
        }
    }
}

impl Default for QueryOptions {
    fn default() -> Self {
        Self::new()
    }
}

impl BlockLocation {
    pub fn new(offset: u64, size: u32, block_type: BlockType) -> Self {
        Self {
            offset,
            size,
            block_type,
            tags: HashSet::new(),
        }
    }
}

impl BlockIndex {
    pub fn new() -> Self {
        Self {
            blocks: HashMap::new(),
            model_index: HashMap::new(),
            timestamp_index: HashMap::new(),
        }
    }

    pub fn add_block(&mut self, id: u32, mut location: BlockLocation) {
        if location.tags.is_empty() {
            location.tags = HashSet::new();
        }
        self.blocks.insert(id, location);
    }

    pub fn get_block_location(&self, id: u32) -> Option<BlockLocation> {
        self.blocks.get(&id).cloned()
    }

    pub fn find_blocks_in_range(&self, start: u64, end: u64) -> Vec<u32> {
        let mut blocks = Vec::new();
        for (&timestamp, ids) in &self.timestamp_index {
            if timestamp >= start && timestamp <= end {
                blocks.extend(ids);
            }
        }
        blocks.sort_unstable();
        blocks.dedup();
        blocks
    }

    pub fn find_blocks_by_model(&self, model: &str) -> Vec<u32> {
        self.model_index
            .get(model)
            .cloned()
            .unwrap_or_default()
    }

    pub fn query_blocks(&self, options: &QueryOptions) -> Vec<u32> {
        let mut blocks: Vec<u32> = self.blocks.keys().copied().collect();

        // Filter by time range
        if let (Some(_start_time), Some(_end_time)) = (options.start_time, options.end_time) {
            blocks.retain(|&id| {
                if let Some(_) = self.get_block_location(id) {
                    // Implement time range filtering logic here
                    true
                } else {
                    false
                }
            });
        }

        // Filter by model
        if let Some(ref model) = options.model {
            let model_blocks = self.find_blocks_by_model(model);
            blocks.retain(|&id| model_blocks.contains(&id));
        }

        // Filter by block type
        if let Some(block_type) = options.block_type {
            blocks.retain(|&id| {
                if let Some(location) = self.get_block_location(id) {
                    location.block_type == block_type
                } else {
                    false
                }
            });
        }

        blocks
    }

    pub fn get_block(&self, id: u32) -> Option<&BlockLocation> {
        self.blocks.get(&id)
    }

    pub fn get_block_mut(&mut self, id: u32) -> Option<&mut BlockLocation> {
        self.blocks.get_mut(&id)
    }
} 