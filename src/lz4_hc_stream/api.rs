#![allow(unsafe_code)]

use crate::{binding, binding::LZ4StreamHC, Error, Result};

use libc::{c_void, size_t};
use std::{
    mem::{size_of, ManuallyDrop, MaybeUninit},
    ptr::NonNull,
};

pub struct CompressionContext {
    stream: NonNull<LZ4StreamHC>,
}

impl CompressionContext {
    pub fn new() -> Result<Self> {
        let ptr = unsafe { NonNull::new(binding::LZ4_createStreamHC()) };
        ptr.ok_or(Error::Generic).map(|stream| Self { stream })
    }
}

impl Drop for CompressionContext {
    fn drop(&mut self) {
        unsafe {
            binding::LZ4_freeStreamHC(self.stream.as_mut());
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn compression_contextX() {
        println!("{}", super::CompressionContext::new().is_ok());
    }
}
