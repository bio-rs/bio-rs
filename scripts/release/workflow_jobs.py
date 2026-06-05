from __future__ import annotations

from typing import Any

from release.workflow_job_assertions import StepCheck, assert_job, require_mapping

RUST_TOOLCHAIN_ACTION = "dtolnay/rust-toolchain@98e1b82157cd469e843cb7f524c1313b4ad9492c"


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

    assert_readiness_jobs(jobs)
    assert_publish_jobs(jobs)
    assert_binary_and_release_jobs(jobs)


def assert_readiness_jobs(jobs: dict[str, Any]) -> None:
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


def assert_publish_jobs(jobs: dict[str, Any]) -> None:
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
            StepCheck(name="Wait for biors-core index", run_contains=["https://crates.io/api/v1/crates/{crate}/{version}", "range(1, 91)"]),
            StepCheck(name="Dry-run publish biors-mcp-server", run_contains=["cargo publish --locked -p biors-mcp-server --dry-run"]),
            StepCheck(name="Publish biors-mcp-server", run_contains=["cargo publish --locked -p biors-mcp-server --token"]),
            StepCheck(name="Dry-run publish biors-backend-candle", run_contains=["cargo publish --locked -p biors-backend-candle --dry-run"]),
            StepCheck(name="Publish biors-backend-candle", run_contains=["cargo publish --locked -p biors-backend-candle --token"]),
            StepCheck(name="Wait for biors-backend-candle index", run_contains=["https://crates.io/api/v1/crates/{crate}/{version}", "range(1, 91)"]),
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
            StepCheck(name="Publish Python distributions to PyPI with token", uses="pypa/gh-action-pypi-publish@cef221092ed1bacb1cc03d23a2d87d1d172e277b"),
            StepCheck(name="Publish Python distributions to PyPI with trusted publishing", uses="pypa/gh-action-pypi-publish@cef221092ed1bacb1cc03d23a2d87d1d172e277b"),
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
            StepCheck(name="Set up Node.js", uses="actions/setup-node@48b55a011bda9f5d6aeb4c2d9c7362e8dae4041e", with_values={"node-version": "${{ env.BIORS_RELEASE_NODE_VERSION }}"}),
            StepCheck(name="Install wasm-pack", run_contains=["github.com/rustwasm/wasm-pack/releases/download", "wasm-pack-v${{ env.BIORS_RELEASE_WASM_PACK_VERSION }}-x86_64-unknown-linux-musl", "BIORS_RELEASE_WASM_PACK_SHA256", "sha256sum -c -"]),
            StepCheck(name="Test WASM package", run_contains=["wasm-pack test --node crates/biors-wasm"]),
            StepCheck(name="Build npm package", run_contains=["scripts/build-wasm-npm-package.sh"]),
            StepCheck(name="Check npm package artifact contents", run_contains=["scripts/check-release-artifact-contents.py wasm-package crates/biors-wasm/pkg"]),
            StepCheck(name="Publish npm package with trusted publishing", run_contains=["npm publish crates/biors-wasm/pkg --access public --provenance"]),
        ],
    )


def assert_binary_and_release_jobs(jobs: dict[str, Any]) -> None:
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
            StepCheck(name="Package release binary", run_contains=["tar -C dist -czf", "biors README.md LICENSE-APACHE LICENSE-MIT"]),
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
            StepCheck(name="Verify binary archive checksums", run_contains=["scripts/write-release-checksums.py --verify dist/biors-v*.tar.gz"]),
            StepCheck(name="Create release if missing", run_contains=["gh release create", "--generate-notes", "dist/biors-v*.tar.gz.sha256"]),
        ],
    )
