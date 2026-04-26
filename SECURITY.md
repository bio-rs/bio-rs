# Security Policy

## Supported Versions

Security reports are accepted for the latest published `0.x` release and the
current `main` branch.

## Reporting a Vulnerability

Please report vulnerabilities privately by opening a GitHub Security Advisory
draft for `bio-rs/bio-rs` or by contacting the repository maintainers through
the project owner profile.

Do not open a public issue for a suspected vulnerability until maintainers have
confirmed the impact and coordinated a disclosure path.

## Scope

The current security surface is limited to local CLI and library processing of
FASTA, JSON manifests, and fixture observations. Reports are most useful when
they include:

- affected command or crate API
- minimal input that reproduces the behavior
- expected impact
- platform and version details

## Disclosure

Maintainers will acknowledge valid reports, assess severity, and publish fixes
through normal GitHub releases and Crates.io package releases.
