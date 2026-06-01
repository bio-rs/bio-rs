## Summary

- 

## Why

- 

## Changes

- 

## Validation

- [ ] `scripts/check.sh`
- [ ] `cargo test --workspace`
- [ ] Python binding changes: built wheel, installed it in a clean venv, and ran `scripts/test-python-wheel.py`
- [ ] WASM/npm changes: ran `wasm-pack test --node packages/rust/biors-wasm` and checked npm package contents
- [ ] MCP changes: ran MCP integration tests or documented why the surface is unchanged
- [ ] Package artifact changes: checked wheel/sdist, npm package, or binary archive contents as applicable
- [ ] Schema parity reviewed for every changed JSON-emitting CLI, Python, WASM, MCP, package, or service surface
- [ ] Docs/README/final release checklist updated (if needed)
- [ ] Benchmarks updated or explicitly scoped as non-claims (if performance-sensitive behavior changed)
- [ ] Benchmark harness smoke is covered by `.github/workflows/benchmarks.yml` or was run locally with `cargo test --workspace --benches --all-features`

## Dependency Review

- [ ] `Cargo.toml` / `Cargo.lock` changes are justified, or no dependency files changed
- [ ] Default `biors-core` / `biors` dependency tree impact reviewed
- [ ] Published crate dependency-count budget impact reviewed
- [ ] Dependency/advisory/license audit impact reviewed

Dependency release-note impact:

-

## Benchmark / Evidence (if applicable)

- Data source (file/command):
- Result summary:

## Checklist

- [ ] Scope is focused and backwards compatibility impact is described
- [ ] New behavior includes tests
- [ ] Error messages / JSON output shape reviewed
