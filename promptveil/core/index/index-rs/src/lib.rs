use std::path::PathBuf;
use thiserror::Error;

mod text_index;
mod vector_index;

#[derive(Debug, Error)]
pub enum IndexError {
    #[error("Failed to initialize index: {0}")]
    InitializationError(String),
    #[error("Text index error: {0}")]
    TextIndex(String),
    #[error("Vector index error: {0}")]
    VectorIndex(String),
    #[error("Invalid vector dimensions: expected {expected}, got {got}")]
    InvalidVectorDimensions { expected: usize, got: usize },
    #[error("Document not found: {0}")]
    DocumentNotFound(String),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

#[derive(Clone, Debug)]
pub struct SearchResult {
    pub conversation_id: String,
    pub score: f32,
    pub snippet: String,
    pub highlights: Vec<String>,
}

#[derive(Clone, Debug)]
pub struct IndexConfig {
    pub vector_dim: usize,
    pub max_elements: usize,
    pub text_index_memory: usize,
    pub m: usize,
    pub ef_construction: usize,
    pub enable_highlighting: bool,
    pub text_analyzer: String,
}

impl Default for IndexConfig {
    fn default() -> Self {
        Self {
            vector_dim: 384,
            max_elements: 10_000,
            text_index_memory: 50_000_000,
            m: 16,
            ef_construction: 200,
            enable_highlighting: true,
            text_analyzer: "simple".to_string(),
        }
    }
}

pub struct IndexManager<'a> {
    text_index: text_index::TextIndex,
    vector_index: vector_index::VectorIndex<'a>,
    config: IndexConfig,
}

impl<'a> IndexManager<'a> {
    pub fn new(index_path: PathBuf, config: IndexConfig) -> Result<Self, IndexError> {
        let text_index = text_index::TextIndex::new(index_path.join("text"), config.clone())?;
        let vector_index = vector_index::VectorIndex::new(
            index_path.join("vector"),
            &config,
        )
        .map_err(|e| IndexError::VectorIndex(e.to_string()))?;

        Ok(Self {
            text_index,
            vector_index,
            config,
        })
    }

    pub fn add_conversation(&mut self, id: &str, text: &str, vector: Vec<f32>) -> Result<(), IndexError> {
        if vector.len() != self.config.vector_dim {
            return Err(IndexError::InvalidVectorDimensions {
                expected: self.config.vector_dim,
                got: vector.len(),
            });
        }

        self.text_index.add_document(id, text)?;
        self.vector_index
            .add_vector(id.to_string(), vector)
            .map_err(|e| IndexError::VectorIndex(e.to_string()))?;
        Ok(())
    }

    pub fn search_text(&self, query: &str, limit: usize) -> Result<Vec<SearchResult>, IndexError> {
        self.text_index.search(query, limit)
    }

    pub fn search_similar(&self, vector: Vec<f32>, limit: usize) -> Result<Vec<(String, f32)>, IndexError> {
        if vector.len() != self.config.vector_dim {
            return Err(IndexError::InvalidVectorDimensions {
                expected: self.config.vector_dim,
                got: vector.len(),
            });
        }

        self.vector_index
            .search_similar(vector, limit)
            .map_err(|e| IndexError::VectorIndex(e.to_string()))
    }

    pub fn delete_conversation(&mut self, id: &str) -> Result<(), IndexError> {
        self.text_index.delete_document(id)?;
        self.vector_index.delete_vector(id)?;
        Ok(())
    }

    pub fn clear(&mut self) -> Result<(), IndexError> {
        self.text_index.clear()?;
        self.vector_index.clear()?;
        Ok(())
    }
}

pub struct Index<'a> {
    text_index: text_index::TextIndex,
    vector_index: vector_index::VectorIndex<'a>,
}

impl<'a> Index<'a> {
    pub fn new(index_path: PathBuf) -> Result<Self, IndexError> {
        Self::new_with_config(index_path, IndexConfig::default())
    }

    pub fn new_with_config(index_path: PathBuf, config: IndexConfig) -> Result<Self, IndexError> {
        let text_index = text_index::TextIndex::new(index_path.join("text"), config.clone())?;
        let vector_index = vector_index::VectorIndex::new(
            index_path.join("vector"),
            &config,
        )
        .map_err(|e| IndexError::VectorIndex(e.to_string()))?;
        
        Ok(Self {
            text_index,
            vector_index,
        })
    }

    pub fn add_conversation(&self, id: &str, text: &str, vector: Vec<f32>) -> Result<(), IndexError> {
        self.text_index.add_document(id, text)?;
        self.vector_index.add_vector(id.to_string(), vector)?;
        Ok(())
    }

    pub fn search_text(&self, query: &str, limit: usize) -> Result<Vec<SearchResult>, IndexError> {
        self.text_index.search(query, limit)
    }

    pub fn search_similar(&self, vector: Vec<f32>, limit: usize) -> Result<Vec<(String, f32)>, IndexError> {
        self.vector_index.search_similar(vector, limit)
    }

    pub fn delete_conversation(&self, id: &str) -> Result<(), IndexError> {
        self.text_index.delete_document(id)?;
        self.vector_index.delete_vector(id)?;
        Ok(())
    }

    pub fn clear(&self) -> Result<(), IndexError> {
        self.text_index.clear()?;
        self.vector_index.clear()?;
        Ok(())
    }
} 