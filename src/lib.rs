//! Yet another liblz4 binding for Rust. 😴

#![deny(unsafe_code)]
#![deny(clippy::all)]
#![cfg_attr(docsrs, feature(doc_cfg))]

mod common;

pub mod lz4;
pub mod lz4_hc;
pub mod lz4f;

pub use common::*;
