import numpy as np
import promptveil_core

"""
Compression Test Results:

1. Basic Test (Hello World):
   - Previous implementation (Rust + zstd): 57 bytes
   - Current implementation (Rust + zstd + bincode serialization): 47 bytes
   - Improvement: ~17.5% smaller
   - Possible reason: Optimization in metadata serialization with bincode

2. Compression Levels:
   - Same result for levels 1 and 9 with highly repetitive data
   - Demonstrates that the algorithm achieves optimal compression even at a low level
   - Compression rate: 276.60x (from 13000 to 47 bytes)

Next Steps:
- Replace zstd with optimized compression in Julia
- Expectation of better compression rates using:
  * GPU Processing
  * Pattern optimization with SVD
  * Quantization of similar tokens
  * SIMD operations
"""

def to_bytes(data):
    """Convert list of integers to bytes"""
    return bytes(data)

def test_compression_comparison():
    """Test compression using the Rust module"""
    # Test data
    data = b"Hello, World!" * 1000
    
    # Compression using Rust
    compressed = to_bytes(promptveil_core.compress_tokens(data, 9))
    decompressed = to_bytes(promptveil_core.decompress_tokens(compressed))
    
    print("\nCompression Results:")
    print(f"Original size: {len(data)} bytes")
    print(f"Compressed size: {len(compressed)} bytes")
    print(f"Compression rate: {len(data)/len(compressed):.2f}x")
    print(f"Data equal after decompress: {data == decompressed}")
    
    assert data == decompressed, "Data is not equal after decompression"
    assert len(compressed) < len(data), "Compression did not reduce data size"
    
    return len(data), len(compressed)

def test_compression_levels():
    """Test different compression levels"""
    data = b"Hello, World!" * 1000
    
    # Compression with low and high level
    compressed_low = to_bytes(promptveil_core.compress_tokens(data, 1))
    compressed_high = to_bytes(promptveil_core.compress_tokens(data, 9))
    
    print("\nCompression Levels Comparison:")
    print(f"Original size: {len(data)} bytes")
    print(f"Compression level 1: {len(compressed_low)} bytes")
    print(f"Compression level 9: {len(compressed_high)} bytes")
    print(f"Relative improvement: {len(compressed_low)/len(compressed_high):.2f}x")
    
    assert len(compressed_high) <= len(compressed_low), "Higher level should provide better compression"
    
    return len(compressed_low), len(compressed_high)

def test_complex_data():
    """Test with more complex and varied data"""
    # Create a sequence of more complex data
    # Simulate an LLM dialogue with different patterns
    dialog = [
        b"User: Can you help me understand quantum computing?",
        b"Assistant: Quantum computing is a type of computing that uses quantum phenomena such as superposition and entanglement.",
        b"User: What is superposition?",
        b"Assistant: Superposition is a fundamental principle of quantum mechanics where a quantum system can exist in multiple states simultaneously.",
        b"User: And entanglement?",
        b"Assistant: Quantum entanglement occurs when pairs or groups of particles are generated, interact, or share spatial proximity in a way such that the quantum state of each particle cannot be described independently.",
    ]
    
    # Repeat the dialogue with some variations
    data = b"\n".join([
        line + b" " + str(i).encode()
        for i in range(100)
        for line in dialog
    ])
    
    # Compression using different levels
    compressed_low = to_bytes(promptveil_core.compress_tokens(data, 1))
    compressed_high = to_bytes(promptveil_core.compress_tokens(data, 9))
    decompressed = to_bytes(promptveil_core.decompress_tokens(compressed_high))
    
    print("\nTest with Complex Data:")
    print(f"Original size: {len(data)} bytes")
    print(f"Compression level 1: {len(compressed_low)} bytes")
    print(f"Compression level 9: {len(compressed_high)} bytes")
    print(f"Compression rate (level 9): {len(data)/len(compressed_high):.2f}x")
    print(f"Improvement level 9 vs level 1: {len(compressed_low)/len(compressed_high):.2f}x")
    print(f"Data equal after decompress: {data == decompressed}")
    
    assert data == decompressed, "Data is not equal after decompression"
    return len(data), len(compressed_high)

def test_compression_empty():
    """Test compression of empty data"""
    data = b""
    compressed = to_bytes(promptveil_core.compress_tokens(data, 9))
    decompressed = to_bytes(promptveil_core.decompress_tokens(compressed))
    
    print("\nTest with Empty Data:")
    print(f"Original size: {len(data)} bytes")
    print(f"Compressed size: {len(compressed)} bytes")
    print(f"Data equal after decompress: {data == decompressed}")
    
    assert data == decompressed, "Empty data is not equal after decompression"
    
    return len(data), len(compressed)

if __name__ == "__main__":
    print("Running compression tests...")
    orig_size, comp_size = test_compression_comparison()
    low_size, high_size = test_compression_levels()
    complex_orig, complex_comp = test_complex_data()
    empty_orig, empty_comp = test_compression_empty() 