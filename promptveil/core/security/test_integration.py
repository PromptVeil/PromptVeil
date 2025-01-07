import numpy as np
from promptveil_core import TokenCompressor

def test_compression():
    # Create some test data
    data = np.array([1, 2, 3, 4, 5, 6, 7, 8], dtype=np.uint32).tobytes()
    chunk_size = 16  # 4 bytes per uint32, so this is 4 tokens

    # Create compressor
    compressor = TokenCompressor(9)
    
    # Test batch compression
    compressed = compressor.compress_batch(data, chunk_size)
    print("Compressed data:", len(compressed), "bytes")

if __name__ == "__main__":
    test_compression()