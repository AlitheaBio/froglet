<div align="center">

# Froglet

**The rail-neutral evidence layer for agent transactions.**

[![CI](https://github.com/AlitheaBio/froglet/actions/workflows/ci.yml/badge.svg)](https://github.com/AlitheaBio/froglet/actions/workflows/ci.yml)
[![Release](https://github.com/AlitheaBio/froglet/actions/workflows/release.yml/badge.svg)](https://github.com/AlitheaBio/froglet/actions/workflows/release.yml)
[![License: Apache-2.0](https://img.shields.io/badge/License-Apache_2.0-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/Rust-1.91.0-orange.svg)](https://www.rust-lang.org/)
[![Edition](https://img.shields.io/badge/Edition-2024-purple.svg)](https://doc.rust-lang.org/edition-guide/)
[![Docker](https://img.shields.io/badge/Docker-ghcr.io-2496ED.svg)](https://github.com/users/armanas/packages/container/package/froglet-provider)

Whatever rail moves the money, Froglet produces a signed, hash-linked record of
what was authorised, under which limits, and what the executing party attested
to — verifiable offline by anyone, with no node and no account.

Verify a chain yourself in one command:

```bash
cargo run -p froglet-verify -- conformance/kernel_v1.json
```

Maintained by [Armanas Povilionis-Muradian](https://armanas.dev).

</div>

---

## Table of Contents

- [Overview](#overview)
- [Product Model](#product-model)
- [Components](#components)
- [Quick Start](#quick-start)
- [Bot Surfaces](#bot-surfaces)
- [Verification](#verification)
- [Current Scope](#current-scope)
- [Documentation](#documentation)

---

## Overview

**Launch preparation, 2 October 2026:** the current scope is free selected-data
sharing and bounded Wasm through native MCP, with optional configured A2A.
Use the [production handoff and explicit remaining gates](docs/RELEASE.md#production-launch-preparation--2-october-2026).
Local software checks, local packages and the public documentation website do
not qualify a new production compute deployment or an immutable release.

**Development checkpoint, 1 October 2026:** work has resumed on the free native
MCP workflow, optional configured A2A transport, and selected-data sharing with
bounded Wasm computation. The final candidate's actual Codex CLI workflow and
extracted demo bundle are qualified on the current Apple Silicon Mac.
The same production inputs also passed 625 library and nine native MCP tests,
plus the terminology workflow, in an isolated Linux arm64 development container.
Thirteen library tests were ignored; Python OS isolation and a Linux agent host
remain unqualified. Claude Code host qualification is blocked on sign-in.
Selected-data/profile examples and bounded local capacity have recorded checks;
clean machines, public compute HTTPS/capacity, and first-user value remain
separate gates. See [current work](TODO.md#current-checkpoint--1-october-2026)
and the [qualification evidence](docs/AGENT_INTEROPERABILITY_PLAN.md#qualification-results--1-october-2026).

The public documentation website was restored on 1 October; seven checked routes
returned HTTP 200 and matched the local build bytes. See the
[deployment record and remaining gates](docs/RELEASE.md#effortless-publishing-qualification).
The last observed hosted compute trial was unavailable; this pass has not
requalified that deployment. Production and available hosted compute qualification
remain open. The immutable [v0.4.7-beta.1 prerelease](https://github.com/AlitheaBio/froglet/releases/tag/v0.4.7-beta.1) is now
published; its downloaded Apple Silicon binary passed a separate deterministic
localhost replay. [Distribution and exact released-Mac evidence](docs/RELEASE.md#versioning)
also record a separate native-MCP Mac-to-Fly proof: six bounded free cases,
restart persistence and offline task recovery on one isolated test host, now
removed. That proof does not qualify production, load, a fresh agent host,
a clean operating system or paid operation.
The [September pause and rollback record](docs-site/README.md#temporary-pause-2026-09-28)
is preserved. File downloads remain experimental; Node/UI/relay qualification
is open.
General CPU jobs, GPU rental, and new payment work are outside the resumed scope.

The optional [research profile](docs/RESEARCH_PROFILE.md) makes namespace,
version, field types, units, provenance and mapping assumptions explicit. The
final local example retrieves a pinned public GO catalog in 100- and 40-row
pages with verified receipts; this establishes data exchange and declared
metadata compatibility, not scientific correctness or independent user demand.

On one shared machine, eight concurrent tiny workflows using independent
requester identities completed without errors in the measured sweep. At 16 and
32, six selected-data jobs hit the four-process limit and returned independently
verified failure chains. Shared requester identities still face about one
identical requester/workload/terms Quote per second; collisions now fail before
Deal admission and leave no pending requester submissions. These measurements
do not establish production capacity, performance superiority or developer-time
savings. See [the exact local qualification](docs/RELEASE.md#october-candidate-local-qualification).

The current product candidate starts with **“Make this catalog usable by another
agent.”** Codex or Claude Code can prepare selected JSON, typed CSV, or SQLite
data, or a small Wasm function; run a local example; ask for exact publication
approval; and return a provider/service share link. Native setup preserves
existing agent settings. Recipients use the same native integration without npm.

See [the publishing workflow](docs-site/src/content/docs/learn/share-services.mdx)
and [the qualification gates](docs/RELEASE.md#effortless-publishing-qualification).
This is a public prerelease: clean-machine agent tests, the real public
relay journey through this profile, and first-time-user acceptance remain
qualification gates. The [first-user trial kit](docs/AGENT_PUBLISH_ACCEPTANCE.md#first-user-trial-kit--1-october-2026)
separates setup usability from evidence of a useful job. Supported
targets are Apple Silicon macOS and Linux x86_64/arm64. The host must stay online.

The local captured-program replay now has complete evidence: Alice's retained
81,222-byte Wasm program matched four independently frozen answers, and all
seven admitted signed chains were verified offline. The seven include an owner
control, selected-data retrieval, four program executions and a separate looping
program's budget failure. Exact completed data and compute retries recovered
cached results after Bob stopped and Alice restarted. In that historical replay,
`get_task` returned HTTP 502 with Bob stopped. The 3 October correction now
revalidates saved terminal task evidence locally; unsigned or unresolved tasks
still require Bob for refresh. Current-source local regressions pass; a fresh
optimized native candidate also passed independent verification of all seven
chains and all six saved Alice terminal task reads after Bob stopped and Alice
restarted. Package/release gates and website publication remain separate checks.

This is deterministic replay of a program created in an earlier actual Codex
session. That earlier fresh-session qualification remains failed because its
complete chains were not captured and persistent Codex configuration changed.
The replay starts no LLM, compiler or agent settings operation. See the
[reusable runbook](examples/README.md#replay-an-already-generated-program-without-an-llm)
and [local finalization evidence](docs/RELEASE.md#local-finalization--captured-program-replay).

Froglet gives one signed economic primitive for three product shapes:

| Shape | Description |
|---|---|
| **Named Services** | Discoverable, published service endpoints |
| **Data-Backed Services** | Services backed by bot-authored data or projects |
| **Open-Ended Compute** | Raw compute targeted via `provider_id` or `provider_url` |

The primary bot-facing integration surfaces are intentionally simple:

- One OpenClaw/NemoClaw plugin id: `froglet` (at `integrations/openclaw/froglet/`)
- One MCP server under `integrations/mcp/froglet/`, published as
  `froglet-mcp` for `npx froglet-mcp`, plus a dependency-minimal native
  bridge in `froglet-node mcp`
- Both surfaces register a single agent-facing tool named `froglet` to
  the host (Claude Code, Codex, Cursor, Windsurf, etc.). The headline
  action is two-step `marketplace_publish`: the first call is a
  non-mutating plan, and the second must carry the exact approved
  `consent_hash`. Both native and JavaScript adapters delegate to the same
  Rust publication module and provider-control contract.
- The `froglet-node` binary is both the daemon (running as a provider)
  and the author CLI (`froglet-node init` / `build` / `publish` /
  `whoami`, `prepare-service`, `invoke`, `doctor`, and `check-updates`).
  `status --open --json` returns a short-lived read-only local status URL.

Bots should be able to create small scriptable services directly, validate them
locally, and publish them without starting from OCI images.
OCI containers remain a supported packaging and deployment path.

---

## Product Model

- Any Froglet node can publish resources and invoke remote resources
- Published resources are execution bindings backed by bot-authored projects,
  explicit source, or prebuilt artifacts
- Easy bot authoring and local checking of scriptable services is a core
  product requirement
- Identity is first-class in signed artifacts
- Clearnet HTTPS, outbound relay HTTPS, and Tor v3 onion transport are
  supported registration paths for self-hosted providers. Bootstrap plans the
  official relay URL and suffix by default but keeps them dormant; publication
  fails closed until an exact durable grant makes the endpoint ready. Planned
  configuration is not a claim that its public operator service is deployed.

> [!NOTE]
> Marketplace, ranking, incentive, and broker policy live above the protocol.
> Payment rails are adapter-level surfaces for local or self-hosted operators,
> not normal buyer onboarding.
> Lightning, Stripe, and x402 are the launch adapters in this repo. Lightning
> and x402 have standardized kernel settlement methods with conformance
> vectors; Stripe's `stripe_mpp.v1` receipt shape is standardized but attested
> rather than cryptographic. Only Lightning escrow uses the signed
> invoice-bundle flow. x402's wire format is settled
> (`x402.eip3009.v1`, [`conformance/x402_v1.json`](conformance/x402_v1.json))
> but its rail is not: no publish-path exposure and no live transcript on any
> network. The first-party hosted `try.froglet.dev` trial was retired on
> 8 October 2026; the free public beta at <https://froglet.dev/services/>
> replaces it. Hosted paid rails must not be claimed live until Lightning
> and Stripe have public payment transcripts, and users should not be asked to
> manage LND channels or payment secrets just to try Froglet.

<details>
<summary><strong>Discovery & Compute model</strong></summary>

- Named services and data services are discovered through discovery
- Open-ended compute uses the provider's direct compute offer via
  `run_compute`, targeted with `provider_id` or `provider_url`
- Publication and bootstrap adapters may include Nostr-style publication
  without making any single relay or network the kernel source of truth

</details>

---

## Components

Product-wise, Froglet is one node that can both provide and consume.
The reference implementation exposes these binaries:

| Binary | Purpose | Default Port |
|---|---|---|
| `froglet-node` | Provider and/or runtime node (role configured via env) | `8080` / `8081` |

> [!TIP]
> Marketplace integration is part of the public Froglet surface. Runtimes can
> point at the default public marketplace with `FROGLET_MARKETPLACE_URL`;
> providers can self-register there after exposing a public HTTPS URL,
> identity-assigned relay URL, Tor v3 onion URL, or claimed
> `*.providers.froglet.dev` hostname. See
> [docs/MARKETPLACE.md](docs/MARKETPLACE.md).

---

## Prerequisites

**Binary install (quickest):** curl, tar, sha256sum (Linux) or shasum (macOS).
Supported: Linux x86_64/arm64, macOS arm64.

**Build from source:** Rust 1.91+, Python 3.12+ (for tests), Node 18+ with npm
(for Claude Code/Codex MCP setup and integration tests).

**Docker (optional fallback/contributor stack):** Docker with Compose v2.

## Quick Start

Canonical onboarding lives in
[docs-site/src/content/docs/docs.mdx](docs-site/src/content/docs/docs.mdx)
(the `learn/` index remains as a legacy route).
Use the repo README for the product and codebase overview, the docs-site
manual for the public launch path, and `docs/` for specs, operator notes, and
integration reference.

The public launch story still has exactly two entry points:

### 1. Try the public beta

- Open <https://froglet.dev/services/> in a browser: no installation, signup or
  invitation. It runs your bounded Wasm program or a synthetic catalog query on
  a real remote node and returns signed evidence the browser verifies.
- Scope and limits:
  [docs-site/src/content/docs/learn/cloud-trial.mdx](docs-site/src/content/docs/learn/cloud-trial.mdx)
- The older hosted trial at `try.froglet.dev` (and its `ai.froglet.dev`
  origin) was retired on 8 October 2026; see
  [docs/HOSTED_TRIAL.md](docs/HOSTED_TRIAL.md).
- The public beta does not prove paid rails, persistent identity, service
  publication, marketplace depth, or general runtime access.

### 2. Run Locally

- Start with
  [docs-site/src/content/docs/learn/quickstart.mdx](docs-site/src/content/docs/learn/quickstart.mdx)
- Then use
  [docs-site/src/content/docs/learn/agents.mdx](docs-site/src/content/docs/learn/agents.mdx)
  and
  [docs-site/src/content/docs/learn/payment-rails.mdx](docs-site/src/content/docs/learn/payment-rails.mdx)
- Self-host and operator follow-ons live in [docs/DOCKER.md](docs/DOCKER.md),
  [docs/GCP_SINGLE_VM.md](docs/GCP_SINGLE_VM.md), and
  [docs/MARKETPLACE.md](docs/MARKETPLACE.md)

Minimal full local stack from zero:

```bash
set -eu
repo=AlitheaBio/froglet
metadata="$(mktemp "${TMPDIR:-/tmp}/froglet-release.XXXXXX")"
bootstrap="$(mktemp "${TMPDIR:-/tmp}/froglet-agent-bootstrap.XXXXXX")"
release_url="$(curl -fsSL --proto '=https' --proto-redir '=https' --tlsv1.2 -o /dev/null -w '%{url_effective}' "https://github.com/$repo/releases/latest")"
tag="${release_url%/}"; tag="${tag##*/}"
printf '%s' "$tag" | grep -Eq '^v[0-9A-Za-z][0-9A-Za-z.+-]*$'
curl -fsSL --proto '=https' --proto-redir '=https' --tlsv1.2 -H 'Accept: application/vnd.github+json' \
  -H 'X-GitHub-Api-Version: 2026-03-10' \
  "https://api.github.com/repos/$repo/releases/tags/$tag" -o "$metadata"
[ "$(sed -n 's/^  "immutable": \([a-z]*\),*$/\1/p' "$metadata")" = true ]
[ "$(sed -n 's/^  "tag_name": "\([^"]*\)",*$/\1/p' "$metadata")" = "$tag" ]
asset_record="$(awk '
  /^    \{/ { in_asset=1; name=digest=state=""; next }
  in_asset && /^      "name":/ { v=$0; sub(/^      "name": "/,"",v); sub(/",*$/,"",v); name=v }
  in_asset && /^      "digest":/ { v=$0; sub(/^      "digest": "/,"",v); sub(/",*$/,"",v); digest=v }
  in_asset && /^      "state":/ { v=$0; sub(/^      "state": "/,"",v); sub(/",*$/,"",v); state=v }
  in_asset && /^    \},*$/ { if (name=="agent-bootstrap.sh") print digest "|" state; in_asset=0 }
' "$metadata")"
[ "$(printf '%s\n' "$asset_record" | sed '/^$/d' | wc -l | tr -d ' ')" = 1 ]
bootstrap_digest="${asset_record%%|*}"; asset_state="${asset_record#*|}"
[ "$asset_state" = uploaded ]
printf '%s' "$bootstrap_digest" | grep -Eq '^sha256:[0-9a-f]{64}$'
bootstrap_digest="${bootstrap_digest#sha256:}"
curl -fsSL --proto '=https' --proto-redir '=https' --tlsv1.2 \
  "https://github.com/$repo/releases/download/$tag/agent-bootstrap.sh" -o "$bootstrap"
if command -v sha256sum >/dev/null 2>&1; then actual="$(sha256sum "$bootstrap" | awk '{print $1}')";
elif command -v shasum >/dev/null 2>&1; then actual="$(shasum -a 256 "$bootstrap" | awk '{print $1}')";
else actual="$(openssl dgst -sha256 "$bootstrap" | sed 's/^.*= //')"; fi
[ "$actual" = "$bootstrap_digest" ]
chmod 0700 "$bootstrap"
VERSION="$tag" sh "$bootstrap" plan
# Show the complete plan and wait for approval. Then copy its exact hash:
VERSION="$tag" sh "$bootstrap" execute '<install_approval_hash>'
rm -f "$bootstrap" "$metadata"
```

`plan` is the safe default and writes nothing outside its temporary workspace.
It resolves the immutable release, verified manifest and target-platform binary
asset digests, exact bootstrap/install/configuration script digests, persistent
paths, and service-manager impact into one canonical approval hash. `execute`
recomputes that contract and stops before any persistent write if the release,
script bytes, profile, paths, or supplied hash changed.

After approval, the agent bootstrap verifies a Release Bundle, installs the checksum-verified
`froglet-node`, starts the native dual-role service through launchd/systemd,
writes native MCP config, and returns only after health plus a non-seeding MCP
status proof and a transient read-only data proof that is confirmed-unpublished.
The proof first requires an empty active-offer feed and no lifecycle for its
exact service ID, records a private cleanup intent before publishing, and
removes both its offer and authoring directory only after an empty-feed check.
A failed proof attempts exact unpublish while the service is still live and
preserves a recovery marker if cleanup cannot be proved.
A digest-pinned dual-role GHCR image is the fallback
when native service management is unavailable.

The approval plan includes the official relay endpoint and DNS suffix by
default. This is dormant configuration: it derives an exact planned HTTPS URL
but opens no WSS connection and exposes no service before an exact durable
publication grant. Set both `FROGLET_RELAY_URL=''` and
`FROGLET_RELAY_PUBLIC_SUFFIX=''` before `plan` to opt out. The first-party
DNS/TLS endpoint is not yet live-proven, so relay publication remains an
explicit external gate rather than an install success claim. See
[docs/RELAY.md](docs/RELAY.md). The implementation/evidence matrix in
[docs/AGENT_FIRST_PUBLICATION_PLAN.md](docs/AGENT_FIRST_PUBLICATION_PLAN.md)
tracks which clean-host and external gates remain open.

From a trusted checkout, run the disposable-host proof with
`bash scripts/fresh_host_quickstart_smoke.sh`.

The lower-level `scripts/install.sh` is an internal release-bundle installer;
agents should use the approval-gated bootstrap above.

Source-checkout Compose and generated host-side agent configs depend on
`FROGLET_HOST_READABLE_CONTROL_TOKEN=true`; the default user path is the
no-clone `/agent` bootstrap. The quickstart page carries the step-by-step
MCP-first explanation, payment-rail decisions, Tor registration, managed
subdomains, and contributor/source-mode fallbacks.

<details>
<summary><strong>Running binaries directly (without Compose)</strong></summary>

```bash
# Provider node
FROGLET_NODE_ROLE=provider \
FROGLET_PRICE_EXEC_WASM=10 \
FROGLET_PAYMENT_BACKEND=lightning \
FROGLET_LIGHTNING_MODE=mock \
cargo run -p froglet --bin froglet-node
```

```bash
# Runtime node
FROGLET_NODE_ROLE=runtime \
FROGLET_PAYMENT_BACKEND=lightning \
FROGLET_LIGHTNING_MODE=mock \
cargo run -p froglet --bin froglet-node
```

The normal model is one node running both provider and runtime roles
(`FROGLET_NODE_ROLE=dual`), so it can publish local resources and invoke
remote ones.

Set `FROGLET_MARKETPLACE_URL` on runtime nodes to search through an external
marketplace. Providers can self-register with the default public marketplace
after they advertise a matching public HTTPS origin, assigned relay HTTPS URL,
Tor v3 onion URL, or claimed `*.providers.froglet.dev` hostname.

</details>

---

## Bot Surfaces

OpenClaw, NemoClaw, and MCP-compatible hosts are the primary bot-facing
surfaces today. Distribution status and marketplace/plugin ordering live in
[PLUGIN_DISTRIBUTION.md](docs/PLUGIN_DISTRIBUTION.md).

### OpenClaw & NemoClaw

Use the shared plugin package in
[integrations/openclaw/froglet](integrations/openclaw/froglet).
Current OpenClaw plugin install/inspect and gateway invocation require
Node.js `22.14.0` or newer. Gateway-mediated local actions should launch with
`FROGLET_PROVIDER_AUTH_TOKEN_PATH` and `FROGLET_RUNTIME_AUTH_TOKEN_PATH`
pointing at the local `data/runtime/` token files.

<details>
<summary><strong>Configuration keys</strong></summary>

| Key | Purpose |
|---|---|
| `hostProduct` | Target host product |
| `providerUrl` | Provider/public API base URL |
| `runtimeUrl` | Runtime API base URL |
| `providerAuthTokenPath` | Path to the provider control token |
| `runtimeAuthTokenPath` | Path to the runtime auth token |
| `baseUrl` | Legacy single-surface fallback URL |
| `authTokenPath` | Legacy single-token fallback path |
| `requestTimeoutMs` | HTTP request timeout |
| `defaultSearchLimit` | Default discovery result limit |
| `maxSearchLimit` | Maximum discovery result limit |

</details>

The generated local OpenClaw config uses the split provider/runtime keys above.
Legacy `baseUrl` and `authTokenPath` remain supported for single-surface
configs such as the checked-in NemoClaw examples.

The one `froglet` tool covers:

- Service discovery and invocation
- Agent-grade service publication via `marketplace_publish`
- Local artifact publication via `publish_artifact`
- Settlement visibility and current marketplace wrappers
- Status and task polling
- Exact install approval via non-mutating `plan_install`, then approved command
  generation via `get_install_guide`
- Post-install workflow planning via `plan_use_case`
- Raw compute

<details>
<summary><strong>Important behavior notes</strong></summary>

- `summary` is metadata only; it does not generate code
- `publish_artifact` is the current local publication path
- `run_compute` is the low-level path for open-ended compute and should include
  `provider_id` or `provider_url`
- Project authoring, log tailing, and node restart are not part of the current
  public tool API

</details>

### MCP: native default and JavaScript compatibility

The no-clone native install configures the released binary directly:

```text
froglet-node mcp
```

This bridge needs no Node.js runtime. Its deliberately focused `froglet` tool
supports `status`, `invoke_service`, `run_compute`, `get_task`, two-step
`marketplace_publish`, and publication status/logs/pause/resume/rollback/
confirmed-unpublish. Durable cloud-adapter work is separately visible through
managed-operation status, confirmed reconciliation, and confirmed compensation.
`local_proof` is available only when an operator has
deliberately enabled the bundled demo catalog; clean installation does not.
Publication delegates to the same canonical project
loader, consent logic, and provider-control API as the CLI.

For bounded inline Wasm computation and optional A2A transport, use a binary
built from this checkout or a release that includes these actions. The local
[MCP/A2A demo](examples/README.md#local-mcp-and-a2a-compute-demo) starts isolated
Alice and Bob Nodes and exercises the real native MCP path. It uses free
loopback computation and separate private A2A credentials; it does not configure
a production wallet or publish a service externally. The supported profile and
remaining release gates are in the
[interoperability specification](docs/AGENT_INTEROPERABILITY_PLAN.md).

The JavaScript MCP package is the broader compatibility and contributor
surface for discovery, settlement, install planning, raw compute, and existing
host integrations:

```bash
npx froglet-mcp
```

The npm package defaults to `FROGLET_PROFILE=local`, with provider/runtime URLs
pointing at `http://127.0.0.1:8080` and `http://127.0.0.1:8081`. Agents should
call `status` first. If the local node or token files are missing, call
`plan_install`, show its immutable release tag, manifest/bootstrap SHA-256
values, persistent paths, process-manager impact, and exact command preview,
then wait for approval. Only after approval, pass the returned `release_tag`
and `install_approval_hash` unchanged to `get_install_guide` and run its
verified-temp-file command through the host shell. After local health is
verified, use `plan_use_case` before
implementing consumer, provider, evidence, payments, batch, or GPU workflows.
The native `froglet-node mcp` bridge also prepares fixed HTTPS JSON services,
issues expiring invitations into private files, and controls durable admission
limits. See [bounded HTTP services and invitations](docs/HTTP_SERVICES.md).
Payments remain a separate operator decision. Public marketplace canaries
currently require anonymous access; private/invite services use direct provider
invocation.

GPU execution is currently unavailable: the reference OCI worker does not
attach or account for devices. GPU capabilities are not advertised and GPU
requests fail explicitly. Earlier Docker/GCP hardware proof does not qualify
this worker. Batch fan-out, GPU scheduling and general storage remain separate
work. The public no-install proof remains the HTTP flow at
`https://froglet.dev/llms.txt`; it is not an installed MCP action.

For a local node, use the local profile:

```bash
FROGLET_PROFILE=local \
FROGLET_PROVIDER_URL=http://127.0.0.1:8080 \
FROGLET_RUNTIME_URL=http://127.0.0.1:8081 \
FROGLET_PROVIDER_AUTH_TOKEN_PATH=/absolute/path/to/froglet/data/runtime/froglet-control.token \
FROGLET_RUNTIME_AUTH_TOKEN_PATH=/absolute/path/to/froglet/data/runtime/auth.token \
  npx froglet-mcp
```

From a source checkout, the same server can be run directly:

```bash
npm ci --prefix integrations/mcp/froglet
node integrations/mcp/froglet/server.js
```

The npm, source-checkout, and digest-pinned MCP-image modes expose the broader
JavaScript surface over MCP stdio. The native mode exposes the smaller
dependency-free publication/lifecycle surface above; it does not claim action
parity with JavaScript.

For normal users, the `/agent` bootstrap writes the local MCP config without a
repo clone. From a source checkout, contributors can still generate the exact
config file instead of editing JSON or TOML by hand:

```bash
cd froglet && ./scripts/setup-agent.sh --target claude-code
cd froglet && ./scripts/setup-agent.sh --target codex
```

---

## Portable Managed Hosting and OCI Isolation

`froglet-service/v4` expresses managed hosting as a provider-neutral
`target`/`profile`. It does not contain AWS regions, Lightsail names, Fly apps,
ECR repositories, or cloud SDK types. The sibling
[Managed Deployment operator](https://github.com/armanas/froglet-services/tree/main/services/operator)
implements the corresponding portable desired-state lifecycle through the
current Lightsail compatibility adapter or generic SSH + OCI adapter and
returns the same normalized result shape. Publish-engine-to-operator wiring
and credentialed live canaries remain separate open gates.

V3 manifests remain readable. `hosting.default = "fly"` and `hosting.fly.*`
are deprecated compatibility inputs only; new authoring uses v4 `managed`, and
provider selection belongs in operator-owned adapter configuration. See
[docs/PUBLICATION_CONTRACT.md](docs/PUBLICATION_CONTRACT.md#service-manifest-v4-and-hosting-portability).

Arbitrary OCI execution does not give the Froglet Node a Docker/Podman socket.
The node sends a digest-only, bounded, capability-reduced request to an
authenticated worker endpoint configured by `FROGLET_OCI_WORKER_URL` and
`FROGLET_OCI_WORKER_TOKEN_PATH`. The reference
[OCI worker](https://github.com/armanas/froglet-services/tree/main/services/oci-worker)
owns rootless engine access and defaults to read-only, non-root, no-network
execution. With no worker configured, OCI execution fails closed.

---

## Verification

### Verifying artifact chains (what a counterparty does)

`froglet-verify` is a standalone offline verifier: no node, no network, no
account, and no clock unless you supply one. Point it at a chain and it checks
every envelope signature, every per-artifact semantic rule, and every hash link
between artifacts.

```bash
cargo run -p froglet-verify -- conformance/kernel_v1.json
```

It accepts a single artifact, a JSON array, a `/v1/feed` page, or a conformance
fixture, from a file or stdin; `--json` emits a machine-readable report and
`--now <unix>` turns on expiry checks. Exit codes: `0` valid, `1` invalid,
`2` usage error.

To check what a running node publishes, pipe its feed straight in. The feed is
paged, so the verdict covers the page you fetched:

```bash
curl -s 'http://127.0.0.1:8080/v1/feed?limit=50' | cargo run -q -p froglet-verify -- -
```

The Rust facade, browser WASM build, and in-repo Rust conformance runners share
the `froglet-protocol` implementation. [python/froglet-verify](python/froglet-verify)
is independently implemented and checks the same public vectors. Multiple Rust
runners exercise distribution paths; they do not provide implementation diversity. See
[conformance/README.md](conformance/README.md) for what a conforming runner must
assert, and [docs/SPEC.md](docs/SPEC.md) for the normative rules.

### Build and repo checks

**Targeted checks:**

```bash
cargo check -q
cargo test -q --lib
node --check integrations/openclaw/froglet/index.js
node --check integrations/openclaw/froglet/scripts/doctor.mjs
node --test integrations/openclaw/froglet/test/plugin.test.js \
  integrations/openclaw/froglet/test/config-profiles.test.mjs \
  integrations/openclaw/froglet/test/doctor.test.mjs \
  integrations/openclaw/froglet/test/froglet-client.test.mjs
npm run check:mcp
npm run test:mcp
```

**Full repo checks:**

```bash
./scripts/strict_checks.sh
```

<details>
<summary><strong>Compose-backed smoke tests</strong></summary>

Optional compose-backed bot-surface smoke coverage:

```bash
FROGLET_RUN_COMPOSE_SMOKE=1 ./scripts/strict_checks.sh
```

Manual compose-backed smoke commands:

```bash
node integrations/openclaw/froglet/test/compose-smoke.mjs
node integrations/mcp/froglet/test/compose-smoke.mjs
```

</details>

---

## Current Scope

**In this repo now:**

- Protocol and supporting specifications under `docs/` and `conformance/` —
  stability guarantees in [docs/VERSIONING.md](docs/VERSIONING.md); to build a
  second implementation in another language, start from the canonical test
  vectors in [`conformance/kernel_v1.json`](conformance/kernel_v1.json) (guide:
  [froglet.dev/spec/conformance](https://froglet.dev/spec/conformance/))
- Reference Froglet node implementation: a single `froglet-node` binary
  serving split provider and runtime planes (published as `froglet-provider`
  and `froglet-runtime` container images)
- OpenClaw source-plugin integration and shared NemoClaw plugin code, with
  host-specific verification status documented separately
- Native dependency-minimal MCP bridge plus the broader JavaScript MCP server
  for external agent hosts and automations
- Python-backed helpers and tests for the public node and protocol surface
- Local project authoring, build, test, and publish flows for bot-authored
  services
- Direct artifact publication for prebuilt Wasm and OCI-backed profiles
- Reference execution profiles for Wasm, Python, container, and confidential
  execution paths
- Local/self-hosted reference settlement support for operator-controlled
  Lightning, Stripe, and x402
- Clearnet and outbound relay transports plus optional self-hosted Tor and
  Nostr-facing adapter support; live public relay readiness is an external
  deployment gate, not inferred from the client implementation
- Tests, validation scripts, and release docs for the public repo surface
- Public-facing self-host documentation and examples

**Later or separately deployed:**

- First-party hosted paid rail claims for Lightning and Stripe, pending public
  live transcripts; hosted x402 remains desirable but non-blocking
- Higher-layer marketplace ranking, reputation, and policy services
- Long-running batch orchestration, which remains out of scope for the current
  v1 runtime surface
- Credentialed live proof for the provider-neutral Lightsail and generic SSH +
  OCI Managed Deployment adapters maintained in `froglet-services`
- Additional provisioning adapters for GCP, OVH, and similar providers when
  demand justifies them; no new provider enum belongs in the manifest
- Zip or archive packaging as a first-class execution submission format
- First-party hosted control-plane operations and runbooks

> [!WARNING]
> Execution hardening is not uniform across all runtimes.
> Wasm is the strongest runnable in-process isolation path. Confidential/TEE
> artifact types remain protocol scaffolding only: this build rejects mock
> attestation/key-release policies and does not advertise `tee.*` runtimes.
> Python requires Linux Landlock ABI v3 plus seccomp; OCI/container execution
> inherits the separately configured worker's isolation characteristics.

---

## Documentation

| Document | Topic |
|---|---|
| [docs-site/src/content/docs/docs.mdx](docs-site/src/content/docs/docs.mdx) | Canonical onboarding manual for the public launch story |
| [docs-site/src/content/docs/learn/cloud-trial.mdx](docs-site/src/content/docs/learn/cloud-trial.mdx) | Public beta scope and the retired hosted-trial notice |
| [docs-site/src/content/docs/learn/quickstart.mdx](docs-site/src/content/docs/learn/quickstart.mdx) | Local self-host quickstart |
| [docs/README.md](docs/README.md) | Reference-doc map for specs, operations, and integrations |
| [ARCHITECTURE.md](docs/ARCHITECTURE.md) | System architecture overview |
| [ADAPTERS.md](docs/ADAPTERS.md) | Payment and network adapters |
| [RUNTIME.md](docs/RUNTIME.md) | Runtime internals |
| [SERVICE_BINDING.md](docs/SERVICE_BINDING.md) | Service binding model |
| [IDENTITY_ATTESTATION.md](docs/IDENTITY_ATTESTATION.md) | Optional DNS + OAuth identity bindings for Froglet keys |
| [PLUGIN_DISTRIBUTION.md](docs/PLUGIN_DISTRIBUTION.md) | MCP registry and agent-plugin distribution order |
| [OPENCLAW.md](docs/OPENCLAW.md) | OpenClaw integration |
| [NEMOCLAW.md](docs/NEMOCLAW.md) | NemoClaw integration |
| [KERNEL.md](docs/KERNEL.md) | Protocol kernel spec |
| [CONFIDENTIAL.md](docs/CONFIDENTIAL.md) | Confidential execution |
| [NOSTR.md](docs/NOSTR.md) | Nostr publication adapter |
| [STORAGE_PROFILE.md](docs/STORAGE_PROFILE.md) | Storage profiles |
| [GCP_SINGLE_VM.md](docs/GCP_SINGLE_VM.md) | Single-VM self-host deployment wrapper |
| [MARKETPLACE.md](docs/MARKETPLACE.md) | Marketplace integration and the default public marketplace |
| [RELAY.md](docs/RELAY.md) | Relay ingress v1 contract (outbound tunnel, zero-DNS public HTTPS) |
| [ARBITER.md](docs/ARBITER.md) | MVP complaint and marketplace enforcement boundary |
| [HOSTED_TRIAL.md](docs/HOSTED_TRIAL.md) | Retirement record for the former hosted trial |
| [RELEASE.md](docs/RELEASE.md) | Release process |
| [NAME_COHERENCE.md](docs/NAME_COHERENCE.md) | Lightweight launch name and registry-risk note |
| [PAYMENT_MATRIX.md](docs/PAYMENT_MATRIX.md) | Supported payment rails and verification coverage |
| [CONFIGURATION.md](docs/CONFIGURATION.md) | Environment-variable configuration reference |
| [MANIFEST.md](docs/MANIFEST.md) | Service manifest format (froglet-service.toml) |
| [PROVIDER_ONBOARDING.md](docs/PROVIDER_ONBOARDING.md) | Publish path and provider onboarding |
| [API_ERRORS.md](docs/API_ERRORS.md) | API error reference — status codes and error shapes |
| [THREAT_MODEL.md](docs/THREAT_MODEL.md) | Assets, trust boundaries, key-compromise runbook |
| [DOCKER.md](docs/DOCKER.md) | Local compose and container deployment |
| [MOUNTS.md](docs/MOUNTS.md) | Capability-gated data mounts for published services |
| [ROLE_TOOL_ARCHITECTURE.md](docs/ROLE_TOOL_ARCHITECTURE.md) | Role and tool architecture |
| [FEEDBACK.md](docs/FEEDBACK.md) | MVP feedback channel and first-four-weeks triage loop |
| [CODE_OF_CONDUCT.md](CODE_OF_CONDUCT.md) | Community standards (Contributor Covenant 2.1) |
| [CONTRIBUTING.md](CONTRIBUTING.md) | How to contribute |

First-party hosted deployment tooling and operator runbooks are maintained
separately from the public protocol and self-host docs in this repo.

---

<div align="center">

**[Docs Manual Source](docs-site/src/content/docs/docs.mdx)** &middot; **[Releases](https://github.com/AlitheaBio/froglet/releases)** &middot; **[Discussions](https://github.com/AlitheaBio/froglet/discussions)** &middot; **[Issues](https://github.com/AlitheaBio/froglet/issues)** &middot; **[License](LICENSE)**

</div>
