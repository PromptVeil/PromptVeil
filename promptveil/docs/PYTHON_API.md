# Python API Reference

## Overview

The PromptVeil Python API provides a high-level interface for managing LLM conversations. This document describes both currently implemented features and planned functionality.

## Feature Status

- ✅ Implemented
- 🔄 In Progress (Next Release)
- 📅 Planned (Future Release)

## Core Classes

### `Conversation`

The basic unit for managing individual conversations.

#### Current Features (✅)
```python
from promptveil import Conversation

# Create and manage conversations
conv = Conversation()
conv.add_message("user", "What is quantum computing?")
conv.add_message("assistant", "Quantum computing is...")
conv.save("quantum.pveil")
loaded_conv = Conversation.load("quantum.pveil")
```

#### Next Release Features (🔄)
```python
# Search within conversation
results = conv.search("quantum", context_size=2)
for match in results:
    print(f"Found in {match.message.role}: {match.context}")

# Basic analysis
stats = conv.analyze()
print(f"Messages: {stats.message_count}")
print(f"Compression: {stats.compression_ratio}%")

# Export to common formats
conv.export("training_data.jsonl", format="jsonl")
```

#### Planned Features (📅)
```python
# Semantic search
semantic_results = conv.semantic_search("computing security")

# Topic extraction
topics = conv.extract_topics()
print("Main topics:", topics.main_topics)
print("Topic flow:", topics.topic_flow)

# Advanced features
conv.share("colleague@company.com", expire_in="7d")
conv.save("project.pveil", version="v1")
quality = conv.evaluate_quality()
```

### `ConversationStore`

A manager for multiple conversations with advanced search and analysis capabilities.

#### Next Release Features (🔄)
```python
from promptveil import ConversationStore

# Initialize and manage store
store = ConversationStore()
conv_id = store.add_conversation(conv)
retrieved_conv = store.get_conversation(conv_id)

# Basic search
results = store.search("quantum computing")
```

#### Planned Features (📅)
```python
# Semantic search across conversations
results = store.semantic_search("security implications")
for result in results:
    print(f"Conversation: {result.conversation_id}")
    print(f"Relevance: {result.score}")

# Topic analysis across conversations
analysis = store.analyze_topics()
print("Global topics:", analysis.global_topics)
print("Trending topics:", analysis.topic_trends)

# Advanced management
store.batch_import(conversations)
clusters = store.cluster_conversations()
trends = store.trend_analysis()
```

## Data Classes

### Current (✅)
```python
@dataclass
class Message:
    role: str
    content: str
    timestamp: float
    metadata: Optional[Dict] = None
```

### Next Release (🔄)
```python
@dataclass
class SearchMatch:
    message: Message
    context: str
    score: float
    span: Tuple[int, int]

@dataclass
class ConversationStats:
    message_count: int
    token_count: int
    compression_ratio: float
    user_messages: int
    assistant_messages: int
    average_response_time: float
```

### Planned (📅)
```python
@dataclass
class TopicAnalysis:
    main_topics: List[str]
    topic_flow: Dict[str, List[str]]
    entities: List[str]
    keywords: List[str]

@dataclass
class StoreTopicAnalysis:
    global_topics: Dict[str, List[str]]
    topic_trends: Dict[str, List[str]]
    common_patterns: List[TopicPattern]
    related_topics: Dict[str, List[str]]
```

## Configuration

### Security Configuration
```python
security_config = {
    "encryption": "hardware_aes_gcm",  # or "chacha20_poly1305"
    "key_management": "file",          # or "hsm", "aws_kms"
    "key_rotation": "30d",            # Key rotation period
}
```

### Compression Configuration
```python
compression_config = {
    "level": "adaptive",     # or 1-9
    "mode": "token_aware",   # or "general"
    "gpu_enabled": True      # Use GPU if available
}
```

## Error Handling

```python
from promptveil.exceptions import (
    PromptVeilError,           # Base exception
    CompressionError,          # Compression-related errors
    SecurityError,             # Security-related errors
    FormatError,              # File format errors
    StoreError                # Store-related errors
)
```

## Best Practices

1. **Resource Management**
   ```python
   # Use context managers
   with store.new_conversation() as conv:
       conv.add_message(...)
   ```

2. **Error Handling**
   ```python
   try:
       conv.save("chat.pveil")
   except SecurityError as e:
       handle_security_error(e)
   ```

3. **Memory Efficiency**
   ```python
   # Stream large conversations
   for message in conv.iter_messages():
       process_message(message)
   ```

## Feature Roadmap

### Release 0.1.0 (Current)
- Core architecture setup
- Basic conversation management
- Initial file storage
- Basic security layer

### Release 0.2.0
- High-performance compression engine
- Hardware-accelerated security
- Text and semantic search
- Conversation store with analytics

### Release 1.0.0
- Topic extraction and analysis
- Export to common formats
- Sharing and collaboration
- Version control
- Quality metrics
- Training data export
- Cloud storage integration 