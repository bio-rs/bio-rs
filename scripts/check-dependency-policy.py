#!/usr/bin/env python3
"""Enforce dependency-light boundaries for release readiness."""

from __future__ import annotations

import subprocess
import tomllib
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
CORE_DEPS = {"serde", "serde_json", "sha2"}
PUBLISHED_CRATE_NORMAL_DEP_BUDGETS = {
    "biors-core": 21,
    "biors": 48,
    "biors-backend-candle": 123,
    "biors-mcp-server": 61,
}
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
CANDLE_EXPECTED_HEAVY_TRANSITIVES = {"rayon", "tokenizers", "zip"}
CANDLE_ALLOWED_DUPLICATE_ROOTS = {"hashbrown", "itertools", "thiserror", "thiserror-impl"}


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

    for package, budget in PUBLISHED_CRATE_NORMAL_DEP_BUDGETS.items():
        count = normal_dependency_count(package)
        if count > budget:
            raise AssertionError(
                f"{package} normal dependency count {count} exceeds budget {budget}; "
                "review dependency growth and update docs/dependency-policy.md if intentional"
            )

    candle_duplicates = cargo_tree_duplicate_roots("biors-backend-candle")
    unexpected_candle_duplicates = sorted(candle_duplicates - CANDLE_ALLOWED_DUPLICATE_ROOTS)
    if unexpected_candle_duplicates:
        raise AssertionError(
            "biors-backend-candle has new duplicate dependency roots: "
            f"{unexpected_candle_duplicates}"
        )

    candle_tree = cargo_tree_normal("biors-backend-candle")
    missing_heavy = sorted(
        dependency for dependency in CANDLE_EXPECTED_HEAVY_TRANSITIVES if dependency not in candle_tree
    )
    if missing_heavy:
        raise AssertionError(
            "biors-backend-candle dependency policy changed; update the documented "
            f"heavy transitive review list, missing {missing_heavy}"
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


def cargo_tree_duplicate_roots(package: str) -> set[str]:
    roots = set()
    for line in cargo_tree_duplicates(package).splitlines():
        if line and line[0].isalnum() and " v" in line:
            roots.add(line.split(" v", 1)[0])
    return roots


def normal_dependency_count(package: str) -> int:
    names = set()
    for line in cargo_tree_normal(package, prefix_none=True).splitlines():
        line = line.strip()
        if not line:
            continue
        names.add(line.split(" ", 1)[0])
    return len(names)


def cargo_tree_normal(package: str, target: str | None = None, prefix_none: bool = False) -> str:
    command = ["cargo", "tree", "--locked", "-p", package, "--edges", "normal"]
    if target is not None:
        command.extend(["--target", target])
    if prefix_none:
        command.extend(["--prefix", "none"])
    completed = subprocess.run(
        command,
        cwd=ROOT,
        check=True,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True,
    )
    return completed.stdout


if __name__ == "__main__":
    raise SystemExit(main())
