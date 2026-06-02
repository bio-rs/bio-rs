#!/usr/bin/env python3
"""Validate release workflow invariants that affect package publication."""

from __future__ import annotations

import json
import re
import subprocess
from pathlib import Path
from typing import Any


WORKFLOW = Path(".github/workflows/release.yml")
RELEASE_TOOL_VERSIONS = Path("scripts/release-tool-versions.env")
UNUSED_RELEASE_TEMPLATE = Path(".github/release_template.md")
RUST_TOOLCHAIN_ACTION = "dtolnay/rust-toolchain@98e1b82157cd469e843cb7f524c1313b4ad9492c"
PINNED_ACTION = re.compile(r"^[^@]+@[0-9a-f]{40}$")


def main() -> None:
    if UNUSED_RELEASE_TEMPLATE.exists():
        raise SystemExit(
            ".github/release_template.md is not used by the release workflow; "
            "keep GitHub --generate-notes as the release body source of truth"
        )

    tool_versions = read_release_tool_versions()
    workflow = load_workflow_yaml(WORKFLOW)
    workflow_text = WORKFLOW.read_text(encoding="utf-8")

    assert_release_triggers(workflow)
    assert_release_env(workflow, tool_versions)
    assert_release_jobs(workflow)
    assert_action_refs_are_pinned(workflow)
    assert_release_tool_scripts_use_pins(tool_versions)
    assert_secondary_text_markers(workflow_text, tool_versions)


def load_workflow_yaml(path: Path) -> dict[str, Any]:
    ruby = (
        "require 'yaml'; require 'json'; "
        "puts JSON.generate(YAML.safe_load(File.read(ARGV[0]), aliases: true))"
    )
    try:
        output = subprocess.check_output(
            ["ruby", "-e", ruby, str(path)],
            text=True,
            stderr=subprocess.PIPE,
        )
    except FileNotFoundError as exc:
        raise SystemExit("Ruby is required to parse release.yml with YAML.safe_load") from exc
    except subprocess.CalledProcessError as exc:
        raise SystemExit(f"failed to parse {path} as YAML: {exc.stderr}") from exc

    workflow = json.loads(output)
    if not isinstance(workflow, dict):
        raise SystemExit(f"{path} must parse to a YAML mapping")
    return workflow


def assert_release_env(
    workflow: dict[str, Any], tool_versions: dict[str, str]
) -> None:
    assert_mapping_value(workflow, ["permissions", "contents"], "read")
    env = require_mapping(workflow, "env")
    for key, expected in tool_versions.items():
        actual = env.get(key)
        if actual != expected:
            raise SystemExit(
                f"release workflow env {key} must match {RELEASE_TOOL_VERSIONS}: "
                f"expected {expected!r}, got {actual!r}"
            )


def assert_release_triggers(workflow: dict[str, Any]) -> None:
    # Ruby's YAML parser follows YAML 1.1 and reads the GitHub Actions `on` key
    # as boolean true. Accept both shapes while still validating the semantics.
    triggers = workflow.get("on", workflow.get("true"))
    if not isinstance(triggers, dict):
        raise SystemExit("release workflow must define workflow_dispatch and push triggers")
    if "workflow_dispatch" not in triggers:
        raise SystemExit("release workflow must keep workflow_dispatch enabled")
    push = triggers.get("push")
    if not isinstance(push, dict):
        raise SystemExit("release workflow must define push trigger details")
    if push.get("branches") != ["main"]:
        raise SystemExit("release workflow push trigger must be limited to main branch")
    if push.get("tags") != ["v*"]:
        raise SystemExit("release workflow push trigger must be limited to v* tags")


