#![cfg(feature = "lz4")]

use lzzzz::lz4;
use rand::{distributions::Standard, rngs::SmallRng, Rng, SeedableRng};
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
                let mut stream = lz4::Compressor::new().unwrap();
                for src in src_set {
                    let mut comp_buf = vec![0; lz4::max_compressed_size(src.len())];
                    let mut decomp_buf = vec![0; src.len()];

                    let len = stream
                        .next(Vec::from(src.as_ref()), &mut comp_buf, mode)
                        .unwrap()
                        .dst_len();

                    lz4::decompress(&comp_buf[..len], &mut decomp_buf).unwrap();
                    assert_eq!(decomp_buf, src);
                }
            });
    }

    #[test]
    fn dictionary() {
        lz4_stream_test_set()
            .par_bridge()
            .for_each(|(src_set, mode)| {
                let dict = SmallRng::seed_from_u64(0)
                    .sample_iter(Standard)
                    .take(64_000)
                    .collect::<Vec<_>>();
                let mut stream = lz4::Compressor::with_dict(&dict).unwrap();
                for src in src_set {
                    let mut comp_buf = vec![0; lz4::max_compressed_size(src.len())];
                    let len = stream
                        .next(Vec::from(src.as_ref()), &mut comp_buf, mode)
                        .unwrap()
                        .dst_len();

                    let mut decomp_buf = vec![0; src.len()];
                    lz4::decompress_with_dict(&comp_buf[..len], &mut decomp_buf, &dict).unwrap();
                    assert_eq!(src, decomp_buf);
                }
            });
    }
}
