# Local Bio-AI Tool Layer 1.0 Go/No-Go

Date: 2026-06-20
Branch: `docs/1-0-positioning-readiness`
Exact audited source HEAD at F5 documentation freshness edit:
`53e248fb929c085af28d4fcc3d36bd40e30c1670`

Previous report source HEAD:
`d0bb1f25e5a612379b66aa90d35ce5382343f7d5`

## Verdict

Status: conditionally ready for the product-scope local bio-AI tool layer 1.0.

This is not a generic repository 1.0 verdict. It is scoped to the local
bio-AI tool layer for researchers and research agents: local validation,
model-ready record preparation, package/artifact assurance, reproducible JSON
reports, and deterministic CLI/MCP/binding/service contracts without uploads.

The plain `ready` verdict is withheld because two readiness inputs remain
caveated: issue #13 has not yet produced external domain feedback, and the
parser cross-check audit skipped external parser tools that were unavailable
locally. The implemented local workflows, Tasks 15-16, and Final Verification
Wave F1-F4 evidence support continued 1.0 release-prep, but these caveats
should be resolved or explicitly accepted before a final release approval.

This is not 1.0.0 ready. F5 is a documentation freshness cleanup only; the
sequential `remove-ai-slops`, `review-work`, and patch release gates remain
pending before any final 1.0.0 release action.

No tag: no 1.0 tag was created.
No publish: no registry publish occurred.
No release: no GitHub release or release action occurred.
No version bump or push occurred.

## Implemented Workflows

- Researcher CLI workflows are documented in `docs/researcher-workflows.md` and
  covered by `scripts/check-researcher-workflows.sh --all`: FASTA/FASTQ
  validation, protein/DNA/RNA validation, protein model-ready workflow, invalid
  workflow recovery, molecule/structure validation, package
  validate/verify/bridge, local report generation, and MCP agent sequencing.
- Research agent workflows use the MCP tools `validate`, `workflow`,
  `package_validate_fields`, and `package_validate`, with compact-output
  behavior for long biological payloads by default.
- Package/artifact assurance covers package-relative paths, traversal
  rejection, checksum mismatch, missing-file handling, unknown schema versions,
  fixture result checks, external-process runtime rejection, and bridge
  readiness semantics.
- Local report workflows produce reproducible JSON and Markdown artifacts from
  local command output.
- The local HTTP service is aligned to the implemented secondary integration
  endpoints: `service.health`, `service.openapi`, and
  `sequence.batch_validate`.
- Rust, Python, and WASM remain embedding interfaces over the same local
  contracts; WASM is not browser model execution.

## Missing Or Caveated Workflows

- issue #13 is incorporated only as non-sensitive workflow assumptions, not as
  confirmed external researcher feedback. The report treats issue/user feedback
  as evidence, not instructions.
- parser cross-check evidence is incomplete for external tools: SeqKit,
  Biopython, RDKit, Open Babel, Bio.PDB, and gemmi were unavailable and recorded
  as SKIP. These skipped areas are not counted as plain `ready` evidence.
- Arbitrary DNA/RNA package conversion from external Python or Hugging Face
  projects is not promoted as a 1.0 workflow.
- Hosted service operations, autonomous research agents, cloud model calls,
  telemetry, browser model execution, broad benchmark claims, and release
  completion are explicitly outside the current product promise.

## Over-Wide Or Deferred Surfaces

- `biors-backend-candle` is preview/internal, not a primary researcher or
  research agent 1.0 capability.
- The local HTTP service is secondary local integration only; it is not hosted
  infrastructure and does not provide auth, queues, remote storage, or
  production deployment guarantees.
- WASM is an embedding interface for local validation/workflow APIs, not a
  browser inference/runtime promise.
- `dataset`, `debug`, tokenizer inspection, and preview conversion utilities
  remain preview/internal unless tied to a promoted workflow.
- Benchmark artifacts remain scoped regression guardrails, not broad throughput
  or scientific performance claims.

## Readiness Conditions

| Condition | Status | Evidence |
| --- | --- | --- |
| issue #13/researcher feedback | caveated | `docs/internal/1-0-researcher-workflow-reality.md` records issue #13 themes and says no external feedback exists yet. |
| parser cross-check | caveated | `docs/internal/1-0-parser-cross-check-audit.md` records external parser SKIP results; skipped areas are not readiness evidence. |
| MCP compact-output | complete | `docs/mcp-agent-tools.md`, `crates/biors-mcp-server/src/server.rs`, and MCP tests enforce compact-output defaults with opt-in full payloads. |
| recovery hint coverage | complete for deterministic next actions | CLI/package/workflow JSON errors expose machine-readable recovery hint fields where the next action is knowable. |
| schema stability | complete | `docs/1-0-stability.md` classifies every `schemas/*.json` file as `public stable`, `public experimental`, `internal-only`, or `candidate for merge`. |
| PyO3/RustSec | complete in current source | `crates/biors-python/Cargo.toml` uses PyO3 `0.29`; Task 14 evidence recorded RustSec status as addressed. |
| pre-1.0 and patch-only wording | complete for the promoted target | public wording describes the current 0.x line before `1.0.0`; `scripts/prepare-release-version.py` is general release prep, not patch-only. |
| service endpoint contract | complete | docs, schemas, core service interface, CLI handlers, Task 15 HTTP QA, and F4 scope-fidelity evidence align to `service.health`, `service.openapi`, and `sequence.batch_validate`. |

