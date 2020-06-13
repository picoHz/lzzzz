use lzzzz::{
    lz4f,
    lz4f::{
        AutoFlush, BlockChecksum, BlockMode, BlockSize, CompressionLevel, ContentChecksum,
        FavorDecSpeed, PreferencesBuilder,
    },
};
use rand::{distributions::Standard, rngs::SmallRng, Rng, SeedableRng};

#[test]
fn parallel_compression_decompression() {
    super::run(|state| {
        let data = generate_data(state, 1024);
        let pref = generate_preference(state).build();
        let err = |_| (data.clone(), pref);

        let mut comp = Vec::new();
        lz4f::compress_to_vec(&data, &mut comp, &pref).map_err(err)?;

        let mut decomp = Vec::new();
        lz4f::decompress_to_vec(&comp, &mut decomp).map_err(err)?;

        if decomp == data {
            Ok(())
        } else {
            Err((data, pref))
        }
    });
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
