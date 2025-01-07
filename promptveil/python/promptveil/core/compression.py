"""Compression functionality using Rust and Julia cores."""

from typing import Union, BinaryIO
from pathlib import Path
import promptveil_core as rust_core
try:
    from julia import Tokenizer
    JULIA_AVAILABLE = True
except ImportError:
    JULIA_AVAILABLE = False


def compress(data: Union[str, bytes, Path], compression_level: int = 9) -> bytes:
    """
    Compress data using the hybrid Rust/Julia compression engine.
    
    Args:
        data: Input data to compress (string, bytes or file path)
        compression_level: Compression level (1-9)
        
    Returns:
        Compressed binary data
    """
    # Convert input to bytes
    if isinstance(data, str):
        input_bytes = data.encode('utf-8')
    elif isinstance(data, Path):
        input_bytes = data.read_bytes()
    else:
        input_bytes = data

    # Pre-process with Julia if available
    if JULIA_AVAILABLE:
        # Convert bytes to tokens (assuming UTF-8 for now)
        tokens = [x for x in input_bytes]
        # Optimize tokens using Julia
        optimized_tokens = Tokenizer.optimize_tokens(tokens)
        input_bytes = bytes(optimized_tokens)

    # Use Rust core for compression
    return rust_core.compress_tokens(input_bytes, compression_level)


def decompress(data: Union[bytes, BinaryIO]) -> bytes:
    """
    Decompress data using the hybrid Rust/Julia decompression engine.
    
    Args:
        data: Compressed data (bytes or file-like object)
        
    Returns:
        Decompressed data as bytes
    """
    # Handle file-like objects
    if hasattr(data, 'read'):
        input_bytes = data.read()
    else:
        input_bytes = data

    # Use Rust core for decompression
    decompressed = rust_core.decompress_tokens(input_bytes)

    # Post-process with Julia if available
    if JULIA_AVAILABLE:
        tokens = [x for x in decompressed]
        # Reverse optimization if needed
        processed_tokens = Tokenizer.optimize_tokens(tokens)  # In this case, same function works for reverse
        return bytes(processed_tokens)

    return decompressed 