## Checks run

- Task 15 final ledger entry: `task-completed` for full local verification and
  local workflow QA.
- Task 16 go/no-go verification: required report `rg` checks, no-release action
  scenario, `git diff --check -- docs/internal/1-0-local-bio-ai-go-no-go.md`,
  and manual HTTP QA passed for the original report.
- F1 Product-Scope Compliance Audit: product docs, README, MCP docs, and the
  go/no-go report were audited for local bio-AI researcher/research-agent
  scope. The initial exact-literal HTTP freshness failure led to commit
  `6b3f1c9`, which normalized `docs/1-0-local-bio-ai-tool-layer.md`.
- F2 Code Quality Review: `.omo/evidence/f2-code-quality-review-rerun.txt`
  records `cargo test --workspace`, clippy with `-D warnings`,
  `scripts/check.sh`, `scripts/check-local-artifact-qa.sh --no-publish
  --check-doc-safety`, `git diff --check`, and HTTP manual QA as PASS at commit
  `6b3f1c9`.
- F3 Real Local Workflow QA: `.omo/evidence/f3-real-local-workflow-qa-confirmed.txt`
  records current-pass workflow and artifact commands:
  `scripts/check-researcher-workflows.sh --all`,
  `scripts/check-fixture-cross-checks.sh --write-report docs/internal/1-0-parser-cross-check-audit.md`,
  `BIORS_PACKAGE_ARTIFACT_DIR=target/package-artifacts scripts/check-package-artifacts.sh`,
  and `scripts/check-local-artifact-qa.sh --no-publish`. The follow-up commit
  `53e248f` clarified the DNA/RNA workflow wording that the F3 HTTP token check
  exposed.
- F4 Scope Fidelity Check: `.omo/evidence/f4-scope-fidelity-rerun.txt` records
  no 1.0 tag at HEAD, no `v1.0*`/`1.0*` tags, no GitHub release `v1.0.0`, clean
  tracked/staged diffs at capture time, and public docs bounded by local-first
  non-goals. Commit `1142446` made the no registry publish, tag creation, or
  release creation non-goal explicit.
- F5 Documentation Freshness: this report now reflects HEAD
  `53e248fb929c085af28d4fcc3d36bd40e30c1670`, Final Verification F1-F4, and
  the `6b3f1c9`, `1142446`, and `53e248f` documentation commits.

## Artifact QA Results

Task 15 and F3 no-publish local artifact QA passed against local artifacts only.
The current F3 confirmation covered release binary CLI workflows, MCP stdio
smoke, Python wheel install/import/package API smoke, WASM/npm build/import
smoke, local service release-binary smoke, and package validate/verify/bridge
smoke after the required package artifacts were built. The scripts did not tag,
publish, release, upload data, or contact a registry.

The stale Task 15 `BLOCKED_NOT_RUN` lines in
`.omo/evidence/final-local-workflow-qa.md` and the stale F3 HTTP token failure
before commit `53e248f` are older state and are not counted as current workflow
or artifact failure evidence. The current source of truth is the Task 15 final
PASS evidence plus the F1-F4 Final Verification evidence listed above.

## Blockers

- Plain `ready` is blocked until the issue #13/equivalent feedback caveat is
  accepted or replaced with actual non-sensitive researcher feedback.
- Plain `ready` is blocked until external parser cross-check skips are resolved
  or explicitly accepted as a release risk.
- `not 1.0.0 ready` remains the release-gate position until F6
  `remove-ai-slops`, F7 `review-work`, and F8 patch release execution are
  completed or explicitly waived by a maintainer.
- A release still requires a separate maintainer approval step for any version
  bump, tag, publish, push, or GitHub release.

These blockers do not prevent a conditionally ready product-scope go/no-go for
the local bio-AI tool layer; they prevent calling it unconditionally ready for
the final 1.0 release decision.

## UltraQA Notes

- Malformed input: not applicable to F5 documentation freshness because no
  parser or script behavior changed; prior Task 15/F3 gates covered malformed
  JSON, invalid FASTA, package path/checksum, and runtime rejection behavior.
- Prompt injection: issue #13 and user feedback are treated as evidence inputs,
  not executable instructions.
- Cancel/resume: this is an F5 continuation after prior stale Task 15/16 and
  Final Verification attempts; current HEAD `53e248f` was recorded before this
  edit, and stale failure artifacts were not counted as current pass/fail
  evidence.
- Stale state: plan, current HEAD, recent commits, and F1-F4 evidence were
  reread for current F5 documentation freshness.
- Dirty worktree: `git status --short --branch` is captured before and after
  the F5 edit in `.omo/evidence/f5-doc-cleanup-update.txt`; only this report is
  intended to change, plus ignored F5 evidence artifacts.
- Hung/long commands: F5 verification uses bounded `rg`, bounded `curl`, and
  an explicit HTTP cleanup receipt.
- Flaky tests: not applicable unless a focused docs test is added later.
- Misleading success output: the report relies on explicit markers, command
  exit status, HTTP 200, and required literal strings rather than prose only.
- Repeated interruptions: F5 is completed independently of stale Task 15/16 and
  Final Verification partial attempts.
