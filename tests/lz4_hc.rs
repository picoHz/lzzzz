use lzzzz::{lz4, lz4_hc};
use rayon::{iter::ParallelBridge, prelude::*};
use std::io::Cursor;

mod common;
use common::lz4_hc_test_set;

mod compress {
    use super::*;

    #[test]
    fn default() {
        lz4_hc_test_set().par_bridge().for_each(|(src, level)| {
            let mut comp_buf = vec![0; lz4::max_compressed_size(src.len())];
            let mut decomp_buf = vec![0; src.len()];
            let len = lz4_hc::compress(&src, &mut comp_buf, level).unwrap();
            lz4::decompress(&comp_buf[..len], &mut decomp_buf).unwrap();
            assert_eq!(decomp_buf, src);
        });
    }
}

mod compress_partial {
    use super::*;

    #[test]
    fn default() {
        lz4_hc_test_set()
            .map(|(src, level)| (0..20).map(move |n| (src.clone(), level, 16 << n)))
            .flatten()
            .par_bridge()
            .for_each(|(src, level, len)| {
                let mut comp_buf = vec![0; len];
                let mut decomp_buf = Vec::new();
                let mut src = Cursor::new(src);
                let pos = src.get_ref().len() / 2;
                src.set_position(pos as u64);
                let dst_len = lz4_hc::compress_partial(&mut src, &mut comp_buf, level).unwrap();
                decomp_buf.resize(src.position() as usize - pos, 0);
                lz4::decompress(&comp_buf[..dst_len], &mut decomp_buf).unwrap();
                assert!(src.get_ref()[pos..].starts_with(&decomp_buf));
            });
    }
}

mod compress_to_vec {
    use super::*;

    #[test]
    fn default() {
        lz4_hc_test_set().par_bridge().for_each(|(src, level)| {
            let header = &b"HEADER"[..];
            let mut comp_buf = Vec::from(header);
            let mut decomp_buf = vec![0; src.len()];
            lz4_hc::compress_to_vec(&src, &mut comp_buf, level).unwrap();
            assert!(comp_buf.starts_with(header));
            lz4::decompress(&comp_buf[header.len()..], &mut decomp_buf).unwrap();
            assert_eq!(decomp_buf, src);
        });
    }
}
