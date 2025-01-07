"""
Performance metrics for PromptVeil.
"""

from dataclasses import dataclass
from typing import Dict, List, Optional
import time
import numpy as np
from collections import deque


@dataclass
class CompressionMetrics:
    """Detailed compression performance metrics."""
    original_size: int
    compressed_size: int
    processing_time_ms: int
    used_gpu: bool
    sequence_length: int
    token_entropy: float
    pattern_matches: int
    gpu_memory_used: Optional[int] = None
    
    @property
    def compression_ratio(self) -> float:
        """Calculate compression ratio."""
        return self.compressed_size / self.original_size
    
    @property
    def throughput(self) -> float:
        """Calculate tokens per second."""
        return self.sequence_length / (self.processing_time_ms / 1000)
    
    @property
    def efficiency_score(self) -> float:
        """Calculate efficiency score (0-1)."""
        # Higher is better
        size_score = 1 - self.compression_ratio
        speed_score = min(1.0, self.throughput / 10000)  # Normalize to 10k tokens/s
        return (size_score + speed_score) / 2


class MetricsTracker:
    """
    Tracks compression performance metrics over time.
    
    Example:
        >>> tracker = MetricsTracker()
        >>> with tracker.track() as metrics:
        ...     result = compressor.compress_tokens(tokens)
        >>> print(tracker.get_statistics())
    """
    
    def __init__(self, window_size: int = 100):
        """
        Initialize tracker.
        
        Args:
            window_size: Number of metrics to keep in rolling window
        """
        self.window_size = window_size
        self.metrics: deque[CompressionMetrics] = deque(maxlen=window_size)
        self.start_time: Optional[float] = None
        self._current_metrics: Optional[Dict] = None
    
    def __enter__(self):
        """Start tracking metrics."""
        self.start_time = time.time()
        self._current_metrics = {}
        return self._current_metrics
    
    def __exit__(self, exc_type, exc_val, exc_tb):
        """Stop tracking and record metrics."""
        if exc_type is None and self._current_metrics:
            processing_time = int((time.time() - self.start_time) * 1000)
            
            # Create metrics object
            metrics = CompressionMetrics(
                original_size=self._current_metrics.get('original_size', 0),
                compressed_size=self._current_metrics.get('compressed_size', 0),
                processing_time_ms=processing_time,
                used_gpu=self._current_metrics.get('used_gpu', False),
                sequence_length=self._current_metrics.get('sequence_length', 0),
                token_entropy=self._current_metrics.get('token_entropy', 0.0),
                pattern_matches=self._current_metrics.get('pattern_matches', 0),
                gpu_memory_used=self._current_metrics.get('gpu_memory_used')
            )
            
            # Add to history
            self.metrics.append(metrics)
        
        self.start_time = None
        self._current_metrics = None
    
    def get_statistics(self) -> Dict[str, float]:
        """Get statistical summary of tracked metrics."""
        if not self.metrics:
            return {}
        
        # Calculate statistics
        ratios = [m.compression_ratio for m in self.metrics]
        throughputs = [m.throughput for m in self.metrics]
        efficiency = [m.efficiency_score for m in self.metrics]
        
        return {
            'compression_ratio': {
                'mean': float(np.mean(ratios)),
                'std': float(np.std(ratios)),
                'min': float(np.min(ratios)),
                'max': float(np.max(ratios))
            },
            'throughput': {
                'mean': float(np.mean(throughputs)),
                'std': float(np.std(throughputs)),
                'min': float(np.min(throughputs)),
                'max': float(np.max(throughputs))
            },
            'efficiency': {
                'mean': float(np.mean(efficiency)),
                'std': float(np.std(efficiency)),
                'min': float(np.min(efficiency)),
                'max': float(np.max(efficiency))
            },
            'gpu_usage': {
                'percent': sum(1 for m in self.metrics if m.used_gpu) / len(self.metrics)
            }
        }
    
    def get_history(self) -> List[CompressionMetrics]:
        """Get full metrics history."""
        return list(self.metrics)
    
    def clear(self):
        """Clear metrics history."""
        self.metrics.clear()


class BatchMetricsTracker(MetricsTracker):
    """
    Tracks metrics for batch compression operations.
    
    Example:
        >>> tracker = BatchMetricsTracker()
        >>> with tracker.track_batch(batch_size=10) as metrics:
        ...     result = compressor.compress_batch(tokens)
        >>> print(tracker.get_batch_statistics())
    """
    
    def __init__(self, window_size: int = 100):
        super().__init__(window_size)
        self.batch_sizes: deque[int] = deque(maxlen=window_size)
    
    def track_batch(self, batch_size: int):
        """Track metrics for a batch operation."""
        self.batch_sizes.append(batch_size)
        return self
    
    def get_batch_statistics(self) -> Dict[str, Dict[str, float]]:
        """Get statistical summary for batch operations."""
        stats = self.get_statistics()
        
        if not self.batch_sizes:
            return stats
        
        # Add batch-specific metrics
        throughput_per_item = [
            m.throughput / size
            for m, size in zip(self.metrics, self.batch_sizes)
        ]
        
        stats['per_item_throughput'] = {
            'mean': float(np.mean(throughput_per_item)),
            'std': float(np.std(throughput_per_item)),
            'min': float(np.min(throughput_per_item)),
            'max': float(np.max(throughput_per_item))
        }
        
        return stats 