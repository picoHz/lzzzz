use super::Decompressor;
use crate::{
    lz4f::{DecompressorBuilder, FrameInfo},
    Error, LZ4Error,
};
use std::{
    borrow::Cow,
    convert::TryInto,
    io::{BufRead, Read, Result},
};

/// BufRead-based streaming decompressor
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
/// use lzzzz::lz4f::decomp::BufReadDecompressor;
/// use std::{
///     fs::File,
///     io::{prelude::*, BufReader},
/// };
///
/// let mut f = File::open("foo.lz4")?;
/// let mut b = BufReader::new(f);
/// let mut r = BufReadDecompressor::new(&mut b)?;
///
/// let mut buf = Vec::new();
/// r.read_to_end(&mut buf)?;
/// # Ok::<(), std::io::Error>(())
/// ```
pub struct BufReadDecompressor<'a, R: BufRead> {
    device: R,
    inner: Decompressor<'a>,
    consumed: usize,
}

impl<'a, R: BufRead> BufReadDecompressor<'a, R> {
    pub fn new(reader: R) -> crate::Result<Self> {
        Self::from_builder(reader)
    }

    pub(super) fn from_builder(device: R) -> crate::Result<Self> {
        Ok(Self {
            device,
            inner: Decompressor::new()?,
            consumed: 0,
        })
    }

    pub fn set_dict(&mut self, dict: Cow<'a, [u8]>) {
        self.inner.set_dict(dict);
    }

    pub fn read_frame_info(&mut self) -> Result<FrameInfo> {
        loop {
            if let Some(frame) = self.inner.frame_info() {
                return Ok(frame);
            }
            let _ = self.read(&mut [])?;
        }
    }
}

impl<R: BufRead> Read for BufReadDecompressor<'_, R> {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        loop {
            let inner_buf = self.device.fill_buf()?;
            let report = self.inner.decompress(inner_buf)?;
            self.device.consume(report.src_len().unwrap());
            if report.dst_len() > 0 {
                break;
            }
        }

        let len = std::cmp::min(buf.len(), self.inner.buf().len() - self.consumed);
        buf[..len].copy_from_slice(&self.inner.buf()[self.consumed..][..len]);
        self.consumed += len;
        if self.consumed >= self.inner.buf().len() {
            self.inner.clear_buf();
            self.consumed = 0;
        }
        Ok(len)
    }
}

impl<R: BufRead> BufRead for BufReadDecompressor<'_, R> {
    fn fill_buf(&mut self) -> Result<&[u8]> {
        let _ = self.read(&mut [])?;
        Ok(&self.inner.buf()[self.consumed..])
    }

    fn consume(&mut self, amt: usize) {
        self.consumed += amt;
        if self.consumed >= self.inner.buf().len() {
            self.inner.clear_buf();
            self.consumed = 0;
        }
    }
}

impl<'a, R: BufRead> TryInto<BufReadDecompressor<'a, R>> for DecompressorBuilder<R> {
    type Error = crate::Error;
    fn try_into(self) -> crate::Result<BufReadDecompressor<'a, R>> {
        BufReadDecompressor::from_builder(self.device)
    }
}

#[cfg(test)]
mod tests {
    use crate::lz4f::{
        comp::WriteCompressor,
        compress_to_vec,
        decomp::{BufReadDecompressor, WriteDecompressor},
        decompress_to_vec, CompressorBuilder, DecompressorBuilder,
    };
    use std::{
        fs::File,
        io::{BufReader, Read, Write},
    };

    #[test]
    fn read() -> std::io::Result<()> {
        {
            let mut file = File::create("foo.lz4")?;
            let mut file = CompressorBuilder::new(&mut file).build::<WriteCompressor<_>>()?;
            file.write_all(b"hello")?;
        }
        let mut comp = vec![];
        {
            let mut file = BufReader::new(File::open("foo.lz4")?);
            file.read_to_end(&mut comp)?;
        }
        {
            let mut buf = Vec::new();
            let mut decomp = Vec::new();
            compress_to_vec(b"Hello world!", &mut buf, &Default::default())?;
            decompress_to_vec(&buf, &mut decomp)?;
            let mut file = File::create("foo.lz4.dec")?;
            let mut file = DecompressorBuilder::new(&mut file).build::<WriteDecompressor<_>>()?;
            println!(">>> {:?}", &decomp);
            println!(">>> {:?}", file.frame_info());
            file.write_all(&buf)?;
            //println!(">>> {:?}", file.frame_info());
        }
        Ok(())
    }
}
