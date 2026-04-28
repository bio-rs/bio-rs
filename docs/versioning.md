# Versioning Policy

`biors-core` and `biors` currently ship in lockstep.

This is intentional for the `0.x` stabilization line:

- the CLI is a thin public wrapper over the core contracts
- CLI JSON schemas expose core data structures directly
- package verification and model-input behavior must stay reproducible across both crates
- lockstep publishing keeps pre-1.0 support and bug triage simpler

Documentation-only changes do not require a version bump or package release.

After `1.0.0`, independent patch releases can be considered only if the change is isolated:

- CLI-only release: command help text, packaging metadata, or human-readable formatting that does not change JSON contracts or core behavior
- core-only release: internal library bug fix that does not affect CLI output, schemas, or package verification behavior

Minor or breaking releases should stay lockstep whenever public contracts, schemas, tokenizer behavior, model-input behavior, or package manifests change.
