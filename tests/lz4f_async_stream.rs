#![cfg(all(feature = "lz4f", feature = "tokio-io"))]

use futures::future::join_all;
use lzzzz::{lz4f, lz4f::*};
use rand::{distributions::Standard, rngs::SmallRng, Rng, SeedableRng};

mod common;
use common::lz4f_test_set;

mod async_read_compressor {
    use super::*;
    use lzzzz::lz4f::{comp::AsyncReadCompressor, CompressorBuilder};
    use tokio::io::AsyncReadExt;

    #[tokio::test]
    async fn default() {
        join_all(lz4f_test_set().map(|(src, prefs)| async move {
            let mut comp_buf = Vec::new();
            let mut decomp_buf = Vec::new();
            {
                let mut src = src.as_ref();
                let mut r = CompressorBuilder::new(&mut src)
                    .preferences(prefs)
                    .build::<AsyncReadCompressor<_>>()
                    .unwrap();
                r.read_to_end(&mut comp_buf).await.unwrap();
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

    #[tokio::test]
    async fn random_chunk() {
        join_all(lz4f_test_set().map(|(src, prefs)| async move {
            let mut comp_buf = Vec::new();
            let mut decomp_buf = Vec::new();
            {
                let mut src = src.as_ref();
                let mut r = CompressorBuilder::new(&mut src)
                    .preferences(prefs)
                    .build::<AsyncReadCompressor<_>>()
                    .unwrap();

                let mut offset = 0;
                let mut rng = SmallRng::seed_from_u64(0);

                loop {
                    if offset >= comp_buf.len() {
                        comp_buf.resize_with(offset + 1024, Default::default);
                    }
                    let len = rng.gen_range(0, comp_buf.len() - offset + 1);
                    let dst = &mut comp_buf[offset..][..len];
                    let len = r.read(dst).await.unwrap();
                    if dst.len() > 0 && len == 0 {
                        break;
                    }
                    offset += len;
                }
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

mod async_bufread_compressor {
    use super::*;
    use lzzzz::lz4f::{comp::AsyncBufReadCompressor, CompressorBuilder};
    use tokio::io::AsyncReadExt;

    #[tokio::test]
    async fn default() {
        join_all(lz4f_test_set().map(|(src, prefs)| async move {
            let mut comp_buf = Vec::new();
            let mut decomp_buf = Vec::new();
            {
                let mut src = src.as_ref();
                let mut r = CompressorBuilder::new(&mut src)
                    .preferences(prefs)
                    .build::<AsyncBufReadCompressor<_>>()
                    .unwrap();
                r.read_to_end(&mut comp_buf).await.unwrap();
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

    #[tokio::test]
    async fn random_chunk() {
        join_all(lz4f_test_set().map(|(src, prefs)| async move {
            let mut comp_buf = Vec::new();
            let mut decomp_buf = Vec::new();
            {
                let mut src = src.as_ref();
                let mut r = CompressorBuilder::new(&mut src)
                    .preferences(prefs)
                    .build::<AsyncBufReadCompressor<_>>()
                    .unwrap();

                let mut offset = 0;
                let mut rng = SmallRng::seed_from_u64(0);

                loop {
                    if offset >= comp_buf.len() {
                        comp_buf.resize_with(offset + 1024, Default::default);
                    }
                    let len = rng.gen_range(0, comp_buf.len() - offset + 1);
                    let dst = &mut comp_buf[offset..][..len];
                    let len = r.read(dst).await.unwrap();
                    if dst.len() > 0 && len == 0 {
                        break;
                    }
                    offset += len;
                }
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

mod async_write_compressor {
    use super::*;
    use futures::future::join_all;
    use lzzzz::lz4f::{comp::AsyncWriteCompressor, CompressorBuilder};
    use tokio::io::AsyncWriteExt;

    #[tokio::test]
    async fn default() {
        join_all(lz4f_test_set().map(|(src, prefs)| async move {
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

mod async_read_decompressor {
    use super::*;
    use futures::future::join_all;
    use lzzzz::lz4f::{comp::WriteCompressor, decomp::AsyncReadDecompressor};
    use std::io::Write;
    use tokio::io::AsyncReadExt;

    #[tokio::test]
    async fn default() {
        join_all(lz4f_test_set().map(|(src, prefs)| async move {
            let mut comp_buf = Vec::new();
            let mut decomp_buf = Vec::new();
            assert_eq!(
                lz4f::compress_to_vec(&src, &mut comp_buf, &prefs)
                    .unwrap()
                    .dst_len(),
                comp_buf.len()
            );
            {
                let mut src = comp_buf.as_slice();
                let mut r = AsyncReadDecompressor::new(&mut src).unwrap();
                r.read_to_end(&mut decomp_buf).await.unwrap();
            }
            assert_eq!(decomp_buf, src);
        }))
        .await;
    }

    #[tokio::test]
    async fn dictionary() {
        join_all(lz4f_test_set().map(|(src, prefs)| async move {
            let mut comp_buf = Vec::new();
            let mut decomp_buf = Vec::new();
            let dict = SmallRng::seed_from_u64(0)
                .sample_iter(Standard)
                .take(64_000)
                .collect::<Vec<_>>();
            {
                let mut w = CompressorBuilder::new(&mut comp_buf)
                    .preferences(prefs)
                    .dict(Dictionary::new(&dict).unwrap())
                    .build::<WriteCompressor<_>>()
                    .unwrap();
                w.write_all(&src).unwrap();
            }
            {
                let mut src = comp_buf.as_slice();
                let mut r = AsyncReadDecompressor::new(&mut src).unwrap();
                r.set_dict(&dict);
                r.read_to_end(&mut decomp_buf).await.unwrap();
            }
            assert_eq!(decomp_buf, src);
        }))
        .await;
    }

    #[tokio::test]
    async fn random_chunk() {
        join_all(lz4f_test_set().map(|(src, prefs)| async move {
            let mut comp_buf = Vec::new();
            let mut decomp_buf = vec![0; src.len()];
            assert_eq!(
                lz4f::compress_to_vec(&src, &mut comp_buf, &prefs)
                    .unwrap()
                    .dst_len(),
                comp_buf.len()
            );
            {
                let mut src = comp_buf.as_slice();
                let mut r = AsyncReadDecompressor::new(&mut src).unwrap();

                let mut offset = 0;
                let mut rng = SmallRng::seed_from_u64(0);

                let dst_len = decomp_buf.len();
                while offset < dst_len {
                    let dst = &mut decomp_buf[offset..][..rng.gen_range(0, dst_len - offset + 1)];
                    let len = r.read(dst).await.unwrap();
                    assert!(dst.len() == 0 || len > 0);
                    offset += len;
                }
            }
            assert_eq!(decomp_buf.len(), src.len());
        }))
        .await;
    }
}

mod async_bufread_decompressor {
    use super::*;
    use futures::future::join_all;
    use lzzzz::lz4f::{comp::WriteCompressor, decomp::AsyncBufReadDecompressor};
    use std::io::Write;
    use tokio::io::AsyncReadExt;

    #[tokio::test]
    async fn default() {
        join_all(lz4f_test_set().map(|(src, prefs)| async move {
            let mut comp_buf = Vec::new();
            let mut decomp_buf = Vec::new();
            assert_eq!(
                lz4f::compress_to_vec(&src, &mut comp_buf, &prefs)
                    .unwrap()
                    .dst_len(),
                comp_buf.len()
            );
            {
                let mut src = comp_buf.as_slice();
                let mut r = AsyncBufReadDecompressor::new(&mut src).unwrap();
                r.read_to_end(&mut decomp_buf).await.unwrap();
            }
            assert_eq!(decomp_buf, src);
        }))
        .await;
    }

    #[tokio::test]
    async fn dictionary() {
        join_all(lz4f_test_set().map(|(src, prefs)| async move {
            let mut comp_buf = Vec::new();
            let mut decomp_buf = Vec::new();
            let dict = SmallRng::seed_from_u64(0)
                .sample_iter(Standard)
                .take(64_000)
                .collect::<Vec<_>>();
            {
                let mut w = CompressorBuilder::new(&mut comp_buf)
                    .preferences(prefs)
                    .dict(Dictionary::new(&dict).unwrap())
                    .build::<WriteCompressor<_>>()
                    .unwrap();
                w.write_all(&src).unwrap();
            }
            {
                let mut src = comp_buf.as_slice();
                let mut r = AsyncBufReadDecompressor::new(&mut src).unwrap();
                r.set_dict(&dict);
                r.read_to_end(&mut decomp_buf).await.unwrap();
            }
            assert_eq!(decomp_buf, src);
        }))
        .await;
    }

    #[tokio::test]
    async fn random_chunk() {
        join_all(lz4f_test_set().map(|(src, prefs)| async move {
            let mut comp_buf = Vec::new();
            let mut decomp_buf = vec![0; src.len()];
            assert_eq!(
                lz4f::compress_to_vec(&src, &mut comp_buf, &prefs)
                    .unwrap()
                    .dst_len(),
                comp_buf.len()
            );
            {
                let mut src = comp_buf.as_slice();
                let mut r = AsyncBufReadDecompressor::new(&mut src).unwrap();

                let mut offset = 0;
                let mut rng = SmallRng::seed_from_u64(0);

                let dst_len = decomp_buf.len();
                while offset < dst_len {
                    let dst = &mut decomp_buf[offset..][..rng.gen_range(0, dst_len - offset + 1)];
                    let len = r.read(dst).await.unwrap();
                    assert!(dst.len() == 0 || len > 0);
                    offset += len;
                }
            }
            assert_eq!(decomp_buf.len(), src.len());
        }))
        .await;
    }
}

mod async_write_decompressor {
    use super::*;
    use futures::future::join_all;
    use lzzzz::lz4f::{comp::WriteCompressor, decomp::AsyncWriteDecompressor};
    use std::io::Write;
    use tokio::io::AsyncWriteExt;

    #[tokio::test]
    async fn default() {
        join_all(lz4f_test_set().map(|(src, prefs)| async move {
            let mut comp_buf = Vec::new();
            let mut decomp_buf = Vec::new();
            assert_eq!(
                lz4f::compress_to_vec(&src, &mut comp_buf, &prefs)
                    .unwrap()
                    .dst_len(),
                comp_buf.len()
            );
            {
                let mut w = AsyncWriteDecompressor::new(&mut decomp_buf).unwrap();
                w.write_all(&comp_buf).await.unwrap();
            }
            assert_eq!(decomp_buf, src);
        }))
        .await;
    }

    #[tokio::test]
    async fn dictionary() {
        join_all(lz4f_test_set().map(|(src, prefs)| async move {
            let mut comp_buf = Vec::new();
            let mut decomp_buf = Vec::new();
            let dict = SmallRng::seed_from_u64(0)
                .sample_iter(Standard)
                .take(64_000)
                .collect::<Vec<_>>();
            {
                let mut w = CompressorBuilder::new(&mut comp_buf)
                    .preferences(prefs)
                    .dict(Dictionary::new(&dict).unwrap())
                    .build::<WriteCompressor<_>>()
                    .unwrap();
                w.write_all(&src).unwrap();
            }
            {
                let mut w = AsyncWriteDecompressor::new(&mut decomp_buf).unwrap();
                w.set_dict(&dict);
                w.write_all(&comp_buf).await.unwrap();
            }
            assert_eq!(decomp_buf, src);
        }))
        .await;
    }

    #[tokio::test]
    async fn random_chunk() {
        join_all(lz4f_test_set().map(|(src, prefs)| async move {
            let mut comp_buf = Vec::new();
            let mut decomp_buf = Vec::new();
            assert_eq!(
                lz4f::compress_to_vec(&src, &mut comp_buf, &prefs)
                    .unwrap()
                    .dst_len(),
                comp_buf.len()
            );
            {
                let mut w = AsyncWriteDecompressor::new(&mut decomp_buf).unwrap();

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
        join_all(lz4f_test_set().map(|(src, prefs)| async move {
            let mut comp_buf = Vec::new();
            let mut decomp_buf = Vec::new();
            assert_eq!(
                lz4f::compress_to_vec(&src, &mut comp_buf, &prefs)
                    .unwrap()
                    .dst_len(),
                comp_buf.len()
            );
            {
                let mut w = AsyncWriteDecompressor::new(&mut decomp_buf).unwrap();
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
