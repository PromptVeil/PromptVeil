# Index-RS API Documentation

This document describes the public API of the index-rs crate, which provides hybrid text and vector search functionality for the PromptVeil system.

## Core Components

### IndexManager

The `IndexManager` is the main entry point for managing both text and vector indices.

```rust
pub struct IndexManager<'a> {
    // internal fields omitted
}

impl<'a> IndexManager<'a> {
    // Creates a new IndexManager with the given path and configuration
    pub fn new(index_path: PathBuf, config: IndexConfig) -> Result<Self, IndexError>

    // Adds a conversation to both text and vector indices
    pub fn add_conversation(&mut self, id: &str, text: &str, vector: Vec<f32>) -> Result<(), IndexError>

    // Performs text-based search
    pub fn search_text(&self, query: &str, limit: usize) -> Result<Vec<SearchResult>, IndexError>

    // Performs vector similarity search
    pub fn search_similar(&self, vector: Vec<f32>, limit: usize) -> Result<Vec<(String, f32)>, IndexError>

    // Deletes a conversation from both indices
    pub fn delete_conversation(&mut self, id: &str) -> Result<(), IndexError>

    // Clears all indices
    pub fn clear(&mut self) -> Result<(), IndexError>
}
```

### Configuration

```rust
pub struct IndexConfig {
    pub vector_dim: usize,        // Dimension of vectors
    pub max_elements: usize,      // Maximum number of elements in the index
    pub ef_construction: usize,   // HNSW index construction parameter
    pub m: usize,                 // HNSW index connectivity parameter
    pub enable_highlighting: bool, // Enable text search highlighting
    pub text_analyzer: String,    // Text analyzer configuration
    pub text_index_memory: usize, // Memory limit for text index
}
```

### Search Results

```rust
pub struct SearchResult {
    pub conversation_id: String,  // Unique identifier
    pub score: f32,              // Relevance score
    pub snippet: String,         // Text snippet
    pub highlights: Vec<String>, // Highlighted matches
}
```

### Error Handling

```rust
pub enum IndexError {
    InitializationError(String),
    TextIndex(String),
    VectorIndex(String),
    InvalidVectorDimensions { expected: usize, got: usize },
    DocumentNotFound(String),
    Io(std::io::Error),
}
```

## Usage Examples

### Basic Index Operations

```rust
// Create a new index manager
let config = IndexConfig::default();
let index_manager = IndexManager::new(path, config)?;

// Add a conversation
let id = "conv123";
let text = "Conversation content...";
let vector = vec![0.1, 0.2, 0.3]; // 384-dimensional vector
index_manager.add_conversation(id, text, vector)?;

// Search by text
let results = index_manager.search_text("query", 10)?;
for result in results {
    println!("Found: {} (score: {})", result.conversation_id, result.score);
}

// Search by vector similarity
let similar = index_manager.search_similar(query_vector, 10)?;
for (id, score) in similar {
    println!("Similar: {} (score: {})", id, score);
}
```

### Configuration Example

```rust
let config = IndexConfig {
    vector_dim: 384,
    max_elements: 10000,
    ef_construction: 100,
    m: 16,
    enable_highlighting: true,
    text_analyzer: "default".to_string(),
    text_index_memory: 1000000,
};
```

## Best Practices

1. Choose appropriate index configuration based on your dataset size and performance requirements
2. Use appropriate vector dimensions (default is 384 for common embedding models)
3. Handle index errors appropriately in your application
4. Consider memory constraints when configuring text index
5. Use batch operations when possible for better performance
6. Regularly maintain and optimize indices for large datasets 