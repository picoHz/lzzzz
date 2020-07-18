use lzzzz::lz4;
use rayon::{iter::ParallelBridge, prelude::*};
use std::cmp;

mod common;
use common::lz4_test_set;

mod compress {
    use super::*;

    #[test]
    fn default() {
        lz4_test_set().par_bridge().for_each(|(src, mode)| {
            let mut comp_buf = vec![0; lz4::max_compressed_size(src.len())];
            let mut decomp_buf = vec![0; src.len()];
            let len = lz4::compress(&src, &mut comp_buf, mode).unwrap();
            lz4::decompress(&comp_buf[..len], &mut decomp_buf).unwrap();
            assert_eq!(decomp_buf, src);
        });
    }

    fn aaa() -> Result<(), std::io::Error> {
        use lzzzz::lz4;

        const ORIGINAL_SIZE: usize = 44;
        const COMPRESSED_DATA: &str =
            "8B1UaGUgcXVpY2sgYnJvd24gZm94IGp1bXBzIG92ZXIgdGhlIGxhenkgZG9nLg==";

        let data = base64::decode(COMPRESSED_DATA).unwrap();

        let mut decomp = lz4::Decompressor::new()?;
        let result = decomp.next(&data[..], ORIGINAL_SIZE)?;

        assert_eq!(result, &b"The quick brown fox jumps over the lazy dog."[..]);
        // Ok::<(), std::io::Error>(())
        Ok(())
    }
}

mod compress_to_vec {
    use super::*;

    #[test]
    fn default() {
        lz4_test_set().par_bridge().for_each(|(src, mode)| {
            let mut comp_buf = Vec::new();
            let mut decomp_buf = vec![0; src.len()];
            lz4::compress_to_vec(&src, &mut comp_buf, mode).unwrap();
            lz4::decompress(&comp_buf, &mut decomp_buf).unwrap();
            assert_eq!(decomp_buf, src);
        });
    }
}

mod decompress {
    use super::*;

    #[test]
    fn partial() {
        lz4_test_set()
            .map(|(src, mode)| (0..20).map(move |n| (src.clone(), mode, 16 << n)))
            .flatten()
            .par_bridge()
            .for_each(|(src, mode, len)| {
                let mut comp_buf = Vec::new();
                let mut decomp_buf = vec![0; cmp::min(src.len(), len)];
                lz4::compress_to_vec(&src, &mut comp_buf, mode).unwrap();
                lz4::decompress_partial(&comp_buf, &mut decomp_buf, src.len()).unwrap();
                assert!(src.starts_with(&decomp_buf));
            });
    }
}
