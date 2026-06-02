#!/usr/bin/env python3
"""Print a compact release status snapshot for the current checkout."""

from __future__ import annotations

import json
import re
import subprocess
import sys


CRATES = [
    "biors-core",
    "biors-mcp-server",
    "biors-backend-candle",
    "biors",
]
NPM_PACKAGE = "@bio-rs/biors-wasm"
PYPI_PACKAGE = "biors"


def main() -> int:
    sha = command(["git", "rev-parse", "HEAD"])
    short_sha = command(["git", "rev-parse", "--short", "HEAD"])
    branch = command(["git", "branch", "--show-current"])
    version = workspace_version()
    tag = f"v{version}" if version else "unknown"

    print("# bio-rs release status")
    print()
    print(f"- branch: {branch or 'detached'}")
    print(f"- commit: {short_sha} ({sha})")
    print(f"- workspace version: {version or 'unknown'}")
    print(f"- expected tag: {tag}")
    print()

    print("## GitHub Actions for current commit")
    print(actions_for_commit(sha))
    print()

    print("## Registry visibility")
    for crate in CRATES:
        print(f"- crates.io {crate}: {crate_version(crate)}")
    print(f"- PyPI {PYPI_PACKAGE}: {pypi_version(PYPI_PACKAGE)}")
    print(f"- npm {NPM_PACKAGE}: {npm_version(NPM_PACKAGE)}")
    return 0


def workspace_version() -> str:
    metadata = command(["cargo", "metadata", "--no-deps", "--format-version", "1"])
    if not metadata:
        return ""
    parsed = json.loads(metadata)
    for package in parsed.get("packages", []):
        if package.get("name") == "biors":
            return str(package.get("version", ""))
    return ""


def actions_for_commit(sha: str) -> str:
    output = command(
        [
            "gh",
            "run",
            "list",
            "--commit",
            sha,
            "--limit",
            "8",
            "--json",
            "databaseId,name,status,conclusion,url",
        ],
        allow_failure=True,
    )
    if not output:
        return "- unavailable: gh is not authenticated or no runs were found"
    runs = json.loads(output)
    if not runs:
        return "- no workflow runs found for this commit"
    lines = []
    for run in runs:
        conclusion = run.get("conclusion") or run.get("status") or "unknown"
        lines.append(
            f"- {run.get('name')}: {conclusion} "
            f"({run.get('url') or 'no url'}, run {run.get('databaseId')})"
        )
    return "\n".join(lines)


def crate_version(crate: str) -> str:
    output = command(["cargo", "search", crate, "--limit", "1"], allow_failure=True)
    match = re.search(rf"^{re.escape(crate)} = \"([^\"]+)\"", output, re.MULTILINE)
    return match.group(1) if match else "unavailable"


def pypi_version(package: str) -> str:
    output = command(
        [
            sys.executable,
            "-m",
            "pip",
            "index",
            "versions",
            package,
            "--disable-pip-version-check",
        ],
        allow_failure=True,
    )
    match = re.search(rf"^{re.escape(package)} \(([^)]+)\)", output, re.MULTILINE)
    return match.group(1) if match else "unavailable"


def npm_version(package: str) -> str:
    output = command(["npm", "view", package, "version"], allow_failure=True)
    return output.strip() or "unavailable"


def command(args: list[str], *, allow_failure: bool = False) -> str:
    try:
        completed = subprocess.run(
            args,
            check=not allow_failure,
            text=True,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            timeout=120,
        )
    except (FileNotFoundError, subprocess.CalledProcessError, subprocess.TimeoutExpired):
        if allow_failure:
            return ""
        raise
    if completed.returncode != 0 and allow_failure:
        return ""
    return completed.stdout.strip()


if __name__ == "__main__":
    raise SystemExit(main())
