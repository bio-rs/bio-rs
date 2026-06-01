# Dependency Policy

bio-rs keeps the default researcher workflow dependency-light. Release prep must
review dependency growth before publishing crates, wheels, npm packages, or
binary archives.

## Boundaries

- `biors-core` normal dependencies are limited to `serde`, `serde_json`, and
  `sha2`.
- The `biors` CLI may depend on core CLI/config crates, but must not directly
  link optional integration crates such as Candle, MCP, Python, or WASM.
- Default CLI pipeline config support is limited to JSON and TOML; YAML parser
  dependencies are intentionally excluded from the default researcher workflow.
- Heavy or platform-specific integrations stay isolated in package-specific
  crates:
  - `biors-backend-candle` owns Candle dependencies.
  - `biors-mcp-server` owns MCP/Tokio dependencies.
  - `biors-python` owns PyO3 dependencies.
  - `biors-wasm` owns wasm-bindgen/js-sys dependencies.

## Release Gate

Run:

```bash
python3 scripts/check-dependency-policy.py
scripts/check-security-audit.sh
```

The local dependency policy script fails if:

- `biors-core` gains normal dependencies outside its budget.
- `biors-core` or `biors` directly depends on optional integration crates.
- a required heavy integration dependency moves out of its isolated crate.
- any published Rust crate exceeds its current normal dependency-count budget:
  - `biors-core`: 21
  - `biors`: 42
  - `biors-backend-candle`: 123
  - `biors-mcp-server`: 61
- the default `biors` dependency tree includes YAML parser dependencies such as
  `serde_yml`, `serde_norway`, `unsafe-libyaml`, or `unsafe-libyaml-norway`.
- `cargo tree --locked -p biors-core --duplicates` or
  `cargo tree --locked -p biors --duplicates` reports duplicate dependencies.
- `biors-backend-candle` gains duplicate dependency roots beyond the currently
  reviewed Candle transitive set: `hashbrown`, `itertools`, `thiserror`, and
  `thiserror-impl`.

This policy gate is also run by `scripts/check-fast.sh` and `scripts/check.sh`.
The security audit script requires `cargo-deny` and is run by
`scripts/check-final-release.sh` and the release workflow. It checks RustSec
advisories, license allowlist compliance, duplicate-version warnings, wildcard
dependencies, and unknown source registries/git dependencies.

## Review Notes

When a PR or release changes `Cargo.lock` or any `Cargo.toml`, record why the
dependency is needed, which crate owns it, whether it changes the default
CLI/core dependency tree, and how any `cargo-deny` advisory or license finding
was resolved.

For `biors-backend-candle`, the current normal dependency tree is intentionally
budgeted rather than treated as dependency-light. `candle-core v0.10.2` pulls
`tokenizers`, `zip`, `rayon`, GEMM crates, and the duplicate roots listed above.
Keep the crate isolated from `biors-core` and `biors`; do not add CUDA, Metal,
Accelerate, MKL, or model-family-specific features without a new dependency
budget and platform CI policy.

Run `cargo machete` and `cargo udeps` during release-readiness when those tools
are installed. If they are not available, record that in release notes and rely
on the scripted direct-dependency, dependency-count, duplicate-tree, and
`cargo-deny` gates before tagging.
