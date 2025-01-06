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

class StorageError(PromptVeilError):
    """Raised when storage operations fail."""
    pass

class FormatError(StorageError):
    """Raised when file format operations fail."""
    pass 