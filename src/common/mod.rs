mod api;
mod binding;
mod buffer;
mod error;

pub use api::{version_number, version_string};
pub use buffer::Buffer;
pub use error::{Error, ErrorKind, Result};

pub(crate) const DEFAULT_BUF_SIZE: usize = 8 * 1024;
