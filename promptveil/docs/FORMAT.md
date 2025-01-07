# PromptVeil File Format (.pveil)

## Overview
The `.pveil` format is a binary file format designed for efficient storage and retrieval of large-scale conversation data, with built-in support for hardware-accelerated compression, encryption, and multi-level indexing.

## File Structure

### Header
```
Magic (4 bytes): "PVEIL"
Version (4 bytes): u32
Flags (4 bytes): u32
Partition Count (8 bytes): u64
Schema Offset (8 bytes): u64
```

### Security Layer
- Master Key ID: Identifier for the root encryption key
- Key Rotation Policy: Configuration for key rotation periods
- Encryption Config:
  - Algorithm (AES-GCM/ChaCha20)
  - Hardware acceleration settings
  - HSM configuration

### Compression Layer
- Token Model Configuration:
  - Vocabulary size
  - Pattern learning settings
  - Batch processing parameters
- GPU Settings:
  - Acceleration flags
  - Memory limits
  - Device preferences
- Pattern Dictionary:
  - Common token sequences
  - Frequency statistics
  - Compression ratios

### Partitions
Each partition is a self-contained unit optimized for parallel processing:

#### Metadata
- Time Range:
  - Start timestamp
  - End timestamp
- Size Statistics:
  - Raw size
  - Compressed size
  - Message count
- Index Statistics:
  - Index sizes
  - Update timestamps
  - Coverage metrics

#### Conversations
- Encrypted Blocks:
  - Block header
  - Encryption metadata
  - Compressed data
- Token Patterns:
  - Local pattern dictionary
  - Pattern usage statistics
  - Optimization hints

#### Local Indices
- B-Tree (Time-based):
  - Timestamp → Block mapping
  - Range query optimization
- LSH (Semantic):
  - Similarity hashes
  - Nearest neighbor lookup
- Bloom Filters:
  - Quick membership testing
  - Pattern presence checks

### Global Indices
- Partition Map:
  - Partition locations
  - Time ranges
  - Size information
- Security Index:
  - Key mappings
  - Access controls
  - Rotation history
- Pattern Index:
  - Global patterns
  - Cross-partition statistics
  - Optimization data

## Binary Layout
```
[Header]
  ├─ Magic ("PVEIL")
  ├─ Version
  └─ Global Config

[Security Layer]
  ├─ Master Key ID
  ├─ Key Rotation Policy
  └─ Encryption Config

[Compression Layer]
  ├─ Token Model Config
  ├─ GPU Settings
  └─ Pattern Dictionary

[Partitions]
  ├─ Partition 1
  │  ├─ Metadata
  │  │  ├─ Time Range
  │  │  ├─ Size Stats
  │  │  └─ Index Stats
  │  │
  │  ├─ Conversations
  │  │  ├─ Encrypted Blocks
  │  │  └─ Token Patterns
  │  │
  │  └─ Local Indices
  │     ├─ B-Tree (Time)
  │     ├─ LSH (Semantic)
  │     └─ Bloom Filters
  │
  └─ Partition N

[Global Indices]
  ├─ Partition Map
  ├─ Security Index
  └─ Pattern Index
```

## Performance Characteristics

### Time Complexity
- Block Lookup: O(1)
- Range Query: O(log n)
- Model Query: O(1)
- Tag Query: O(1)
- Complex Query: O(m log n) where m is the number of criteria

### Space Complexity
- Block Index: O(n)
- Timestamp Index: O(n)
- Model Index: O(n)
- Tag Index: O(n)

### Memory Optimizations
- Pre-allocated collections
- Result caching
- Efficient data structures
- Adaptive algorithms

## Implementation Details

### Writing Data
```rust
let mut store = ConversationStore::new();
let mut partition = Partition::new();

// Add conversations
partition.add_conversation(conversation);

// Update indices
partition.update_indices();

// Add to store
store.add_partition(partition);

// Save to disk
store.save("conversations.pveil")?;
```

### Reading Data
```rust
// Load store
let store = ConversationStore::load("conversations.pveil")?;

// Query by time range
let convs = store.find_in_range(start, end)?;

// Complex query
let options = QueryOptions {
    time_range: Some(start..end),
    semantic_query: Some("security implications"),
    limit: Some(100),
    offset: Some(0),
};
let results = store.query(&options)?;
```

## Future Considerations
1. Streaming support for real-time processing
2. Distributed indices for cluster deployments
3. Custom compression plugins
4. Advanced semantic indexing
5. Cloud storage integration 