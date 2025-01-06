"""
Compression module for PromptVeil.
Provides high-level interface for token compression with GPU acceleration.
"""

from dataclasses import dataclass
from typing import List, Optional, Tuple
import numpy as np
from numpy.typing import NDArray

from .exceptions import CompressionError


@dataclass
class CompressionConfig:
    """Configuration for token compression."""
    gpu_enabled: bool = True
    batch_size: int = 1000
    min_gpu_tokens: int = 1000
    simd_enabled: bool = True
    pattern_learning: bool = True

    def __post_init__(self):
        """Validate configuration values."""
        if self.batch_size < 1:
            raise ValueError("batch_size must be positive")
        if self.min_gpu_tokens < 1:
            raise ValueError("min_gpu_tokens must be positive")


@dataclass
class CompressionStats:
    """Statistics from compression operations."""
    original_size: int
    compressed_size: int
    compression_ratio: float
    processing_time_ms: int
    used_gpu: bool

    @property
    def size_reduction_percent(self) -> float:
        """Calculate size reduction as a percentage."""
        return (1.0 - self.compression_ratio) * 100


class TokenCompressor:
    """
    High-level interface for token compression.
    
    Example:
        >>> compressor = TokenCompressor()
        >>> tokens = [1, 2, 3, 4, 5]
        >>> compressed, stats = compressor.compress_tokens(tokens)
        >>> decompressed = compressor.decompress_tokens(compressed)
        >>> assert tokens == decompressed
    """
    
    def __init__(self, config: Optional[CompressionConfig] = None):
        """
        Initialize compressor with optional configuration.
        
        Args:
            config: Compression configuration
        """
        self.config = config or CompressionConfig()
    
    def compress_tokens(
        self,
        tokens: List[int]
    ) -> Tuple[bytes, CompressionStats]:
        """
        Compress a sequence of tokens.
        
        Args:
            tokens: List of token IDs to compress
            
        Returns:
            Tuple of (compressed bytes, compression statistics)
            
        Raises:
            CompressionError: If compression fails
        """
        try:
            # Convert to numpy array
            tokens_array = np.array(tokens, dtype=np.uint32)
            
            # Call Rust compression
            from ._promptveil_core import compress_tokens
            compressed, stats = compress_tokens(
                tokens_array,
                self.config.gpu_enabled,
                self.config.batch_size,
                self.config.min_gpu_tokens,
                self.config.simd_enabled,
                self.config.pattern_learning
            )
            
            return compressed, CompressionStats(
                original_size=stats['original_size'],
                compressed_size=stats['compressed_size'],
                compression_ratio=stats['compression_ratio'],
                processing_time_ms=stats['processing_time_ms'],
                used_gpu=stats['used_gpu']
            )
            
        except Exception as e:
            raise CompressionError(f"Token compression failed: {str(e)}")
    
    def decompress_tokens(self, data: bytes) -> List[int]:
        """
        Decompress tokens from bytes.
        
        Args:
            data: Compressed token data
            
        Returns:
            List of decompressed token IDs
            
        Raises:
            CompressionError: If decompression fails
        """
        try:
            # Call Rust decompression
            from ._promptveil_core import decompress_tokens
            tokens = decompress_tokens(data)
            return tokens.tolist()
            
        except Exception as e:
            raise CompressionError(f"Token decompression failed: {str(e)}")
    
    def compress_batch(
        self,
        tokens: NDArray[np.uint32]
    ) -> Tuple[NDArray[np.uint32], CompressionStats]:
        """
        Compress a batch of token sequences.
        
        Args:
            tokens: 2D array of token IDs (shape: [batch_size, sequence_length])
            
        Returns:
            Tuple of (compressed tokens array, compression statistics)
            
        Raises:
            CompressionError: If compression fails
            ValueError: If input dimensions are invalid
        """
        try:
            if tokens.ndim != 2:
                raise ValueError("Input must be a 2D array")
                
            # Call Rust batch compression
            from ._promptveil_core import compress_batch
            compressed, stats = compress_batch(
                tokens,
                self.config.gpu_enabled,
                self.config.batch_size,
                self.config.min_gpu_tokens,
                self.config.simd_enabled,
                self.config.pattern_learning
            )
            
            return compressed, CompressionStats(
                original_size=stats['original_size'],
                compressed_size=stats['compressed_size'],
                compression_ratio=stats['compression_ratio'],
                processing_time_ms=stats['processing_time_ms'],
                used_gpu=stats['used_gpu']
            )
            
        except Exception as e:
            raise CompressionError(f"Batch compression failed: {str(e)}")
    
    def decompress_batch(
        self,
        tokens: NDArray[np.uint32],
        original_shape: Tuple[int, int]
    ) -> NDArray[np.uint32]:
        """
        Decompress a batch of token sequences.
        
        Args:
            tokens: Compressed tokens array
            original_shape: Shape of the original uncompressed array (rows, cols)
            
        Returns:
            Decompressed tokens array
            
        Raises:
            CompressionError: If decompression fails
        """
        try:
            # Call Rust batch decompression
            from ._promptveil_core import decompress_batch
            return decompress_batch(tokens, original_shape[0], original_shape[1])
            
        except Exception as e:
            raise CompressionError(f"Batch decompression failed: {str(e)}") 