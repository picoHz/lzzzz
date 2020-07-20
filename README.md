# lzzzz

Yet another [liblz4](https://github.com/lz4/lz4) binding for Rust.

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
    - Streaming I/O (`AsyncRead` / `AsyncBufRead` / `AsyncWrite`)
    - [optional] Asynchronous I/O (`AsyncRead` / `AsyncBufRead` / `AsyncWrite`)

### Asynchronous I/O

The `tokio-io` feature flag enables asynchronous LZ4F streaming compressors and decompressors.

```toml
lzzzz = { version = "0.1", features = ["tokio-io"] }
```

## Usage

```toml
[dependencies]
lzzzz = "0.1"
```

## Example

```rust
use lzzzz::lz4;

let data = b"The quick brown fox jumps over the lazy dog.";
let mut buf = Vec::new();

lz4::compress_to_vec(data, &mut buf, lz4::ACC_LEVEL_DEFAULT)?;
```
