# CRATES KNOWLEDGE BASE

## OVERVIEW

`crates/` is a Cargo workspace with thin public surfaces around shared
biological domain logic.

## STRUCTURE

```text
crates/
|-- biors-core/            # shared domain contracts and validation logic
|-- biors/                 # CLI binary and local service command
|-- biors-python/          # PyO3/maturin binding package
|-- biors-wasm/            # wasm-bindgen/npm binding package
|-- biors-mcp-server/      # MCP tool server
`-- biors-backend-candle/  # optional Candle backend adapter
```

## WHERE TO LOOK

| Task | Location | Notes |
| --- | --- | --- |
| Shared records, validators, parsers | `biors-core/src` | Keep reusable and non-CLI-specific |
| CLI command behavior | `biors/src/cli` | Mirrors `docs/cli-contract.md` and `schemas/` |
| CLI contract tests | `biors/tests` | One concern per integration test file |
| Python API | `biors-python/src`, `biors-python/python/biors` | Rust wrappers plus typed Python surface |
| WASM API | `biors-wasm/src`, `biors-wasm/index.d.ts` | JS/WASM boundary and TS declarations |
| MCP tools | `biors-mcp-server/src` | Server/tool routing outside CLI |
| Backend adapter | `biors-backend-candle/src` | Optional runtime bridge only |

## CONVENTIONS

- Keep crate ownership clear. Move shared biological contracts into
  `biors-core`; keep command parsing, output formatting, packaging CLIs,
  Python-specific wrappers, WASM-specific wrappers, and MCP transport code in
  their surface crates.
- Do not introduce a dependency into `biors-core` just because a surface crate
  needs it.
- Workspace package metadata is centralized at the root `Cargo.toml`; avoid
  local drift unless a crate truly has surface-specific packaging metadata.
- Binding crates must preserve parity with core behavior and CLI schemas where
  their API claims the same capability.
- Optional backend crates should report capability/error boundaries without
  changing core contracts.

## CHECKS

```bash
cargo test -p biors-core
cargo test -p biors
cargo test -p biors-mcp-server --all-targets
maturin build --manifest-path crates/biors-python/Cargo.toml
wasm-pack test --node crates/biors-wasm
```

Use the surface-specific command only when that surface changes; run root gates
for cross-crate or release-facing work.

## ANTI-PATTERNS

- Duplicating parser, validation, schema, or package rules across surface
  crates instead of sharing a core contract.
- Letting Python, WASM, MCP, or Candle behavior silently diverge from core
  validation semantics.
- Treating package metadata files as generated throwaways; they are published
  surfaces.
