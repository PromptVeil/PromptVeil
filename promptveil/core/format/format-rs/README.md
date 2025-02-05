# Format-RS

A Rust implementation of the format layer for PromptVeil, providing efficient message serialization, versioning, and schema validation.

## Features

- Multiple serialization formats support (JSON, MessagePack, Bincode)
- Message versioning with automatic conversion
- Schema validation and registry
- Asynchronous API for all operations
- Comprehensive testing suite

## Installation

To use this module, add it as a dependency in your `Cargo.toml`:

```toml
[dependencies]
format-rs = { path = "path/to/format-rs" }
```

## Usage

### Basic Serialization

```rust
use format_rs::{FormatManager, JsonProvider, Message, MessageMetadata};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
struct MyData {
    field1: String,
    field2: i32,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let manager = FormatManager::new(JsonProvider);
    
    let message = Message {
        metadata: MessageMetadata {
            version: "1.0".to_string(),
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)?
                .as_secs() as i64,
            format_type: "json".to_string(),
        },
        content: MyData {
            field1: "test".to_string(),
            field2: 42,
        },
    };

    let serialized = manager.serialize(&message).await?;
    let deserialized = manager.deserialize(&serialized).await?;
    
    Ok(())
}
```

### Version Management

```rust
use format_rs::{VersionRegistry, VersionConverter};

struct V1ToV2Converter;

impl VersionConverter<Message<MyData>, Message<MyData>> for V1ToV2Converter {
    fn convert(&self, mut data: Message<MyData>) -> Result<Message<MyData>, FormatError> {
        // Convert data from V1 to V2 format
        Ok(data)
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let registry = VersionRegistry::new("2.0".to_string());
    registry.register_converter("1.0".to_string(), V1ToV2Converter).await?;
    
    // Automatically converts old messages to current version
    let converted = registry.convert_to_current(old_message).await?;
    
    Ok(())
}
```

### Schema Validation

```rust
use format_rs::{SchemaRegistry, JsonSchemaValidator};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let registry = SchemaRegistry::new();
    registry
        .register_validator("myschema".to_string(), JsonSchemaValidator::<MyData>::new())
        .await?;
    
    let is_valid = registry.validate("myschema", &data).await?;
    
    Ok(())
}
```

## Security

This module implements several security measures:

- Schema validation to prevent invalid data
- Version control to maintain data integrity
- Safe serialization/deserialization

## Development

To build and test the module:

```bash
cargo build --release
cargo test
```

## Contributing

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/MyFeature`)
3. Commit your changes (`git commit -am 'Add new feature'`)
4. Push to the branch (`git push origin feature/MyFeature`)
5. Create a Pull Request

## License

This project is licensed under the MIT License - see the LICENSE file for details. 