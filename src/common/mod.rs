mod api;
mod binding;
mod buffer;
use std::{convert, fmt, io};

pub use api::{version_number, version_string};
pub use buffer::Buffer;

pub(crate) const DEFAULT_BUF_SIZE: usize = 8 * 1024;

/// A result of successful compression/decompression
#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, Hash)]
pub struct Report {
    pub(crate) dst_len: usize,
    pub(crate) src_len: Option<usize>,
}

impl Report {
    /// Return the length of the data written to the destination buffer.
    pub const fn dst_len(&self) -> usize {
        self.dst_len
    }

    /// Return the length of the data consumed from the source buffer.
    ///
    /// The value is present only if the underlying liblz4 API
    /// explicitly returns one.
    /// In most cases, the consumed length must be equal to the length of the source buffer
    /// and this method just returns [`None`].
    ///
    /// [`None`]: https://doc.rust-lang.org/nightly/core/option/enum.Option.html#variant.None
    pub const fn src_len(&self) -> Option<usize> {
        self.src_len
    }
}

/// A specialized Result type for compression/decompression operations.
pub type Result<T> = std::result::Result<T, Error>;

/// Compression/Decompression error
#[derive(Debug)]
pub enum Error {
    IOError(io::Error),
    CompressionFailed,
    DecompressionFailed,
    StreamResetFailed,
    CompressedDataIncomplete,
    NullPointerUnexpected,
    CompressionModeInvalid,
    DecompressionModeInvalid,
    DictionaryChangedDuringDecompression,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), fmt::Error> {
        <Self as fmt::Debug>::fmt(self, f)
    }
}

impl convert::From<Error> for io::Error {
    fn from(err: Error) -> Self {
        match err {
            Error::IOError(err) => err,
            _ => Self::new(io::ErrorKind::Other, err),
        }
    }
}

impl convert::From<io::Error> for Error {
    fn from(err: io::Error) -> Self {
        Self::IOError(err)
    }
}

impl std::error::Error for Error {}

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

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct Error2 {
    kind: ErrorKind,
}

impl Error2 {
    pub const fn new(kind: ErrorKind) -> Self {
        Self { kind }
    }

    pub const fn kind(self) -> ErrorKind {
        self.kind
    }
}

impl convert::From<Error2> for io::Error {
    fn from(err: Error2) -> Self {
        Self::new(io::ErrorKind::Other, err)
    }
}

impl fmt::Display for Error2 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), fmt::Error> {
        <Self as fmt::Debug>::fmt(self, f)
    }
}

impl std::error::Error for Error2 {}
