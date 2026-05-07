# Schema Versioning Policy

bio-rs schemas are public contracts. Each schema-bearing artifact must carry an
explicit version string so tools can parse, validate, migrate, and reject inputs
deterministically.

## Current Contracts

| Contract | Current version | Supported versions | Policy API |
|---|---|---|---|
| Package manifest | `biors.package.v1` | `biors.package.v0`, `biors.package.v1` | `package_manifest_policy()` |
| Pipeline config | `biors.pipeline.v0` | `biors.pipeline.v0` | `pipeline_config_policy()` |

Schema policies are exposed from `biors_core::versioning` so downstream tools
can inspect support status instead of scraping docs.

## Deprecation Policy

- A supported schema version must remain readable for at least two minor
  releases after deprecation is announced.
- Deprecation must be documented in this file and the relevant package or
  pipeline format guide.
- Validators must emit a structured warning before a deprecated version can be
  removed.
- Patch releases must not deprecate or remove schema versions.

`biors.package.v0` is supported, not deprecated.

## Breaking Change Policy

A change is breaking when an existing valid artifact cannot be parsed,
validated, or interpreted with the same biological meaning.

Breaking changes require:

- a new schema version string
- migration notes
- tests for old-version parsing and new-version validation
- a minor release before 1.0, or a major release after 1.0

Patch releases may tighten implementation bugs only when the accepted public
schema contract stays the same.

## Manifest Versioning

Package manifest readers must read all supported manifest versions. Writers
should emit the current version.

The v0 to v1 migration is not fully automatic because v1 adds research metadata
that must come from the package author:

- `package_layout`
- `metadata.license`
- `metadata.citation`
- `metadata.model_card`

`manifest_schema_migration_plan(biors.package.v0, biors.package.v1)` exposes
the required steps for tooling and future CLI helpers.

## Pipeline Config Versioning

Pipeline configs use `schema_version: "biors.pipeline.v0"` starting with the
config MVP. Pipeline config readers must parse the schema tag before
normalization or validation. Unknown schema versions must be rejected with a
stable validation error.

Pipeline config migrations may rewrite syntax when the biological input
semantics, tokenizer behavior, model-input policy, and export target remain
unchanged.

## Backward Compatibility Rules

- Readers must accept every `supported` version returned by the policy API.
- Writers should emit the current version.
- Optional fields may be added in minor releases when old inputs remain valid.
- Required fields, enum removals, renamed fields, and changed defaults require
  a new schema version.
- Validation must not silently infer missing scientific metadata for a migration.

## Migration Strategy

Migrations are schema-tagged and explicit:

1. Parse the old schema by version.
2. Validate it under the old schema.
3. Build a migration plan.
4. Apply deterministic rewrites only when data can be preserved exactly.
5. Require user-supplied metadata when a new schema field changes scientific,
   licensing, citation, or reproducibility meaning.
6. Validate the result under the target schema.

For v0 package manifests, bio-rs can preserve artifact paths and checksums, but
it must not invent license, citation, or model-card metadata.
