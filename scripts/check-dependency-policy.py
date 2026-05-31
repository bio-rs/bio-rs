#!/usr/bin/env python3
"""Enforce dependency-light boundaries for release readiness."""

from __future__ import annotations

import subprocess
import tomllib
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
CORE_DEPS = {"serde", "serde_json", "sha2"}
FORBIDDEN_CORE_OR_CLI_DEPS = {
    "biors-backend-candle",
    "biors-mcp-server",
    "biors-python",
    "biors-wasm",
    "candle-core",
    "js-sys",
    "pyo3",
    "rmcp",
    "wasm-bindgen",
}
EXPECTED_INTEGRATION_DEPS = {
    "packages/rust/biors-backend-candle/Cargo.toml": {"candle-core"},
    "packages/rust/biors-mcp-server/Cargo.toml": {"rmcp", "tokio"},
    "packages/rust/biors-python/Cargo.toml": {"pyo3"},
    "packages/rust/biors-wasm/Cargo.toml": {"js-sys", "wasm-bindgen"},
}


def main() -> int:
    core = dependencies("packages/rust/biors-core/Cargo.toml")
    if core != CORE_DEPS:
        raise AssertionError(
            f"biors-core normal dependencies must stay {sorted(CORE_DEPS)}, found {sorted(core)}"
        )

    for manifest in ["packages/rust/biors-core/Cargo.toml", "packages/rust/biors/Cargo.toml"]:
        direct = dependencies(manifest)
        forbidden = sorted(direct & FORBIDDEN_CORE_OR_CLI_DEPS)
        if forbidden:
            raise AssertionError(f"{manifest} must not depend directly on {forbidden}")

    for manifest, expected in EXPECTED_INTEGRATION_DEPS.items():
        direct = dependencies(manifest)
        missing = sorted(expected - direct)
        if missing:
            raise AssertionError(f"{manifest} should isolate integration deps {missing}")

    for package in ["biors-core", "biors"]:
        duplicate_tree = cargo_tree_duplicates(package)
        if duplicate_tree:
            raise AssertionError(
                f"{package} must not have duplicate dependencies:\n{duplicate_tree}"
            )

    wasm_tree = cargo_tree_normal("biors-wasm", "wasm32-unknown-unknown")
    if "console_error_panic_hook" in wasm_tree:
        raise AssertionError("biors-wasm default features must not enable console_error_panic_hook")

    print("Dependency policy checks passed")
    return 0


def dependencies(manifest: str) -> set[str]:
    data = tomllib.loads((ROOT / manifest).read_text())
    return set(data.get("dependencies", {}))


def cargo_tree_duplicates(package: str) -> str:
    completed = subprocess.run(
        ["cargo", "tree", "--locked", "-p", package, "--duplicates"],
        cwd=ROOT,
        check=True,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True,
    )
    return completed.stdout.strip()


def cargo_tree_normal(package: str, target: str) -> str:
    completed = subprocess.run(
        ["cargo", "tree", "--locked", "-p", package, "--target", target, "--edges", "normal"],
        cwd=ROOT,
        check=True,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True,
    )
    return completed.stdout


if __name__ == "__main__":
    raise SystemExit(main())
