#![allow(unsafe_code)]

use crate::{binding, binding::LZ4StreamHC, Error, Result};

use libc::{c_void, size_t};
use std::{
    mem::{size_of, MaybeUninit},
    ptr::NonNull,
};

enum Stream {
    Stack(LZ4StreamHC),
    Heap(NonNull<LZ4StreamHC>),
}

pub struct CompressionContext {
    stream: Stream,
}

impl CompressionContext {
    pub fn new() -> Result<Self> {
        let mut stream = MaybeUninit::<LZ4StreamHC>::zeroed();
        unsafe {
            let ptr = binding::LZ4_initStreamHC(
                stream.as_mut_ptr() as *mut c_void,
                size_of::<LZ4StreamHC>() as size_t,
            );
            if !ptr.is_null() {
                return Ok(Self {
                    stream: Stream::Stack(stream.assume_init()),
                });
            }
            let ptr = NonNull::new(binding::LZ4_createStreamHC());
            ptr.ok_or(Error::Generic).map(|stream| Self {
                stream: Stream::Heap(stream),
            })
        }
    }
}

impl Drop for CompressionContext {
    fn drop(&mut self) {
        if let Stream::Heap(mut ptr) = self.stream {
            unsafe {
                binding::LZ4_freeStreamHC(ptr.as_mut());
            }
        }
    }
}
