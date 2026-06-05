# PROJECT KNOWLEDGE BASE

**Generated:** 2026-06-05T20:45:07Z
**Commit:** 2595486
**Branch:** chore/init-deep-agents

## OVERVIEW

bio-rs is a Rust workspace for biological AI tooling infrastructure: core
validation/conversion logic, a CLI, optional backend adapters, MCP service
tools, Python bindings, and WASM/npm bindings. GitHub code plus checked-in
tests, schemas, contracts, and fixtures are the source of truth.

## STRUCTURE

```text
bio-rs/
|-- crates/      # workspace crates and binding surfaces
|-- docs/        # public docs for implemented behavior and contracts
|-- schemas/     # JSON Schema contracts for CLI/API outputs and package data
|-- contracts/   # small checked-in reference contracts
|-- testdata/    # end-to-end package, pipeline, and sequence fixtures
|-- scripts/     # local gates, release checks, benchmark tooling
|-- .github/     # CI, benchmark, and release workflows
|-- benchmarks/  # committed benchmark regression artifacts and reports
`-- deploy/      # local deployment templates only
```

## WHERE TO LOOK

| Task | Location | Notes |
| --- | --- | --- |
| Core APIs, parsing, validation | `crates/biors-core` | Deterministic, dependency-light domain logic |
| CLI behavior | `crates/biors` | Command routing, JSON envelopes, service command |
| Python bindings | `crates/biors-python` | PyO3 plus maturin packaging |
| WASM/npm bindings | `crates/biors-wasm` | wasm-bindgen plus npm packaging |
| MCP server | `crates/biors-mcp-server` | Tool routing separate from CLI |
| Optional Candle backend | `crates/biors-backend-candle` | Adapter crate; do not pull backend weight into core |
| Public docs | `docs/` | Describe implemented behavior only |
| JSON schemas | `schemas/` | Public output/package/service contracts |
| Fixtures | `testdata/`, `crates/*/tests/fixtures` | Repo-level end-to-end data vs crate-local focused data |
| Release and CI gates | `scripts/`, `.github/workflows/` | Scripts are the executable policy |

## CONVENTIONS

- Source of truth order: implementation and tests first, then schemas,
  contracts, fixtures, docs, README, roadmap text.
- Keep implementation, tests, test data, schemas, contracts, docs, and
  integration templates aligned whenever behavior changes.
- Do not claim roadmap or "not yet" items as current features.
- Workspace crates ship in lockstep pre-1.0; docs-only changes do not require a
  version bump, but public contract changes usually do.
- Keep `biors-core` deterministic and dependency-light; CLI, bindings,
  packaging, runtime adapters, and integration logic belong outside core unless
  a shared contract requires core ownership.
- Local-first and privacy-first defaults matter: no network, telemetry,
  uploads, external model calls, hosted workspace behavior, or persistence
  unless explicitly requested and documented.
- Benchmark reports are scoped regression guardrails. Do not turn them into
  broad performance claims without fresh workload-specific evidence.
- Package paths must stay package-relative. Reject absolute paths, `..`
  traversal, missing files, checksum mismatches, and unknown schema versions
  with stable errors.

## AUTONOMY

- Work without asking for routine decisions when the repository can be
  inspected.
- Do not ask the user to choose branch names, commit messages, file paths, or
  task splits.
- Ask only when the task is destructive, requires secrets, intentionally breaks
  public behavior, or is genuinely ambiguous in a user-facing way.
- When unsure, choose the smallest safe change that preserves current behavior.

## ANTI-PATTERNS

- Broad rewrites, speculative abstractions, hidden global state, and optional
  heavy dependencies in shared code.
- Generated artifacts, caches, wheels, npm package output, `target/`, local
  virtualenvs, logs, secrets, or internal audit notes in commits.
- Editing release metadata by blind text replacement when a tool can regenerate
  it safely.
- Public docs that overstate DNA/RNA, hosted, browser, service, benchmark, or
  model-execution support beyond what code and tests enforce.

## COMMANDS

```bash
scripts/check-fast.sh
scripts/check.sh
scripts/check-final-release.sh
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

## SHIPPING

- Before editing or committing, inspect branch and working tree.
- Stage only intended files; preserve unrelated user work.
- Use the local repo-shipping workflow for commits, pushes, releases, and
  deployment safety.
- Commits and pushes may proceed when explicitly requested and safe.
- Tags, releases, registry publishing, deployment, and version bumps need
  explicit user approval immediately before execution.
- For release-affecting work, `scripts/check-final-release.sh` is the local
  source of truth before pushing a tag.

## QUALITY

- Run `scripts/check.sh` when behavior, release, or cross-surface contracts
  change.
- For docs-only or instruction-only changes, code checks are not required unless
  the text describes changed behavior.
- If a relevant check cannot be run, state why.

## SECURITY

- Keep vulnerability reports private.
- Do not upload biological data, model inputs, package artifacts, or user
  content to external services by default.
- Do not commit credentials, private URLs, telemetry, analytics, or external
  reporting hooks.

## FINAL SUMMARY

Report changed files, checks run, public behavior changes, docs updated, and
anything intentionally left out. Do not claim completion without verification.
