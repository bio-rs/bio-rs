# Dependency Policy

bio-rs keeps the default researcher workflow dependency-light. Release prep must
review dependency growth before publishing crates, wheels, npm packages, or
binary archives.

## Boundaries

- `biors-core` normal dependencies are limited to `serde`, `serde_json`, and
  `sha2`.
- The `biors` CLI may depend on core CLI/config crates, but must not directly
  link optional integration crates such as Candle, MCP, Python, or WASM.
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
```

The script fails if:

- `biors-core` gains normal dependencies outside its budget.
- `biors-core` or `biors` directly depends on optional integration crates.
- a required heavy integration dependency moves out of its isolated crate.
- `cargo tree --locked -p biors-core --duplicates` or
  `cargo tree --locked -p biors --duplicates` reports duplicate dependencies.

This gate is also run by `scripts/check-fast.sh` and `scripts/check.sh`.

## Review Notes

When a PR or release changes `Cargo.lock` or any `Cargo.toml`, record why the
dependency is needed, which crate owns it, and whether it changes the default
CLI/core dependency tree. If advisory/license tooling such as `cargo deny`,
`cargo machete`, or `cargo udeps` is adopted later, keep those checks additive
to this boundary policy rather than replacing it.
