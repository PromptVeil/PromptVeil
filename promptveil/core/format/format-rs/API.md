# Format-RS API Documentation

This document describes the public API of the format-rs crate, which provides serialization and format handling functionality for the PromptVeil system.

## Core Components

### FormatManager

The `FormatManager` is the main entry point for handling data serialization and deserialization operations.

```rust
pub struct FormatManager<P> {
    // internal fields omitted
}

impl<P: FormatProvider> FormatManager<P> {
    // Creates a new FormatManager with the given provider
    pub fn new(provider: P) -> Self

    // Serializes data using the configured provider
    pub async fn serialize(&self, data: &P::Input) -> Result<Vec<u8>, FormatError>

    // Deserializes data using the configured provider
    pub async fn deserialize(&self, data: &[u8]) -> Result<P::Output, FormatError>

    // Validates data against a schema
    pub async fn validate(&self, data: &[u8]) -> Result<bool, FormatError>
}
```

### Message Types

```rust
pub struct MessageMetadata {
    pub version: String,
    pub timestamp: i64,
    pub format_type: String,
}

pub struct Message<T> {
    pub metadata: MessageMetadata,
    pub content: T,
}
```

### Format Providers

The crate includes several built-in format providers:

- `JsonProvider`: JSON serialization/deserialization
- `BincodeProvider`: Binary format serialization
- `MessagePackProvider`: MessagePack format support

### Error Handling

```rust
pub enum FormatError {
    SerializationError(String),
    DeserializationError(String),
    VersionError(String),
    SchemaError(String),
}
```

## Usage Examples

### Basic Serialization/Deserialization

```rust
// Create a format manager with JSON provider
let provider = JsonProvider::<MyData>::new();
let manager = FormatManager::new(provider);

// Serialize data
let data = MyData { /* ... */ };
let serialized = manager.serialize(&data).await?;

// Deserialize data
let deserialized: MyData = manager.deserialize(&serialized).await?;
```

### Message Handling

```rust
// Create a message with metadata
let message = Message {
    metadata: MessageMetadata {
        version: "1.0".to_string(),
        timestamp: SystemTime::now().as_secs() as i64,
        format_type: "json".to_string(),
    },
    content: my_data,
};

// Serialize the message
let serialized = manager.serialize(&message).await?;
```

### Schema Validation

```rust
// Validate data against schema
if manager.validate(&serialized).await? {
    // Data is valid
} else {
    // Data is invalid
}
```

## Best Practices

1. Always include proper metadata with version information
2. Handle format errors appropriately in your application
3. Choose the appropriate format provider based on your needs:
   - Use `JsonProvider` for human-readable formats
   - Use `BincodeProvider` for efficient binary serialization
   - Use `MessagePackProvider` for compact binary format with schema support
4. Validate data when receiving from untrusted sources 