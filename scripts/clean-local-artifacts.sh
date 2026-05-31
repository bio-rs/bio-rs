#!/bin/sh
set -eu

repo_root="$(git rev-parse --show-toplevel)"
cd "$repo_root"

remove_target=false
case "${1:-}" in
  "")
    ;;
  --target)
    remove_target=true
    ;;
  *)
    echo "usage: scripts/clean-local-artifacts.sh [--target]" >&2
    exit 2
    ;;
esac

find . \( -path ./.git -o -path ./target \) -prune -o -name .DS_Store -type f -delete
find . \( -path ./.git -o -path ./target \) -prune -o -name __pycache__ -type d -prune -exec rm -rf {} +
find . \( -path ./.git -o -path ./target \) -prune -o -name .pytest_cache -type d -prune -exec rm -rf {} +
find . \( -path ./.git -o -path ./target \) -prune -o -name '*.pyc' -type f -delete

rm -rf \
  .benchmark-wasm \
  .venv \
  packages/rust/biors-python/python/biors/biors.abi3.so \
  packages/rust/biors-wasm/pkg

if [ "$remove_target" = true ]; then
  rm -rf target
fi

echo "Local generated artifacts cleaned."
