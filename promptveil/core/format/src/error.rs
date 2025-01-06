use std::fmt;
use std::io;

#[derive(Debug)]
pub enum Error {
    Io(io::Error),
    Serialization(String),
    Deserialization(String),
    InvalidMagic,
    InvalidVersion,
    InvalidPartition(String),
    InvalidSchema(String),
    InvalidBlock(String),
    Compression(String),
    Encryption(String),
}

impl std::error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Io(err) => write!(f, "IO error: {}", err),
            Error::Serialization(msg) => write!(f, "Serialization error: {}", msg),
            Error::Deserialization(msg) => write!(f, "Deserialization error: {}", msg),
            Error::InvalidMagic => write!(f, "Invalid file magic number"),
            Error::InvalidVersion => write!(f, "Unsupported file version"),
            Error::InvalidPartition(msg) => write!(f, "Invalid partition: {}", msg),
            Error::InvalidSchema(msg) => write!(f, "Invalid schema: {}", msg),
            Error::InvalidBlock(msg) => write!(f, "Invalid block: {}", msg),
            Error::Compression(msg) => write!(f, "Compression error: {}", msg),
            Error::Encryption(msg) => write!(f, "Encryption error: {}", msg),
        }
    }
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Self {
        Error::Io(err)
    }
}

impl From<bincode::Error> for Error {
    fn from(err: bincode::Error) -> Self {
        Error::Serialization(err.to_string())
    }
} 