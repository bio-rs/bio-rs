#!/usr/bin/env python3
"""Install the built Python wheel into a clean venv and run package tests."""

from __future__ import annotations

import argparse
import subprocess
import sys
import tempfile
import venv
from pathlib import Path


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--dist-dir", type=Path, default=Path("dist"))
    parser.add_argument(
        "--tests-dir",
        type=Path,
        default=Path("packages/rust/biors-python/tests"),
    )
    args = parser.parse_args()

    wheel = find_single_wheel(args.dist_dir)
    with tempfile.TemporaryDirectory(prefix="biors-python-wheel-") as temp_dir:
        venv_dir = Path(temp_dir) / "venv"
        venv.EnvBuilder(with_pip=True).create(venv_dir)
        python = venv_python(venv_dir)
        run([python, "-m", "pip", "install", "--upgrade", "pip"])
        run([python, "-m", "pip", "install", "pytest", "jsonschema", str(wheel)])
        run([python, "-m", "pytest", str(args.tests_dir), "-q"])
    return 0


def find_single_wheel(dist_dir: Path) -> Path:
    wheels = sorted(dist_dir.glob("*.whl"))
    if len(wheels) != 1:
        raise SystemExit(f"expected exactly one wheel in {dist_dir}, found {len(wheels)}")
    return wheels[0]


def venv_python(venv_dir: Path) -> str:
    if sys.platform == "win32":
        return str(venv_dir / "Scripts" / "python.exe")
    return str(venv_dir / "bin" / "python")


def run(command: list[str]) -> None:
    subprocess.run(command, check=True)


if __name__ == "__main__":
    raise SystemExit(main())
