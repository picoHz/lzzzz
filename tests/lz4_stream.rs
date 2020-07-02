#![cfg(feature = "lz4")]

use lzzzz::lz4;
use rayon::{iter::ParallelBridge, prelude::*};

mod common;
use common::lz4_stream_test_set;

mod compressor {
    use super::*;

    #[test]
    fn default() {
        lz4_stream_test_set()
            .par_bridge()
            .for_each(|(src_set, mode)| {
                for src in src_set {
                    let mut comp_buf = vec![0; lz4::max_compressed_size(src.len())];
                    let mut decomp_buf = vec![0; src.len()];

                    let mut stream = lz4::Compressor::new().unwrap();
                    let len = stream
                        .next(src.as_ref(), &mut comp_buf, mode)
                        .unwrap()
                        .dst_len();

                    lz4::decompress(
                        &comp_buf[..len],
                        &mut decomp_buf,
                        lz4::DecompressionMode::Default,
                    )
                    .unwrap();
                    assert_eq!(decomp_buf, src);
                }
            });
    }
}
