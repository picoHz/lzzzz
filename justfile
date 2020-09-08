test:
    cargo test --release --all-features

bench:
    cargo bench --all-features

fmt:
    cargo +nightly fmt

doc:
    cargo +nightly doc --all-features

export RUSTDOCFLAGS := "--cfg docsrs"