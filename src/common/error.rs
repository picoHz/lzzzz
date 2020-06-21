use std::{convert, fmt, io};

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum ErrorKind {
    CompressionFailed,
    DecompressionFailed,
    StreamResetFailed,
    CompressedDataIncomplete,
    NullPointerUnexpected,
    CompressionModeInvalid,
    DecompressionModeInvalid,
    DictionaryChangedDuringDecompression,
}

impl fmt::Display for ErrorKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), fmt::Error> {
        <Self as fmt::Debug>::fmt(self, f)
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct Error {
    kind: ErrorKind,
}

impl Error {
    pub(crate) const fn new(kind: ErrorKind) -> Self {
        Self { kind }
    }

    pub const fn kind(self) -> ErrorKind {
        self.kind
    }
}

impl convert::From<Error> for io::Error {
    fn from(err: Error) -> Self {
        Self::new(io::ErrorKind::Other, err)
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), fmt::Error> {
        <ErrorKind as fmt::Display>::fmt(&self.kind, f)
    }
}

impl std::error::Error for Error {}

/// A specialized Result type for compression/decompression operations.
pub type Result<T> = std::result::Result<T, Error>;
