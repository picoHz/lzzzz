use futures::future::join_all;
use lzzzz::{lz4, lz4f::*};

mod common;
use common::lz4_test_set;

mod compress {
    use super::*;

    #[tokio::test]
    async fn normal() {
        join_all(
            lz4_test_set()
                .map(|src| async move {
                    let mut comp_buf = vec![0; lz4::max_compressed_size(src.len())];
                    //let mut decomp_buf = Vec::new();

                    let len = lz4::compress(&src, &mut comp_buf, lz4::CompressionMode::Default)
                        .unwrap()
                        .dst_len();
                })
                .map(|task| tokio::spawn(task)),
        )
        .await;
    }
}
