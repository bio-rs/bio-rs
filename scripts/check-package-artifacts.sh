#!/bin/sh
set -eu

repo_root="$(git rev-parse --show-toplevel)"
cd "$repo_root"

. scripts/release-tool-versions.env

artifact_root="${BIORS_PACKAGE_ARTIFACT_DIR:-target/package-artifacts}"
python_dist="$artifact_root/python-dist"

echo "==> Python wheel and sdist artifacts"
if ! command -v maturin >/dev/null 2>&1; then
  echo "maturin is required. Install with: python -m pip install 'maturin==$BIORS_RELEASE_MATURIN_VERSION'" >&2
  exit 1
fi
maturin_version="$(maturin --version | awk '{print $2}')"
if [ "$maturin_version" != "$BIORS_RELEASE_MATURIN_VERSION" ]; then
  echo "maturin $BIORS_RELEASE_MATURIN_VERSION is required; found $maturin_version" >&2
  echo "Install with: python -m pip install 'maturin==$BIORS_RELEASE_MATURIN_VERSION'" >&2
  exit 1
fi
rm -rf "$python_dist"
mkdir -p "$python_dist"
maturin build --release --manifest-path crates/biors-python/Cargo.toml --out "$python_dist" --compatibility pypi
maturin sdist --manifest-path crates/biors-python/Cargo.toml --out "$python_dist"
python3 scripts/check-release-artifact-contents.py python-dist "$python_dist" --require-sdist
python3 scripts/test-python-wheel.py --dist-dir "$python_dist"

echo "==> WASM tests and npm artifact"
wasm-pack test --node crates/biors-wasm
scripts/build-wasm-npm-package.sh

echo "==> crate package artifacts"
cargo package --locked -p biors-core

for package in biors-mcp-server biors-backend-candle biors; do
  # These crates depend on the same-release biors-core package. Before
  # publication, cargo resolves that dependency from the registry index, so the
  # local preflight can only verify package file inclusion. The release workflow
  # performs the full dry-run publish after biors-core is visible in the index.
  cargo package --list -p "$package" >/dev/null
done
