test:
    cargo test --release --all-features

doc:
    cargo +nightly doc --all-features

export RUSTDOCFLAGS := "--cfg docsrs"