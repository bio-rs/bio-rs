#!/bin/sh
set -eu
export PYTHONDONTWRITEBYTECODE=1

if [ -f "$HOME/.cargo/env" ]; then
  # shellcheck disable=SC1090
  . "$HOME/.cargo/env"
fi

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

echo "==> benchmark docs"
scripts/check-benchmark-docs.sh

echo "==> release workflow"
python3 scripts/check-release-workflow.py

echo "==> dependency policy"
python3 scripts/check-dependency-policy.py

echo "==> cargo fmt --check"
cargo fmt --all --check

echo "==> cargo check --workspace --all-targets --all-features"
cargo check --locked --workspace --all-targets --all-features
