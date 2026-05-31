#!/usr/bin/env python3
"""Check release artifacts include required redistribution files."""

from __future__ import annotations

import argparse
import json
import subprocess
import tarfile
import zipfile
from pathlib import Path

LICENSES = {"LICENSE-APACHE", "LICENSE-MIT"}


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    subcommands = parser.add_subparsers(dest="command", required=True)

    python_dist = subcommands.add_parser("python-dist")
    python_dist.add_argument("dist_dir", type=Path)
    python_dist.add_argument("--require-sdist", action="store_true")

    wasm_package = subcommands.add_parser("wasm-package")
    wasm_package.add_argument("package_dir", type=Path)

    binary_tarball = subcommands.add_parser("binary-tarball")
    binary_tarball.add_argument("archive", type=Path)

    args = parser.parse_args()
    if args.command == "python-dist":
        check_python_dist(args.dist_dir, require_sdist=args.require_sdist)
    elif args.command == "wasm-package":
        check_wasm_package(args.package_dir)
    elif args.command == "binary-tarball":
        check_binary_tarball(args.archive)
    else:
        raise AssertionError(f"unhandled command: {args.command}")
    return 0


def check_python_dist(dist_dir: Path, *, require_sdist: bool) -> None:
    wheels = sorted(dist_dir.glob("*.whl"))
    sdists = sorted(dist_dir.glob("*.tar.gz"))
    if not wheels:
        raise SystemExit(f"{dist_dir} does not contain a wheel")
    if require_sdist and not sdists:
        raise SystemExit(f"{dist_dir} does not contain a source distribution")

    for wheel in wheels:
        with zipfile.ZipFile(wheel) as archive:
            require_entry_basenames(wheel, archive.namelist(), LICENSES)

    for sdist in sdists:
        with tarfile.open(sdist) as archive:
            require_entry_basenames(sdist, archive.getnames(), LICENSES)


def check_wasm_package(package_dir: Path) -> None:
    completed = subprocess.run(
        ["npm", "pack", str(package_dir), "--dry-run", "--json"],
        check=True,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True,
    )
    print(completed.stdout, end="")
    pack = json.loads(completed.stdout)
    files = [file["path"] for file in pack[0]["files"]]
    require_entry_basenames(package_dir, files, LICENSES)


def check_binary_tarball(archive_path: Path) -> None:
    with tarfile.open(archive_path) as archive:
        names = archive.getnames()
    require_entry_basenames(archive_path, names, LICENSES | {"README.md", "biors"})


def require_entry_basenames(artifact: Path, entries: list[str], required: set[str]) -> None:
    present = {Path(entry).name for entry in entries}
    missing = sorted(required - present)
    if missing:
        raise SystemExit(f"{artifact} is missing required file(s): {', '.join(missing)}")


if __name__ == "__main__":
    raise SystemExit(main())
