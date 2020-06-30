use futures::future::join_all;
use lzzzz::{lz4f, lz4f::*};

mod common;
use common::lz4f_test_set;

mod compress_to_vec {
    use super::*;

    #[tokio::test]
    async fn normal() {
        join_all(
            lz4f_test_set()
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
            lz4f_test_set()
                .map(|(src, prefs)| async move {
                    let mut comp_buf = vec![0; lz4f::max_compressed_size(src.len(), &prefs)];
                    let mut decomp_buf = Vec::new();

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
    async fn too_small_dst() {
        join_all(
            lz4f_test_set()
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
            lz4f_test_set()
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
            lz4f_test_set()
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
