# Security Policy

## Supported Versions

Only the latest released version of Bullpen receives security updates. The project is pre-1.0, and older releases are considered unsupported once a newer tag ships.

| Version | Supported |
| ------- | --------- |
| latest  | Yes       |
| older   | No        |

## Reporting a Vulnerability

Please report vulnerabilities privately. Do **not** open a public GitHub issue for suspected security bugs.

Preferred channel:

- [Open a private security advisory](https://github.com/puemos/bullpen/security/advisories/new) on GitHub.

Alternative channel:

- Email the maintainer at **puemos@gmail.com** with the subject line `SECURITY: bullpen`.

When reporting, please include:

- A description of the issue and its impact.
- Steps to reproduce or a proof-of-concept, if available.
- The commit SHA or release version you observed it on.
- Your preferred contact for follow-up.

## Response Expectations

- **Acknowledgement**: within 72 hours of receipt.
- **Initial assessment**: within 7 days, including severity and next steps.
- **Fix + disclosure**: coordinated via the private advisory. A fix and public advisory are published together, with credit to the reporter unless anonymity is requested.

## Scope

In scope:

- The Bullpen desktop application (Rust backend, React frontend, Tauri shell).
- The analysis MCP server and ACP integration code in this repository.
- The landing site under `landing/`.

Out of scope:

- Third-party ACP agents (e.g. Codex, Claude Code) — report upstream.
- Vulnerabilities that require a pre-compromised host or physical access.
- Issues that only affect unsupported builds or local forks.
