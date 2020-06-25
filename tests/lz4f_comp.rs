use lzzzz::{
    lz4f,
    lz4f::{
        AutoFlush, BlockChecksum, BlockMode, BlockSize, CompressionLevel, ContentChecksum, Error,
        ErrorKind, FavorDecSpeed, Preferences, PreferencesBuilder,
    },
};
use rand::{distributions::Standard, rngs::SmallRng, Rng, SeedableRng};
use rayon::prelude::*;
use std::{i32, u32};

fn generate_data() -> impl Iterator<Item = Vec<u8>> {
    (0..20).map(|n| {
        let rng = SmallRng::seed_from_u64(n as u64);
        rng.sample_iter(Standard).take(16 << n).collect()
    })
}

fn preferences_set() -> Vec<Preferences> {
    vec![
        PreferencesBuilder::new().build(),
        PreferencesBuilder::new()
            .block_size(BlockSize::Max64KB)
            .build(),
        PreferencesBuilder::new()
            .block_size(BlockSize::Max256KB)
            .build(),
        PreferencesBuilder::new()
            .block_size(BlockSize::Max1MB)
            .build(),
        PreferencesBuilder::new()
            .block_size(BlockSize::Max4MB)
            .build(),
        PreferencesBuilder::new()
            .block_mode(BlockMode::Independent)
            .build(),
        PreferencesBuilder::new()
            .content_checksum(ContentChecksum::Enabled)
            .build(),
        PreferencesBuilder::new().dict_id(u32::MAX).build(),
        PreferencesBuilder::new()
            .block_checksum(BlockChecksum::Enabled)
            .build(),
        PreferencesBuilder::new()
            .compression_level(CompressionLevel::High)
            .build(),
        PreferencesBuilder::new()
            .compression_level(CompressionLevel::Max)
            .build(),
        PreferencesBuilder::new()
            .compression_level(CompressionLevel::Custom(i32::MAX))
            .build(),
        PreferencesBuilder::new()
            .compression_level(CompressionLevel::Custom(i32::MIN))
            .build(),
        PreferencesBuilder::new()
            .favor_dec_speed(FavorDecSpeed::Enabled)
            .build(),
        PreferencesBuilder::new()
            .auto_flush(AutoFlush::Enabled)
            .build(),
    ]
}

#[test]
fn compress_decompress_to_vec() {
    preferences_set().par_iter().for_each(|prefs| {
        for src in generate_data() {
            let mut comp_buf = Vec::new();
            let mut decomp_buf = Vec::new();

            assert_eq!(
                lz4f::compress_to_vec(&src, &mut comp_buf, prefs)
                    .unwrap()
                    .dst_len(),
                comp_buf.len()
            );
            assert_eq!(
                lz4f::decompress_to_vec(&comp_buf, &mut decomp_buf)
                    .unwrap()
                    .dst_len(),
                decomp_buf.len()
            );
            assert_eq!(decomp_buf, src);
        }
    });
}

#[test]
fn compress() {
    preferences_set().par_iter().for_each(|prefs| {
        for src in generate_data() {
            let mut comp_buf = Vec::new();
            let mut decomp_buf = Vec::new();

            comp_buf.resize_with(
                lz4f::max_compressed_size(src.len(), &prefs),
                Default::default,
            );
            let len = lz4f::compress(&src, &mut comp_buf, prefs)
                .unwrap()
                .dst_len();
            comp_buf.resize_with(len, Default::default);
            assert_eq!(
                lz4f::decompress_to_vec(&comp_buf, &mut decomp_buf)
                    .unwrap()
                    .dst_len(),
                decomp_buf.len()
            );
            assert_eq!(decomp_buf, src);
        }
    });
}

#[test]
fn compress_too_small_dst() {
    preferences_set().par_iter().for_each(|prefs| {
        for src in generate_data() {
            let mut comp_buf = Vec::new();
            assert_eq!(
                lz4f::compress(&src, &mut comp_buf, prefs),
                Err(Error::Lz4f(ErrorKind::DstMaxSizeTooSmall))
            );
        }
    });
}

#[test]
fn decompress_to_vec_invalid_header() {
    preferences_set().par_iter().for_each(|prefs| {
        for src in generate_data() {
            let header = Vec::from("hello!".as_bytes());
            let mut comp_buf = Vec::new();
            let mut decomp_buf = header.clone();
            assert_eq!(
                lz4f::compress_to_vec(&src, &mut comp_buf, prefs)
                    .unwrap()
                    .dst_len(),
                comp_buf.len()
            );
            assert_eq!(
                lz4f::decompress_to_vec(&comp_buf[1..], &mut decomp_buf),
                Err(Error::Lz4f(ErrorKind::FrameTypeUnknown))
            );
            assert_eq!(decomp_buf, header);
        }
    });
}

#[test]
fn decompress_to_vec_incomplete_src() {
    preferences_set().par_iter().for_each(|prefs| {
        for src in generate_data() {
            let header = Vec::from("hello!".as_bytes());
            let mut comp_buf = Vec::new();
            let mut decomp_buf = header.clone();
            assert_eq!(
                lz4f::compress_to_vec(&src, &mut comp_buf, prefs)
                    .unwrap()
                    .dst_len(),
                comp_buf.len()
            );
            assert_eq!(
                lz4f::decompress_to_vec(&comp_buf[..comp_buf.len() - 1], &mut decomp_buf),
                Err(Error::Common(lzzzz::ErrorKind::CompressedDataIncomplete))
            );
            assert_eq!(decomp_buf, header);
        }
    });
}
