#![cfg(feature = "lz4-hc")]

use lzzzz::{lz4, lz4_hc};
use rayon::{iter::ParallelBridge, prelude::*};

mod common;
use common::lz4_hc_test_set;

mod compress {
    use super::*;

    #[test]
    fn default() {
        lz4_hc_test_set().par_bridge().for_each(|(src, level)| {
            let mut comp_buf = vec![0; lz4::max_compressed_size(src.len())];
            let mut decomp_buf = vec![0; src.len()];
            let len =
                lz4_hc::compress(&src, &mut comp_buf, lz4_hc::CompressionMode::Default, level)
                    .unwrap()
                    .dst_len();
            lz4::decompress(
                &comp_buf[..len],
                &mut decomp_buf,
                lz4::DecompressionMode::Default,
            )
            .unwrap();
            assert_eq!(decomp_buf, src);
        });
    }

    #[test]
    fn partial() {
        lz4_hc_test_set()
            .map(|(src, level)| (0..20).map(move |n| (src.clone(), level, 16 << n)))
            .flatten()
            .par_bridge()
            .for_each(|(src, level, len)| {
                let mut comp_buf = vec![0; len];
                let mut decomp_buf = Vec::new();
                let report =
                    lz4_hc::compress(&src, &mut comp_buf, lz4_hc::CompressionMode::Partial, level)
                        .unwrap();
                decomp_buf.resize(report.src_len().unwrap(), 0);
                lz4::decompress(
                    &comp_buf[..report.dst_len()],
                    &mut decomp_buf,
                    lz4::DecompressionMode::Default,
                )
                .unwrap();
                assert!(src.starts_with(&decomp_buf));
            });
    }
}

mod compress_to_vec {
    use super::*;

    #[test]
    fn default() {
        lz4_hc_test_set().par_bridge().for_each(|(src, level)| {
            let mut comp_buf = Vec::new();
            let mut decomp_buf = vec![0; src.len()];
            let len = lz4_hc::compress_to_vec(&src, &mut comp_buf, level)
                .unwrap()
                .dst_len();
            lz4::decompress(
                &comp_buf[..len],
                &mut decomp_buf,
                lz4::DecompressionMode::Default,
            )
            .unwrap();
            assert_eq!(decomp_buf, src);
        });
    }
}
