mod compress_to_vec {
    use lzzzz::{lz4f, lz4f::Preferences};
    use rand::{distributions::Standard, rngs::SmallRng, Rng, SeedableRng};
    use std::io::Result;

    fn test(src: &[u8], prefs: &lz4f::Preferences) {
        (|| -> Result<()> {
            let mut comp_buf = Vec::new();
            let mut decomp_buf = Vec::new();

            lz4f::compress_to_vec(src, &mut comp_buf, prefs)?;
            lz4f::decompress_to_vec(&comp_buf, &mut decomp_buf)?;
            assert_eq!(decomp_buf.as_slice(), src);

            Ok(())
        })()
        .unwrap();
    }

    pub fn generate_data() -> impl Iterator<Item = impl Iterator<Item = u8>> {
        (0..24).map(|n| {
            let rng = SmallRng::seed_from_u64(n as u64);
            rng.sample_iter(Standard).take(2 << n)
        })
    }

    #[test]
    fn default() {
        let prefs = Preferences::default();
        test(&[], &prefs);
        test("Hello world!".as_bytes(), &prefs);
        for i in generate_data() {
            test(&i.collect::<Vec<_>>(), &prefs);
        }
    }
}
