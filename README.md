<h1 align="center">lzzzz</h1>
<div align="center">
 <strong>
   Yet another liblz4 binding for Rust.
 </strong>
</div>

<br/>

[![Build Status](https://dev.azure.com/picoHz/lzzzz/_apis/build/status/lzzzz-CI?branchName=master)](https://dev.azure.com/picoHz/lzzzz/_build/latest?definitionId=2&branchName=master)

- **Designed for Rust:** Lzzzz is a high-level wrapper of liblz4 provides comprehensible API complies with Rust's manner without losing performance and flexibility. You have no concern about memory management and concurrency safety.

- **Various Modes:** `LZ4`, `LZ4_HC`, `LZ4F`, `LZ4 Streaming`, `LZ4_HC Streaming` and `LZ4F Streaming` are supported.

- **Flexible Streaming:** All the compressor/decompressor streams support `Read`, `BufRead` and `Write` operations. 
With `tokio` feature, `AsyncRead`, `AsyncBufRead` and `AsyncWrite` are also supported.

- LZ4 Block Compression/Decompression
- LZ4 Streaming Compression/Decompression (Reader/Writer)
- LZ4_HC Block Compression
- LZ4_HC Streaming Compression (Reader/Writer)
- LZ4F Compression/Decompression
- LZ4F Streaming Compression/Decompression (Reader/Writer)

# Asynchronous I/O feature

The `tokio-io` feature flag enables asynchronous streaming compressors and decompressors.

```toml
lzzzz = { version = "...", features = ["tokio-io"] }
```
