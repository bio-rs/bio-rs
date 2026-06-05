#!/usr/bin/env python3
"""Check release artifacts include required redistribution files."""

from __future__ import annotations

import argparse
import json
import subprocess
import tarfile
import tempfile
import zipfile
from pathlib import Path

LICENSES = {"LICENSE-APACHE", "LICENSE-MIT"}
PYTHON_TYPING_FILES = {"__init__.pyi", "py.typed"}
WASM_PACKAGE_FILES = {
    "README.md",
    "biors_wasm.js",
    "biors_wasm.d.ts",
    "biors_wasm_bg.js",
    "biors_wasm_bg.wasm",
    "biors_wasm_bg.wasm.d.ts",
    "index.d.ts",
}


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
            require_entry_basenames(wheel, archive.namelist(), LICENSES | PYTHON_TYPING_FILES)

    for sdist in sdists:
        with tarfile.open(sdist) as archive:
            require_entry_basenames(sdist, archive.getnames(), LICENSES | PYTHON_TYPING_FILES)


def check_wasm_package(package_dir: Path) -> None:
    package_dir = package_dir.resolve()
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
    require_entry_basenames(package_dir, files, LICENSES | WASM_PACKAGE_FILES)
    smoke_test_wasm_package(package_dir)


def smoke_test_wasm_package(package_dir: Path) -> None:
    with tempfile.TemporaryDirectory(prefix="biors-wasm-pack-") as temp_dir:
        work_dir = Path(temp_dir)
        completed = subprocess.run(
            ["npm", "pack", str(package_dir), "--json"],
            cwd=work_dir,
            check=True,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            text=True,
        )
        pack = json.loads(completed.stdout)
        tarball = work_dir / pack[0]["filename"]
        subprocess.run(
            ["npm", "install", "--ignore-scripts", "--no-audit", "--fund=false", str(tarball)],
            cwd=work_dir,
            check=True,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            text=True,
        )
        script = """
const mod = await import("@bio-rs/biors-wasm");
const required = [
  "browserExecutionPolicy",
  "inspectBrowserFile",
  "validateBrowserFile",
  "tokenizeBrowserFile",
  "parseFasta",
  "validateFasta",
  "tokenize",
  "buildModelInputWithPolicy",
  "runWorkflow",
];
for (const name of required) {
  if (typeof mod[name] !== "function") {
    throw new Error(`${name} is not a function`);
  }
}
if ("default" in mod) {
  throw new Error("unexpected default export");
}
const bytes = new TextEncoder().encode(">seq1\\nACDE\\n");
const records = mod.parseFasta(bytes);
if (records.length !== 1 || records[0].id !== "seq1") {
  throw new Error("parseFasta smoke check failed");
}
const browserInput = { name: "protein.fasta", bytes, kind: "protein", profile: "protein-20" };
const policy = mod.browserExecutionPolicy();
if (policy.network_access !== "none" || policy.uploads_input_data !== false) {
  throw new Error("browser execution policy is not local-only");
}
const inspection = mod.inspectBrowserFile(browserInput);
if (inspection.file.format !== "fasta") {
  throw new Error("inspectBrowserFile smoke check failed");
}
const validation = mod.validateBrowserFile(browserInput);
if (validation.report.error_count !== 0) {
  throw new Error("validateBrowserFile smoke check failed");
}
const tokenization = mod.tokenizeBrowserFile(browserInput);
if (tokenization.tokenization.ids[0] !== "seq1") {
  throw new Error("tokenizeBrowserFile smoke check failed");
}
"""
        subprocess.run(
            ["node", "--input-type=module", "-e", script],
            cwd=work_dir,
            check=True,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            text=True,
        )


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
