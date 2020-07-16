use std::{convert, fmt, io};

/// A list specifying general categories of compression/decompression error.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum ErrorKind {
    InitializationFailed,
    CompressionFailed,
    DecompressionFailed,
    FrameHeaderInvalid,
    CompressedDataIncomplete,
    /// The specified compression mode was not valid.
    CompressionModeInvalid,
    /// The specified decompression mode was not valid.
    DecompressionModeInvalid,
    /// The dictionary data was not consistent during the decompression.
    DictionaryChangedDuringDecompression,
}

impl fmt::Display for ErrorKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), fmt::Error> {
        <Self as fmt::Debug>::fmt(self, f)
    }
}

/// The error type for compression/decompression operations
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct Error {
    kind: ErrorKind,
}

impl Error {
    pub(crate) const fn new(kind: ErrorKind) -> Self {
        Self { kind }
    }

    /// Return the corresponding `ErrorKind` for this error.
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

/// A specialized [`Result`] type for compression/decompression operations.
///
/// [`Result`]: https://doc.rust-lang.org/std/io/type.Result.html
pub type Result<T> = std::result::Result<T, Error>;
