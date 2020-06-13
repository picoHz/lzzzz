#![allow(unsafe_code)]

use crate::{
    binding,
    binding::{LZ4DecStream, LZ4Stream},
    Error, Result,
};

use libc::{c_char, c_int, c_void, size_t};
use std::{
    mem::{size_of, MaybeUninit},
    ptr::NonNull,
};

enum Stream {
    Stack(LZ4Stream),
    Heap(NonNull<LZ4Stream>),
}

pub struct CompressionContext {
    stream: Stream,
}

impl CompressionContext {
    pub fn new() -> Result<Self> {
        let mut stream = MaybeUninit::<LZ4Stream>::zeroed();
        unsafe {
            let ptr = binding::LZ4_initStream(
                stream.as_mut_ptr() as *mut c_void,
                size_of::<LZ4Stream>() as size_t,
            );
            if !ptr.is_null() {
                return Ok(Self {
                    stream: Stream::Stack(stream.assume_init()),
                });
            }
            let ptr = NonNull::new(binding::LZ4_createStream());
            ptr.ok_or(Error::NullPointerUnexprected).map(|stream| Self {
                stream: Stream::Heap(stream),
            })
        }
    }

    fn get_ptr(&mut self) -> *mut LZ4Stream {
        match &mut self.stream {
            Stream::Stack(stream) => stream as *mut LZ4Stream,
            Stream::Heap(ptr) => ptr.as_ptr(),
        }
    }

    pub fn set_dict(&mut self, dict: &[u8]) {
        unsafe {
            binding::LZ4_loadDict(
                self.get_ptr(),
                dict.as_ptr() as *const c_char,
                dict.len() as c_int,
            );
        }
    }

    pub fn next(&mut self, src: &[u8], dst: &mut [u8], acceleration: i32) -> usize {
        unsafe {
            binding::LZ4_compress_fast_continue(
                self.get_ptr(),
                src.as_ptr() as *const c_char,
                dst.as_mut_ptr() as *mut c_char,
                src.len() as c_int,
                dst.len() as c_int,
                acceleration as c_int,
            ) as usize
        }
    }
}

impl Drop for CompressionContext {
    fn drop(&mut self) {
        if let Stream::Heap(mut ptr) = self.stream {
            unsafe {
                binding::LZ4_freeStream(ptr.as_mut());
            }
        }
    }
}

pub struct DecompressionContext {
    stream: NonNull<LZ4DecStream>,
}

impl DecompressionContext {
    pub fn new() -> Result<Self> {
        unsafe {
            let ptr = NonNull::new(binding::LZ4_createStreamDecode());
            ptr.ok_or(Error::NullPointerUnexprected)
                .map(|stream| Self { stream })
        }
    }
}

impl Drop for DecompressionContext {
    fn drop(&mut self) {
        unsafe {
            binding::LZ4_freeStreamDecode(self.stream.as_mut());
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn compression_context() {
        println!("{}", super::CompressionContext::new().is_ok());
    }
}
