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
        config = CompressionConfig()
        self.assertTrue(config.gpu_enabled)
        self.assertEqual(config.batch_size, 1000)
        self.assertEqual(config.min_gpu_tokens, 1000)
        self.assertTrue(config.simd_enabled)
        self.assertTrue(config.pattern_learning)
    
    def test_invalid_config(self):
        """Test configuration validation."""
        with self.assertRaises(ValueError):
            CompressionConfig(batch_size=0)
        
        with self.assertRaises(ValueError):
            CompressionConfig(min_gpu_tokens=0)

class TestTokenCompressor(unittest.TestCase):
    """Test token compression functionality."""
    
    def setUp(self):
        """Set up test compressor."""
        self.compressor = TokenCompressor()
        self.test_tokens = [1, 2, 3, 4, 5]
    
    def test_compress_decompress(self):
        """Test basic compression and decompression."""
        compressed, stats = self.compressor.compress_tokens(self.test_tokens)
        
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
        self.assertEqual(self.test_tokens, decompressed)
    
    def test_batch_compression(self):
        """Test batch compression and decompression."""
        # Create test batch
        batch = np.array([
            [1, 2, 3],
            [4, 5, 6],
            [7, 8, 9]
        ], dtype=np.uint32)
        
        # Compress batch
        compressed, stats = self.compressor.compress_batch(batch)
        
        # Check compression stats
        self.assertGreater(stats.original_size, 0)
        self.assertGreater(stats.compressed_size, 0)
        self.assertGreater(stats.compression_ratio, 0.0)
        
        # Decompress and verify
        decompressed = self.compressor.decompress_batch(compressed, batch.shape)
        np.testing.assert_array_equal(batch, decompressed)
    
    def test_gpu_config(self):
        """Test GPU configuration."""
        # Force CPU-only
        config = CompressionConfig(
            gpu_enabled=False,
            min_gpu_tokens=999999  # High threshold
        )
        compressor = TokenCompressor(config)
        
        # Compress tokens
        compressed, stats = compressor.compress_tokens(self.test_tokens)
        self.assertFalse(stats.used_gpu)
        
        # Verify data
        decompressed = compressor.decompress_tokens(compressed)
        self.assertEqual(self.test_tokens, decompressed)
    
    def test_invalid_input(self):
        """Test error handling for invalid input."""
        # Empty input
        with self.assertRaises(CompressionError):
            self.compressor.compress_tokens([])
        
        # Invalid batch dimensions
        with self.assertRaises(ValueError):
            self.compressor.compress_batch(np.array([1, 2, 3]))  # 1D array
        
        # Invalid compressed data
        with self.assertRaises(CompressionError):
            self.compressor.decompress_tokens(b"invalid data")
    
    def test_large_batch(self):
        """Test compression of large batches."""
        # Create large batch
        batch = np.random.randint(
            0, 1000,
            size=(100, 50),  # 5000 tokens
            dtype=np.uint32
        )
        
        # Compress with GPU
        config = CompressionConfig(
            gpu_enabled=True,
            min_gpu_tokens=1000
        )
        compressor = TokenCompressor(config)
        
        compressed, stats = compressor.compress_batch(batch)
        self.assertTrue(stats.used_gpu)  # Should use GPU
        
        # Verify decompression
        decompressed = compressor.decompress_batch(compressed, batch.shape)
        np.testing.assert_array_equal(batch, decompressed)

if __name__ == '__main__':
    unittest.main() 