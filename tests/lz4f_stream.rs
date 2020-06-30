use futures::future::join_all;
use lzzzz::{lz4f, lz4f::*};
use rand::{distributions::Standard, rngs::SmallRng, Rng, SeedableRng};

mod common;
use common::lz4f_test_set;

mod write_compressor {
    use super::*;
    use lzzzz::lz4f::{comp::WriteCompressor, CompressorBuilder};
    use std::io::prelude::*;

    #[tokio::test]
    async fn normal() {
        join_all(
            lz4f_test_set()
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

    #[tokio::test]
    async fn random_chunk() {
        join_all(
            lz4f_test_set()
                .map(|(src, prefs)| async move {
                    let mut comp_buf = Vec::new();
                    let mut decomp_buf = Vec::new();
                    {
                        let mut w = CompressorBuilder::new(&mut comp_buf)
                            .preferences(prefs)
                            .build::<WriteCompressor<_>>()
                            .unwrap();

                        let mut offset = 0;
                        let mut rng = SmallRng::seed_from_u64(0);

                        while offset < src.len() {
                            let len = w
                                .write(&src[offset..][..rng.gen_range(0, src.len() - offset + 1)])
                                .unwrap();
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
                })
                .map(|task| tokio::spawn(task)),
        )
        .await;
    }
}

mod read_compressor {
    use super::*;
    use lzzzz::lz4f::{comp::ReadCompressor, CompressorBuilder};
    use std::io::prelude::*;

    #[tokio::test]
    async fn normal() {
        join_all(
            lz4f_test_set()
                .map(|(src, prefs)| async move {
                    let mut comp_buf = Vec::new();
                    let mut decomp_buf = Vec::new();
                    {
                        let mut src = src.as_slice();
                        let mut r = CompressorBuilder::new(&mut src)
                            .preferences(prefs)
                            .build::<ReadCompressor<_>>()
                            .unwrap();
                        r.read_to_end(&mut comp_buf).unwrap();
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

    #[tokio::test]
    async fn random_chunk() {
        join_all(
            lz4f_test_set()
                .map(|(src, prefs)| async move {
                    let mut comp_buf = Vec::new();
                    let mut decomp_buf = Vec::new();
                    {
                        let mut src = src.as_slice();
                        let mut r = CompressorBuilder::new(&mut src)
                            .preferences(prefs)
                            .build::<ReadCompressor<_>>()
                            .unwrap();

                        let mut offset = 0;
                        let mut rng = SmallRng::seed_from_u64(0);

                        loop {
                            if offset >= comp_buf.len() {
                                comp_buf.resize_with(offset + 1024, Default::default);
                            }
                            let len = rng.gen_range(0, comp_buf.len() - offset + 1);
                            let dst = &mut comp_buf[offset..][..len];
                            let len = r.read(dst).unwrap();
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
                })
                .map(|task| tokio::spawn(task)),
        )
        .await;
    }
}

mod bufread_compressor {
    use super::*;
    use lzzzz::lz4f::{comp::BufReadCompressor, CompressorBuilder};
    use std::io::prelude::*;

    #[tokio::test]
    async fn normal() {
        join_all(
            lz4f_test_set()
                .map(|(src, prefs)| async move {
                    let mut comp_buf = Vec::new();
                    let mut decomp_buf = Vec::new();
                    {
                        let mut src = src.as_slice();
                        let mut r = CompressorBuilder::new(&mut src)
                            .preferences(prefs)
                            .build::<BufReadCompressor<_>>()
                            .unwrap();
                        r.read_to_end(&mut comp_buf).unwrap();
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

    #[tokio::test]
    async fn random_chunk() {
        join_all(
            lz4f_test_set()
                .map(|(src, prefs)| async move {
                    let mut comp_buf = Vec::new();
                    let mut decomp_buf = Vec::new();
                    {
                        let mut src = src.as_slice();
                        let mut r = CompressorBuilder::new(&mut src)
                            .preferences(prefs)
                            .build::<BufReadCompressor<_>>()
                            .unwrap();

                        let mut offset = 0;
                        let mut rng = SmallRng::seed_from_u64(0);

                        loop {
                            if offset >= comp_buf.len() {
                                comp_buf.resize_with(offset + 1024, Default::default);
                            }
                            let len = rng.gen_range(0, comp_buf.len() - offset + 1);
                            let dst = &mut comp_buf[offset..][..len];
                            let len = r.read(dst).unwrap();
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
                })
                .map(|task| tokio::spawn(task)),
        )
        .await;
    }
}

mod write_decompressor {
    use super::*;
    use lzzzz::lz4f::{
        comp::WriteCompressor, decomp::WriteDecompressor, DecompressorBuilder, Dictionary,
    };
    use std::io::prelude::*;

    #[tokio::test]
    async fn normal() {
        join_all(
            lz4f_test_set()
                .map(|(src, prefs)| async move {
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
                            .build::<WriteDecompressor<_>>()
                            .unwrap();
                        w.write_all(&comp_buf).unwrap();
                    }
                    assert_eq!(decomp_buf, src);
                })
                .map(|task| tokio::spawn(task)),
        )
        .await;
    }

    #[tokio::test]
    async fn dictionary() {
        join_all(
            lz4f_test_set()
                .map(|(src, prefs)| async move {
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
                        let mut w = DecompressorBuilder::new(&mut decomp_buf)
                            .build::<WriteDecompressor<_>>()
                            .unwrap();
                        w.set_dict(&dict);
                        w.write_all(&comp_buf).unwrap();
                    }
                    assert_eq!(decomp_buf, src);
                })
                .map(|task| tokio::spawn(task)),
        )
        .await;
    }

    #[tokio::test]
    async fn small_buffer_capacity() {
        join_all(
            lz4f_test_set()
                .map(|(src, prefs)| async move {
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
                            .capacity(1)
                            .build::<WriteDecompressor<_>>()
                            .unwrap();
                        w.write_all(&comp_buf).unwrap();
                    }
                    assert_eq!(decomp_buf, src);
                })
                .map(|task| tokio::spawn(task)),
        )
        .await;
    }

    #[tokio::test]
    async fn random_chunk() {
        join_all(
            lz4f_test_set()
                .map(|(src, prefs)| async move {
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
                            .build::<WriteDecompressor<_>>()
                            .unwrap();

                        let mut offset = 0;
                        let mut rng = SmallRng::seed_from_u64(0);

                        while offset < comp_buf.len() {
                            let src = &comp_buf[offset..]
                                [..rng.gen_range(0, comp_buf.len() - offset + 1)];
                            let len = w.write(src).unwrap();
                            assert!(src.len() == 0 || len > 0);
                            offset += len;
                        }
                    }
                    assert_eq!(decomp_buf.len(), src.len());
                })
                .map(|task| tokio::spawn(task)),
        )
        .await;
    }

    #[tokio::test]
    async fn invalid_header() {
        join_all(
            lz4f_test_set()
                .map(|(src, prefs)| async move {
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
                            .build::<WriteDecompressor<_>>()
                            .unwrap();
                        let err = w
                            .write_all(&comp_buf[1..])
                            .unwrap_err()
                            .into_inner()
                            .unwrap();
                        assert_eq!(
                            err.downcast::<lz4f::Error>().unwrap(),
                            Box::new(lz4f::Error::Common(lzzzz::ErrorKind::DecompressionFailed))
                        );
                    }
                })
                .map(|task| tokio::spawn(task)),
        )
        .await;
    }
}

mod read_decompressor {
    use super::*;
    use futures::future::join_all;
    use lzzzz::lz4f::{comp::WriteCompressor, decomp::ReadDecompressor, DecompressorBuilder};
    use std::io::{Read, Write};

    #[tokio::test]
    async fn normal() {
        join_all(
            lz4f_test_set()
                .map(|(src, prefs)| async move {
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
                        let mut r = DecompressorBuilder::new(&mut src)
                            .build::<ReadDecompressor<_>>()
                            .unwrap();
                        r.read_to_end(&mut decomp_buf).unwrap();
                    }
                    assert_eq!(decomp_buf, src);
                })
                .map(|task| tokio::spawn(task)),
        )
        .await;
    }

    #[tokio::test]
    async fn small_buffer_capacity() {
        join_all(
            lz4f_test_set()
                .map(|(src, prefs)| async move {
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
                        let mut r = DecompressorBuilder::new(&mut src)
                            .capacity(1)
                            .build::<ReadDecompressor<_>>()
                            .unwrap();
                        r.read_to_end(&mut decomp_buf).unwrap();
                    }
                    assert_eq!(decomp_buf, src);
                })
                .map(|task| tokio::spawn(task)),
        )
        .await;
    }

    #[tokio::test]
    async fn dictionary() {
        join_all(
            lz4f_test_set()
                .map(|(src, prefs)| async move {
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
                        let mut r = DecompressorBuilder::new(&mut src)
                            .build::<ReadDecompressor<_>>()
                            .unwrap();
                        r.set_dict(&dict);
                        r.read_to_end(&mut decomp_buf).unwrap();
                    }
                    assert_eq!(decomp_buf, src);
                })
                .map(|task| tokio::spawn(task)),
        )
        .await;
    }

    #[tokio::test]
    async fn random_chunk() {
        join_all(
            lz4f_test_set()
                .map(|(src, prefs)| async move {
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
                        let mut r = DecompressorBuilder::new(&mut src)
                            .build::<ReadDecompressor<_>>()
                            .unwrap();

                        let mut offset = 0;
                        let mut rng = SmallRng::seed_from_u64(0);

                        let dst_len = decomp_buf.len();
                        while offset < dst_len {
                            let dst =
                                &mut decomp_buf[offset..][..rng.gen_range(0, dst_len - offset + 1)];
                            let len = r.read(dst).unwrap();
                            assert!(dst.len() == 0 || len > 0);
                            offset += len;
                        }
                    }
                    assert_eq!(decomp_buf.len(), src.len());
                })
                .map(|task| tokio::spawn(task)),
        )
        .await;
    }
}

mod bufread_decompressor {
    use super::*;
    use futures::future::join_all;
    use lzzzz::lz4f::{comp::WriteCompressor, decomp::BufReadDecompressor, DecompressorBuilder};
    use std::io::{Read, Write};

    #[tokio::test]
    async fn normal() {
        join_all(
            lz4f_test_set()
                .map(|(src, prefs)| async move {
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
                        let mut r = DecompressorBuilder::new(&mut src)
                            .build::<BufReadDecompressor<_>>()
                            .unwrap();
                        r.read_to_end(&mut decomp_buf).unwrap();
                    }
                    assert_eq!(decomp_buf, src);
                })
                .map(|task| tokio::spawn(task)),
        )
        .await;
    }

    #[tokio::test]
    async fn small_buffer_capacity() {
        join_all(
            lz4f_test_set()
                .map(|(src, prefs)| async move {
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
                        let mut r = DecompressorBuilder::new(&mut src)
                            .capacity(1)
                            .build::<BufReadDecompressor<_>>()
                            .unwrap();
                        r.read_to_end(&mut decomp_buf).unwrap();
                    }
                    assert_eq!(decomp_buf, src);
                })
                .map(|task| tokio::spawn(task)),
        )
        .await;
    }

    #[tokio::test]
    async fn dictionary() {
        join_all(
            lz4f_test_set()
                .map(|(src, prefs)| async move {
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
                        let mut r = DecompressorBuilder::new(&mut src)
                            .build::<BufReadDecompressor<_>>()
                            .unwrap();
                        r.set_dict(&dict);
                        r.read_to_end(&mut decomp_buf).unwrap();
                    }
                    assert_eq!(decomp_buf, src);
                })
                .map(|task| tokio::spawn(task)),
        )
        .await;
    }

    #[tokio::test]
    async fn random_chunk() {
        join_all(
            lz4f_test_set()
                .map(|(src, prefs)| async move {
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
                        let mut r = DecompressorBuilder::new(&mut src)
                            .build::<BufReadDecompressor<_>>()
                            .unwrap();

                        let mut offset = 0;
                        let mut rng = SmallRng::seed_from_u64(0);

                        let dst_len = decomp_buf.len();
                        while offset < dst_len {
                            let dst =
                                &mut decomp_buf[offset..][..rng.gen_range(0, dst_len - offset + 1)];
                            let len = r.read(dst).unwrap();
                            assert!(dst.len() == 0 || len > 0);
                            offset += len;
                        }
                    }
                    assert_eq!(decomp_buf.len(), src.len());
                })
                .map(|task| tokio::spawn(task)),
        )
        .await;
    }
}
