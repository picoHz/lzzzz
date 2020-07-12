#![cfg(feature = "lz4-hc")]

use lzzzz::{lz4, lz4_hc};
use rand::{distributions::Standard, rngs::SmallRng, Rng, SeedableRng};
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
                let mut comp = lz4_hc::Compressor::new().unwrap();
                let mut decomp = lz4::Decompressor::new().unwrap();
                comp.set_compression_level(level);
                for src in src_set {
                    let mut comp_buf = vec![0; lz4::max_compressed_size(src.len())];
                    let len = comp.next(Vec::from(src.as_ref()), &mut comp_buf).unwrap();
                    assert_eq!(
                        decomp.next(&comp_buf[..len], src.len()).unwrap(),
                        src.as_ref()
                    );
                }
            });
    }

    #[test]
    fn dictionary() {
        lz4_hc_stream_test_set()
            .par_bridge()
            .for_each(|(src_set, level)| {
                let dict = SmallRng::seed_from_u64(0)
                    .sample_iter(Standard)
                    .take(64_000)
                    .collect::<Vec<_>>();
                let mut comp = lz4_hc::Compressor::with_dict(&dict).unwrap();
                let mut decomp = lz4::Decompressor::with_dict(&dict).unwrap();
                comp.set_compression_level(level);
                for src in src_set {
                    let mut comp_buf = vec![0; lz4::max_compressed_size(src.len())];
                    let len = comp.next(Vec::from(src.as_ref()), &mut comp_buf).unwrap();
                    assert_eq!(
                        decomp.next(&comp_buf[..len], src.len()).unwrap(),
                        src.as_ref()
                    );
                }
            });
    }

    #[test]
    fn dynamic_adaptation() {
        lz4_hc_stream_test_set()
            .par_bridge()
            .for_each(|(src_set, _)| {
                let mut comp = lz4_hc::Compressor::new().unwrap();
                let mut decomp = lz4::Decompressor::new().unwrap();
                let mut rng = SmallRng::seed_from_u64(0);
                for src in src_set {
                    let mut comp_buf = vec![0; lz4::max_compressed_size(src.len())];

                    comp.set_compression_level(rng.gen());
                    comp.set_favor_dec_speed(if rng.gen_bool(0.5) {
                        lz4_hc::FavorDecSpeed::Enabled
                    } else {
                        lz4_hc::FavorDecSpeed::Disabled
                    });

                    let len = comp.next(Vec::from(src.as_ref()), &mut comp_buf).unwrap();
                    assert_eq!(
                        decomp.next(&comp_buf[..len], src.len()).unwrap(),
                        src.as_ref()
                    );
                }
            });
    }
}
