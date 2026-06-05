# SCHEMAS KNOWLEDGE BASE

## OVERVIEW

`schemas/` contains public JSON Schema contracts for CLI output, service
requests/responses, package manifests, reports, pipeline files, browser/WASM
outputs, and validation diagnostics.

## WHERE TO LOOK

| Task | Location | Notes |
| --- | --- | --- |
| CLI output schema | `*-output.v0.json`, `cli-*.v0.json` | Must match command tests and docs |
| Package manifests | `package-manifest.v0.json`, `package-manifest.v1.json` | Versioned public package contract |
| Service contracts | `service-*.v0.json` | Local service request/response shape |
| Pipeline contracts | `pipeline-*.v0.json` | Pipeline config, lock, output |
| Validation diagnostics | `*-validation-output.v0.json`, `cli-error.v0.json` | Stable issue/error fields |

## CONVENTIONS

- Schema files are public contracts, not loose examples.
- Preserve explicit schema identifiers and version tags. Unknown schema versions
  must remain rejectable by readers with stable validation errors.
- Change schema, implementation, tests, docs, and fixtures in the same slice.
- Prefer additive fields for patch/minor evolution. Tightening required fields,
  removing enum values, or changing field types is a breaking contract change
  unless compatibility handling is implemented.
- Keep package schemas strict about package-relative paths, declared layout
  directories, checksums, and required research metadata.
- Use schema names that match the command/service/report surface; avoid
  placeholder or roadmap schemas.

## CHECKS

```bash
cargo test -p biors --test schema_contract
cargo test -p biors --test schema_cli_package_contract
cargo test -p biors-core --test schema_versioning
scripts/check-fast.sh
```

Add focused schema tests for new surfaces before expanding broad release gates.

## ANTI-PATTERNS

- Keeping unused schemas "just in case".
- Marking future product surfaces as supported before code and tests produce
  matching payloads.
- Weakening validation to make a fixture pass without preserving researcher-use
  error quality.
