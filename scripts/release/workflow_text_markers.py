from __future__ import annotations

from pathlib import Path

RELEASE_TOOL_VERSIONS = Path("scripts/release-tool-versions.env")


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
        "BIORS_RELEASE_WASM_PACK_SHA256",
        "BIORS_RELEASE_NODE_VERSION",
    }
    missing = sorted(required - versions.keys())
    if missing:
        raise SystemExit(f"release tool versions file is missing: {', '.join(missing)}")
    return versions


def assert_secondary_text_markers(
    workflow_text: str, tool_versions: dict[str, str]
) -> None:
    required_text = [
        f"BIORS_RELEASE_MATURIN_VERSION: '{tool_versions['BIORS_RELEASE_MATURIN_VERSION']}'",
        f"BIORS_RELEASE_WASM_PACK_VERSION: '{tool_versions['BIORS_RELEASE_WASM_PACK_VERSION']}'",
        f"BIORS_RELEASE_WASM_PACK_SHA256: '{tool_versions['BIORS_RELEASE_WASM_PACK_SHA256']}'",
        f"BIORS_RELEASE_NODE_VERSION: '{tool_versions['BIORS_RELEASE_NODE_VERSION']}'",
        "node-version: ${{ env.BIORS_RELEASE_NODE_VERSION }}",
        '"maturin==${{ env.BIORS_RELEASE_MATURIN_VERSION }}"',
        "github.com/rustwasm/wasm-pack/releases/download",
        "wasm-pack-v${{ env.BIORS_RELEASE_WASM_PACK_VERSION }}-x86_64-unknown-linux-musl",
        "sha256sum -c -",
        "x86_64-unknown-linux-gnu",
        "aarch64-apple-darwin",
        "scripts/check-release-artifact-contents.py python-dist dist",
        "scripts/test-python-wheel.py --dist-dir dist",
        "scripts/check-release-artifact-contents.py wasm-package crates/biors-wasm/pkg",
        'scripts/check-release-artifact-contents.py binary-tarball "${{ matrix.archive }}"',
        'scripts/write-release-checksums.py "${{ matrix.archive }}"',
        'scripts/write-release-checksums.py --verify "${{ matrix.archive }}"',
        "scripts/write-release-checksums.py --verify dist/biors-v*.tar.gz",
        "${{ matrix.archive }}.sha256",
        "dist/biors-v*.tar.gz.sha256",
        "npm publish crates/biors-wasm/pkg --access public --provenance",
        "tar -C dist -czf \"${{ matrix.archive }}\" biors README.md LICENSE-APACHE LICENSE-MIT",
        "scripts/check-registry-versions.py",
        "cargo install --locked cargo-deny",
        "scripts/check-security-audit.sh",
        "--generate-notes",
    ]
    for text in required_text:
        if text not in workflow_text:
            raise SystemExit(f"release workflow is missing packaging marker: {text}")

    for text in forbidden_release_text():
        if text in workflow_text:
            raise SystemExit(
                "npm trusted publishing must use GitHub OIDC instead of "
                f"long-lived npm token configuration: found {text}"
            )


def forbidden_release_text() -> list[str]:
    return [
        "NODE_AUTH_TOKEN",
        "NPM_TOKEN",
        "registry-url: 'https://registry.npmjs.org'",
        'registry-url: "https://registry.npmjs.org"',
    ]


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

    sha256 = tool_versions["BIORS_RELEASE_WASM_PACK_SHA256"]
    if len(sha256) != 64 or any(char not in "0123456789abcdef" for char in sha256):
        raise SystemExit(
            "BIORS_RELEASE_WASM_PACK_SHA256 must be a lowercase 64-character sha256"
        )

    for key, version in tool_versions.items():
        if key.endswith("_SHA256"):
            continue
        if not version or version.count(".") < 2:
            raise SystemExit(f"{key} must be pinned to an exact patch version, got {version!r}")
