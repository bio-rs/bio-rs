#!/usr/bin/env python3
"""Validate the declared Rust MSRV across package metadata and CI."""

from pathlib import Path
import json
import subprocess
import sys
import tomllib


MSRV = "1.88"
TOOLCHAIN = "1.88.0"
RUST_TOOLCHAIN_ACTION = "dtolnay/rust-toolchain@98e1b82157cd469e843cb7f524c1313b4ad9492c"
WORKFLOWS = [
    Path(".github/workflows/ci.yml"),
    Path(".github/workflows/release.yml"),
]


def main() -> int:
    errors: list[str] = []
    workspace = tomllib.loads(Path("Cargo.toml").read_text(encoding="utf-8"))
    rust_version = workspace["workspace"]["package"].get("rust-version")
    if rust_version != MSRV:
        errors.append(f"workspace rust-version must be {MSRV}, found {rust_version!r}")

    toolchain = tomllib.loads(Path("rust-toolchain.toml").read_text(encoding="utf-8"))
    channel = toolchain["toolchain"].get("channel")
    if channel != TOOLCHAIN:
        errors.append(f"rust-toolchain channel must be {TOOLCHAIN}, found {channel!r}")

    for manifest in sorted(Path("crates").glob("*/Cargo.toml")):
        package = tomllib.loads(manifest.read_text(encoding="utf-8"))["package"]
        if package.get("rust-version", {}).get("workspace") is not True:
            errors.append(f"{manifest} must inherit rust-version.workspace = true")

    metadata = json.loads(
        subprocess.check_output(
            ["cargo", "metadata", "--locked", "--format-version", "1", "--no-deps"],
            text=True,
        )
    )
    for package in metadata["packages"]:
        if package.get("rust_version") != MSRV:
            errors.append(
                f"{package['name']} metadata rust_version must be {MSRV}, "
                f"found {package.get('rust_version')!r}"
            )

    for workflow in WORKFLOWS:
        text = workflow.read_text(encoding="utf-8")
        if RUST_TOOLCHAIN_ACTION not in text:
            errors.append(f"{workflow} must install pinned Rust toolchain action {RUST_TOOLCHAIN_ACTION}")

    if errors:
        for error in errors:
            print(error, file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
