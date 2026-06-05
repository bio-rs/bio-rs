# CLI CRATE KNOWLEDGE BASE

## OVERVIEW

`crates/biors` owns the `biors` binary, command parsing, CLI JSON envelopes,
local service command, package tooling commands, and release/readiness tests.

## STRUCTURE

```text
src/
|-- main.rs          # binary entry
|-- cli/mod.rs      # command dispatcher
|-- cli/*_args.rs   # clap argument definitions
|-- cli/*           # command handlers and output assembly
|-- input/          # file/stdin/input-source handling
`-- output.rs       # CLI output envelope helpers
tests/
|-- cli_*.rs        # user-facing command behavior
|-- schema_*.rs     # schema contract coverage
`-- release_*.rs    # release/readiness policy assertions
```

## WHERE TO LOOK

| Task | Location | Notes |
| --- | --- | --- |
| Command shape | `src/cli/args.rs`, `src/cli/*_args.rs` | Keep in sync with `docs/cli-contract.md` |
| Command handlers | `src/cli/handlers.rs`, `src/cli/*.rs` | Keep JSON output schema-backed |
| Service command | `src/cli/serve*.rs`, `src/cli/service_args.rs` | Local-first HTTP surface |
| Package commands | `src/cli/package*` | Must preserve package layout and checksums |
| Dataset/batch input resolution | `src/input`, `src/cli/dataset`, `src/cli/batch.rs` | File, directory, stdin, and glob behavior |

## CONVENTIONS

- Treat `docs/cli-contract.md` and `schemas/*.json` as the public CLI contract.
  Change code, tests, docs, and schema together.
- CLI stdout is for the documented output envelope. Keep diagnostics and human
  guidance on stderr unless a schema says otherwise.
- `biors serve` remains local-first: default `127.0.0.1:8787`, no network
  uploads, no hosted workspace behavior, and no external model calls.
- Integration tests should use repo fixtures and shared helpers instead of
  shelling out through ad hoc temp layouts.
- Release policy tests are deliberate guardrails. Update them only when the
  release process or supported surface actually changes.

## CHECKS

```bash
cargo test -p biors
cargo test -p biors --test schema_contract --test schema_cli_package_contract
scripts/check-fast.sh
```

Run schema and release tests for changes that affect CLI JSON, docs inventory,
release workflow assumptions, or packaging commands.

## ANTI-PATTERNS

- Adding a CLI command without schema coverage, docs coverage, and a fixture or
  integration test for expected researcher-facing behavior.
- Hiding validation failures behind generic exit messages.
- Treating service mode as hosted infrastructure.
