test:
    cargo test --release

bench:
    cargo bench

fmt:
    cargo +nightly fmt

doc:
    cargo +nightly doc

export RUSTDOCFLAGS := "--cfg docsrs"