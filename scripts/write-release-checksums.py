#!/usr/bin/env python3
"""Write or verify SHA-256 sidecar files for release artifacts."""

from __future__ import annotations

import argparse
import hashlib
from pathlib import Path


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--verify", action="store_true")
    parser.add_argument("artifacts", nargs="+", type=Path)
    args = parser.parse_args()

    for artifact in args.artifacts:
        if args.verify:
            verify_checksum(artifact)
        else:
            write_checksum(artifact)
    return 0


def write_checksum(artifact: Path) -> None:
    digest = sha256_file(artifact)
    checksum_path = checksum_sidecar(artifact)
    checksum_path.write_text(f"{digest}  {artifact.name}\n", encoding="utf-8")


def verify_checksum(artifact: Path) -> None:
    checksum_path = checksum_sidecar(artifact)
    parts = checksum_path.read_text(encoding="utf-8").strip().split()
    if len(parts) != 2:
        raise SystemExit(f"{checksum_path} must contain '<sha256>  <filename>'")
    expected_digest, expected_name = parts
    if expected_name != artifact.name:
        raise SystemExit(f"{checksum_path} names {expected_name}, expected {artifact.name}")
    actual_digest = sha256_file(artifact)
    if actual_digest != expected_digest:
        raise SystemExit(
            f"{checksum_path} does not match {artifact}: "
            f"expected {expected_digest}, got {actual_digest}"
        )


def checksum_sidecar(artifact: Path) -> Path:
    return artifact.with_name(f"{artifact.name}.sha256")


def sha256_file(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as handle:
        for chunk in iter(lambda: handle.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


if __name__ == "__main__":
    raise SystemExit(main())
