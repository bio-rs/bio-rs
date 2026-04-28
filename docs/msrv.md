# MSRV Policy Draft

bio-rs currently validates against the Rust stable toolchain in CI.

For the 1.0 line, the intended policy is:

- Document a concrete minimum supported Rust version before the first stable
  release.
- Keep the MSRV unchanged for patch releases.
- Raise the MSRV only in a minor release and call it out in release notes.
- Keep `biors-core` usable on `wasm32-unknown-unknown` unless a release note
  explicitly says otherwise.

Until that policy is finalized, contributors should run `scripts/check.sh`,
which includes the workspace checks, tests, clippy, and the
`biors-core` WebAssembly build check.
