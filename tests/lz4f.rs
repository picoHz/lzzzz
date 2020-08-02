use lzzzz::{lz4f, lz4f::*};
use rayon::{iter::ParallelBridge, prelude::*};

mod common;
use common::lz4f_test_set;

mod compress_to_vec {
    use super::*;

    #[test]
    fn default() {
        lz4f_test_set().par_bridge().for_each(|(src, prefs)| {
            let header = &b"HEADER"[..];
            let mut comp_buf = Vec::from(header);
            let mut decomp_buf = Vec::from(header);

            assert_eq!(
                lz4f::compress_to_vec(&src, &mut comp_buf, &prefs).unwrap(),
                comp_buf.len() - header.len()
            );
            assert_eq!(
                lz4f::decompress_to_vec(&comp_buf[header.len()..], &mut decomp_buf).unwrap(),
                decomp_buf.len() - header.len()
            );
            assert_eq!(&decomp_buf[header.len()..], &src[..]);
        });
    }
}

mod compress {
    use super::*;

    #[test]
    fn default() {
        lz4f_test_set().par_bridge().for_each(|(src, prefs)| {
            let mut comp_buf = vec![0; lz4f::max_compressed_size(src.len(), &prefs)];
            let mut decomp_buf = Vec::new();

            let len = lz4f::compress(&src, &mut comp_buf, &prefs).unwrap();
            comp_buf.resize_with(len, Default::default);
            assert_eq!(
                lz4f::decompress_to_vec(&comp_buf, &mut decomp_buf).unwrap(),
                decomp_buf.len()
            );
            assert_eq!(decomp_buf, src);
        });
    }

    #[test]
    fn too_small_dst() {
        lz4f_test_set().par_bridge().for_each(|(src, prefs)| {
            let mut comp_buf = Vec::new();
            assert_eq!(
                lz4f::compress(&src, &mut comp_buf, &prefs),
                Err(Error::Lz4f(ErrorKind::DstMaxSizeTooSmall))
            );
        });
    }

    #[test]
    fn content_size() {
        lz4f_test_set().par_bridge().for_each(|(src, prefs)| {
            let prefs = PreferencesBuilder::from(prefs).content_size(1).build();
            let mut comp_buf = vec![0; lz4f::max_compressed_size(src.len(), &prefs)];
            let mut decomp_buf = Vec::new();

            let len = lz4f::compress(&src, &mut comp_buf, &prefs).unwrap();
            comp_buf.resize_with(len, Default::default);
            assert_eq!(
                lz4f::decompress_to_vec(&comp_buf, &mut decomp_buf).unwrap(),
                decomp_buf.len()
            );
            assert_eq!(decomp_buf, src);

            let mut comp_buf = comp_buf.as_slice();
            let mut r = ReadDecompressor::new(&mut comp_buf).unwrap();
            assert_eq!(r.read_frame_info().unwrap().content_size(), src.len());
        });
    }
}

mod decompress_to_vec {
    use super::*;

    #[test]
    fn invalid_header() {
        lz4f_test_set().par_bridge().for_each(|(src, prefs)| {
            let header = &b"HEADER"[..];
            let mut comp_buf = Vec::from(header);
            let mut decomp_buf = Vec::from(header);
            assert_eq!(
                lz4f::compress_to_vec(&src, &mut comp_buf, &prefs).unwrap(),
                comp_buf.len() - header.len()
            );
            assert_eq!(
                lz4f::decompress_to_vec(&comp_buf[1..], &mut decomp_buf),
                Err(Error::Lz4f(ErrorKind::FrameTypeUnknown))
            );
            assert_eq!(decomp_buf, header);
        });
    }

    #[test]
    fn incomplete_src() {
        lz4f_test_set().par_bridge().for_each(|(src, prefs)| {
            let header = &b"HEADER"[..];
            let mut comp_buf = Vec::new();
            let mut decomp_buf = Vec::from(header);
            assert_eq!(
                lz4f::compress_to_vec(&src, &mut comp_buf, &prefs).unwrap(),
                comp_buf.len()
            );
            assert_eq!(
                lz4f::decompress_to_vec(&comp_buf[..comp_buf.len() - 1], &mut decomp_buf),
                Err(Error::Common(lzzzz::ErrorKind::CompressedDataIncomplete))
            );
            assert_eq!(decomp_buf, header);
        });
    }
}
