//! LZ4 Frame Streaming Compressor/Decompressor

mod api;
mod binding;

use crate::{
    lz4f::{FrameInfo, Preferences},
    Result,
};
use api::{CompressionContext, DictionaryHandle, LZ4Buffer};
use std::{cmp, io, ops, sync::Arc};

enum CompressorState<D> {
    Created,
    WriteActive {
        finalizer: fn(&mut StreamCompressor<D>) -> Result<()>,
    },
    WriteFinalized,
    ReadActive {
        buffered: ops::Range<usize>,
    },
}

/// The `StreamCompressor<D>` provides a transparent compression to any reader
/// and writer.
///
/// If the underlying I/O device `D` implements `Read`, `BufRead` or `Write`,
/// the `StreamCompressor<D>` also implements `Read`, `BufRead` or `Write`.
///
/// Note that this doesn't mean "Bidirectional stream".
/// Making read and write operations on a same instance causes a panic!
pub struct StreamCompressor<D> {
    pref: Preferences,
    ctx: CompressionContext,
    device: D,
    state: CompressorState<D>,
    buffer: LZ4Buffer,
}

impl<D> StreamCompressor<D> {
    /// Create a new `StreamCompressor<D>` instance with the default
    /// configuration.
    pub fn new(device: D, pref: Preferences) -> Result<Self> {
        Ok(Self {
            pref: pref,
            ctx: CompressionContext::new(None)?,
            device,
            state: CompressorState::Created,
            buffer: LZ4Buffer::new(),
        })
    }
}

impl<D: io::Write> StreamCompressor<D> {
    /// Finalize this LZ4 frame explicitly.
    ///
    /// Dropping a `StreamCompressor` automatically finalize a frame
    /// so you don't have to call this unless you need a `Result`.
    pub fn end(&mut self) -> Result<()> {
        self.finalize_write()
    }

    fn finalize_write(&mut self) -> Result<()> {
        self.ensure_write();
        if let CompressorState::WriteActive { .. } = &self.state {
            self.state = CompressorState::WriteFinalized;
            let len = self.ctx.end(&mut self.buffer, None)?;
            self.device.write_all(&self.buffer[..len])?;
            self.device.flush()?;
        }
        Ok(())
    }

    fn ensure_write(&self) {
        if let CompressorState::ReadActive { .. } = self.state {
            panic!("Read operations are not permitted")
        }
    }
}

impl<D: io::Write> io::Write for StreamCompressor<D> {
    fn write(&mut self, src: &[u8]) -> io::Result<usize> {
        self.ensure_write();
        self.buffer.grow(src.len(), &self.pref);
        if let CompressorState::Created = self.state {
            self.state = CompressorState::WriteActive {
                finalizer: StreamCompressor::<D>::finalize_write,
            };
            let len = self.ctx.begin(&mut self.buffer, &self.pref)?;
            self.device.write(&self.buffer[..len])?;
        }
        let len = self.ctx.update(&mut self.buffer, src, None)?;
        self.device.write(&self.buffer[..len])?;
        Ok(src.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        self.ensure_write();
        let len = self.ctx.flush(&mut self.buffer, None)?;
        self.device.write_all(&self.buffer[..len])?;
        self.device.flush()
    }
}

impl<D: io::Read> StreamCompressor<D> {
    fn ensure_read(&self) {
        match self.state {
            CompressorState::WriteActive { .. } | CompressorState::WriteFinalized => {
                panic!("Write operations are not permitted")
            }
            _ => (),
        }
    }
}

impl<D: io::Read> io::Read for StreamCompressor<D> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.ensure_read();

        let header_len = if let CompressorState::Created = self.state {
            self.state = CompressorState::ReadActive { buffered: 0..0 };
            self.buffer.grow(0, &self.pref);
            self.ctx.begin(&mut self.buffer, &self.pref)?
        } else if let CompressorState::ReadActive { buffered } = &self.state {
            let len = buffered.end - buffered.start;
            let min_len = cmp::min(len, buf.len());
            buf[..min_len].copy_from_slice(&self.buffer[buffered.start..buffered.start + min_len]);
            self.state = CompressorState::ReadActive {
                buffered: if min_len < len {
                    buffered.start + min_len..buffered.end
                } else {
                    0..0
                },
            };
            return Ok(min_len);
        } else {
            0
        };

        let mut tmp = [0u8; 2048];
        let len = self.device.read(&mut tmp[..])?;
        self.buffer.grow(len, &self.pref);

        let len = if len == 0 {
            self.ctx.flush(&mut self.buffer[header_len..], None)?
        } else {
            self.ctx
                .update(&mut self.buffer[header_len..], &tmp[..len], None)?
        };
        let len = header_len + len;
        let min_len = cmp::min(len, buf.len());
        buf[..min_len].copy_from_slice(&self.buffer[..min_len]);
        if min_len < len {
            self.state = CompressorState::ReadActive {
                buffered: min_len..len,
            };
        }
        Ok(min_len)
    }
}

impl<D: io::BufRead> io::BufRead for StreamCompressor<D> {
    fn fill_buf(&mut self) -> io::Result<&[u8]> {
        use std::io::Read;
        self.read(&mut [])?;
        if let CompressorState::ReadActive { buffered } = &self.state {
            Ok(&self.buffer[buffered.clone()])
        } else {
            Ok(&[])
        }
    }

    fn consume(&mut self, amt: usize) {
        self.ensure_read();
        if let CompressorState::ReadActive { buffered } = &self.state {
            let len = buffered.end - buffered.start;
            self.state = CompressorState::ReadActive {
                buffered: if amt >= len {
                    0..0
                } else {
                    buffered.start + amt..buffered.end
                },
            };
        }
    }
}

impl<D> Drop for StreamCompressor<D> {
    fn drop(&mut self) {
        let finalizer = if let CompressorState::WriteActive { finalizer } = &self.state {
            finalizer
        } else {
            return;
        };
        let _ = (finalizer)(self);
    }
}

enum DecompressorState {
    Created,
}

/// The `FrameDeompressor<D>` provides a transparent decompression to any reader
/// and writer.
pub struct StreamDecompressor<'a, D> {
    device: D,
    state: DecompressorState,
    dict: &'a [u8],
}

impl<'a, D> StreamDecompressor<'a, D> {
    pub fn new(device: D) -> Result<Self> {
        Ok(Self {
            device,
            state: DecompressorState::Created,
            dict: &[],
        })
    }

    pub fn with_dict(device: D, dict: &'a [u8]) -> Result<Self> {
        Ok(Self {
            device,
            state: DecompressorState::Created,
            dict,
        })
    }

    pub fn set_dict(&mut self, dict: &'a [u8]) {
        self.dict = dict;
    }

    pub fn frame_info(&mut self) -> Result<FrameInfo> {
        todo!();
    }
}

/// A user-defined dictionary for the efficient compression.
///
/// **Cited from lz4frame.h:**
///
/// A Dictionary is useful for the compression of small messages (KB range).
/// It dramatically improves compression efficiency.
///
/// LZ4 can ingest any input as dictionary, though only the last 64 KB are
/// useful. Best results are generally achieved by using Zstandard's Dictionary
/// Builder to generate a high-quality dictionary from a set of samples.
#[derive(Clone)]
pub struct Dictionary(Arc<DictionaryHandle>);

impl Dictionary {
    pub fn new(data: &[u8]) -> Self {
        Self(Arc::new(DictionaryHandle::new(data)))
    }
}
