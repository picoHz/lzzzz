# lzzzz

Yet another [liblz4](https://github.com/lz4/lz4) binding for Rust.

## Usage

```toml
[dependencies]
lzzzz = "0.1"
```

## Features

- LZ4
    - Compression (Block / Streaming)
    - Decompression (Block / Streaming)
    - Custom Dictionary
- LZ4_HC 
    - Compression (Block / Streaming)
    - Custom Dictionary
- LZ4F 
    - Compression
    - Decompression
    - Custom Dictionary
    - Streaming I/O (`Read` / `BufRead` / `Write`)
    - [optional] Asynchronous I/O (`AsyncRead` / `AsyncBufRead` / `AsyncWrite`)

### Asynchronous I/O

The `tokio-io` feature flag enables asynchronous LZ4F streaming compressors and decompressors.

```toml
[dependencies]
lzzzz = { version = "0.1", features = ["tokio-io"] }
```

## Example

```rust
use lzzzz::{lz4, lz4_hc, lz4f};

let data = b"The quick brown fox jumps over the lazy dog.";

let mut compressed = Vec::new();
lz4::compress_to_vec(data, &mut compressed, lz4::ACC_LEVEL_DEFAULT)?;

let mut compressed = Vec::new();
lz4_hc::compress_to_vec(data, &mut compressed, lz4_hc::CLEVEL_DEFAULT)?;

let mut decompressed = vec![0; data.len()];
lz4::decompress(&compressed, &mut decompressed)?;
```
