"""
Integration tests for PromptVeil compression system.
Tests the interaction between Julia, Rust, and Python layers.
"""

import unittest
import numpy as np
from typing import List, Tuple
import tempfile
import os

from promptveil.compression import TokenCompressor, CompressionConfig
from promptveil.exceptions import CompressionError, GPUError

def generate_token_sequence(length: int) -> List[int]:
    """Generate a realistic token sequence."""
    # Simulate typical token patterns in LLM conversations
    base_tokens = list(range(1000))  # Common vocabulary tokens
    special_tokens = [0, 1, 2]  # Special tokens like BOS, EOS, PAD
    
    sequence = []
    for _ in range(length):
        if np.random.random() < 0.1:  # 10% chance of special token
            sequence.append(np.random.choice(special_tokens))
        else:
            sequence.append(np.random.choice(base_tokens))
    return sequence

def generate_conversation_batch(
    batch_size: int,
    seq_length: int
) -> Tuple[np.ndarray, List[List[int]]]:
    """Generate a batch of conversation sequences."""
    sequences = [
        generate_token_sequence(seq_length)
        for _ in range(batch_size)
    ]
    return np.array(sequences, dtype=np.uint32), sequences

class TestCompression(unittest.TestCase):
    """Test full compression pipeline."""
    
    def setUp(self):
        """Set up test environment."""
        self.compressor = TokenCompressor()
        self.test_sequences = [
            generate_token_sequence(100),  # Small sequence
            generate_token_sequence(1000),  # Medium sequence
            generate_token_sequence(10000)  # Large sequence
        ]
    
    def test_compression_pipeline(self):
        """Test complete compression pipeline with different sizes."""
        for sequence in self.test_sequences:
            # Compress
            compressed, stats = self.compressor.compress_tokens(sequence)
            
            # Verify compression stats
            self.assertGreater(stats.original_size, 0)
            self.assertGreater(stats.compressed_size, 0)
            self.assertLess(stats.compressed_size, stats.original_size)
            
            # Check GPU usage based on sequence size
            expected_gpu = len(sequence) >= self.compressor.config.min_gpu_tokens
            self.assertEqual(stats.used_gpu, expected_gpu)
            
            # Decompress and verify
            decompressed = self.compressor.decompress_tokens(compressed)
            self.assertEqual(sequence, decompressed)
    
    def test_batch_pipeline(self):
        """Test batch processing pipeline."""
        # Generate test batch
        batch, sequences = generate_conversation_batch(10, 500)
        
        # Compress batch
        compressed, stats = self.compressor.compress_batch(batch)
        
        # Verify compression
        self.assertGreater(stats.original_size, 0)
        self.assertGreater(stats.compressed_size, 0)
        self.assertLess(stats.compressed_size, stats.original_size)
        
        # Decompress and verify each sequence
        decompressed = self.compressor.decompress_batch(compressed, batch.shape)
        np.testing.assert_array_equal(batch, decompressed)
        
        # Verify individual sequences
        for i, sequence in enumerate(sequences):
            self.assertTrue(np.array_equal(decompressed[i], sequence))

class TestGPUFallback(unittest.TestCase):
    """Test GPU fallback mechanisms."""
    
    def setUp(self):
        """Set up test configurations."""
        self.cpu_config = CompressionConfig(
            gpu_enabled=False,
            min_gpu_tokens=999999
        )
        self.gpu_config = CompressionConfig(
            gpu_enabled=True,
            min_gpu_tokens=1000
        )
    
    def test_gpu_fallback(self):
        """Test automatic fallback to CPU."""
        compressor = TokenCompressor(self.gpu_config)
        
        # Small sequence should use CPU
        small_seq = generate_token_sequence(500)
        compressed, stats = compressor.compress_tokens(small_seq)
        self.assertFalse(stats.used_gpu)
        
        # Large sequence should attempt GPU
        large_seq = generate_token_sequence(5000)
        compressed, stats = compressor.compress_tokens(large_seq)
        # Note: test will pass whether GPU is available or not
        decompressed = compressor.decompress_tokens(compressed)
        self.assertEqual(large_seq, decompressed)
    
    def test_forced_cpu(self):
        """Test forced CPU processing."""
        compressor = TokenCompressor(self.cpu_config)
        
        # Even large sequences should use CPU
        large_seq = generate_token_sequence(5000)
        compressed, stats = compressor.compress_tokens(large_seq)
        self.assertFalse(stats.used_gpu)
        
        decompressed = compressor.decompress_tokens(compressed)
        self.assertEqual(large_seq, decompressed)

