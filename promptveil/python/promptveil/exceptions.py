"""
Exception classes for PromptVeil.
"""

class PromptVeilError(Exception):
    """Base exception for all PromptVeil errors."""
    pass

class CompressionError(PromptVeilError):
    """Raised when compression or decompression fails."""
    pass

class GPUError(CompressionError):
    """Raised when GPU operations fail."""
    pass

class MemoryError(CompressionError):
    """Raised when memory operations fail."""
    pass

class SecurityError(PromptVeilError):
    """Raised when security operations fail."""
    pass

class StoreError(PromptVeilError):
    """Raised when conversation store operations fail."""
    pass

class FormatError(StoreError):
    """Raised when file format operations fail."""
    pass 