def assert_release_jobs(workflow: dict[str, Any]) -> None:
    jobs = require_mapping(workflow, "jobs")

    expected_jobs = {
        "release-readiness",
        "pre-tag-registry-check",
        "publish-crates",
        "build-python-wheels",
        "publish-python",
        "publish-wasm-npm",
        "build-release-binaries",
        "create-github-release",
    }
    missing = expected_jobs - jobs.keys()
    if missing:
        raise SystemExit(f"release workflow is missing jobs: {', '.join(sorted(missing))}")

    assert_job(
        jobs,
        "release-readiness",
        needs=[],
        permissions=None,
        steps=[
            StepCheck(uses=RUST_TOOLCHAIN_ACTION),
            StepCheck(name="Install cargo-deny", run_contains=["cargo install --locked cargo-deny"]),
            StepCheck(name="Run dependency security audit", run_contains=["scripts/check-security-audit.sh"]),
            StepCheck(name="Run checks", run_contains=["scripts/check.sh"]),
        ],
    )
    assert_job(
        jobs,
        "pre-tag-registry-check",
        needs=["release-readiness"],
        permissions={"contents": "read"},
        tag_only=True,
        steps=[
            StepCheck(uses=RUST_TOOLCHAIN_ACTION),
            StepCheck(
                name="Set up Node.js",
                uses="actions/setup-node@48b55a011bda9f5d6aeb4c2d9c7362e8dae4041e",
                with_values={"node-version": "${{ env.BIORS_RELEASE_NODE_VERSION }}"},
            ),
            StepCheck(name="Check package versions are unpublished", run_contains=["scripts/check-registry-versions.py"]),
        ],
    )
    assert_job(
        jobs,
        "publish-crates",
        needs=["release-readiness", "pre-tag-registry-check"],
        permissions={"contents": "read"},
        tag_only=True,
        steps=[
            StepCheck(uses=RUST_TOOLCHAIN_ACTION),
            StepCheck(name="Dry-run publish biors-core", run_contains=["cargo publish --locked -p biors-core --dry-run"]),
            StepCheck(name="Publish biors-core", run_contains=["cargo publish --locked -p biors-core --token"]),
            StepCheck(
                name="Wait for biors-core index",
                run_contains=["https://crates.io/api/v1/crates/{crate}/{version}", "range(1, 91)"],
            ),
            StepCheck(name="Dry-run publish biors-mcp-server", run_contains=["cargo publish --locked -p biors-mcp-server --dry-run"]),
            StepCheck(name="Publish biors-mcp-server", run_contains=["cargo publish --locked -p biors-mcp-server --token"]),
            StepCheck(name="Dry-run publish biors-backend-candle", run_contains=["cargo publish --locked -p biors-backend-candle --dry-run"]),
            StepCheck(name="Publish biors-backend-candle", run_contains=["cargo publish --locked -p biors-backend-candle --token"]),
            StepCheck(
                name="Wait for biors-backend-candle index",
                run_contains=["https://crates.io/api/v1/crates/{crate}/{version}", "range(1, 91)"],
            ),
            StepCheck(name="Dry-run publish biors", run_contains=["cargo publish --locked -p biors --dry-run"]),
            StepCheck(name="Publish biors", run_contains=["cargo publish --locked -p biors --token"]),
        ],
    )
    assert_job(
        jobs,
        "build-python-wheels",
        needs=["release-readiness", "pre-tag-registry-check"],
        permissions=None,
        tag_only=True,
        matrix={"os": ["ubuntu-latest", "macos-latest", "windows-latest"]},
        steps=[
            StepCheck(uses=RUST_TOOLCHAIN_ACTION),
            StepCheck(uses="actions/setup-python@a309ff8b426b58ec0e2a45f0f869d46889d02405"),
            StepCheck(name="Install maturin", run_contains=['"maturin==${{ env.BIORS_RELEASE_MATURIN_VERSION }}"']),
            StepCheck(name="Build wheel", run_contains=["maturin build --release"]),
            StepCheck(name="Build source distribution", run_contains=["maturin sdist"]),
            StepCheck(name="Check Python distribution license files", run_contains=["scripts/check-release-artifact-contents.py python-dist dist"]),
            StepCheck(name="Check Python source distribution license files", run_contains=["--require-sdist"]),
            StepCheck(name="Test installed Python wheel", run_contains=["scripts/test-python-wheel.py --dist-dir dist"]),
            StepCheck(uses="actions/upload-artifact@043fb46d1a93c77aae656e7c1c64a875d1fc6a0a"),
        ],
    )
    assert_job(
        jobs,
        "publish-python",
        needs=["release-readiness", "pre-tag-registry-check", "build-python-wheels"],
        permissions={"contents": "read", "id-token": "write"},
        tag_only=True,
        env={"PYPI_API_TOKEN": "${{ secrets.PYPI_API_TOKEN }}"},
        steps=[
            StepCheck(uses="actions/download-artifact@3e5f45b2cfb9172054b4087a40e8e0b5a5461e7c"),
            StepCheck(
                name="Publish Python distributions to PyPI with token",
                uses="pypa/gh-action-pypi-publish@cef221092ed1bacb1cc03d23a2d87d1d172e277b",
            ),
            StepCheck(
                name="Publish Python distributions to PyPI with trusted publishing",
                uses="pypa/gh-action-pypi-publish@cef221092ed1bacb1cc03d23a2d87d1d172e277b",
            ),
        ],
    )
    assert_job(
        jobs,
        "publish-wasm-npm",
        needs=["release-readiness", "pre-tag-registry-check"],
        permissions={"contents": "read", "id-token": "write"},
        tag_only=True,
        steps=[
            StepCheck(uses=RUST_TOOLCHAIN_ACTION, with_values={"targets": "wasm32-unknown-unknown"}),
            StepCheck(
                name="Set up Node.js",
                uses="actions/setup-node@48b55a011bda9f5d6aeb4c2d9c7362e8dae4041e",
                with_values={"node-version": "${{ env.BIORS_RELEASE_NODE_VERSION }}"},
            ),
            StepCheck(
                name="Install wasm-pack",
                run_contains=[
                    "github.com/rustwasm/wasm-pack/releases/download",
                    "wasm-pack-v${{ env.BIORS_RELEASE_WASM_PACK_VERSION }}-x86_64-unknown-linux-musl",
                ],
            ),
            StepCheck(name="Test WASM package", run_contains=["wasm-pack test --node packages/rust/biors-wasm"]),
            StepCheck(name="Build npm package", run_contains=["scripts/build-wasm-npm-package.sh"]),
            StepCheck(name="Check npm package artifact contents", run_contains=["scripts/check-release-artifact-contents.py wasm-package packages/rust/biors-wasm/pkg"]),
            StepCheck(
                name="Publish npm package with trusted publishing",
                run_contains=[
                    "npm publish packages/rust/biors-wasm/pkg --access public --provenance",
                ],
            ),
        ],
    )
    assert_job(
        jobs,
        "build-release-binaries",
        needs=["release-readiness", "pre-tag-registry-check"],
        permissions={"contents": "read", "id-token": "write", "attestations": "write"},
        tag_only=True,
        matrix={
            "include": [
                {"os": "ubuntu-latest", "target": "x86_64-unknown-linux-gnu"},
                {"os": "macos-latest", "target": "aarch64-apple-darwin"},
            ]
        },
        steps=[
            StepCheck(uses=RUST_TOOLCHAIN_ACTION, with_values={"targets": "${{ matrix.target }}"}),
            StepCheck(name="Build release binary", run_contains=["cargo build --locked --release -p biors --target"]),
            StepCheck(
                name="Package release binary",
                run_contains=["tar -C dist -czf", "biors README.md LICENSE-APACHE LICENSE-MIT"],
            ),
            StepCheck(name="Check binary archive contents", run_contains=["scripts/check-release-artifact-contents.py binary-tarball"]),
            StepCheck(name="Write binary archive checksum", run_contains=["scripts/write-release-checksums.py"]),
            StepCheck(name="Verify binary archive checksum", run_contains=["scripts/write-release-checksums.py --verify"]),
            StepCheck(uses="actions/attest-build-provenance@43d14bc2b83dec42d39ecae14e916627a18bb661"),
            StepCheck(uses="actions/upload-artifact@043fb46d1a93c77aae656e7c1c64a875d1fc6a0a"),
        ],
    )
    assert_job(
        jobs,
        "create-github-release",
        needs=["publish-crates", "publish-python", "publish-wasm-npm", "build-release-binaries"],
        permissions={"contents": "write"},
        tag_only=True,
        steps=[
            StepCheck(uses="actions/download-artifact@3e5f45b2cfb9172054b4087a40e8e0b5a5461e7c"),
            StepCheck(name="Verify binary archive checksums", run_contains=["scripts/write-release-checksums.py --verify dist/*.tar.gz"]),
            StepCheck(name="Create release if missing", run_contains=["gh release create", "--generate-notes", "dist/*.tar.gz.sha256"]),
        ],
    )


