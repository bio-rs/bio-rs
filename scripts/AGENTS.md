# SCRIPTS KNOWLEDGE BASE

## OVERVIEW

`scripts/` is executable repository policy: local gates, release preflights,
benchmark report generation, package checks, and maintenance helpers.

## STRUCTURE

```text
scripts/
|-- check*.sh / check*.py       # local, CI, release, dependency gates
|-- release/                    # release workflow invariant helpers
|-- benchmarks/                 # benchmark runners, artifact checks, renderers
|-- *release* / *registry*      # version, registry, checksum, status tooling
`-- build-wasm-npm-package.sh   # WASM/npm package build path
```

## WHERE TO LOOK

| Task | Location | Notes |
| --- | --- | --- |
| Fast local gate | `check-fast.sh` | Iteration and pre-commit subset |
| Full gate | `check.sh` | CI-equivalent local check |
| Final release preflight | `check-final-release.sh` | Strongest local release gate |
| Workflow invariants | `check-release-workflow.py`, `release/` | Pinning, job order, release markers |
| Package artifacts | `check-package-artifacts.sh`, `check-release-artifact-contents.py` | Python, npm, crate, binary contents |
| Registry versions | `check-registry-versions.py` | Tag-release publish preflight |
| Benchmarks | `benchmarks/`, `check-benchmark-docs.sh` | Reports must match JSON artifacts |

## CONVENTIONS

- Match the existing shebang. Most shell gates are `#!/bin/sh`; keep them
  portable. Bash-specific scripts must declare Bash and justify it by existing
  behavior.
- Python scripts should be deterministic CLI tools with clear nonzero failures;
  avoid hidden network calls except explicit registry/status checks.
- Release tool versions are pinned in `release-tool-versions.env`; workflow and
  script pins must stay aligned.
- GitHub Actions references are pinned by policy; update pin checks with
  workflow changes.
- Benchmark scripts update both JSON artifacts and rendered Markdown. Keep
  `check-benchmark-docs.sh` synchronized with every committed benchmark report.
- Registry/version scripts must distinguish local dry-run checks from tag-time
  publish checks.

## CHECKS

```bash
scripts/check-fast.sh
scripts/check.sh
python3 scripts/check-release-workflow.py
scripts/check-final-release.sh
```

Run only the focused script while editing, then the containing gate before
shipping release- or CI-affecting changes.

## ANTI-PATTERNS

- Bypassing script gates by duplicating command lists in docs or workflows.
- Leaving generated wheels, npm packages, binary archives, local caches, or
  temporary release output in the repository.
- Making broad benchmark or release-readiness claims from stale artifacts.
