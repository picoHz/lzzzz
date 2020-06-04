#![deny(unsafe_code)]

pub mod lz4f;
pub(crate) mod sys;

type Result<T> = std::result::Result<T, &'static str>;
