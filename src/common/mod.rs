mod api;
mod binding;
use std::{convert, fmt, io};

pub use api::{version_number, version_string};

/// A result of successful compression/decompression
#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, Hash)]
pub struct Report {
    pub(crate) dst_len: usize,
    pub(crate) src_len: Option<usize>,
    pub(crate) expected_src_len: Option<usize>,
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

    pub(crate) const fn expected_src_len(&self) -> Option<usize> {
        self.expected_src_len
    }
}

/// A specialized Result type for compression/decompression operations.
pub type Result<T> = std::result::Result<T, Error>;

/// Compression/Decompression error
#[derive(Debug)]
pub enum Error {
    LZ4Error(LZ4Error),
    IOError(io::Error),
    CompressionFailed,
    CompressedDataIncomplete,
    NullPointerUnexpected,
    CompressionModeInvalid,
    DecompressionModeInvalid,
    DictionaryChangedDuringDecompression,
}

/// Errors from liblz4
#[derive(Debug)]
pub enum LZ4Error {
    Generic,
    MaxBlockSizeInvalid,
    BlockModeInvalid,
    ContentChecksumFlagInvalid,
    CompressionLevelInvalid,
    HeaderVersionWrong,
    BlockChecksumInvalid,
    ReservedFlagSet,
    AllocationFailed,
    SrcSizeTooLarge,
    DstMaxSizeTooSmall,
    FrameHeaderIncomplete,
    FrameTypeUnknown,
    FrameSizeWrong,
    SrcPtrWrong,
    DecompressionFailed,
    HeaderChecksumInvalid,
    ContentChecksumInvalid,
    FrameDecodingAlreadyStarted,
    Unspecified,
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
            _ => io::Error::new(io::ErrorKind::Other, err),
        }
    }
}

impl convert::From<io::Error> for Error {
    fn from(err: io::Error) -> Self {
        Self::IOError(err)
    }
}

impl convert::From<usize> for Error {
    fn from(value: usize) -> Self {
        Self::LZ4Error(match value.wrapping_neg() {
            1 => LZ4Error::Generic,
            2 => LZ4Error::MaxBlockSizeInvalid,
            3 => LZ4Error::BlockModeInvalid,
            4 => LZ4Error::ContentChecksumFlagInvalid,
            5 => LZ4Error::CompressionLevelInvalid,
            6 => LZ4Error::HeaderVersionWrong,
            7 => LZ4Error::BlockChecksumInvalid,
            8 => LZ4Error::ReservedFlagSet,
            9 => LZ4Error::AllocationFailed,
            10 => LZ4Error::SrcSizeTooLarge,
            11 => LZ4Error::DstMaxSizeTooSmall,
            12 => LZ4Error::FrameHeaderIncomplete,
            13 => LZ4Error::FrameTypeUnknown,
            14 => LZ4Error::FrameSizeWrong,
            15 => LZ4Error::SrcPtrWrong,
            16 => LZ4Error::DecompressionFailed,
            17 => LZ4Error::HeaderChecksumInvalid,
            18 => LZ4Error::ContentChecksumInvalid,
            19 => LZ4Error::FrameDecodingAlreadyStarted,
            _ => LZ4Error::Unspecified,
        })
    }
}

impl std::error::Error for Error {}
