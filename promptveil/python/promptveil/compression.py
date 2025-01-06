"""
Compression module for PromptVeil.
Provides high-level interface for token compression with GPU acceleration.
"""

from dataclasses import dataclass
from typing import List, Optional, Tuple
import numpy as np
from numpy.typing import NDArray

from .exceptions import CompressionError
import promptveil_core


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
            # Convert to list of uint32
            tokens_list = [int(t) for t in tokens]
            
            # Call Julia compression via FFI
            compressed = promptveil_core.julia_optimize_tokens_config(
                tokens_list,
                len(tokens_list),
                self.config.gpu_enabled,
                self.config.simd_enabled,
                self.config.pattern_learning
            )
            
            # Convert to bytes
            compressed_bytes = np.array(compressed, dtype=np.uint32).tobytes()
            
            # Calculate stats
            stats = CompressionStats(
                original_size=len(tokens) * 4,
                compressed_size=len(compressed_bytes),
                compression_ratio=len(compressed_bytes) / (len(tokens) * 4),
                processing_time_ms=0,  # TODO: Add timing
                used_gpu=self.config.gpu_enabled
            )
            
            return compressed_bytes, stats
            
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
            # Convert bytes to list of uint32
            tokens_array = np.frombuffer(data, dtype=np.uint32)
            tokens_list = tokens_array.tolist()
            
            # Call Julia decompression via FFI
            decompressed = promptveil_core.julia_decompress_batch(
                [tokens_list],  # Convert to 2D list
                1,  # Single sequence
                len(tokens_list)
            )
            
            return decompressed[0]  # Return first sequence
            
        except Exception as e:
            raise CompressionError(f"Token decompression failed: {str(e)}")
    
    def compress_batch(
        self,
        tokens: NDArray[np.uint32]
    ) -> Tuple[bytes, CompressionStats]:
        """
        Compress a batch of token sequences.
        
        Args:
            tokens: 2D array of token IDs (shape: [batch_size, sequence_length])
            
        Returns:
            Tuple of (compressed bytes, compression statistics)
            
        Raises:
            CompressionError: If compression fails
            ValueError: If input dimensions are invalid
        """
        try:
            if tokens.ndim != 2:
                raise ValueError("Input must be a 2D array")
                
            # Convert to list of lists
            print(f"DEBUG: Converting array of shape {tokens.shape} to list")
            tokens_list = tokens.tolist()
            print(f"DEBUG: Converted to list: {tokens_list}")
            print(f"DEBUG: Data types in first row: {[type(x) for x in tokens_list[0]]}")
            
            print(f"DEBUG: Calling julia_compress_batch_config with:")
            print(f"  - tokens_list: {tokens_list}")
            print(f"  - rows: {tokens.shape[0]}")
            print(f"  - cols: {tokens.shape[1]}")
            print(f"  - gpu_enabled: {self.config.gpu_enabled}")
            print(f"  - simd_enabled: {self.config.simd_enabled}")
            print(f"  - pattern_learning: {self.config.pattern_learning}")
            
            try:
                # Call Julia batch compression via FFI
                compressed = promptveil_core.julia_compress_batch_config(
                    tokens_list,
                    tokens.shape[0],  # rows
                    tokens.shape[1],  # cols
                    self.config.gpu_enabled,
                    self.config.simd_enabled,
                    self.config.pattern_learning
                )
                print("DEBUG: Julia compression call returned successfully")
                print(f"DEBUG: Compressed result type: {type(compressed)}")
                if compressed is not None:
                    print(f"DEBUG: Compressed result length: {len(compressed)}")
            except Exception as e:
                print(f"DEBUG: Julia compression call failed with error: {type(e).__name__}: {str(e)}")
                raise
            
            # Convert to bytes
            print("DEBUG: Converting compressed data to bytes")
            compressed_bytes = np.array(compressed).tobytes()
            print(f"DEBUG: Converted to {len(compressed_bytes)} bytes")
            
            # Calculate stats
            stats = CompressionStats(
                original_size=tokens.size * 4,
                compressed_size=len(compressed_bytes),
                compression_ratio=len(compressed_bytes) / (tokens.size * 4),
                processing_time_ms=0,  # TODO: Add timing
                used_gpu=self.config.gpu_enabled
            )
            print(f"DEBUG: Created compression stats: {stats}")
            
            return compressed_bytes, stats
            
        except Exception as e:
            print(f"DEBUG: Compression failed with error: {str(e)}")
            print(f"DEBUG: Error type: {type(e)}")
            import traceback
            print(f"DEBUG: Traceback:\n{traceback.format_exc()}")
            raise CompressionError(f"Batch compression failed: {str(e)}")
    
    def decompress_batch(
        self,
        data: bytes,
        original_shape: Tuple[int, int]
    ) -> NDArray[np.uint32]:
        """
        Decompress a batch of token sequences.
        
        Args:
            data: Compressed token data
            original_shape: Shape of the original uncompressed array (rows, cols)
            
        Returns:
            Decompressed tokens array
            
        Raises:
            CompressionError: If decompression fails
        """
        try:
            # Convert bytes to list of lists
            tokens_array = np.frombuffer(data, dtype=np.uint32)
            tokens_list = tokens_array.reshape(original_shape).tolist()
            
            # Call Julia batch decompression via FFI
            decompressed = promptveil_core.julia_decompress_batch(
                tokens_list,
                original_shape[0],  # rows
                original_shape[1]   # cols
            )
            
            # Convert back to numpy array
            return np.array(decompressed, dtype=np.uint32)
            
        except Exception as e:
            raise CompressionError(f"Batch decompression failed: {str(e)}") 