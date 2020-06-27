use futures::future::join_all;
use lzzzz::{lz4f, lz4f::*};
use rand::{distributions::Standard, rngs::SmallRng, Rng, SeedableRng};
use std::{i32, u32};

fn generate_data() -> impl Iterator<Item = Vec<u8>> {
    (0..20).map(|n| {
        let rng = SmallRng::seed_from_u64(n as u64);
        rng.sample_iter(Standard).take(16 << n).collect()
    })
}

fn preferences_set() -> impl Iterator<Item = Preferences> {
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
    .into_iter()
}

fn test_set() -> impl Iterator<Item = (Vec<u8>, Preferences)> {
    generate_data()
        .map(|data| preferences_set().map(move |prefs| (data.clone(), prefs)))
        .flatten()
}

mod compress_to_vec {
    use super::*;

    #[tokio::test]
    async fn normal() {
        join_all(
            test_set()
                .map(|(src, prefs)| async move {
                    let header = Vec::from("hello!".as_bytes());
                    let mut comp_buf = header.clone();
                    let mut decomp_buf = header.clone();

                    assert_eq!(
                        lz4f::compress_to_vec(&src, &mut comp_buf, &prefs)
                            .unwrap()
                            .dst_len(),
                        comp_buf.len() - header.len()
                    );
                    assert_eq!(
                        lz4f::decompress_to_vec(&comp_buf[header.len()..], &mut decomp_buf)
                            .unwrap()
                            .dst_len(),
                        decomp_buf.len() - header.len()
                    );
                    assert_eq!(&decomp_buf[header.len()..], &src[..]);
                })
                .map(|task| tokio::spawn(task)),
        )
        .await;
    }
}

mod compress {
    use super::*;

    #[tokio::test]
    async fn normal() {
        join_all(
            test_set()
                .map(|(src, prefs)| async move {
                    let mut comp_buf = Vec::new();
                    let mut decomp_buf = Vec::new();

                    comp_buf.resize_with(
                        lz4f::max_compressed_size(src.len(), &prefs),
                        Default::default,
                    );
                    let len = lz4f::compress(&src, &mut comp_buf, &prefs)
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
                })
                .map(|task| tokio::spawn(task)),
        )
        .await;
    }

    #[tokio::test]
    async fn too_small_dts() {
        join_all(
            test_set()
                .map(|(src, prefs)| async move {
                    let mut comp_buf = Vec::new();
                    assert_eq!(
                        lz4f::compress(&src, &mut comp_buf, &prefs),
                        Err(Error::Lz4f(ErrorKind::DstMaxSizeTooSmall))
                    );
                })
                .map(|task| tokio::spawn(task)),
        )
        .await;
    }
}

mod decompress_to_vec {
    use super::*;

    #[tokio::test]
    async fn invalid_header() {
        join_all(
            test_set()
                .map(|(src, prefs)| async move {
                    let header = Vec::from("hello!".as_bytes());
                    let mut comp_buf = Vec::new();
                    let mut decomp_buf = header.clone();
                    assert_eq!(
                        lz4f::compress_to_vec(&src, &mut comp_buf, &prefs)
                            .unwrap()
                            .dst_len(),
                        comp_buf.len()
                    );
                    assert_eq!(
                        lz4f::decompress_to_vec(&comp_buf[1..], &mut decomp_buf),
                        Err(Error::Lz4f(ErrorKind::FrameTypeUnknown))
                    );
                    assert_eq!(decomp_buf, header);
                })
                .map(|task| tokio::spawn(task)),
        )
        .await;
    }

    #[tokio::test]
    async fn incomplete_src() {
        join_all(
            test_set()
                .map(|(src, prefs)| async move {
                    let header = Vec::from("hello!".as_bytes());
                    let mut comp_buf = Vec::new();
                    let mut decomp_buf = header.clone();
                    assert_eq!(
                        lz4f::compress_to_vec(&src, &mut comp_buf, &prefs)
                            .unwrap()
                            .dst_len(),
                        comp_buf.len()
                    );
                    assert_eq!(
                        lz4f::decompress_to_vec(&comp_buf[..comp_buf.len() - 1], &mut decomp_buf),
                        Err(Error::Common(lzzzz::ErrorKind::CompressedDataIncomplete))
                    );
                    assert_eq!(decomp_buf, header);
                })
                .map(|task| tokio::spawn(task)),
        )
        .await;
    }
}

mod write_compressor {
    use super::*;
    use lzzzz::lz4f::{comp::WriteCompressor, CompressorBuilder};
    use std::io::prelude::*;

