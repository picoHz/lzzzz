#![cfg(feature = "async-io")]

use async_std::{fs::File, io::BufReader};
use futures::future::join_all;
use lzzzz::{lz4f, lz4f::*};
use rand::{distributions::Standard, rngs::SmallRng, Rng, SeedableRng};
use static_assertions::assert_impl_all;

mod common;
use common::lz4f_test_set;

assert_impl_all!(lz4f::AsyncBufReadCompressor<BufReader<File>>: Send);
assert_impl_all!(lz4f::AsyncReadCompressor<File>: Send);
assert_impl_all!(lz4f::AsyncWriteCompressor<File>: Send);
assert_impl_all!(lz4f::AsyncBufReadDecompressor<BufReader<File>>: Send);
assert_impl_all!(lz4f::AsyncReadDecompressor<File>: Send);
assert_impl_all!(lz4f::AsyncWriteDecompressor<File>: Send);

mod async_read_compressor {
    use super::*;
    use futures_lite::AsyncReadExt;
    use lzzzz::lz4f::AsyncReadCompressor;

    #[test]
    fn default() {
        smol::run(async {
            join_all(lz4f_test_set().map(|(src, prefs)| async move {
                let mut comp_buf = Vec::new();
                let mut decomp_buf = Vec::new();
                {
                    let mut src = src.as_ref();
                    let mut r = AsyncReadCompressor::new(&mut src, prefs).unwrap();
                    r.read_to_end(&mut comp_buf).await.unwrap();
                }
                assert_eq!(
                    lz4f::decompress_to_vec(&comp_buf, &mut decomp_buf).unwrap(),
                    decomp_buf.len()
                );
                assert_eq!(decomp_buf, src);
            }))
            .await;
        })
    }

    #[test]
    fn random_chunk() {
        smol::run(async {
            join_all(lz4f_test_set().map(|(src, prefs)| async move {
                let mut comp_buf = Vec::new();
                let mut decomp_buf = Vec::new();
                {
                    let mut src = src.as_ref();
                    let mut r = AsyncReadCompressor::new(&mut src, prefs).unwrap();

                    let mut offset = 0;
                    let mut rng = SmallRng::seed_from_u64(0);

                    loop {
                        if offset >= comp_buf.len() {
                            comp_buf.resize_with(offset + 1024, Default::default);
                        }
                        let len = rng.gen_range(0, comp_buf.len() - offset + 1);
                        let dst = &mut comp_buf[offset..][..len];
                        let len = r.read(dst).await.unwrap();
                        if !dst.is_empty() && len == 0 {
                            break;
                        }
                        offset += len;
                    }
                }
                assert_eq!(
                    lz4f::decompress_to_vec(&comp_buf, &mut decomp_buf).unwrap(),
                    decomp_buf.len()
                );
                assert_eq!(decomp_buf, src);
            }))
            .await;
        })
    }
}

mod async_bufread_compressor {
    use super::*;
    use futures_lite::AsyncReadExt;
    use lzzzz::lz4f::AsyncBufReadCompressor;

    #[test]
    fn default() {
        smol::run(async {
            join_all(lz4f_test_set().map(|(src, prefs)| async move {
                let mut comp_buf = Vec::new();
                let mut decomp_buf = Vec::new();
                {
                    let mut src = src.as_ref();
                    let mut r = AsyncBufReadCompressor::new(&mut src, prefs).unwrap();
                    r.read_to_end(&mut comp_buf).await.unwrap();
                }
                assert_eq!(
                    lz4f::decompress_to_vec(&comp_buf, &mut decomp_buf).unwrap(),
                    decomp_buf.len()
                );
                assert_eq!(decomp_buf, src);
            }))
            .await;
        })
    }

