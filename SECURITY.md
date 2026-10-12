# Security Policy

## Reporting a Vulnerability

If you discover a security vulnerability in Froglet, please report it
responsibly by emailing **security@alithea-bio.com**.

Please include:

- A description of the vulnerability and its impact
- Steps to reproduce or a proof-of-concept
- The affected version(s)

## What to expect

- **Acknowledgement within 48 hours** of your report.
- **An assessment within 7 days**: severity, affected surfaces, and whether we
  agree it is a vulnerability.
- **A fix or documented mitigation within 90 days** for confirmed issues —
  usually much sooner; this is a small codebase with a single maintainer and
  security fixes jump the queue.
- **Coordinated disclosure**: we ask that you not publish details until a fix
  has shipped or 90 days have elapsed, whichever comes first. We will credit
  you in the changelog and release notes unless you prefer otherwise.
- If a report affects the hosted services (`froglet.dev`,
  `marketplace.froglet.dev`, `relay.froglet.dev`, `arbiter.froglet.dev`), we may
  deploy a mitigation there before the open-source fix lands; the public fix
  still follows the timeline above.

## Scope

This policy covers:

- The Froglet protocol implementation and kernel (`froglet-protocol`,
  `conformance/`)
- The reference node (`froglet-node`, provider and runtime planes, settlement
  drivers including Lightning, Stripe, and the experimental x402 driver)
- The publish engine and bootstrap/install scripts
- The OpenClaw/NemoClaw and MCP integrations and the shared JS client library
- The hosted first-party deployments listed above

Out of scope: vulnerabilities requiring physical access, social engineering of
the maintainer, volumetric denial-of-service against the hosted free tier, and
issues in third-party dependencies that are already public (report those
upstream; we track them via `cargo audit` / `npm audit` in CI).

## Threat model

The protocol's trust boundaries, key-compromise blast radius and response
runbook, and known accepted risks are documented in
[docs/THREAT_MODEL.md](docs/THREAT_MODEL.md). Read it before reporting — it
also tells you which behaviors are deliberate (e.g. non-refundable base fees,
operator-adjudicated marketplace enforcement).

## Supported Versions

Only the latest release on the `main` branch is actively supported with
security fixes. Signed `froglet/v1` artifacts remain verifiable across
versions per [docs/VERSIONING.md](docs/VERSIONING.md), so upgrading the node
never invalidates existing evidence.
