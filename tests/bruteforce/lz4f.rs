use lzzzz::lz4f;
use lzzzz::lz4f::{
    AutoFlush, BlockChecksum, BlockMode, BlockSize, CompressionLevel, ContentChecksum, Preferences,
};
use rand::{distributions::Standard, rngs::SmallRng, Rng, SeedableRng};
use rayon::prelude::*;

#[test]
fn parallel_compression_decompression() {
    let all_ok = (0..4095usize)
        .into_par_iter()
        .map(|n| {
            let rng = SmallRng::seed_from_u64(n as u64);
            rng.sample_iter(Standard).take(n).collect::<Vec<_>>()
        })
        .map(|plain| {
            preferences(4095).map(move |pref| {
                let mut comp = Vec::new();
                lz4f::compress_to_vec(&plain, &mut comp, &pref).unwrap();
                (plain.clone(), comp)
            })
        })
        .flatten()
        .map(|(plain, comp)| {
            let mut decomp = Vec::new();
            lz4f::decompress_to_vec(&comp, &mut decomp).unwrap();
            (plain, decomp)
        })
        .all(|(plain, decomp)| plain == decomp);
    assert!(all_ok);
}

fn preferences(n: usize) -> impl ParallelIterator<Item = Preferences> {
    (0..n).into_par_iter().map(|n| {
        let mut rng = SmallRng::seed_from_u64(n as u64);
        lz4f::PreferencesBuilder::new()
            /*
            .auto_flush(if rng.gen_bool(0.5) {
                AutoFlush::Disabled
            } else {
                AutoFlush::Enabled
            })
            .block_checksum(if rng.gen_bool(0.5) {
                BlockChecksum::Disabled
            } else {
                BlockChecksum::Enabled
            })
            .content_checksum(if rng.gen_bool(0.5) {
                ContentChecksum::Disabled
            } else {
                ContentChecksum::Enabled
            })
            .block_mode(if rng.gen_bool(0.5) {
                BlockMode::Independent
            } else {
                BlockMode::Linked
            })
            .block_size(match rng.gen_range(0, 5) {
                0 => BlockSize::Max1MB,
                1 => BlockSize::Max4MB,
                2 => BlockSize::Max64KB,
                3 => BlockSize::Max256KB,
                _ => BlockSize::Default,
            })
            .dict_id(rng.gen())
            */
            .compression_level(CompressionLevel::Custom(rng.gen()))
            .build()
    })
}
