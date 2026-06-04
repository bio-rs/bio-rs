#!/bin/sh
set -eu
export PYTHONDONTWRITEBYTECODE=1

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
python3 scripts/check-python-syntax.py

echo "==> module size"
python3 scripts/check-module-size.py

echo "==> sequence-kind support docs"
python3 scripts/check-sequence-kind-support-docs.py

echo "==> benchmark docs"
# Keep this aligned with check-fast.sh: the release gate must diff rendered
# benchmark Markdown, not only validate the machine-readable artifact.
scripts/check-benchmark-docs.sh

echo "==> release workflow"
python3 scripts/check-release-workflow.py

echo "==> github actions pinning"
python3 scripts/check-github-actions-pinning.py

echo "==> dependency policy"
python3 scripts/check-dependency-policy.py

echo "==> rust version policy"
python3 scripts/check-rust-version-policy.py

echo "==> cargo check --workspace --all-targets --all-features"
cargo check --locked --workspace --all-targets --all-features

echo "==> cargo check -p biors-core --target wasm32-unknown-unknown --all-features"
if ! rustup target list --installed | grep -qx wasm32-unknown-unknown; then
  rustup target add wasm32-unknown-unknown
fi
cargo check --locked -p biors-core --target wasm32-unknown-unknown --all-features

echo "==> cargo test --workspace --all-targets --all-features"
cargo test --locked --workspace --all-targets --all-features

echo "==> install smoke"
scripts/check-install-smoke.sh

echo "==> cargo clippy --workspace --all-targets --all-features -- -D warnings"
cargo clippy --locked --workspace --all-targets --all-features -- -D warnings
