#!/bin/bash
set -e

# Get version from Cargo.toml
VERSION=$(grep "^version =" Cargo.toml | head -1 | cut -d '"' -f 2)
echo "Syncing all packages to version $VERSION..."

# Update npm package.json
if [ -f packages/rust/biors-wasm/package.json ]; then
  sed -i '' "s/\"version\": \".*\"/\"version\": \"$VERSION\"/" packages/rust/biors-wasm/package.json
  echo "Updated packages/rust/biors-wasm/package.json"
fi

# Update python pyproject.toml
if [ -f packages/rust/biors-python/pyproject.toml ]; then
  sed -i '' "s/^version = \".*\"/version = \"$VERSION\"/" packages/rust/biors-python/pyproject.toml
  echo "Updated packages/rust/biors-python/pyproject.toml"
fi

# Update python __init__.py
if [ -f packages/rust/biors-python/src/biors/__init__.py ]; then
  sed -i '' "s/^__version__ = \".*\"/__version__ = \"$VERSION\"/" packages/rust/biors-python/src/biors/__init__.py
  echo "Updated packages/rust/biors-python/src/biors/__init__.py"
fi

echo "✅ Version sync complete."
