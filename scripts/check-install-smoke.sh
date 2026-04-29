#!/bin/sh
set -eu

tmp_dir="$(mktemp -d)"
cleanup() {
  rm -rf "$tmp_dir"
}
trap cleanup EXIT INT TERM

install_root="$tmp_dir/install"
cargo install \
  --locked \
  --path packages/rust/biors \
  --root "$install_root"

"$install_root/bin/biors" --version
"$install_root/bin/biors" tokenize examples/protein.fasta >/dev/null
