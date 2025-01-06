"""
Tests for the compression module.
"""

import unittest
import numpy as np
from promptveil.compression import (
    TokenCompressor,
    CompressionConfig,
    CompressionError
)

class TestCompressionConfig(unittest.TestCase):
    """Test compression configuration."""
    
    def test_default_config(self):
        """Test default configuration values."""
        print("\nTesting default configuration...")
        config = CompressionConfig()
        self.assertTrue(config.gpu_enabled)
        self.assertEqual(config.batch_size, 1000)
        self.assertEqual(config.min_gpu_tokens, 1000)
        self.assertTrue(config.simd_enabled)
        self.assertTrue(config.pattern_learning)
        print("Default configuration test passed!")
    
    def test_invalid_config(self):
        """Test configuration validation."""
        print("\nTesting invalid configurations...")
        with self.assertRaises(ValueError):
            CompressionConfig(batch_size=0)
        
        with self.assertRaises(ValueError):
            CompressionConfig(min_gpu_tokens=0)
        print("Invalid configuration tests passed!")

class TestTokenCompressor(unittest.TestCase):
    """Test token compression functionality."""
    
    def setUp(self):
        """Set up test compressor."""
        self.compressor = TokenCompressor()
        self.test_tokens = [1, 2, 3, 4, 5]
    
    def test_compress_decompress(self):
        """Test basic compression and decompression."""
        print("\nTesting basic compression/decompression...")
        print(f"Original tokens: {self.test_tokens}")
        
        compressed, stats = self.compressor.compress_tokens(self.test_tokens)
        print(f"Compression stats: {stats}")
        
        # Check compression stats
        self.assertGreater(stats.original_size, 0)
        self.assertGreater(stats.compressed_size, 0)
        self.assertGreater(stats.compression_ratio, 0.0)
        self.assertGreaterEqual(stats.processing_time_ms, 0)
        self.assertIsInstance(stats.used_gpu, bool)
        
        # Check size reduction
        self.assertGreaterEqual(stats.size_reduction_percent, 0.0)
        self.assertLessEqual(stats.size_reduction_percent, 100.0)
        
        # Check decompression
        decompressed = self.compressor.decompress_tokens(compressed)
        print(f"Decompressed tokens: {decompressed}")
        self.assertEqual(self.test_tokens, decompressed)
        print("Basic compression/decompression test passed!")
    
    def test_batch_compression(self):
        """Test batch compression and decompression."""
        print("\nTesting batch compression/decompression...")
        
        # Create a larger test batch
        batch = np.array([
            [1, 2, 3, 4],
            [5, 6, 7, 8],
            [9, 10, 11, 12],
            [13, 14, 15, 16]
        ], dtype=np.uint32)
        
        print(f"Original batch shape: {batch.shape}")
        print("Original batch:")
        print(batch)
        
        compressor = TokenCompressor()
        print("\nAttempting compression...")
        
        try:
            compressed, stats = compressor.compress_batch(batch)
            print("\nCompression successful!")
            print(f"Compression stats: {stats}")
            
            print("\nAttempting decompression...")
            decompressed = compressor.decompress_batch(compressed, batch.shape)
            print("Decompression successful!")
            
            print("\nVerifying results...")
            np.testing.assert_array_equal(batch, decompressed)
            print("Batch compression/decompression test passed!")
            
        except Exception as e:
            print(f"\nError during compression: {type(e).__name__}: {str(e)}")
            import traceback
            print(f"Traceback:\n{traceback.format_exc()}")
            raise
    
    def test_gpu_config(self):
        """Test GPU configuration."""
        print("\nTesting GPU configuration...")
        # Force CPU-only
        config = CompressionConfig(
            gpu_enabled=False,
            min_gpu_tokens=999999  # High threshold
        )
        compressor = TokenCompressor(config)
        
        # Compress tokens
        compressed, stats = compressor.compress_tokens(self.test_tokens)
        print(f"Compression stats (CPU-only): {stats}")
        self.assertFalse(stats.used_gpu)
        
        # Verify data
        decompressed = compressor.decompress_tokens(compressed)
        self.assertEqual(self.test_tokens, decompressed)
        print("GPU configuration test passed!")
    
    def test_invalid_input(self):
        """Test error handling for invalid input."""
        print("\nTesting invalid inputs...")
        # Empty input
        with self.assertRaises(CompressionError):
            self.compressor.compress_tokens([])
        
        # Invalid batch dimensions
        with self.assertRaises(ValueError):
            self.compressor.compress_batch(np.array([1, 2, 3]))  # 1D array
        
        # Invalid compressed data
        with self.assertRaises(CompressionError):
            self.compressor.decompress_tokens(b"invalid data")
        print("Invalid input tests passed!")
    
    def test_large_batch(self):
        """Test compression of large batches."""
        print("\nTesting large batch compression...")
        # Create large batch
        batch = np.random.randint(
            0, 1000,
            size=(100, 50),  # 5000 tokens
            dtype=np.uint32
        )
        print(f"Large batch shape: {batch.shape}")
        
        # Compress with GPU
        config = CompressionConfig(
            gpu_enabled=True,
            min_gpu_tokens=1000
        )
        compressor = TokenCompressor(config)
        
        compressed, stats = compressor.compress_batch(batch)
        print(f"Large batch compression stats: {stats}")
        self.assertTrue(stats.used_gpu)  # Should use GPU
        
        # Verify decompression
        decompressed = compressor.decompress_batch(compressed, batch.shape)
        np.testing.assert_array_equal(batch, decompressed)
        print("Large batch compression test passed!")

if __name__ == '__main__':
    # Run tests with more verbose output
    unittest.main(verbosity=2) 