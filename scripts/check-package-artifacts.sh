#!/bin/sh
set -eu

repo_root="$(git rev-parse --show-toplevel)"
cd "$repo_root"

artifact_root="${BIORS_PACKAGE_ARTIFACT_DIR:-target/package-artifacts}"
python_dist="$artifact_root/python-dist"

echo "==> Python wheel and sdist artifacts"
if ! command -v maturin >/dev/null 2>&1; then
  echo "maturin is required. Install with: python -m pip install 'maturin>=1.0,<2.0'" >&2
  exit 1
fi
rm -rf "$python_dist"
mkdir -p "$python_dist"
maturin build --release --manifest-path packages/rust/biors-python/Cargo.toml --out "$python_dist" --compatibility pypi
maturin sdist --manifest-path packages/rust/biors-python/Cargo.toml --out "$python_dist"
python3 scripts/check-release-artifact-contents.py python-dist "$python_dist" --require-sdist
python3 scripts/test-python-wheel.py --dist-dir "$python_dist"

echo "==> WASM tests and npm artifact"
wasm-pack test --node packages/rust/biors-wasm
scripts/build-wasm-npm-package.sh

echo "==> crate package artifacts"
cargo package --locked -p biors-core

for package in biors-mcp-server biors-backend-candle biors; do
  # These crates depend on the same-release biors-core package. Before
  # publication, cargo's package verifier resolves that dependency from the
  # registry index, so create the tarballs locally and leave source verification
  # to the workspace test gates plus the post-core publish dry-run workflow.
  cargo package --locked --no-verify -p "$package"
done
