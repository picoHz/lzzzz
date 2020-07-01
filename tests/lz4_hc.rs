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
}
