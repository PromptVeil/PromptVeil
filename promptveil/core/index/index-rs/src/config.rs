use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexConfig {
    // Vector index configuration
    pub vector_dim: usize,
    pub max_elements: usize,
    pub ef_construction: usize,
    pub m: usize,

    // Text index configuration
    pub text_analyzer: String,
    pub text_index_memory: usize,  // in bytes
    pub enable_highlighting: bool,
}

impl Default for IndexConfig {
    fn default() -> Self {
        Self {
            vector_dim: 768,
            max_elements: 100_000,
            ef_construction: 200,
            m: 16,
            text_analyzer: "default".to_string(),
            text_index_memory: 50_000_000,  // 50MB
            enable_highlighting: true,
        }
    }
} 