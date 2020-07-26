use lzzzz::{lz4f, lz4f::*};
use rand::{distributions::Standard, rngs::SmallRng, Rng, SeedableRng};
use rayon::{iter::ParallelBridge, prelude::*};
use static_assertions::assert_impl_all;
use std::{
    fs::File,
    io::{prelude::*, BufReader},
};

mod common;
use common::lz4f_test_set;

assert_impl_all!(lz4f::BufReadCompressor<BufReader<File>>: Send);
assert_impl_all!(lz4f::ReadCompressor<File>: Send);
assert_impl_all!(lz4f::WriteCompressor<File>: Send);
assert_impl_all!(lz4f::BufReadDecompressor<BufReader<File>>: Send);
assert_impl_all!(lz4f::ReadDecompressor<File>: Send);
assert_impl_all!(lz4f::WriteDecompressor<File>: Send);

mod write_compressor {
    use super::*;
    use lzzzz::lz4f::WriteCompressor;

    #[test]
    fn default() {
        lz4f_test_set().par_bridge().for_each(|(src, prefs)| {
            let mut comp_buf = Vec::new();
            let mut decomp_buf = Vec::new();
            {
                let mut w = WriteCompressor::new(&mut comp_buf, prefs).unwrap();
                w.write_all(&src).unwrap();
            }
            assert_eq!(
                lz4f::decompress_to_vec(&comp_buf, &mut decomp_buf).unwrap(),
                decomp_buf.len()
            );
            assert_eq!(decomp_buf, src);
        });
    }

    #[test]
    fn random_chunk() {
        lz4f_test_set().par_bridge().for_each(|(src, prefs)| {
            let mut comp_buf = Vec::new();
            let mut decomp_buf = Vec::new();
            {
                let mut w = WriteCompressor::new(&mut comp_buf, prefs).unwrap();

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
                lz4f::decompress_to_vec(&comp_buf, &mut decomp_buf).unwrap(),
                decomp_buf.len()
            );
            assert_eq!(decomp_buf, src);
        });
    }
}

mod read_compressor {
    use super::*;
    use lzzzz::lz4f::ReadCompressor;

    #[test]
    fn default() {
        lz4f_test_set().par_bridge().for_each(|(src, prefs)| {
            let mut comp_buf = Vec::new();
            let mut decomp_buf = Vec::new();
            {
                let mut src = src.as_ref();
                let mut r = ReadCompressor::new(&mut src, prefs).unwrap();
                r.read_to_end(&mut comp_buf).unwrap();
            }
            assert_eq!(
                lz4f::decompress_to_vec(&comp_buf, &mut decomp_buf).unwrap(),
                decomp_buf.len()
            );
            assert_eq!(decomp_buf, src);
        });
    }

