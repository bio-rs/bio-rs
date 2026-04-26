# AGENTS.md instructions for bio-rs

## Project

bio-rs is an open-source Rust project rebuilding bio-AI tooling infrastructure.

The goal is to make biological AI tooling faster, more reproducible, easier to run, and easier to package outside Python notebooks.

## Source Of Truth

- The GitHub repository is the implementation source of truth.
- If roadmap notes, planning documents, or implementation conflict, inspect the code and preserve current working behavior unless asked to change it.

## Architecture Rules

- Keep core library code small, stable, and dependency-light.
- Keep command-line interfaces as thin wrappers around core library behavior.
- Do not add heavy dependencies to default builds.
- Inference backends, language bindings, WASM bindings, HTTP servers, and chemistry/structure-heavy dependencies must be optional features or separate crates.
- Prefer module boundaries first. Split into crates only when dependency profile, release cadence, or external usage clearly justifies it.
- Do not introduce global state unless necessary.

## Implementation Rules

- Inspect the repository before editing.
- Do not guess file paths.
- Prefer small, focused changes.
- Preserve public behavior unless the task explicitly asks for a breaking change.
- If changing public CLI behavior, JSON output, schemas, public data contracts, or Rust public API, update docs and tests.
- Prefer clear error types over stringly typed errors.
- Prefer deterministic output.
- Avoid unnecessary abstractions until repeated use justifies them.

## Testing And Quality

Before finishing a task:

- Run `cargo fmt`.
- Run `cargo test`.
- Run relevant targeted tests if the full suite is expensive.
- Add or update tests for changed behavior.
- Add or update fixtures for parser, tokenizer, or schema behavior when relevant.
- Do not introduce unnecessary heavy dependencies.

Default acceptance criteria:

- `cargo fmt` passes.
- `cargo test` passes.
- Relevant tests are added or updated.
- Public behavior is documented if changed.
- No unnecessary heavy dependencies are introduced.

## Documentation

Update documentation when public behavior changes.

Public behavior includes:

- CLI commands and flags
- JSON output schema
- error codes and messages
- package manifest schema
- public data contracts
- Rust public API

## Release Discipline

- Do not bump versions automatically for cleanup or docs-only work.
- Keep versions sequential when a release is requested.
- Ensure each release maps to a real, publishable feature slice.
- Keep package manifests, dependency versions, README, tags, and release notes aligned.
- Publish packages in dependency order discovered from the current workspace.
- Inspect the current release workflow before tagging or publishing.
- Verify the shipped state after release with CI and registry checks.

## README And Public Copy

Keep the main `README.md` aligned with the currently shipped state of `bio-rs`.

Use neutral open-source project tone. Do not write public README copy in a building-in-public or diary style.
