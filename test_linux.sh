#/bin/sh
cargo test \
    --target x86_64-unknown-linux-gnu \
    -Z panic-abort-tests \
    -Z build-std="std,panic_abort" \
    --no-default-features \
    --features testing
