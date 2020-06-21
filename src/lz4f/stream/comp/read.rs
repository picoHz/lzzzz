use super::{BufReadCompressor, Dictionary, Preferences};
use crate::lz4f::CompressorBuilder;
use std::{
    convert::TryInto,
    io::{BufReader, Read, Result},
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
/// let mut r = ReadCompressor::new(&mut f)?;
///
/// let mut buf = Vec::new();
/// r.read_to_end(&mut buf)?;
/// # Ok::<(), std::io::Error>(())
/// ```
pub struct ReadCompressor<R: Read> {
    inner: BufReadCompressor<BufReader<R>>,
}

impl<R: Read> ReadCompressor<R> {
    pub fn new(reader: R) -> crate::lz4f::Result<Self> {
        CompressorBuilder::new(reader).build()
    }

    fn from_builder(
        device: R,
        pref: Preferences,
        dict: Option<Dictionary>,
    ) -> crate::lz4f::Result<Self> {
        Ok(Self {
            inner: BufReadCompressor::from_builder(BufReader::new(device), pref, dict)?,
        })
    }
}

impl<R: Read> Read for ReadCompressor<R> {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        self.inner.read(buf)
    }
}

impl<R: Read> TryInto<ReadCompressor<R>> for CompressorBuilder<R> {
    type Error = crate::lz4f::Error;
    fn try_into(self) -> crate::lz4f::Result<ReadCompressor<R>> {
        ReadCompressor::from_builder(self.device, self.pref, self.dict)
    }
}
