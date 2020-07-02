#![cfg(feature = "lz4-hc")]

use lzzzz::{lz4, lz4_hc};
use rayon::{iter::ParallelBridge, prelude::*};

mod common;
use common::lz4_hc_stream_test_set;

mod compressor {
    use super::*;

    #[test]
    fn default() {
        lz4_hc_stream_test_set()
            .par_bridge()
            .for_each(|(src_set, level)| {
                let mut stream = lz4_hc::Compressor::new().unwrap();
                stream.set_compression_level(level);
                for src in src_set {
                    let mut comp_buf = vec![0; lz4::max_compressed_size(src.len())];
                    let mut decomp_buf = vec![0; src.len()];

                    let len = stream
                        .next(
                            Vec::from(src.as_ref()),
                            &mut comp_buf,
                            lz4_hc::CompressionMode::Default,
                        )
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