    #[test]
    fn random_chunk() {
        smol::run(async {
            join_all(lz4f_test_set().map(|(src, prefs)| async move {
                let mut comp_buf = Vec::new();
                let mut decomp_buf = Vec::new();
                {
                    let mut src = src.as_ref();
                    let mut r = AsyncBufReadCompressor::new(&mut src, prefs).unwrap();

                    let mut offset = 0;
                    let mut rng = SmallRng::seed_from_u64(0);

                    loop {
                        if offset >= comp_buf.len() {
                            comp_buf.resize_with(offset + 1024, Default::default);
                        }
                        let len = rng.gen_range(0, comp_buf.len() - offset + 1);
                        let dst = &mut comp_buf[offset..][..len];
                        let len = r.read(dst).await.unwrap();
                        if !dst.is_empty() && len == 0 {
                            break;
                        }
                        offset += len;
                    }
                }
                assert_eq!(
                    lz4f::decompress_to_vec(&comp_buf, &mut decomp_buf).unwrap(),
                    decomp_buf.len()
                );
                assert_eq!(decomp_buf, src);
            }))
            .await;
        })
    }
}

mod async_write_compressor {
    use super::*;
    use futures::future::join_all;
    use futures_lite::AsyncWriteExt;
    use lzzzz::lz4f::AsyncWriteCompressor;

    #[test]
    fn default() {
        smol::run(async {
            join_all(lz4f_test_set().map(|(src, prefs)| async move {
                let mut comp_buf = Vec::new();
                let mut decomp_buf = Vec::new();
                {
                    let mut w = AsyncWriteCompressor::new(&mut comp_buf, prefs).unwrap();
                    w.write_all(&src).await.unwrap();
                    w.close().await.unwrap();
                }
                assert_eq!(
                    lz4f::decompress_to_vec(&comp_buf, &mut decomp_buf).unwrap(),
                    decomp_buf.len()
                );
                assert_eq!(decomp_buf, src);
            }))
            .await;
        })
    }
}

mod async_read_decompressor {
    use super::*;
    use futures::future::join_all;
    use futures_lite::AsyncReadExt;
    use lzzzz::lz4f::{AsyncReadDecompressor, WriteCompressor};
    use std::io::Write;

