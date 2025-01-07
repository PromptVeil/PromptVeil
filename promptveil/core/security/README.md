# PromptVeil Core

Core functionality for PromptVeil, providing token compression and encryption capabilities.

## Features

- Token compression using Julia optimizations
- Batch compression for large token sequences
- Secure encryption and decryption
- Python bindings for easy integration
- Cross-platform support (Windows, Linux, macOS)

## Requirements

- Python 3.7 or later
- Julia 1.6 or later
- Rust toolchain (cargo, rustc)
- C compiler (gcc, clang, or MSVC)

## Installation

### From PyPI

```bash
pip install promptveil-core
```

### From Source

1. Install Julia and ensure it's in your PATH
2. Install Rust toolchain
3. Clone the repository
4. Build and install:

```bash
# Windows
python -m pip install maturin
maturin develop

# Linux/macOS
python3 -m pip install maturin
maturin develop
```

## Usage

```python
from promptveil_core import (
    compress_tokens, 
    decompress_tokens, 
    compress_batch, 
    decompress_batch
)

# Single sequence compression
compressed = compress_tokens(data)
decompressed = decompress_tokens(compressed)

# Batch compression
chunk_size = 16
compressed = compress_batch(data, chunk_size)
rows = len(data) // chunk_size
cols = chunk_size // 4
decompressed = decompress_batch(compressed, rows, cols)

# Encryption
from promptveil_core import encrypt, decrypt, generate_key

key = generate_key()
encrypted = encrypt(data, key)
decrypted = decrypt(encrypted, key)
```

## Building from Source

### Windows

1. Install Visual Studio Build Tools or Visual Studio Community
2. Install LLVM (optional, for better optimizations)
3. Run:
```bash
set LIBCLANG_PATH="C:\Program Files\LLVM\bin"
maturin develop
```

### Linux

1. Install build dependencies:
```bash
# Ubuntu/Debian
sudo apt-get install build-essential

# Fedora
sudo dnf groupinstall "Development Tools"
```

2. Build and install:
```bash
maturin develop
```

### macOS

1. Install Xcode Command Line Tools:
```bash
xcode-select --install
```

2. Build and install:
```bash
maturin develop
```

## Contributing

Contributions are welcome! Please read our contributing guidelines for details. 