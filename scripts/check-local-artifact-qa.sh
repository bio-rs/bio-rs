#!/bin/sh
set -eu

repo_root="$(git rev-parse --show-toplevel)"
cd "$repo_root"

mode=""
check_doc_safety_only=0

usage() {
  echo "usage: scripts/check-local-artifact-qa.sh --no-publish [--check-doc-safety]" >&2
}

fail() {
  echo "check-local-artifact-qa: $*" >&2
  exit 1
}

while [ "$#" -gt 0 ]; do
  case "$1" in
    --no-publish)
      mode="no-publish"
      ;;
    --check-doc-safety)
      check_doc_safety_only=1
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      usage
      fail "unknown argument: $1"
      ;;
  esac
  shift
done

[ "$mode" = "no-publish" ] || {
  usage
  fail "--no-publish is required"
}

artifact_tmp=""

cleanup() {
  if [ -n "$artifact_tmp" ] && [ -d "$artifact_tmp" ]; then
    rm -rf "$artifact_tmp"
  fi
}
trap cleanup EXIT INT TERM

check_doc_safety() {
  python3 - docs/release-qa.md <<'PY'
from pathlib import Path
import re
import sys

path = Path(sys.argv[1])
text = path.read_text()

required = [
    "## No-Publish Local QA",
    "Release binary CLI workflows",
    "MCP stdio tool smoke",
    "Python wheel install/import/package API smoke",
    "WASM/npm build/import smoke",
    "Local service release-binary smoke",
    "Package validate/verify/bridge smoke",
    "## Post-Publish Approval Gate",
    "explicit approval",
]
for marker in required:
    if marker not in text:
        raise SystemExit(f"release QA doc missing: {marker}")

start = text.index("## No-Publish Local QA")
end = text.index("## Post-Publish Approval Gate", start)
no_publish = text[start:end]

for parts in [
    ("cargo", "publish"),
    ("npm", "publish"),
    ("twine", "upload"),
    ("maturin", "upload"),
    ("gh", "release", "create"),
    ("git", "tag"),
    ("git", "push", "--tags"),
    ("actions", "upload-artifact"),
]:
    needle = " ".join(parts)
    if re.search(r"(^|\n)\s*(?:\$ )?" + re.escape(needle), no_publish):
        raise SystemExit(f"no-publish section contains release command: {needle}")

post_publish = text[end:]
for parts in [("cargo", "publish"), ("npm", "publish"), ("gh", "release", "create")]:
    needle = " ".join(parts)
    if needle not in post_publish:
        raise SystemExit(f"post-publish section missing approval-gated command: {needle}")
PY
}

require_executable() {
  path="$1"
  label="$2"
  [ -x "$path" ] || fail "missing executable $label at $path"
}

require_file() {
  path="$1"
  label="$2"
  [ -f "$path" ] || fail "missing $label at $path"
}

check_release_binary_cli() {
  biors_bin="${BIORS_BIN:-target/release/biors}"
  require_executable "$biors_bin" "release binary"

  echo "==> release binary CLI workflows"
  BIORS_BIN="$biors_bin" BIORS_DEMO_OUT_DIR="${BIORS_DEMO_OUT_DIR:-target/local-artifact-qa/demo}" \
    sh scripts/launch-demo.sh
}

check_mcp_stdio() {
  mcp_bin="${BIORS_MCP_BIN:-target/release/biors-mcp-server}"
  require_executable "$mcp_bin" "MCP stdio binary"

  echo "==> MCP stdio tool smoke"
  artifact_tmp="${artifact_tmp:-$(mktemp -d "${TMPDIR:-/tmp}/biors-local-artifact-qa.XXXXXX")}"
  fifo="$artifact_tmp/mcp.stdin"
  mkfifo "$fifo"
  exec 3<>"$fifo"
  "$mcp_bin" <"$fifo" >"$artifact_tmp/mcp.stdout" 2>"$artifact_tmp/mcp.stderr" &
  mcp_pid="$!"
  sleep 1
  if ! kill -0 "$mcp_pid" 2>/dev/null; then
    cat "$artifact_tmp/mcp.stderr" >&2 || true
    exec 3>&-
    fail "MCP stdio server exited before client attach"
  fi
  kill "$mcp_pid" 2>/dev/null || true
  wait "$mcp_pid" 2>/dev/null || true
  exec 3>&-
}

