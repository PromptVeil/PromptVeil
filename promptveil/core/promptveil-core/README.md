# PromptVeil Core

Core integration module for PromptVeil, providing a unified interface for all core functionality.

## Features

- Unified interface for all core modules
- Secure conversation storage and retrieval
- Advanced search capabilities:
  - Full-text search with highlighting
  - Semantic search using embeddings
  - Hybrid search combining both approaches
- Hardware-accelerated encryption
- Efficient data formatting and compression
- Caching and performance optimizations

## Architecture

The module integrates several main components:

1. **Conversation Manager**
   - Message and conversation handling
   - Metadata management
   - Efficient storage and retrieval

2. **Embedding Manager**
   - Text-to-vector conversion using all-MiniLM-L6-v2
   - Efficient caching with TTL
   - Batch processing support

3. **Index Manager**
   - Full-text search using Tantivy
   - Vector similarity search using HNSW
   - Efficient indexing and retrieval

4. **Security Manager**
   - Hardware-accelerated encryption
   - Secure key management
   - Memory protection

5. **Format Manager**
   - Efficient binary format
   - Data compression
   - Metadata handling

## Usage Examples

### Basic Usage

```rust
use promptveil_core::{PromptVeilCore, Config};

// Initialize with default configuration
let config = Config::default();
let core = PromptVeilCore::new("path/to/data", config).await?;

// Add a conversation
let conversation = Conversation {
    id: "conv1".to_string(),
    messages: vec![
        Message {
            role: "user".to_string(),
            content: "What is machine learning?".to_string(),
            timestamp: chrono::Utc::now().timestamp(),
        }
    ],
    metadata: None,
};

core.add_conversation(conversation).await?;
```

### Search Capabilities

```rust
// Text-based search
let text_results = core.search_text("machine learning", 10).await?;

// Semantic search
let semantic_results = core.search_semantic(
    "artificial intelligence and neural networks", 
    10
).await?;

// Hybrid search with custom weights
let hybrid_results = core.hybrid_search(
    "deep learning techniques",
    0.3, // text weight
    0.7, // semantic weight
    10   // limit
).await?;
```

### Python Integration

```python
from promptveil import PromptVeilCore

# Initialize core
core = PromptVeilCore("path/to/data")

# Add a conversation
conversation = {
    "id": "conv1",
    "messages": [{
        "role": "user",
        "content": "What is machine learning?",
        "timestamp": 1234567890
    }],
    "metadata": {"tags": ["ai", "education"]}
}

await core.add_conversation(conversation)

# Search conversations
results = await core.search_text("machine learning", limit=10)
semantic_results = await core.search_semantic("artificial intelligence", limit=10)
```

## Performance Optimizations

- Async operations for non-blocking I/O
- Efficient embedding caching with TTL
- Batch processing for embeddings
- Hardware acceleration where available
- Thread-safe concurrent access
- Optimized memory usage

## Dependencies

- `index-rs`: Search and indexing
- `security-rs`: Encryption and security
- `format-rs`: Data formatting
- `rust-bert`: Text embeddings
- `tokio`: Async runtime
- `pyo3`: Python bindings

## Contributing

1. Fork the repository
2. Create your feature branch
3. Commit your changes
4. Push to the branch
5. Create a Pull Request

## License

MIT License - See LICENSE file for details 