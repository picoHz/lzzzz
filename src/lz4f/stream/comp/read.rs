use super::{BufReadCompressor, Dictionary, Preferences};
use crate::lz4f::{CompressorBuilder, Error, Result};
use std::{
    convert::TryInto,
    io::{BufReader, Read},
};

/// Read-based streaming compressor
///
/// # Examples
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
/// use lzzzz::lz4f::comp::ReadCompressor;
/// use std::{fs::File, io::prelude::*};
///
/// let mut f = File::open("foo.txt")?;
/// let mut r = ReadCompressor::new(&mut f, Default::default())?;
///
/// let mut buf = Vec::new();
/// r.read_to_end(&mut buf)?;
/// # Ok::<(), std::io::Error>(())
/// ```
pub struct ReadCompressor<R: Read> {
    inner: BufReadCompressor<BufReader<R>>,
}

impl<R: Read> ReadCompressor<R> {
    pub fn new(reader: R, prefs: Preferences) -> Result<Self> {
        Ok(Self {
            inner: BufReadCompressor::from_builder(BufReader::new(reader), prefs, None)?,
        })
    }

    pub fn with_dict(reader: R, prefs: Preferences, dict: Dictionary) -> Result<Self> {
        Ok(Self {
            inner: BufReadCompressor::from_builder(BufReader::new(reader), prefs, Some(dict))?,
        })
    }

    fn from_builder(device: R, pref: Preferences, dict: Option<Dictionary>) -> Result<Self> {
        Ok(Self {
            inner: BufReadCompressor::from_builder(BufReader::new(device), pref, dict)?,
        })
    }
}

impl<R: Read> Read for ReadCompressor<R> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        self.inner.read(buf)
    }
}

impl<R: Read> TryInto<ReadCompressor<R>> for CompressorBuilder<R> {
    type Error = Error;
    fn try_into(self) -> Result<ReadCompressor<R>> {
        ReadCompressor::from_builder(self.device, self.pref, self.dict)
    }
}
