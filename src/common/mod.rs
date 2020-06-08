mod api;
use std::{convert, fmt, io};

pub use api::{version_number, version_string};

/// A result of successful compression/decompression
#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, Hash)]
pub struct Report {
    pub(crate) src_len: Option<usize>,
    pub(crate) dst_len: usize,
}

impl Report {
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

    /// Return the length of the data written to the destination buffer.
    pub const fn dst_len(&self) -> usize {
        self.dst_len
    }
}

#[derive(Debug)]
pub enum LZ4Error {
    LZ4(&'static str),
    IO(io::Error),
}

impl fmt::Display for LZ4Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), fmt::Error> {
        match self {
            Self::LZ4(err) => write!(f, "{}", err),
            Self::IO(err) => err.fmt(f),
        }
    }
}

impl std::error::Error for LZ4Error {}

impl From<&'static str> for LZ4Error {
    fn from(err: &'static str) -> Self {
        Self::LZ4(err)
    }
}

impl convert::From<io::Error> for LZ4Error {
    fn from(err: io::Error) -> Self {
        Self::IO(err)
    }
}

impl convert::From<LZ4Error> for io::Error {
    fn from(err: LZ4Error) -> Self {
        match err {
            LZ4Error::LZ4(err) => io::Error::new(io::ErrorKind::Other, err),
            LZ4Error::IO(err) => err,
        }
    }
}

pub type Result<T> = std::result::Result<T, LZ4Error>;
