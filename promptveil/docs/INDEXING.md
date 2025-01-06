# Indexing and Search in PromptVeil

This document details the indexing and search functionality in PromptVeil.

## Overview

PromptVeil provides efficient search capabilities through:

1. In-memory term indexing
2. TF-IDF based relevance scoring
3. Phrase matching support
4. Role-based filtering
5. Recency-aware ranking

## Index Structure

### Term Index

The core index structure maps terms to conversation and message locations:

```python
term_index: Dict[str, Dict[str, Set[int]]] = {
    "term": {
        "conversation_id": {message_indices}
    }
}
```

### Metadata Store

Additional metadata for efficient search operations:

```python
conversation_metadata: Dict[str, Dict] = {
    "conversation_id": {
        "timestamp": float,
        "message_count": int,
        "roles": List[str],
        "messages": Dict[str, Dict]
    }
}
```

## Search Features

### Text Processing

1. Case-insensitive matching
2. Stop word removal
3. Special character handling
4. Phrase detection
5. Word boundary respect

### Ranking Factors

The search ranking considers multiple factors:

1. Term frequency (TF)
2. Inverse document frequency (IDF)
3. Phrase matches (2x boost)
4. Term proximity
5. Message recency
6. Message position (first message boost)

### Role Filtering

Support for filtering results by message role:
- User messages only
- Assistant messages only
- All messages (default)

## Implementation Details

### Tokenization

```python
def _tokenize(text: str) -> Set[str]:
    """Convert text to searchable terms."""
    # Lowercase and clean
    text = text.lower()
    text = re.sub(r'[^\w\s]', ' ', text)
    
    # Split and filter
    words = text.split()
    terms = {word for word in words 
            if len(word) > 2 and word not in stop_words}
    
    # Add phrases
    pairs = {f"{words[i]} {words[i+1]}"
            for i in range(len(words)-1)}
    terms.update(pairs)
    
    return terms
```

### Scoring Algorithm

The relevance score is calculated as:

```python
score = (tf * phrase_boost) / (df + 1)
score *= position_boost  # 1.5x for first message
score *= recency_boost   # 1.0 + 1/(days_old + 1)
score *= proximity_boost # 1.0 + 1/(min_distance + 1)
```

## Integration

### Store Integration

- Index updates on conversation changes
- Persistence with store saves
- Automatic rebuilding if needed

### Security Integration

- Index encrypted with store
- Search on decrypted data
- Secure cleanup after operations

## Performance

### Optimization Techniques

1. In-memory index for fast lookups
2. Lazy phrase matching
3. Early termination on filters
4. Efficient metadata access

### Memory Usage

- Term dictionary sharing
- Compact message references
- Metadata caching

## Best Practices

### Index Management

1. Regular index maintenance
2. Periodic reindexing
3. Monitoring index size

### Search Optimization

1. Use specific search terms
2. Leverage role filters
3. Consider recency needs

## Future Improvements

1. Fuzzy matching support
2. Semantic search capabilities
3. Distributed index support
4. Custom ranking factors
5. Advanced query syntax 