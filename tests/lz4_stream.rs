use lzzzz::lz4;
use rand::{distributions::Standard, rngs::SmallRng, Rng, SeedableRng};
use rayon::{iter::ParallelBridge, prelude::*};
use static_assertions::assert_impl_all;

mod common;
use common::lz4_stream_test_set;

assert_impl_all!(lz4::Compressor: Send);
assert_impl_all!(lz4::Decompressor: Send);

mod compressor {
    use super::*;

    #[test]
    fn default() {
        lz4_stream_test_set()
            .par_bridge()
            .for_each(|(src_set, mode)| {
                let mut comp = lz4::Compressor::new().unwrap();
                let mut decomp = lz4::Decompressor::new().unwrap();
                for src in src_set {
                    let mut comp_buf = vec![0; lz4::max_compressed_size(src.len())];
                    let len = comp.next(&src, &mut comp_buf, mode).unwrap();
                    assert_eq!(decomp.next(&comp_buf[..len], src.len()).unwrap(), &src);
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
                    .take(64 * 1024)
                    .collect::<Vec<_>>();
                let mut comp = lz4::Compressor::with_dict(&dict).unwrap();
                let mut decomp = lz4::Decompressor::with_dict(&dict).unwrap();
                for src in src_set {
                    let mut comp_buf = vec![0; lz4::max_compressed_size(src.len())];
                    let len = comp.next(&src, &mut comp_buf, mode).unwrap();
                    assert_eq!(decomp.next(&comp_buf[..len], src.len()).unwrap(), &src);
                }
            });
    }
}
