//! LZ4 Streaming Compressor/Decompressor

mod api;

use std::{
    any::Any,
    marker::PhantomData,
    rc::{Rc, Weak},
};

pub struct Outlive<'a> {
    owner: Weak<()>,
    phantom: PhantomData<&'a ()>,
}

impl<'a> Outlive<'a> {
    fn new(owner: Weak<()>) -> Self {
        Self {
            owner,
            phantom: PhantomData,
        }
    }
}

pub struct StreamCompressor {
    rc: Rc<()>,
}

impl StreamCompressor {
    pub fn new() -> Self {
        Self { rc: Rc::new(()) }
    }

    pub fn begin<'a, 'b>(&mut self, src: &'a [u8], dst: &'b mut [u8]) -> Outlive<'a> {
        // Invalidate the current Outlive.
        self.rc = Rc::new(());
        Outlive::new(Rc::downgrade(&self.rc))
    }

    pub fn next<'a, 'b, 'c>(
        &mut self,
        prev: Outlive<'a>,
        src: &'b [u8],
        dst: &'c mut [u8],
    ) -> Outlive<'b> {
        if !prev.owner.ptr_eq(&Rc::downgrade(&self.rc)) {
            panic!("aaaa");
        }
        Outlive::new(Rc::downgrade(&self.rc))
    }

    pub fn end<'a, 'b>(&mut self, prev: Outlive<'a>, dst: &'b mut [u8]) {
        if !prev.owner.ptr_eq(&Rc::downgrade(&self.rc)) {
            panic!("aaaa");
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn compression_context() {
        let mut a = super::StreamCompressor::new();
        let c = vec![0, 4];
        let r = { a.begin(&c, &mut []) };
        let c = vec![0, 4];
        let r = a.next(r, &c, &mut []);
    }
}
