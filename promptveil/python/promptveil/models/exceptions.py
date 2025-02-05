"""Exception classes for PromptVeil."""

class PromptVeilError(Exception):
    """Base exception for all PromptVeil errors."""
    pass

class ConfigurationError(PromptVeilError):
    """Error in configuration."""
    pass

class SecurityError(PromptVeilError):
    """Security-related error."""
    pass

class IndexError(PromptVeilError):
    """Index-related error."""
    pass

class FormatError(PromptVeilError):
    """Format-related error."""
    pass

class VectorError(PromptVeilError):
    """Vector computation error."""
    pass

class ConversationError(PromptVeilError):
    """Conversation-related error."""
    pass

class StorageError(PromptVeilError):
    """Storage-related error."""
    pass

class NotFoundError(PromptVeilError):
    """Resource not found error."""
    pass

class ValidationError(PromptVeilError):
    """Data validation error."""
    pass 