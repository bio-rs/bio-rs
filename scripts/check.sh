#!/bin/sh
set -eu

if [ -f "$HOME/.cargo/env" ]; then
  # shellcheck disable=SC1090
  . "$HOME/.cargo/env"
fi

echo "==> cargo fmt --check"
cargo fmt --all --check

echo "==> cargo check --workspace --all-targets --all-features"
cargo check --locked --workspace --all-targets --all-features

echo "==> cargo check -p biors-core --target wasm32-unknown-unknown --all-features"
rustup target add wasm32-unknown-unknown
cargo check --locked -p biors-core --target wasm32-unknown-unknown --all-features

echo "==> cargo test --workspace --all-targets --all-features"
cargo test --locked --workspace --all-targets --all-features

echo "==> cargo clippy --workspace --all-targets --all-features -- -D warnings"
cargo clippy --locked --workspace --all-targets --all-features -- -D warnings
