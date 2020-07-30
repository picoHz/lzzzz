use super::{BufReadCompressor, Dictionary, Preferences};
use crate::lz4f::Result;
use std::{
    fmt,
    io::{BufReader, Read},
};

/// The [`Read`]-based streaming compressor.
///
/// # Example
///
/// ```
/// # use std::env;
/// # use std::path::Path;
/// # use lzzzz::{Error, Result};
/// # use assert_fs::prelude::*;
/// # let tmp_dir = assert_fs::TempDir::new().unwrap().into_persistent();
/// # env::set_current_dir(tmp_dir.path()).unwrap();
/// #
/// # tmp_dir.child("foo.txt").write_str("Hello").unwrap();
/// #
/// use lzzzz::lz4f::ReadCompressor;
/// use std::{fs::File, io::prelude::*};
///
/// let mut f = File::open("foo.txt")?;
/// let mut r = ReadCompressor::new(&mut f, Default::default())?;
///
/// let mut buf = Vec::new();
/// r.read_to_end(&mut buf)?;
/// # Ok::<(), std::io::Error>(())
/// ```
///
/// [`Read`]: https://doc.rust-lang.org/std/io/trait.Read.html

pub struct ReadCompressor<R: Read> {
    inner: BufReadCompressor<BufReader<R>>,
}

impl<R: Read> ReadCompressor<R> {
    /// Creates a new `ReadCompressor<R>`.
    pub fn new(reader: R, prefs: Preferences) -> Result<Self> {
        Ok(Self {
            inner: BufReadCompressor::new(BufReader::new(reader), prefs)?,
        })
    }

    /// Creates a new `ReadCompressor<R>` with a dictionary.
    pub fn with_dict(reader: R, prefs: Preferences, dict: Dictionary) -> Result<Self> {
        Ok(Self {
            inner: BufReadCompressor::with_dict(BufReader::new(reader), prefs, dict)?,
        })
    }

    /// Returns ownership of the reader.
    pub fn into_inner(self) -> R {
        self.inner.into_inner().into_inner()
    }
}

impl<R> fmt::Debug for ReadCompressor<R>
where
    R: Read + fmt::Debug,
{
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt.debug_struct("ReadCompressor")
            .field("reader", &self.inner.device.get_ref())
            .field("prefs", &self.inner.inner.prefs())
            .finish()
    }
}

impl<R: Read> Read for ReadCompressor<R> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        self.inner.read(buf)
    }
}
