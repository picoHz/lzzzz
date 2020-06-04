mod sys;

pub fn version_number() -> u32 {
    unsafe { sys::LZ4_versionNumber() as u32 }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(crate::version_number(), 10902);
    }
}
