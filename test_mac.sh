#/bin/sh

set -ex

cargo test \
    --target aarch64-apple-darwin \
    -Z panic-abort-tests \
    -Z build-std="std,panic_abort" \
    --no-default-features \
    --features testing #\
    # -- \
    # --nocapture