class StepCheck:
    def __init__(
        self,
        *,
        name: str | None = None,
        uses: str | None = None,
        run_contains: list[str] | None = None,
        with_values: dict[str, Any] | None = None,
    ) -> None:
        self.name = name
        self.uses = uses
        self.run_contains = run_contains or []
        self.with_values = with_values or {}

    def matches(self, step: dict[str, Any]) -> bool:
        if self.name is not None and step.get("name") != self.name:
            return False
        if self.uses is not None and step.get("uses") != self.uses:
            return False
        run = step.get("run", "")
        if any(text not in run for text in self.run_contains):
            return False
        with_block = step.get("with", {})
        return all(with_block.get(key) == value for key, value in self.with_values.items())

    def describe(self) -> str:
        parts = []
        if self.name:
            parts.append(f"name={self.name!r}")
        if self.uses:
            parts.append(f"uses={self.uses!r}")
        if self.run_contains:
            parts.append(f"run contains {self.run_contains!r}")
        if self.with_values:
            parts.append(f"with {self.with_values!r}")
        return ", ".join(parts)


def assert_job(
    jobs: dict[str, Any],
    job_name: str,
    *,
    needs: list[str],
    permissions: dict[str, str] | None,
    steps: list[StepCheck],
    tag_only: bool = False,
    env: dict[str, str] | None = None,
    matrix: dict[str, Any] | None = None,
) -> None:
    job = require_mapping(jobs, job_name)
    if normalize_needs(job.get("needs")) != needs:
        raise SystemExit(f"release job {job_name} has unsafe needs: {job.get('needs')!r}")
    if tag_only and job.get("if") != "startsWith(github.ref, 'refs/tags/v')":
        raise SystemExit(f"release job {job_name} must only run on v* tags")
    if permissions is not None and job.get("permissions") != permissions:
        raise SystemExit(
            f"release job {job_name} permissions must be {permissions!r}, "
            f"got {job.get('permissions')!r}"
        )
    if env is not None and job.get("env") != env:
        raise SystemExit(f"release job {job_name} env must be {env!r}")
    if matrix is not None:
        assert_matrix(job_name, job, matrix)
    assert_steps(job_name, require_list(job, "steps"), steps)


