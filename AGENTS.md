# AGENTS.md

## Role

Work as an autonomous coding agent for bio-rs.

bio-rs is an open-source Rust project for biological AI tooling infrastructure. Keep changes small, inspectable, reproducible, dependency-light, and honest about what is implemented.

## Source of Truth

GitHub code is the implementation source of truth.

Planning notes, roadmap text, issues, README copy, and release ideas are direction unless the behavior is implemented, tested, and documented.

Before changing behavior, inspect the relevant code and existing repo docs:

- `README.md`
- `CONTRIBUTING.md`
- `docs/`
- `schemas/`
- `examples/`
- `.github/`
- `scripts/check.sh`

Do not duplicate detailed contracts here.

Keep implementation, tests, fixtures, schemas, docs, and examples aligned.

Do not claim roadmap items as current features.

## Autonomy

Work without asking for routine decisions.

Do not ask the user to choose branch names, commit messages, file paths, or task splits when the repository can be inspected.

Ask only when the task is destructive, requires secrets, intentionally breaks public behavior, or is genuinely ambiguous in a way that changes user-facing behavior.

When unsure, choose the smallest safe change that preserves current behavior.

## Repository Safety

Before editing or committing, inspect the current branch and working tree.

Do not overwrite unrelated user changes.

Do not stage unrelated files.

Do not commit generated artifacts, caches, logs, build outputs, temporary files, or `target/`.

If generated or local-only files appear and are not ignored, update `.gitignore` in a separate logical commit when appropriate.

## Architecture

Keep reusable domain logic separate from CLI, bindings, packaging, runtime, and integration layers.

Keep core behavior deterministic, testable, and dependency-light.

Write code with Clean Code and SOLID discipline, especially single responsibility.

Each file, module, type, and function should have one clear reason to change.

Split code by logical responsibility before files become hard to scan; external contributors should be able to infer the design from module names and small, focused files.

Make heavy, platform-specific, experimental, or integration-oriented capabilities optional or isolated.

Prefer clear module boundaries before adding crates.

Avoid speculative abstractions, global state, large rewrites, and unnecessary dependencies.

## Quality

Use the repository’s existing checks and CI conventions.

Run `scripts/check.sh` when relevant.

Add or update tests, fixtures, schemas, docs, and examples when behavior changes.

For docs-only changes, code checks are not required unless the docs describe changed behavior.

If a check cannot be run, state why.

## Shipping

For commit, push, release, publish, deploy, versioning, or shipping-safety work, use the repo-shipping workflow.

Commits and pushes may proceed when explicitly requested and safe.

Releases, package publishing, tags, deploys, and version bumps require explicit user approval immediately before execution.

Do not bump versions, create tags, publish packages, deploy, or mark work as released unless the user clearly confirms that exact action.

Do not make benchmark or performance claims without reproducible evidence.

## Security

Do not commit secrets, credentials, private URLs, local env files, telemetry, analytics, or external reporting.

Do not upload biological data or user content to external services unless explicitly requested and documented.

## Final Summary

When finished, report what changed, checks run, public behavior changes, docs updated, and follow-up work left out.

Do not claim work was completed unless it was actually completed.
