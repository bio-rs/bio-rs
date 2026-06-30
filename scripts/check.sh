#!/bin/sh
set -eu
export PYTHONDONTWRITEBYTECODE=1

if [ -f "$HOME/.cargo/env" ]; then
  # shellcheck disable=SC1090
  . "$HOME/.cargo/env"
fi

: "${CARGO_BUILD_JOBS:=1}"
: "${CARGO_INCREMENTAL:=0}"
: "${CARGO_PROFILE_DEV_DEBUG:=0}"
export CARGO_BUILD_JOBS CARGO_INCREMENTAL CARGO_PROFILE_DEV_DEBUG

workspace_gate_args="--workspace --exclude biors-backend-candle --all-targets --all-features"

echo "==> cargo fmt --check"
cargo fmt --all --check

echo "==> shell syntax"
find scripts .githooks -type f -print | while IFS= read -r file; do
  case "$file" in
    *.sh|.githooks/*) sh -n "$file" ;;
  esac
done

echo "==> python syntax"
python3 scripts/check-python-syntax.py

echo "==> module size"
python3 scripts/check-module-size.py

echo "==> sequence-kind support docs"
python3 scripts/check-sequence-kind-support-docs.py

echo "==> benchmark artifacts"
scripts/check-benchmark-docs.sh

echo "==> release workflow"
python3 scripts/check-release-workflow.py

echo "==> github actions pinning"
python3 scripts/check-github-actions-pinning.py

echo "==> dependency policy"
python3 scripts/check-dependency-policy.py

echo "==> rust version policy"
python3 scripts/check-rust-version-policy.py

echo "==> cargo check --workspace --exclude biors-backend-candle --all-targets --all-features"
cargo check --locked $workspace_gate_args

echo "==> cargo check -p biors-core --target wasm32-unknown-unknown --all-features"
if ! rustup target list --installed | grep -qx wasm32-unknown-unknown; then
  rustup target add wasm32-unknown-unknown
fi
cargo check --locked -p biors-core --target wasm32-unknown-unknown --all-features

echo "==> cargo test --workspace --exclude biors-backend-candle --all-targets --all-features"
cargo test --locked $workspace_gate_args

echo "==> cargo test -p biors-backend-candle --test candle_backend"
CARGO_BUILD_JOBS=1 cargo test --locked -p biors-backend-candle --test candle_backend

echo "==> install smoke"
scripts/check-install-smoke.sh

echo "==> cargo clippy --workspace --exclude biors-backend-candle --all-targets --all-features -- -D warnings"
cargo clippy --locked $workspace_gate_args -- -D warnings
