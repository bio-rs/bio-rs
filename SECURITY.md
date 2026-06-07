# Security Policy

## Supported Versions

Security reports are accepted for the latest published `0.x` release and the
current `main` branch.

## Reporting a Vulnerability

Please report vulnerabilities privately by opening a GitHub Security Advisory
draft for `bio-rs/bio-rs` or by contacting the repository maintainers through
the project owner profile.

Do not open a public issue for a suspected vulnerability until maintainers have
confirmed the impact and coordinated a disclosure path.

## Scope

Security reports are in scope for promoted or published bio-rs surfaces in the
repository:

- Rust crates: `biors-core`, `biors`, `biors-backend-candle`,
  `biors-mcp-server`, `biors-python`, and `biors-wasm`
- CLI commands, including local FASTA/JSON processing, package validation, and
  package conversion
- Python bindings, WASM/npm package APIs, and MCP tool inputs
- offline service contracts and JSON schemas
- package artifact validation, fixture verification, manifest migration, and
  runtime bridge planning
- guarded external-process backend contracts and process I/O limits
- optional Candle model artifact loading from local safetensors files

The most security-sensitive areas are local filesystem safety, path traversal
and symlink handling, package artifact checksums, malformed JSON and FASTA
inputs, bounded process execution, package path guards, and model artifact
loading boundaries.

bio-rs should not upload biological data, model inputs, package artifacts, or
user content to external services by default. Reports about unintended network
access, telemetry, or data exfiltration are in scope.

## External Process Runtime Boundary

`biors-core` includes a guarded external-process runtime adapter for local
integration work. It is not promoted as a package manifest runtime backend:
package runtime bridge planning rejects `external-process` manifests.

The adapter executes the configured program directly without a shell, passes
arguments without shell interpolation, clears the parent environment by default,
and exposes only explicitly configured environment variables unless
`inherit_environment` is intentionally enabled. Each execution has a timeout,
stdout/stderr byte caps, and stderr byte-count diagnostics that avoid embedding
stderr content in error messages.

Treat the configured program path and optional working directory as trusted local
operator inputs. Do not expose them to untrusted package manifests or remote user
requests without adding a separate trust policy.

Reports are most useful when they include:

- affected command or crate API
- minimal input that reproduces the behavior
- expected impact
- platform and version details

## Disclosure

Maintainers will acknowledge valid reports, assess severity, and publish fixes
through normal GitHub releases and Crates.io package releases.