    #[test]
    fn default() {
        smol::run(async {
            join_all(lz4f_test_set().map(|(src, prefs)| async move {
                let mut comp_buf = Vec::new();
                let mut decomp_buf = Vec::new();
                assert_eq!(
                    lz4f::compress_to_vec(&src, &mut comp_buf, &prefs).unwrap(),
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
        })
    }

    #[test]
    fn dictionary() {
        smol::run(async {
            join_all(lz4f_test_set().map(|(src, prefs)| async move {
                let mut comp_buf = Vec::new();
                let mut decomp_buf = Vec::new();
                let dict = SmallRng::seed_from_u64(0)
                    .sample_iter(Standard)
                    .take(64_000)
                    .collect::<Vec<_>>();
                {
                    let mut w = WriteCompressor::with_dict(
                        &mut comp_buf,
                        prefs,
                        Dictionary::new(&dict).unwrap(),
                    )
                    .unwrap();
                    w.write_all(&src).unwrap();
                }
                {
                    let mut src = comp_buf.as_slice();
                    let mut r = AsyncReadDecompressor::new(&mut src).unwrap();
                    assert_eq!(
                        r.read_frame_info().await.unwrap().dict_id(),
                        prefs.frame_info().dict_id()
                    );
                    r.set_dict(&dict);
                    r.read_to_end(&mut decomp_buf).await.unwrap();
                }
                assert_eq!(decomp_buf, src);
            }))
            .await;
        })
    }

    #[test]
    fn random_chunk() {
        smol::run(async {
            join_all(lz4f_test_set().map(|(src, prefs)| async move {
                let mut comp_buf = Vec::new();
                let mut decomp_buf = vec![0; src.len()];
                assert_eq!(
                    lz4f::compress_to_vec(&src, &mut comp_buf, &prefs).unwrap(),
                    comp_buf.len()
                );
                {
                    let mut src = comp_buf.as_slice();
                    let mut r = AsyncReadDecompressor::new(&mut src).unwrap();

                    let mut offset = 0;
                    let mut rng = SmallRng::seed_from_u64(0);

                    let dst_len = decomp_buf.len();
                    while offset < dst_len {
                        let dst =
                            &mut decomp_buf[offset..][..rng.gen_range(0, dst_len - offset + 1)];
                        let len = r.read(dst).await.unwrap();
                        assert!(dst.is_empty() || len > 0);
                        offset += len;
                    }
                }
                assert_eq!(decomp_buf.len(), src.len());
            }))
            .await;
        })
    }
}

mod async_bufread_decompressor {
    use super::*;
    use futures::future::join_all;
    use futures_lite::AsyncReadExt;
    use lzzzz::lz4f::{AsyncBufReadDecompressor, WriteCompressor};
    use std::io::Write;

    #[test]
    fn default() {
        smol::run(async {
            join_all(lz4f_test_set().map(|(src, prefs)| async move {
                let mut comp_buf = Vec::new();
                let mut decomp_buf = Vec::new();
                assert_eq!(
                    lz4f::compress_to_vec(&src, &mut comp_buf, &prefs).unwrap(),
                    comp_buf.len()
                );
                {
                    let mut src = comp_buf.as_slice();
                    let mut r = AsyncBufReadDecompressor::new(&mut src).unwrap();
                    assert_eq!(
                        r.read_frame_info().await.unwrap().dict_id(),
                        prefs.frame_info().dict_id()
                    );
                    r.read_to_end(&mut decomp_buf).await.unwrap();
                }
                assert_eq!(decomp_buf, src);
            }))
            .await;
        })
    }

    #[test]
    fn dictionary() {
        smol::run(async {
            join_all(lz4f_test_set().map(|(src, prefs)| async move {
                let mut comp_buf = Vec::new();
                let mut decomp_buf = Vec::new();
                let dict = SmallRng::seed_from_u64(0)
                    .sample_iter(Standard)
                    .take(64_000)
                    .collect::<Vec<_>>();
                {
                    let mut w = WriteCompressor::with_dict(
                        &mut comp_buf,
                        prefs,
                        Dictionary::new(&dict).unwrap(),
                    )
                    .unwrap();
                    w.write_all(&src).unwrap();
                }
                {
                    let mut src = comp_buf.as_slice();
                    let mut r = AsyncBufReadDecompressor::new(&mut src).unwrap();
                    assert_eq!(
                        r.read_frame_info().await.unwrap().dict_id(),
                        prefs.frame_info().dict_id()
                    );
                    r.set_dict(&dict);
                    r.read_to_end(&mut decomp_buf).await.unwrap();
                }
                assert_eq!(decomp_buf, src);
            }))
            .await;
        })
    }

    #[test]
    fn random_chunk() {
        smol::run(async {
            join_all(lz4f_test_set().map(|(src, prefs)| async move {
                let mut comp_buf = Vec::new();
                let mut decomp_buf = vec![0; src.len()];
                assert_eq!(
                    lz4f::compress_to_vec(&src, &mut comp_buf, &prefs).unwrap(),
                    comp_buf.len()
                );
                {
                    let mut src = comp_buf.as_slice();
                    let mut r = AsyncBufReadDecompressor::new(&mut src).unwrap();

                    let mut offset = 0;
                    let mut rng = SmallRng::seed_from_u64(0);

                    let dst_len = decomp_buf.len();
                    while offset < dst_len {
                        let dst =
                            &mut decomp_buf[offset..][..rng.gen_range(0, dst_len - offset + 1)];
                        let len = r.read(dst).await.unwrap();
                        assert!(dst.is_empty() || len > 0);
                        offset += len;
                    }
                }
                assert_eq!(decomp_buf.len(), src.len());
            }))
            .await;
        })
    }
}

mod async_write_decompressor {
    use super::*;
    use futures::future::join_all;
    use futures_lite::AsyncWriteExt;
    use lzzzz::lz4f::{AsyncWriteDecompressor, WriteCompressor};
    use std::io::Write;

    #[test]
    fn default() {
        smol::run(async {
            join_all(lz4f_test_set().map(|(src, prefs)| async move {
                let mut comp_buf = Vec::new();
                let mut decomp_buf = Vec::new();
                assert_eq!(
                    lz4f::compress_to_vec(&src, &mut comp_buf, &prefs).unwrap(),
                    comp_buf.len()
                );
                {
                    let mut w = AsyncWriteDecompressor::new(&mut decomp_buf).unwrap();
                    w.write_all(&comp_buf).await.unwrap();
                }
                assert_eq!(decomp_buf, src);
            }))
            .await;
        })
    }

    #[test]
    fn decode_header_only() {
        smol::run(async {
            join_all(lz4f_test_set().map(|(src, prefs)| async move {
                let mut comp_buf = Vec::new();
                let mut decomp_buf = Vec::new();
                assert_eq!(
                    lz4f::compress_to_vec(&src, &mut comp_buf, &prefs).unwrap(),
                    comp_buf.len()
                );
                {
                    let mut w = AsyncWriteDecompressor::new(&mut decomp_buf).unwrap();
                    assert!(w.frame_info().is_none());
                    w.decode_header_only(true);

                    let mut header_len = 0;
                    while w.frame_info().is_none() {
                        header_len += w.write(&comp_buf[header_len..]).await.unwrap();
                    }

                    assert_eq!(w.write(&comp_buf).await.unwrap(), 0);
                    w.decode_header_only(false);
                    w.write_all(&comp_buf[header_len..]).await.unwrap();
                }
                assert_eq!(decomp_buf, src);
            }))
            .await;
        })
    }

    #[test]
    fn dictionary() {
        smol::run(async {
            join_all(lz4f_test_set().map(|(src, prefs)| async move {
                let mut comp_buf = Vec::new();
                let mut decomp_buf = Vec::new();
                let dict = SmallRng::seed_from_u64(0)
                    .sample_iter(Standard)
                    .take(64_000)
                    .collect::<Vec<_>>();
                {
                    let mut w = WriteCompressor::with_dict(
                        &mut comp_buf,
                        prefs,
                        Dictionary::new(&dict).unwrap(),
                    )
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
        })
    }

    #[test]
    fn random_chunk() {
        smol::run(async {
            join_all(lz4f_test_set().map(|(src, prefs)| async move {
                let mut comp_buf = Vec::new();
                let mut decomp_buf = Vec::new();
                assert_eq!(
                    lz4f::compress_to_vec(&src, &mut comp_buf, &prefs).unwrap(),
                    comp_buf.len()
                );
                {
                    let mut w = AsyncWriteDecompressor::new(&mut decomp_buf).unwrap();

                    let mut offset = 0;
                    let mut rng = SmallRng::seed_from_u64(0);

                    while offset < comp_buf.len() {
                        let src =
                            &comp_buf[offset..][..rng.gen_range(0, comp_buf.len() - offset + 1)];
                        let len = w.write(src).await.unwrap();
                        assert!(src.is_empty() || len > 0);
                        offset += len;
                    }
                }
                assert_eq!(decomp_buf.len(), src.len());
            }))
            .await;
        })
    }

    #[test]
    fn invalid_header() {
        smol::run(async {
            join_all(lz4f_test_set().map(|(src, prefs)| async move {
                let mut comp_buf = Vec::new();
                let mut decomp_buf = Vec::new();
                assert_eq!(
                    lz4f::compress_to_vec(&src, &mut comp_buf, &prefs).unwrap(),
                    comp_buf.len()
                );
                {
                    let mut w = AsyncWriteDecompressor::new(&mut decomp_buf).unwrap();
                    let err = w
                        .write_all(&comp_buf[1..])
                        .await
                        .unwrap_err()
                        .into_inner()
                        .unwrap()
                        .downcast::<lz4f::Error>()
                        .unwrap();
                    assert_eq!(
                        *err,
                        lz4f::Error::Common(lzzzz::ErrorKind::FrameHeaderInvalid)
                    );
                }
            }))
            .await;
        })
    }
}
