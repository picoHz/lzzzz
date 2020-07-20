<h1 align="center">lzzzz</h1>
<div align="center">
 <strong>
   Yet another liblz4 binding for Rust.
 </strong>
</div>

<br/>

Lzzzz provides high-level liblz4 wrapper API.

# Features

- LZ4
    - Compression (Block / Streaming)
    - Decompression (Block / Streaming)
    - Custom Dictinary
- LZ4_HC 
    - Compression
    - Custom dictinary
- LZ4F 
    - Compression (Block / Streaming)
    - Decompression (Block / Streaming)
    - Custom Dictinary
    - Asynchronous I/O (Optional)

## Asynchronous I/O feature

The `tokio-io` feature flag enables asynchronous LZ4F streaming compressors and decompressors.

```toml
lzzzz = { version = "0.1", features = ["tokio-io"] }
```
