use lzzzz::lz4f::PreferencesBuilder;
use lzzzz::{
    lz4f,
    lz4f::{
        AutoFlush, BlockChecksum, BlockMode, BlockSize, CompressionLevel, ContentChecksum,
        FavorDecSpeed, Preferences,
    },
};
use rand::{
    distributions::{Distribution, Standard},
    prelude::*,
    rngs::SmallRng,
    Rng, SeedableRng,
};
use rayon::prelude::*;

#[test]
fn parallel_compression_decompression() {
    let all_ok = (0..40usize)
        .into_par_iter()
        .map(|n| {
            let rng = SmallRng::seed_from_u64(n as u64);
            rng.sample_iter(Standard).take(n).collect::<Vec<_>>()
        })
        .map(|plain| {
            let pref = generate_preference(0).build();
            let mut comp = Vec::new();
            lz4f::compress_to_vec(&plain, &mut comp, &pref).unwrap();
            (plain, comp)
        })
        .map(|(plain, comp)| {
            let mut decomp = Vec::new();
            lz4f::decompress_to_vec(&comp, &mut decomp).unwrap();
            (plain, decomp)
        })
        .all(|(plain, decomp)| plain == decomp);
    assert!(all_ok);
}

fn compression_decompression(state: u64) -> Result<(), (Vec<u8>, Preferences)> {
    let data = generate_data(state, 1024);
    let pref = generate_preference(state).build();
    let mut comp = Vec::new();
    lz4f::compress_to_vec(&data, &mut comp, &pref).map_err(|_| (data.clone(), pref))?;
    let mut decomp = Vec::new();
    lz4f::decompress_to_vec(&comp, &mut decomp).map_err(|_| (data.clone(), pref))?;
    if decomp == data {
        Ok(())
    } else {
        Err((data.clone(), pref))
    }
}

pub fn generate_data(state: u64, max_len: usize) -> Vec<u8> {
    let mut rng = SmallRng::seed_from_u64(state);
    let len = rng.gen_range(0, max_len + 1);
    rng.sample_iter(Standard).take(len).collect::<Vec<_>>()
}

pub fn generate_preference(state: u64) -> PreferencesBuilder {
    let mut rng = SmallRng::seed_from_u64(state);
    lz4f::PreferencesBuilder::new()
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
        .favor_dec_speed(if rng.gen_bool(0.5) {
            FavorDecSpeed::Disabled
        } else {
            FavorDecSpeed::Enabled
        })
        .compression_level(CompressionLevel::Custom(rng.gen()))
        .dict_id(rng.gen())
}
