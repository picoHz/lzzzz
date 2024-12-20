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

    #[test]
    fn fill() {
        lz4_test_set()
            .flat_map(|(src, mode)| (0..20).map(move |n| (src.clone(), mode, 16 << n)))
            .par_bridge()
            .for_each(|(src, _mode, len)| {
                let mut comp_buf = vec![0; len];
                let mut decomp_buf = Vec::new();
                let (read, wrote) = lz4::compress_fill(&src, &mut comp_buf).unwrap();
                decomp_buf.resize(read, 0);
                let len = lz4::decompress(&comp_buf[..wrote], &mut decomp_buf).unwrap();
                assert_eq!(len, read);
                assert!(src.starts_with(&decomp_buf));
            });
    }
}

mod compress_to_vec {
    use super::*;

    #[test]
    fn default() {
        lz4_test_set().par_bridge().for_each(|(src, mode)| {
            let header = &b"HEADER"[..];
            let mut comp_buf = Vec::from(header);
            let mut decomp_buf = vec![0; src.len()];
            lz4::compress_to_vec(&src, &mut comp_buf, mode).unwrap();
            assert!(comp_buf.starts_with(header));
            lz4::decompress(&comp_buf[header.len()..], &mut decomp_buf).unwrap();
            assert_eq!(decomp_buf, src);
        });
    }
}

mod decompress {
    use super::*;

    #[test]
    fn default() {
        lz4_test_set().par_bridge().for_each(|(src, mode)| {
            let mut comp_buf = Vec::new();
            let mut decomp_buf = vec![0; src.len()];
            lz4::compress_to_vec(&src, &mut comp_buf, mode).unwrap();
            lz4::decompress(&comp_buf, &mut decomp_buf).unwrap();
            assert_eq!(src, &decomp_buf);
        });
    }

    #[test]
    fn partial() {
        lz4_test_set()
            .flat_map(|(src, mode)| (0..20).map(move |n| (src.clone(), mode, 16 << n)))
            .par_bridge()
            .for_each(|(src, mode, len)| {
                let mut comp_buf = Vec::new();
                let mut decomp_buf = vec![0; cmp::min(src.len(), len)];
                lz4::compress_to_vec(&src, &mut comp_buf, mode).unwrap();
                lz4::decompress_partial(&comp_buf, &mut decomp_buf, src.len()).unwrap();
                assert!(src.starts_with(&decomp_buf));
            });
    }

    #[test]
    fn with_dict_and_dict_slow() {
        lz4_test_set().par_bridge().for_each(|(src, mode)| {
            for with_dict in [lz4::Compressor::with_dict, lz4::Compressor::with_dict_slow] {
                let mut comp_buf = Vec::new();
                let mut decomp_buf = vec![0; src.len()];
                let mut comp = with_dict(src.as_ref()).unwrap();
                comp.next_to_vec(&src, &mut comp_buf, mode).unwrap();
                lz4::decompress_with_dict(&comp_buf, &mut decomp_buf, &src).unwrap();
                assert_eq!(src, &decomp_buf);
            }
        });
    }
}
