# Versioning Policy

`biors-core` and `biors` currently ship in lockstep.

This is intentional for the pre-1.0 stabilization line:

- the CLI is a thin public wrapper over the core contracts
- CLI JSON schemas expose core data structures directly
- package verification and model-input behavior must stay reproducible across both crates
- lockstep publishing keeps pre-1.0 support and bug triage simpler

Documentation-only changes do not require a version bump or package release.

After the first stable release, independent patch releases can be considered only if the change is isolated:

- CLI-only release: command help text, packaging metadata, or human-readable formatting that does not change JSON contracts or core behavior
- core-only release: internal library bug fix that does not affect CLI output, schemas, or package verification behavior

Minor or breaking releases should stay lockstep whenever public contracts, schemas, tokenizer behavior, model-input behavior, or package manifests change.

## Schema Versioning

bio-rs schemas are public contracts. Each schema-bearing artifact must carry an
explicit version string so tools can parse, validate, migrate, and reject inputs
deterministically. The code-level policy API lives in `biors_core::versioning`.

Current schema contracts:

| Contract | Current version | Supported versions | Policy API |
|---|---|---|---|
| Package manifest | `biors.package.v1` | `biors.package.v0`, `biors.package.v1` | `package_manifest_policy()` |
| Pipeline config | `biors.pipeline.v0` | `biors.pipeline.v0` | `pipeline_config_policy()` |

Schema policies are exposed from `biors_core::versioning` so downstream tools
can inspect support status instead of scraping docs.

Deprecation policy:

- A supported schema version must remain readable for at least two minor
  releases after deprecation is announced.
- Deprecation must be documented here and in the relevant package or pipeline
  format guide.
- Validators must emit a structured warning before a deprecated version can be
  removed.
- Patch releases must not deprecate or remove schema versions.

`biors.package.v0` is supported, not deprecated.

A change is breaking when an existing valid artifact cannot be parsed,
validated, or interpreted with the same biological meaning. Breaking changes
require a new schema version string, migration notes, tests for old-version
parsing and new-version validation, and a minor release before 1.0 or a major
release after 1.0.

Patch releases may tighten implementation bugs only when the accepted public
schema contract stays the same. Optional fields may be added in minor releases
when old inputs remain valid. Required fields, enum removals, renamed fields,
and changed defaults require a new schema version.

Package manifest readers must read all supported manifest versions. Writers
should emit the current version. The v0 to v1 migration is not fully automatic
because v1 adds research metadata that must come from the package author:

- `package_layout`
- `metadata.license`
- `metadata.citation`
- `metadata.model_card`

`manifest_schema_migration_plan(biors.package.v0, biors.package.v1)` exposes
the required steps for tooling and future CLI helpers.

Pipeline configs use `schema_version: "biors.pipeline.v0"` starting with the
config contract. Pipeline config readers must parse the schema tag before
normalization or validation. Unknown schema versions must be rejected with a
stable validation error.

Migrations are schema-tagged and explicit:

1. Parse the old schema by version.
2. Validate it under the old schema.
3. Build a migration plan.
4. Apply deterministic rewrites only when data can be preserved exactly.
5. Require user-supplied metadata when a new schema field changes scientific,
   licensing, citation, or reproducibility meaning.
6. Validate the result under the target schema.
