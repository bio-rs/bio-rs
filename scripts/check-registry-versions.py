#!/usr/bin/env python3
"""Fail tag releases when configured package versions already exist."""

from __future__ import annotations

import argparse
import json
import os
import subprocess
import tomllib
import urllib.error
import urllib.request
from dataclasses import dataclass
from pathlib import Path


REPO = Path(__file__).resolve().parents[1]
CRATES = ("biors-core", "biors-mcp-server", "biors-backend-candle", "biors")
PYPROJECT = REPO / "packages/rust/biors-python/pyproject.toml"
NPM_PACKAGE = REPO / "packages/rust/biors-wasm/package.json"
TIMEOUT_SECONDS = 20


@dataclass(frozen=True)
class PackageVersion:
    registry: str
    name: str
    version: str
    url: str


def cargo_versions() -> list[PackageVersion]:
    output = subprocess.check_output(
        ["cargo", "metadata", "--no-deps", "--format-version", "1"],
        cwd=REPO,
        text=True,
    )
    metadata = json.loads(output)
    packages = {package["name"]: package["version"] for package in metadata["packages"]}
    missing = [crate for crate in CRATES if crate not in packages]
    if missing:
        raise SystemExit(f"cargo metadata is missing release crate(s): {', '.join(missing)}")

    return [
        PackageVersion(
            "crates.io",
            crate,
            packages[crate],
            f"https://crates.io/api/v1/crates/{crate}/{packages[crate]}",
        )
        for crate in CRATES
    ]


def python_version() -> PackageVersion:
    pyproject = tomllib.loads(PYPROJECT.read_text(encoding="utf-8"))
    project = pyproject["project"]
    name = project["name"]
    version = project["version"]
    return PackageVersion(
        "PyPI",
        name,
        version,
        f"https://pypi.org/pypi/{name}/{version}/json",
    )


def npm_version() -> PackageVersion:
    package = json.loads(NPM_PACKAGE.read_text(encoding="utf-8"))
    name = package["name"]
    version = package["version"]
    escaped_name = name.replace("/", "%2f")
    return PackageVersion(
        "npm",
        name,
        version,
        f"https://registry.npmjs.org/{escaped_name}/{version}",
    )


def release_versions() -> list[PackageVersion]:
    return [*cargo_versions(), python_version(), npm_version()]


def version_exists(package: PackageVersion) -> bool:
    request = urllib.request.Request(
        package.url,
        headers={"User-Agent": "bio-rs-release-check"},
    )
    try:
        with urllib.request.urlopen(request, timeout=TIMEOUT_SECONDS):
            return True
    except urllib.error.HTTPError as error:
        if error.code == 404:
            return False
        raise SystemExit(
            f"could not check {package.registry} version for "
            f"{package.name} {package.version}: HTTP {error.code}"
        ) from error
    except urllib.error.URLError as error:
        fallback = command_version_exists(package)
        if fallback is not None:
            return fallback
        raise SystemExit(
            f"could not check {package.registry} version for "
            f"{package.name} {package.version}: {error.reason}"
        ) from error


def command_version_exists(package: PackageVersion) -> bool | None:
    if package.registry == "crates.io":
        return cargo_version_exists(package.name, package.version)
    if package.registry == "PyPI":
        return pypi_version_exists(package.name, package.version)
    if package.registry == "npm":
        return npm_version_exists_command(package.name, package.version)
    return None


def cargo_version_exists(name: str, version: str) -> bool | None:
    result = run_command(["cargo", "search", name, "--limit", "1"])
    if result is None:
        return None
    return result[0] == 0 and f'{name} = "{version}"' in result[1]


def pypi_version_exists(name: str, version: str) -> bool | None:
    result = run_command(
        ["python3", "-m", "pip", "index", "versions", name, "--disable-pip-version-check"]
    )
    if result is None:
        return None
    return result[0] == 0 and version in result[1]


def npm_version_exists_command(name: str, version: str) -> bool | None:
    result = run_command(["npm", "view", f"{name}@{version}", "version"])
    if result is None:
        return None
    return result[0] == 0 and result[1].strip() == version


def run_command(command: list[str]) -> tuple[int, str, str] | None:
    try:
        result = subprocess.run(
            command,
            cwd=REPO,
            check=False,
            text=True,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
        )
    except FileNotFoundError:
        return None
    return result.returncode, result.stdout, result.stderr


def tag_version() -> str | None:
    tag = os.environ.get("GITHUB_REF_NAME")
    if not tag:
        return None
    return tag.removeprefix("v")


def main() -> None:
    parser = argparse.ArgumentParser(
        description="Check that release package versions are not already published.",
    )
    parser.add_argument(
        "--skip-network",
        action="store_true",
        help="Print configured release package versions without querying registries.",
    )
    args = parser.parse_args()

    versions = release_versions()
    expected = tag_version()
    if expected:
        mismatched = [
            package
            for package in versions
            if package.version != expected
        ]
        if mismatched:
            details = "\n".join(
                f"- {package.registry} {package.name}: {package.version} != tag {expected}"
                for package in mismatched
            )
            raise SystemExit(f"release tag does not match package version(s):\n{details}")

    if args.skip_network:
        for package in versions:
            print(f"{package.registry} {package.name} {package.version}")
        return

    published = [package for package in versions if version_exists(package)]
    if published:
        details = "\n".join(
            f"- {package.registry} {package.name} {package.version}"
            for package in published
        )
        raise SystemExit(
            "release package version(s) already exist; bump versions before tagging:\n"
            f"{details}"
        )

    for package in versions:
        print(f"{package.registry} {package.name} {package.version} is unpublished")


if __name__ == "__main__":
    main()
