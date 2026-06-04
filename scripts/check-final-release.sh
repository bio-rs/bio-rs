#!/bin/sh
set -eu

repo_root="$(git rev-parse --show-toplevel)"
cd "$repo_root"

echo "==> release workflow dry run"
python3 scripts/check-release-workflow.py

echo "==> local registry version preflight"
python3 scripts/check-registry-versions.py --skip-network

echo "==> dependency security audit"
scripts/check-security-audit.sh

echo "==> full release gate"
scripts/check.sh

echo "==> build release binary"
cargo build --locked --release -p biors

echo "==> public demo dry run with release binary"
BIORS_BIN=target/release/biors sh scripts/launch-demo.sh

echo "==> install flow final test"
scripts/check-install-smoke.sh

echo "==> package artifact final test"
scripts/check-package-artifacts.sh