check_python_wheel() {
  dist_dir="${BIORS_PYTHON_DIST_DIR:-target/package-artifacts/python-dist}"
  [ -d "$dist_dir" ] || fail "missing Python dist dir: $dist_dir"

  wheel="$(python3 - "$dist_dir" <<'PY'
from pathlib import Path
import sys

wheels = sorted(Path(sys.argv[1]).glob("*.whl"))
if len(wheels) != 1:
    raise SystemExit(f"expected exactly one wheel in {sys.argv[1]}, found {len(wheels)}")
print(wheels[0])
PY
)"

  echo "==> Python wheel install/import/package API smoke"
  artifact_tmp="${artifact_tmp:-$(mktemp -d "${TMPDIR:-/tmp}/biors-local-artifact-qa.XXXXXX")}"
  python3 -m venv "$artifact_tmp/python-wheel-venv"
  py="$artifact_tmp/python-wheel-venv/bin/python"
  "$py" -m pip install --no-index "$wheel"
  "$py" - <<'PY'
import json

import biors

records = biors.parse_fasta_records(">seq1\nACDE\n")
assert len(records) == 1
report = json.loads(biors.validate_package_manifest_file("testdata/protein-package/manifest.json"))
assert report["valid"] is True
PY
}

check_wasm_npm_package() {
  pkg_dir="${BIORS_WASM_PKG_DIR:-crates/biors-wasm/pkg}"
  require_file "$pkg_dir/package.json" "WASM npm package metadata"
  require_file "$pkg_dir/biors_wasm.js" "WASM npm JS glue"
  require_file "$pkg_dir/biors_wasm_bg.wasm" "WASM binary payload"
  require_file "$pkg_dir/index.d.ts" "WASM TypeScript declarations"

  echo "==> WASM/npm build/import smoke"
  node - "$pkg_dir" <<'NODE'
const fs = require("fs");
const path = require("path");
const pkgDir = process.argv[2];
const manifest = JSON.parse(fs.readFileSync(path.join(pkgDir, "package.json"), "utf8"));
if (manifest.name !== "@bio-rs/biors-wasm") {
  throw new Error(`unexpected package name: ${manifest.name}`);
}
if (manifest.module !== "biors_wasm.js" || manifest.types !== "index.d.ts") {
  throw new Error("WASM package must expose JS module and TypeScript declarations");
}
NODE
}

check_local_service() {
  biors_bin="${BIORS_BIN:-target/release/biors}"
  require_executable "$biors_bin" "release binary"

  echo "==> local service release-binary smoke"
  artifact_tmp="${artifact_tmp:-$(mktemp -d "${TMPDIR:-/tmp}/biors-local-artifact-qa.XXXXXX")}"
  port="$(python3 - <<'PY'
import socket

with socket.socket() as sock:
    sock.bind(("127.0.0.1", 0))
    print(sock.getsockname()[1])
PY
)"
  "$biors_bin" serve --host 127.0.0.1 --port "$port" \
    >"$artifact_tmp/service.stdout" 2>"$artifact_tmp/service.stderr" &
  service_pid="$!"

  attempt=0
  while [ "$attempt" -lt 30 ]; do
    if python3 - "$port" <<'PY'
import socket
import sys

port = int(sys.argv[1])
try:
    with socket.create_connection(("127.0.0.1", port), timeout=0.2) as sock:
        sock.sendall(b"GET /health HTTP/1.1\r\nHost: 127.0.0.1\r\n\r\n")
        response = sock.recv(4096)
except OSError:
    raise SystemExit(1)

if b"200 OK" not in response or b"local_first_no_external_calls" not in response:
    raise SystemExit(1)
PY
    then
      break
    fi
    attempt=$((attempt + 1))
    sleep 0.2
  done

  if [ "$attempt" -eq 30 ]; then
    cat "$artifact_tmp/service.stderr" >&2 || true
    kill "$service_pid" 2>/dev/null || true
    wait "$service_pid" 2>/dev/null || true
    fail "local service health smoke failed"
  fi

  kill "$service_pid" 2>/dev/null || true
  wait "$service_pid" 2>/dev/null || true
}

check_package_workflow() {
  biors_bin="${BIORS_BIN:-target/release/biors}"
  require_executable "$biors_bin" "release binary"

  echo "==> package validate/verify/bridge smoke"
  "$biors_bin" package validate testdata/protein-package/manifest.json >/dev/null
  "$biors_bin" package verify \
    testdata/protein-package/manifest.json \
    testdata/protein-package/observations.json >/dev/null
  "$biors_bin" package bridge testdata/protein-package/manifest.json >/dev/null
}

check_doc_safety

if [ "$check_doc_safety_only" -eq 1 ]; then
  exit 0
fi

check_release_binary_cli
check_mcp_stdio
check_python_wheel
check_wasm_npm_package
check_local_service
check_package_workflow
