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
        run_dictionary_test(|dict| lz4::Compressor::with_dict(dict))
    }
    
    #[test]
    fn dictionary_slow() {
        run_dictionary_test(|dict| lz4::Compressor::with_dict_slow(dict))
    }

    /// Helper function to run dictionary compression tests with either normal or slow mode
    fn run_dictionary_test<F>(compressor_factory: F)
    where
        F: Fn(&[u8]) -> Result<lz4::Compressor, lzzzz::Error> + Sync,
    {
        lz4_stream_test_set()
            .par_bridge()
            .for_each(|(src_set, mode)| {
                let dict = SmallRng::seed_from_u64(0)
                    .sample_iter(Standard)
                    .take(64 * 1024)
                    .collect::<Vec<_>>();
                
                let mut comp = compressor_factory(&dict).unwrap();
                let mut decomp = lz4::Decompressor::with_dict(&dict).unwrap();
                
                for src in src_set {
                    let mut comp_buf = vec![0; lz4::max_compressed_size(src.len())];
                    let len = comp.next(&src, &mut comp_buf, mode).unwrap();
                    assert_eq!(decomp.next(&comp_buf[..len], src.len()).unwrap(), &src);
                }
            });
    }

    #[test]
    fn attach_dictionary() {
        // Basic test data
        let data = b"The quick brown fox jumps over the lazy dog";

        // Create dictionary stream
        let mut dict_comp = lz4::Compressor::with_dict(data).unwrap();
        
        // Create working stream and attach dictionary
        let mut comp = lz4::Compressor::new().unwrap();

        // Compress with attached dictionary
        comp.attach_dict(Some(&mut dict_comp));
        let mut output_attached_dict = Vec::new();
        comp.next_to_vec(data, &mut output_attached_dict, lz4::ACC_LEVEL_DEFAULT).unwrap();

        // Compress with no dictionary
        comp.attach_dict(None);
        let mut output_no_dict = Vec::new();
        comp.next_to_vec(data, &mut output_no_dict, lz4::ACC_LEVEL_DEFAULT).unwrap();

        // Verify the data compresses to same result as regular dictionary compression
        let mut comp_regular = lz4::Compressor::with_dict(data).unwrap();
        let mut output_regular_dict = Vec::new();
        comp_regular.next_to_vec(data, &mut output_regular_dict, lz4::ACC_LEVEL_DEFAULT).unwrap();

        // Results should match
        assert_eq!(output_attached_dict, output_regular_dict, "Compressed data should match");
        assert_ne!(output_attached_dict, output_no_dict, "Data with no dict should be different");
    }
}
