# Local Artifact QA

This playbook verifies bio-rs as an AI-ready biological data I/O, validation,
and tokenization engine before any public release action. The no-publish path
uses only local artifacts, local fixtures, loopback service calls, and local
package files. It must not require network access, API keys, tokens,
credentials, telemetry, uploads, tags, or registry writes.

Run the safety-only document check while editing this playbook:

```bash
scripts/check-local-artifact-qa.sh --no-publish --check-doc-safety
```

Run the full no-publish local artifact smoke after building the required local
artifacts:

```bash
scripts/check-local-artifact-qa.sh --no-publish
```

## No-Publish Local QA

The script expects already-built local artifacts and fails if a required local
artifact is missing. That is intentional: the release decision should verify
the same files a researcher or research agent would install or launch.

| Surface | Required local artifact | Smoke |
| --- | --- | --- |
| Release binary CLI workflows | `BIORS_BIN`, default `target/release/biors` | Release binary CLI workflows run `scripts/launch-demo.sh`, then package checks against committed fixtures. |
| MCP stdio tool smoke | `BIORS_MCP_BIN`, default `target/release/biors-mcp-server` | MCP stdio tool smoke boots the local stdio server binary and confirms it stays alive long enough for a client to attach. Tool-call behavior is covered by the MCP integration tests. |
| Python wheel install/import/package API smoke | `BIORS_PYTHON_DIST_DIR`, default `target/package-artifacts/python-dist` | Python wheel install/import/package API smoke installs exactly one local wheel with `pip --no-index`, imports `biors`, parses FASTA, and calls package inspection APIs. |
| WASM/npm build/import smoke | `BIORS_WASM_PKG_DIR`, default `crates/biors-wasm/pkg` | WASM/npm build/import smoke checks the generated local npm package metadata, JS glue, TypeScript declarations, and `.wasm` payload without contacting a registry. |
| Local service release-binary smoke | `BIORS_BIN`, default `target/release/biors` | Local service release-binary smoke starts `biors serve` on `127.0.0.1`, calls `/health` over loopback, and stops the process. |
| Package validate/verify/bridge smoke | `BIORS_BIN`, default `target/release/biors` | Package validate/verify/bridge smoke runs `package validate`, `package verify`, and `package bridge` against `testdata/protein-package`. |

Suggested local artifact preparation:

```bash
cargo build --locked --release -p biors -p biors-mcp-server
BIORS_PACKAGE_ARTIFACT_DIR=target/package-artifacts scripts/check-package-artifacts.sh
```

The second command builds local Python, WASM/npm, and crate package artifacts.
It does not create tags or registry releases.

## Post-Publish Approval Gate

Stop before this section unless the maintainer has given explicit approval for
public release actions in the current conversation. These commands are isolated
here so the no-publish QA path can prove it does not execute them.

Approval-gated examples:

```bash
cargo publish -p biors-core
npm publish crates/biors-wasm/pkg
gh release create vX.Y.Z
```

After explicit approval and public release actions, verify crates.io, PyPI,
npm, and GitHub release visibility with the registry/version checks used by the
release workflow.
