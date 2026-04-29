#!/bin/sh
set -eu

if [ -f "$HOME/.cargo/env" ]; then
  # shellcheck disable=SC1090
  . "$HOME/.cargo/env"
fi

echo "==> cargo fmt --check"
cargo fmt --all --check

echo "==> shell syntax"
find scripts .githooks -type f -print | while IFS= read -r file; do
  case "$file" in
    *.sh|.githooks/*) sh -n "$file" ;;
  esac
done

echo "==> python syntax"
python3 -m py_compile \
  scripts/benchmark_fasta_vs_biopython.py \
  scripts/check-benchmark-artifact.py \
  scripts/check-release-workflow.py \
  scripts/render_benchmark_report.py

echo "==> benchmark docs"
scripts/check-benchmark-docs.sh

echo "==> release workflow"
python3 scripts/check-release-workflow.py

echo "==> cargo check --workspace --all-targets --all-features"
cargo check --locked --workspace --all-targets --all-features

echo "==> cargo check -p biors-core --target wasm32-unknown-unknown --all-features"
if ! rustup target list --installed | grep -qx wasm32-unknown-unknown; then
  rustup target add wasm32-unknown-unknown
fi
cargo check --locked -p biors-core --target wasm32-unknown-unknown --all-features

echo "==> cargo test --workspace --all-targets --all-features"
cargo test --locked --workspace --all-targets --all-features

echo "==> cargo clippy --workspace --all-targets --all-features -- -D warnings"
cargo clippy --locked --workspace --all-targets --all-features -- -D warnings