    #[test]
    fn random_chunk() {
        lz4f_test_set().par_bridge().for_each(|(src, prefs)| {
            let mut comp_buf = Vec::new();
            let mut decomp_buf = Vec::new();
            {
                let mut src = src.as_ref();
                let mut r = ReadCompressor::new(&mut src, prefs).unwrap();

                let mut offset = 0;
                let mut rng = SmallRng::seed_from_u64(0);

                loop {
                    if offset >= comp_buf.len() {
                        comp_buf.resize_with(offset + 1024, Default::default);
                    }
                    let len = rng.gen_range(0, comp_buf.len() - offset + 1);
                    let dst = &mut comp_buf[offset..][..len];
                    let len = r.read(dst).unwrap();
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
        });
    }
}

mod bufread_compressor {
    use super::*;
    use lzzzz::lz4f::BufReadCompressor;

    #[test]
    fn default() {
        lz4f_test_set().par_bridge().for_each(|(src, prefs)| {
            let mut comp_buf = Vec::new();
            let mut decomp_buf = Vec::new();
            {
                let mut src = src.as_ref();
                let mut r = BufReadCompressor::new(&mut src, prefs).unwrap();
                r.read_to_end(&mut comp_buf).unwrap();
            }
            assert_eq!(
                lz4f::decompress_to_vec(&comp_buf, &mut decomp_buf).unwrap(),
                decomp_buf.len()
            );
            assert_eq!(decomp_buf, src);
        });
    }

    #[test]
    fn random_chunk() {
        lz4f_test_set().par_bridge().for_each(|(src, prefs)| {
            let mut comp_buf = Vec::new();
            let mut decomp_buf = Vec::new();
            {
                let mut src = src.as_ref();
                let mut r = BufReadCompressor::new(&mut src, prefs).unwrap();

                let mut offset = 0;
                let mut rng = SmallRng::seed_from_u64(0);

                loop {
                    if offset >= comp_buf.len() {
                        comp_buf.resize_with(offset + 1024, Default::default);
                    }
                    let len = rng.gen_range(0, comp_buf.len() - offset + 1);
                    let dst = &mut comp_buf[offset..][..len];
                    let len = r.read(dst).unwrap();
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
        });
    }
}

mod write_decompressor {
    use super::*;
    use lzzzz::lz4f::{Dictionary, WriteCompressor, WriteDecompressor};

    #[test]
    fn default() {
        lz4f_test_set().par_bridge().for_each(|(src, prefs)| {
            let mut comp_buf = Vec::new();
            let mut decomp_buf = Vec::new();
            assert_eq!(
                lz4f::compress_to_vec(&src, &mut comp_buf, &prefs).unwrap(),
                comp_buf.len()
            );
            {
                let mut w = WriteDecompressor::new(&mut decomp_buf).unwrap();
                w.write_all(&comp_buf).unwrap();
            }
            assert_eq!(decomp_buf, src);
        });
    }

    #[test]
    fn decode_header_only() {
        lz4f_test_set().par_bridge().for_each(|(src, prefs)| {
            let mut comp_buf = Vec::new();
            let mut decomp_buf = Vec::new();
            assert_eq!(
                lz4f::compress_to_vec(&src, &mut comp_buf, &prefs).unwrap(),
                comp_buf.len()
            );
            {
                let mut w = WriteDecompressor::new(&mut decomp_buf).unwrap();
                assert!(w.frame_info().is_none());
                w.decode_header_only(true);

                let mut header_len = 0;
                while w.frame_info().is_none() {
                    header_len += w.write(&comp_buf[header_len..]).unwrap();
                }

                assert_eq!(w.write(&comp_buf).unwrap(), 0);
                w.decode_header_only(false);
                w.write_all(&comp_buf[header_len..]).unwrap();
            }
            assert_eq!(decomp_buf, src);
        });
    }

    #[test]
    fn dictionary() {
        lz4f_test_set().par_bridge().for_each(|(src, prefs)| {
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
                let mut w = WriteDecompressor::new(&mut decomp_buf).unwrap();
                w.set_dict(&dict);
                w.write_all(&comp_buf).unwrap();
            }
            assert_eq!(decomp_buf, src);
        });
    }

    #[test]
    fn random_chunk() {
        lz4f_test_set().par_bridge().for_each(|(src, prefs)| {
            let mut comp_buf = Vec::new();
            let mut decomp_buf = Vec::new();
            assert_eq!(
                lz4f::compress_to_vec(&src, &mut comp_buf, &prefs).unwrap(),
                comp_buf.len()
            );
            {
                let mut w = WriteDecompressor::new(&mut decomp_buf).unwrap();

                let mut offset = 0;
                let mut rng = SmallRng::seed_from_u64(0);

                while offset < comp_buf.len() {
                    let src = &comp_buf[offset..][..rng.gen_range(0, comp_buf.len() - offset + 1)];
                    let len = w.write(src).unwrap();
                    assert!(src.is_empty() || len > 0);
                    offset += len;
                }
            }
            assert_eq!(decomp_buf.len(), src.len());
        });
    }

    #[test]
    fn invalid_header() {
        lz4f_test_set().par_bridge().for_each(|(src, prefs)| {
            let mut comp_buf = Vec::new();
            let mut decomp_buf = Vec::new();
            assert_eq!(
                lz4f::compress_to_vec(&src, &mut comp_buf, &prefs).unwrap(),
                comp_buf.len()
            );
            {
                let mut w = WriteDecompressor::new(&mut decomp_buf).unwrap();
                let err = w
                    .write_all(&comp_buf[1..])
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
        });
    }
}

mod read_decompressor {
    use super::*;
    use lzzzz::lz4f::{ReadDecompressor, WriteCompressor};

    #[test]
    fn default() {
        lz4f_test_set().par_bridge().for_each(|(src, prefs)| {
            let mut comp_buf = Vec::new();
            let mut decomp_buf = Vec::new();
            assert_eq!(
                lz4f::compress_to_vec(&src, &mut comp_buf, &prefs).unwrap(),
                comp_buf.len()
            );
            {
                let mut src = comp_buf.as_slice();
                let mut r = ReadDecompressor::new(&mut src).unwrap();
                assert_eq!(
                    r.read_frame_info().unwrap().dict_id(),
                    prefs.frame_info().dict_id()
                );
                r.read_to_end(&mut decomp_buf).unwrap();
            }
            assert_eq!(decomp_buf, src);
        });
    }

    #[test]
    fn dictionary() {
        lz4f_test_set().par_bridge().for_each(|(src, prefs)| {
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
                let mut r = ReadDecompressor::new(&mut src).unwrap();
                assert_eq!(
                    r.read_frame_info().unwrap().dict_id(),
                    prefs.frame_info().dict_id()
                );
                r.set_dict(&dict);
                r.read_to_end(&mut decomp_buf).unwrap();
            }
            assert_eq!(decomp_buf, src);
        });
    }

    #[test]
    fn random_chunk() {
        lz4f_test_set().par_bridge().for_each(|(src, prefs)| {
            let mut comp_buf = Vec::new();
            let mut decomp_buf = vec![0; src.len()];
            assert_eq!(
                lz4f::compress_to_vec(&src, &mut comp_buf, &prefs).unwrap(),
                comp_buf.len()
            );
            {
                let mut src = comp_buf.as_slice();
                let mut r = ReadDecompressor::new(&mut src).unwrap();

                let mut offset = 0;
                let mut rng = SmallRng::seed_from_u64(0);

                let dst_len = decomp_buf.len();
                while offset < dst_len {
                    let dst = &mut decomp_buf[offset..][..rng.gen_range(0, dst_len - offset + 1)];
                    let len = r.read(dst).unwrap();
                    assert!(dst.is_empty() || len > 0);
                    offset += len;
                }
            }
            assert_eq!(decomp_buf.len(), src.len());
        });
    }
}

mod bufread_decompressor {
    use super::*;
    use lzzzz::lz4f::{BufReadDecompressor, WriteCompressor};

    #[test]
    fn default() {
        lz4f_test_set().par_bridge().for_each(|(src, prefs)| {
            let mut comp_buf = Vec::new();
            let mut decomp_buf = Vec::new();
            assert_eq!(
                lz4f::compress_to_vec(&src, &mut comp_buf, &prefs).unwrap(),
                comp_buf.len()
            );
            {
                let mut src = comp_buf.as_slice();
                let mut r = BufReadDecompressor::new(&mut src).unwrap();
                assert_eq!(
                    r.read_frame_info().unwrap().dict_id(),
                    prefs.frame_info().dict_id()
                );
                r.read_to_end(&mut decomp_buf).unwrap();
            }
            assert_eq!(decomp_buf, src);
        });
    }

    #[test]
    fn dictionary() {
        lz4f_test_set().par_bridge().for_each(|(src, prefs)| {
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
                let mut r = BufReadDecompressor::new(&mut src).unwrap();
                assert_eq!(
                    r.read_frame_info().unwrap().dict_id(),
                    prefs.frame_info().dict_id()
                );
                r.set_dict(&dict);
                r.read_to_end(&mut decomp_buf).unwrap();
            }
            assert_eq!(decomp_buf, src);
        });
    }

    #[test]
    fn random_chunk() {
        lz4f_test_set().par_bridge().for_each(|(src, prefs)| {
            let mut comp_buf = Vec::new();
            let mut decomp_buf = vec![0; src.len()];
            assert_eq!(
                lz4f::compress_to_vec(&src, &mut comp_buf, &prefs).unwrap(),
                comp_buf.len()
            );
            {
                let mut src = comp_buf.as_slice();
                let mut r = BufReadDecompressor::new(&mut src).unwrap();

                let mut offset = 0;
                let mut rng = SmallRng::seed_from_u64(0);

                let dst_len = decomp_buf.len();
                while offset < dst_len {
                    let dst = &mut decomp_buf[offset..][..rng.gen_range(0, dst_len - offset + 1)];
                    let len = r.read(dst).unwrap();
                    assert!(dst.is_empty() || len > 0);
                    offset += len;
                }
            }
            assert_eq!(decomp_buf.len(), src.len());
        });
    }
}