def assert_matrix(job_name: str, job: dict[str, Any], expected: dict[str, Any]) -> None:
    strategy = require_mapping(job, "strategy")
    matrix = require_mapping(strategy, "matrix")
    for key, value in expected.items():
        if key == "include":
            actual = matrix.get("include")
            if not isinstance(actual, list):
                raise SystemExit(f"release job {job_name} matrix.include must be a list")
            for expected_entry in value:
                if not any(
                    all(entry.get(k) == v for k, v in expected_entry.items())
                    for entry in actual
                    if isinstance(entry, dict)
                ):
                    raise SystemExit(
                        f"release job {job_name} matrix.include missing {expected_entry!r}"
                    )
        elif matrix.get(key) != value:
            raise SystemExit(
                f"release job {job_name} matrix.{key} must be {value!r}, got {matrix.get(key)!r}"
            )


def assert_steps(
    job_name: str, steps: list[Any], expected_steps: list[StepCheck]
) -> None:
    typed_steps = [step for step in steps if isinstance(step, dict)]
    search_from = 0
    for expected in expected_steps:
        for index in range(search_from, len(typed_steps)):
            if expected.matches(typed_steps[index]):
                search_from = index + 1
                break
        else:
            raise SystemExit(
                f"release job {job_name} is missing ordered step: {expected.describe()}"
            )


def assert_action_refs_are_pinned(workflow: dict[str, Any]) -> None:
    jobs = require_mapping(workflow, "jobs")
    unpinned = []
    for job_name, job in jobs.items():
        if not isinstance(job, dict):
            continue
        for step in job.get("steps", []):
            if not isinstance(step, dict) or "uses" not in step:
                continue
            action = step["uses"]
            if not isinstance(action, str) or not PINNED_ACTION.match(action):
                unpinned.append(f"{job_name}: {action!r}")
    if unpinned:
        raise SystemExit(
            "release workflow action refs must be pinned to 40-character SHAs: "
            + ", ".join(unpinned)
        )


