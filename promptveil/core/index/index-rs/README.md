# Index-RS

A high-performance indexing module for PromptVeil, providing both text-based and vector-based search capabilities.

## Features

- Full-text search using Tantivy
  - Field-based queries
  - Relevance scoring
  - Snippet generation
  - Highlighting support
- Vector similarity search using HNSW
  - Optimized for high-dimensional vectors
  - Fast similarity search
  - Memory-efficient storage
  - Support for large-scale datasets
- Async API for all operations
- Hardware-optimized search algorithms
- Efficient memory management
- Persistence support

## Architecture

The module consists of two main components:

1. **Text Index**
   - Built on Tantivy for full-text search
   - Support for field-based queries
   - Relevance scoring
   - Snippet generation
   - Highlighting support

2. **Vector Index**
   - Built on HNSW for approximate nearest neighbor search
   - Optimized for high-dimensional vectors
   - Fast similarity search
   - Memory-efficient storage
   - Support for large-scale datasets

## Usage

```rust
use index_rs::{Index, IndexManager};

// Initialize the index
let index = Index::new("path/to/index").await?;

// Add a conversation
index.add_conversation(
    "conv_id_1",
    "conversation text content",
    vec![0.1, 0.2, ..., 0.768] // 768-dimensional vector
).await?;

// Text-based search
let results = index.search_text("query", 10).await?;
for result in results {
    println!("Found: {} (score: {})", result.conversation_id, result.score);
    println!("Snippet: {}", result.snippet);
    println!("Highlights: {:?}", result.highlights);
}

// Vector similarity search
let similar = index.search_similar(query_vector, 10).await?;
for match in similar {
    println!("Similar: {} (distance: {})", match.conversation_id, match.distance);
}
```

## Performance

- Text search: O(log N) for most queries
- Vector search: O(log N) average case
- Memory usage: ~1KB per document for text index
- Vector index: ~3KB per 768-dimensional vector

## Dependencies

- `tantivy`: Full-text search engine
- `hnsw`: Hierarchical Navigable Small World graphs
- `tokio`: Async runtime
- `serde`: Serialization
- `thiserror`: Error handling

## Roadmap

### Planned Features

1. **Advanced Search**
   - Hybrid search combining text and vector similarity
   - Configurable weights for text and vector scores
   - Advanced filtering (role, timestamp)
   - Custom ranking factors

2. **Performance Optimizations**
   - Distributed index support
   - Batch processing optimizations
   - Memory usage improvements

3. **Additional Features**
   - Real-time index updates
   - Custom tokenizers and analyzers
   - Advanced query syntax

## Contributing

1. Fork the repository
2. Create your feature branch
3. Commit your changes
4. Push to the branch
5. Create a Pull Request

## License

MIT License - See LICENSE file for details 