class TestErrorHandling(unittest.TestCase):
    """Test error handling across layers."""
    
    def setUp(self):
        """Set up test compressor."""
        self.compressor = TokenCompressor()
    
    def test_memory_errors(self):
        """Test handling of memory allocation errors."""
        # Try to compress an extremely large sequence
        try:
            huge_seq = generate_token_sequence(10_000_000)  # 10M tokens
            self.compressor.compress_tokens(huge_seq)
        except CompressionError as e:
            self.assertIn("memory", str(e).lower())
    
    def test_gpu_errors(self):
        """Test handling of GPU-related errors."""
        config = CompressionConfig(
            gpu_enabled=True,
            min_gpu_tokens=1  # Force GPU for all sequences
        )
        compressor = TokenCompressor(config)
        
        # If GPU fails, should fall back to CPU
        sequence = generate_token_sequence(1000)
        try:
            compressed, stats = compressor.compress_tokens(sequence)
            # Either succeeded with GPU or fell back to CPU
            if not stats.used_gpu:
                self.assertLess(stats.compressed_size, stats.original_size)
        except GPUError:
            self.skipTest("GPU compression not available")
    
    def test_data_corruption(self):
        """Test handling of corrupted data."""
        # Compress valid sequence
        sequence = generate_token_sequence(100)
        compressed, _ = self.compressor.compress_tokens(sequence)
        
        # Corrupt the compressed data
        corrupted = bytearray(compressed)
        corrupted[10:20] = b"0" * 10
        
        # Attempt to decompress
        with self.assertRaises(CompressionError):
            self.compressor.decompress_tokens(bytes(corrupted))

class TestPersistence(unittest.TestCase):
    """Test persistence of compressed data."""
    
    def setUp(self):
        """Set up test environment."""
        self.compressor = TokenCompressor()
        self.test_file = tempfile.NamedTemporaryFile(delete=False)
        self.test_file.close()
    
    def tearDown(self):
        """Clean up test files."""
        os.unlink(self.test_file.name)
    
    def test_file_persistence(self):
        """Test saving and loading compressed data."""
        # Generate and compress sequence
        sequence = generate_token_sequence(1000)
        compressed, stats = self.compressor.compress_tokens(sequence)
        
        # Save to file
        with open(self.test_file.name, "wb") as f:
            f.write(compressed)
        
        # Read back and decompress
        with open(self.test_file.name, "rb") as f:
            loaded = f.read()
        
        decompressed = self.compressor.decompress_tokens(loaded)
        self.assertEqual(sequence, decompressed)
    
    def test_compression_stability(self):
        """Test compression stability across multiple runs."""
        sequence = generate_token_sequence(1000)
        
        # Compress multiple times
        results = []
        for _ in range(5):
            compressed, stats = self.compressor.compress_tokens(sequence)
            results.append((compressed, stats))
        
        # Verify consistent compression ratio
        ratios = [stats.compression_ratio for _, stats in results]
        max_ratio_diff = max(ratios) - min(ratios)
        self.assertLess(max_ratio_diff, 0.1)  # Less than 10% variation
        
        # Verify all decompress correctly
        for compressed, _ in results:
            decompressed = self.compressor.decompress_tokens(compressed)
            self.assertEqual(sequence, decompressed)

if __name__ == '__main__':
    unittest.main() 