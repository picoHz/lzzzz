# lzzzz

Yet another liblz4 binding for Rust.

## Features

- LZ4
    - Compression (Block / Streaming)
    - Decompression (Block / Streaming)
    - Custom Dictionary
- LZ4_HC 
    - Compression
    - Custom Dictionary
- LZ4F 
    - Compression (Block / Streaming)
    - Decompression (Block / Streaming)
    - Custom Dictionary
    - Asynchronous I/O (`tokio-io` feature)

### Asynchronous I/O

The `tokio-io` feature flag enables asynchronous LZ4F streaming compressors and decompressors.

```toml
lzzzz = { version = "0.1", features = ["tokio-io"] }
```