    #[tokio::test]
    async fn normal() {
        join_all(
            test_set()
                .map(|(src, prefs)| async move {
                    let mut comp_buf = Vec::new();
                    let mut decomp_buf = Vec::new();
                    {
                        let mut w = CompressorBuilder::new(&mut comp_buf)
                            .preferences(prefs)
                            .build::<WriteCompressor<_>>()
                            .unwrap();
                        w.write_all(&src).unwrap();
                    }
                    assert_eq!(
                        lz4f::decompress_to_vec(&comp_buf, &mut decomp_buf)
                            .unwrap()
                            .dst_len(),
                        decomp_buf.len()
                    );
                    assert_eq!(decomp_buf, src);
                })
                .map(|task| tokio::spawn(task)),
        )
        .await;
    }
}

#[cfg(feature = "use-tokio")]
mod async_write_compressor {
    use super::*;
    use futures::future::join_all;
    use lzzzz::lz4f::{comp::AsyncWriteCompressor, CompressorBuilder};
    use tokio::io::AsyncWriteExt;

    #[tokio::test]
    async fn normal() {
        join_all(test_set().map(|(src, prefs)| async move {
            let mut comp_buf = Vec::new();
            let mut decomp_buf = Vec::new();
            {
                let mut w = CompressorBuilder::new(&mut comp_buf)
                    .preferences(prefs.clone())
                    .build::<AsyncWriteCompressor<_>>()
                    .unwrap();
                w.write_all(&src).await.unwrap();
                w.shutdown().await.unwrap();
            }
            assert_eq!(
                lz4f::decompress_to_vec(&comp_buf, &mut decomp_buf)
                    .unwrap()
                    .dst_len(),
                decomp_buf.len()
            );
            assert_eq!(decomp_buf, src);
        }))
        .await;
    }
}

#[cfg(feature = "use-tokio")]
mod async_write_decompressor {
    use super::*;
    use futures::future::join_all;
    use lzzzz::lz4f::{decomp::AsyncWriteDecompressor, DecompressorBuilder};
    use tokio::io::AsyncWriteExt;

    #[tokio::test]
    async fn normal() {
        join_all(test_set().map(|(src, prefs)| async move {
            let mut comp_buf = Vec::new();
            let mut decomp_buf = Vec::new();
            assert_eq!(
                lz4f::compress_to_vec(&src, &mut comp_buf, &prefs)
                    .unwrap()
                    .dst_len(),
                comp_buf.len()
            );
            {
                let mut w = DecompressorBuilder::new(&mut decomp_buf)
                    .build::<AsyncWriteDecompressor<_>>()
                    .unwrap();
                w.write_all(&comp_buf).await.unwrap();
            }
            assert_eq!(decomp_buf, src);
        }))
        .await;
    }

    #[tokio::test]
    async fn random_chunk() {
        join_all(test_set().map(|(src, prefs)| async move {
            let mut comp_buf = Vec::new();
            let mut decomp_buf = Vec::new();
            assert_eq!(
                lz4f::compress_to_vec(&src, &mut comp_buf, &prefs)
                    .unwrap()
                    .dst_len(),
                comp_buf.len()
            );
            {
                let mut w = DecompressorBuilder::new(&mut decomp_buf)
                    .build::<AsyncWriteDecompressor<_>>()
                    .unwrap();

                let mut offset = 0;
                let mut rng = SmallRng::seed_from_u64(0);

                while offset < comp_buf.len() {
                    let src = &comp_buf[offset..][..rng.gen_range(0, comp_buf.len() - offset + 1)];
                    let len = w.write(src).await.unwrap();
                    assert!(src.len() == 0 || len > 0);
                    offset += len;
                }
            }
            assert_eq!(decomp_buf.len(), src.len());
        }))
        .await;
    }

    #[tokio::test]
    async fn invalid_header() {
        join_all(test_set().map(|(src, prefs)| async move {
            let mut comp_buf = Vec::new();
            let mut decomp_buf = Vec::new();
            assert_eq!(
                lz4f::compress_to_vec(&src, &mut comp_buf, &prefs)
                    .unwrap()
                    .dst_len(),
                comp_buf.len()
            );
            {
                let mut w = DecompressorBuilder::new(&mut decomp_buf)
                    .build::<AsyncWriteDecompressor<_>>()
                    .unwrap();
                let err = w
                    .write_all(&comp_buf[1..])
                    .await
                    .unwrap_err()
                    .into_inner()
                    .unwrap();
                assert_eq!(
                    err.downcast::<lz4f::Error>().unwrap(),
                    Box::new(lz4f::Error::Common(lzzzz::ErrorKind::DecompressionFailed))
                );
            }
        }))
        .await;
    }
}
