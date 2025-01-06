# PromptVeil Python Package

This is the Python package for PromptVeil, providing a high-level interface to the PromptVeil framework.

## Features

- Secure storage of LLM conversations
- Token-aware compression
- Hardware-accelerated encryption
- Distributed storage support

## Installation

```bash
pip install promptveil
```

## Usage

```python
from promptveil import PromptVeil

# Initialize PromptVeil
pv = PromptVeil()

# Store a conversation
conversation_id = pv.store_conversation([
    {"role": "user", "content": "Hello, how are you?"},
    {"role": "assistant", "content": "I'm doing well, thank you for asking!"}
])

# Retrieve a conversation
conversation = pv.get_conversation(conversation_id)

# Search through conversations
results = pv.search("hello")