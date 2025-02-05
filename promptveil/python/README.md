# PromptVeil Python

Python bindings and high-level API for PromptVeil.

## Features

- Async API for all operations
- Type-safe data models using Pydantic
- Hardware-accelerated operations through Rust
- Comprehensive error handling
- Efficient memory management

## Installation

```bash
pip install promptveil
```

## Usage

```python
from promptveil import PromptVeil, Conversation
from pathlib import Path

# Initialize PromptVeil
pv = PromptVeil(base_path="data")

# Create a conversation
conversation = Conversation(id="conv1")
conversation.add_message("user", "What is quantum computing?")
conversation.add_message("assistant", "Quantum computing leverages quantum phenomena...")

# Add conversation asynchronously
async with pv:
    await pv.add_conversation(conversation)

    # Search by text
    results = await pv.search_text("quantum computing")
    for result in results:
        print(f"Found: {result.conversation_id} (score: {result.score})")
        print(f"Snippet: {result.snippet}")

    # Search by similarity
    similar = await pv.search_similar("How do quantum computers work?")
    for match in similar:
        print(f"Similar: {match.conversation_id} (similarity: {match.similarity})")
```

## Architecture

The module is structured as follows:

```
promptveil/
├── _core/           # Rust bindings
├── models/          # Data models
│   ├── conversation.py
│   ├── config.py
│   └── exceptions.py
├── utils/           # Utility functions
│   └── helpers.py
└── __init__.py     # Main interface
```

## Configuration

```python
from promptveil import PromptVeil
from promptveil.models import SecurityConfig, IndexConfig

# Configure with defaults
pv = PromptVeil("data")

# Custom configuration
pv = PromptVeil(
    "data",
    security=SecurityConfig(
        key_rotation_days=7,
        encryption_enabled=True,
        hardware_acceleration=True
    ),
    index=IndexConfig(
        vector_dim=768,
        max_elements=1_000_000
    )
)
```

## Error Handling

```python
from promptveil.models import SecurityError, NotFoundError

try:
    async with pv:
        await pv.add_conversation(conversation)
except SecurityError as e:
    print(f"Security error: {e}")
except NotFoundError as e:
    print(f"Not found: {e}")
```

## Development

1. Clone the repository
2. Install development dependencies:
   ```bash
   pip install -e ".[dev]"
   ```
3. Run tests:
   ```bash
   pytest tests/
   ```

## Contributing

1. Fork the repository
2. Create your feature branch
3. Commit your changes
4. Push to the branch
5. Create a Pull Request

## License

MIT License - See LICENSE file for details