def assert_secondary_text_markers(
    workflow_text: str, tool_versions: dict[str, str]
) -> None:
    required_text = [
        f"BIORS_RELEASE_MATURIN_VERSION: '{tool_versions['BIORS_RELEASE_MATURIN_VERSION']}'",
        f"BIORS_RELEASE_WASM_PACK_VERSION: '{tool_versions['BIORS_RELEASE_WASM_PACK_VERSION']}'",
        f"BIORS_RELEASE_NODE_VERSION: '{tool_versions['BIORS_RELEASE_NODE_VERSION']}'",
        "node-version: ${{ env.BIORS_RELEASE_NODE_VERSION }}",
        '"maturin==${{ env.BIORS_RELEASE_MATURIN_VERSION }}"',
        "github.com/rustwasm/wasm-pack/releases/download",
        "wasm-pack-v${{ env.BIORS_RELEASE_WASM_PACK_VERSION }}-x86_64-unknown-linux-musl",
        "x86_64-unknown-linux-gnu",
        "aarch64-apple-darwin",
        "scripts/check-release-artifact-contents.py python-dist dist",
        "scripts/test-python-wheel.py --dist-dir dist",
        "scripts/check-release-artifact-contents.py wasm-package packages/rust/biors-wasm/pkg",
        'scripts/check-release-artifact-contents.py binary-tarball "${{ matrix.archive }}"',
        'scripts/write-release-checksums.py "${{ matrix.archive }}"',
        'scripts/write-release-checksums.py --verify "${{ matrix.archive }}"',
        "scripts/write-release-checksums.py --verify dist/*.tar.gz",
        "${{ matrix.archive }}.sha256",
        "dist/*.tar.gz.sha256",
        "npm publish packages/rust/biors-wasm/pkg --access public --provenance",
        "tar -C dist -czf \"${{ matrix.archive }}\" biors README.md LICENSE-APACHE LICENSE-MIT",
        "scripts/check-registry-versions.py",
        "cargo install --locked cargo-deny",
        "scripts/check-security-audit.sh",
        "--generate-notes",
    ]
    for text in required_text:
        if text not in workflow_text:
            raise SystemExit(f"release workflow is missing packaging marker: {text}")
    forbidden_text = [
        "NODE_AUTH_TOKEN",
        "NPM_TOKEN",
        "registry-url: 'https://registry.npmjs.org'",
        'registry-url: "https://registry.npmjs.org"',
    ]
    for text in forbidden_text:
        if text in workflow_text:
            raise SystemExit(
                "npm trusted publishing must use GitHub OIDC instead of "
                f"long-lived npm token configuration: found {text}"
            )


def read_release_tool_versions() -> dict[str, str]:
    versions = {}
    for line in RELEASE_TOOL_VERSIONS.read_text(encoding="utf-8").splitlines():
        if not line.strip() or line.startswith("#"):
            continue
        key, value = line.split("=", 1)
        versions[key] = value

    required = {
        "BIORS_RELEASE_MATURIN_VERSION",
        "BIORS_RELEASE_WASM_PACK_VERSION",
        "BIORS_RELEASE_NODE_VERSION",
    }
    missing = sorted(required - versions.keys())
    if missing:
        raise SystemExit(f"release tool versions file is missing: {', '.join(missing)}")
    return versions


def assert_release_tool_scripts_use_pins(tool_versions: dict[str, str]) -> None:
    package_artifacts = Path("scripts/check-package-artifacts.sh").read_text(encoding="utf-8")
    wasm_package = Path("scripts/build-wasm-npm-package.sh").read_text(encoding="utf-8")
    version_printer = Path("scripts/print-release-tool-versions.sh").read_text(
        encoding="utf-8"
    )

    for text, name, script in [
        (
            "maturin==$BIORS_RELEASE_MATURIN_VERSION",
            "maturin pin",
            package_artifacts,
        ),
        (
            "cargo install wasm-pack --locked --version $BIORS_RELEASE_WASM_PACK_VERSION",
            "wasm-pack pin",
            wasm_package,
        ),
    ]:
        if text not in script:
            raise SystemExit(f"release tool local script is missing {name}: {text}")

    for key in tool_versions:
        if key not in version_printer:
            raise SystemExit(f"release tool version printer is missing {key}")

    for key, version in tool_versions.items():
        if not version or version.count(".") < 2:
            raise SystemExit(f"{key} must be pinned to an exact patch version, got {version!r}")


def normalize_needs(value: Any) -> list[str]:
    if value is None:
        return []
    if isinstance(value, str):
        return [value]
    if isinstance(value, list) and all(isinstance(item, str) for item in value):
        return value
    raise SystemExit(f"job needs must be a string or string list, got {value!r}")


def require_mapping(mapping: dict[str, Any], key: str) -> dict[str, Any]:
    value = mapping.get(key)
    if not isinstance(value, dict):
        raise SystemExit(f"release workflow key {key} must be a mapping")
    return value


def require_list(mapping: dict[str, Any], key: str) -> list[Any]:
    value = mapping.get(key)
    if not isinstance(value, list):
        raise SystemExit(f"release workflow key {key} must be a list")
    return value


def assert_mapping_value(
    mapping: dict[str, Any], path: list[str], expected: Any
) -> None:
    current: Any = mapping
    for key in path:
        if not isinstance(current, dict) or key not in current:
            raise SystemExit(f"release workflow is missing {'.'.join(path)}")
        current = current[key]
    if current != expected:
        raise SystemExit(
            f"release workflow {'.'.join(path)} must be {expected!r}, got {current!r}"
        )


if __name__ == "__main__":
    main()
