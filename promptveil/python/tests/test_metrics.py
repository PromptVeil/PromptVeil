"""
Tests for the metrics module.
"""

import unittest
import time
import numpy as np
from promptveil.metrics import (
    CompressionMetrics,
    MetricsTracker,
    BatchMetricsTracker
)

class TestCompressionMetrics(unittest.TestCase):
    """Test compression metrics calculations."""
    
    def setUp(self):
        """Set up test metrics."""
        self.metrics = CompressionMetrics(
            original_size=1000,
            compressed_size=500,
            processing_time_ms=100,
            used_gpu=True,
            sequence_length=200,
            token_entropy=4.5,
            pattern_matches=50,
            gpu_memory_used=1024
        )
    
    def test_compression_ratio(self):
        """Test compression ratio calculation."""
        self.assertEqual(self.metrics.compression_ratio, 0.5)
    
    def test_throughput(self):
        """Test throughput calculation."""
        expected = 200 / (100 / 1000)  # sequence_length / (processing_time_ms / 1000)
        self.assertEqual(self.metrics.throughput, expected)
    
    def test_efficiency_score(self):
        """Test efficiency score calculation."""
        size_score = 1 - 0.5  # 1 - compression_ratio
        speed_score = min(1.0, self.metrics.throughput / 10000)
        expected = (size_score + speed_score) / 2
        self.assertEqual(self.metrics.efficiency_score, expected)

class TestMetricsTracker(unittest.TestCase):
    """Test metrics tracking functionality."""
    
    def setUp(self):
        """Set up test tracker."""
        self.tracker = MetricsTracker(window_size=3)
    
    def test_metrics_tracking(self):
        """Test basic metrics tracking."""
        with self.tracker.track() as metrics:
            # Simulate compression operation
            time.sleep(0.1)
            metrics.update({
                'original_size': 1000,
                'compressed_size': 500,
                'used_gpu': True,
                'sequence_length': 200,
                'token_entropy': 4.5,
                'pattern_matches': 50
            })
        
        # Verify recorded metrics
        history = self.tracker.get_history()
        self.assertEqual(len(history), 1)
        
        metrics = history[0]
        self.assertEqual(metrics.original_size, 1000)
        self.assertEqual(metrics.compressed_size, 500)
        self.assertTrue(metrics.used_gpu)
        self.assertGreater(metrics.processing_time_ms, 0)
    
    def test_window_size(self):
        """Test rolling window behavior."""
        # Add more metrics than window size
        for i in range(5):
            with self.tracker.track() as metrics:
                metrics.update({
                    'original_size': 1000 + i,
                    'compressed_size': 500 + i,
                    'used_gpu': True,
                    'sequence_length': 200,
                    'token_entropy': 4.5,
                    'pattern_matches': 50
                })
        
        # Should only keep last 3
        history = self.tracker.get_history()
        self.assertEqual(len(history), 3)
        self.assertEqual(history[-1].original_size, 1004)
    
    def test_statistics(self):
        """Test statistical calculations."""
        # Add test metrics
        sizes = [1000, 2000, 3000]
        for size in sizes:
            with self.tracker.track() as metrics:
                metrics.update({
                    'original_size': size,
                    'compressed_size': size // 2,
                    'used_gpu': True,
                    'sequence_length': 200,
                    'token_entropy': 4.5,
                    'pattern_matches': 50
                })
        
        stats = self.tracker.get_statistics()
        
        # Check compression ratio stats
        ratio_stats = stats['compression_ratio']
        self.assertEqual(ratio_stats['mean'], 0.5)
        self.assertEqual(ratio_stats['min'], 0.5)
        self.assertEqual(ratio_stats['max'], 0.5)
        
        # Check GPU usage
        self.assertEqual(stats['gpu_usage']['percent'], 1.0)

class TestBatchMetricsTracker(unittest.TestCase):
    """Test batch metrics tracking functionality."""
    
    def setUp(self):
        """Set up test tracker."""
        self.tracker = BatchMetricsTracker(window_size=3)
    
    def test_batch_tracking(self):
        """Test batch metrics tracking."""
        batch_sizes = [10, 20, 30]
        
        for batch_size in batch_sizes:
            with self.tracker.track_batch(batch_size) as metrics:
                # Simulate batch compression
                time.sleep(0.1)
                metrics.update({
                    'original_size': batch_size * 100,
                    'compressed_size': batch_size * 50,
                    'used_gpu': True,
                    'sequence_length': batch_size * 20,
                    'token_entropy': 4.5,
                    'pattern_matches': batch_size * 5
                })
        
        # Check batch statistics
        stats = self.tracker.get_batch_statistics()
        
        # Should have per-item throughput
        self.assertIn('per_item_throughput', stats)
        
        # Check that larger batches have better per-item throughput
        throughputs = [
            m.throughput / size
            for m, size in zip(self.tracker.get_history(), batch_sizes)
        ]
        self.assertTrue(all(t1 >= t2 for t1, t2 in zip(throughputs[1:], throughputs[:-1])))
    
    def test_batch_window(self):
        """Test batch metrics window behavior."""
        # Add more batches than window size
        for i in range(5):
            with self.tracker.track_batch(i + 1) as metrics:
                metrics.update({
                    'original_size': 1000,
                    'compressed_size': 500,
                    'used_gpu': True,
                    'sequence_length': 200,
                    'token_entropy': 4.5,
                    'pattern_matches': 50
                })
        
        # Should only keep last 3 batch sizes
        self.assertEqual(len(self.tracker.batch_sizes), 3)
        self.assertEqual(list(self.tracker.batch_sizes), [3, 4, 5])

if __name__ == '__main__':
    unittest.main() 