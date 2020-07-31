test:
    cargo test --release --all-features

fmt:
    cargo +nightly fmt

doc:
    cargo +nightly doc --all-features

export RUSTDOCFLAGS := "--cfg docsrs"