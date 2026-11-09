# Changelog

All notable changes to this repo should be recorded here.

The format follows Keep a Changelog. The current release line is `0.4.x`:
`0.2.x` added agent-grade publish (one MCP call from intent to live offer) on
top of the `0.1.x` protocol core, `0.3.x` put paid services on the publish
path (Lightning + Stripe settlement), and `0.4.x` added the phoenixd prepaid
Lightning rail and MCP registry distribution.

## [Unreleased]

### Added

- Synthetic immunopeptidomics reference: two provider-local SQLite datasets, NVIDIA FLARE task components, a bounded Wasm count workload, explicit denominators/missingness, signed evidence and incomplete-result rejection.
- Standalone OpenShell plugin qualification recipe and disposable local backend/probe; corrected directory-upload destinations in NemoClaw setup instructions and doctor output.

## [0.4.7-beta.1] - Prepared 2026-10-03

This section records the beta candidate at preparation, before immutable
publication. Its compute and interoperability evidence is local; new hosted compute, fresh
agent-host sessions, clean-machine installation and paid operation still need
their own qualification. Website publication does not establish those gates.

### Added

- Native MCP `run_compute` and `get_task` for requester-supplied bounded Wasm,
  including a local module-path option. An optional A2A 1.0 adapter carries
  Froglet's signed transactions between explicitly configured requester keys
  and exact Offer hashes. It is disabled by default and supports polling,
  without cancellation, streaming or public federation.
- Explicit selected-data research declarations for field types, namespaces,
  versions, units and provenance. These compare declared metadata; they do not
  establish ontology alignment or scientific correctness.
- A shared website functionality matrix separates current source support,
  local qualification, experimental paths, disabled capabilities and plans.
- A provider-only Fly smoke helper requires an exact released image digest,
  expected node version and explicit organization. Its default private profile
  allows no work and checks authenticated controls and anonymous refusals;
  reachability does not qualify computation or payments.

- `froglet-wasm`: the signing half of the kernel compiled for the browser
  (fresh identities, artifact signing for the five free-deal artifacts, and
  canonical hashing). It reproduces the descriptor, offer, quote, deal, and
  receipt vectors byte for byte and is a thin layer over `froglet-protocol`.
  It runs the in-tab provider and consumer playground on the Developers page.
  No protocol, hashing, signing, or artifact change.
- The Developers-page playground has an editor. Bob's functions are
  TypeScript-like AssemblyScript that the page compiles, in the tab, to a
  `froglet.wasm.run_json.v1` module that imports nothing, then publishes and
  runs. The compiler is loaded from the site's own origin, in a Worker, the
  first time someone compiles. Two opt-in tests, which need a built
  `froglet-node`, check the playground against a real node: one has a node's
  requester call the tab's provider, and the other has a node run the modules
  the editor compiles and compares its answers, receipts, and failures with the
  tab's.
- `examples/wasm-services`: two dependency-free `froglet.wasm.run_json.v1`
  sample services (an adder and a Fibonacci function), built for wasm32. The
  playground's editor starts from AssemblyScript ports of them, and tests hold
  the ports to the same answers.
- `froglet-verify`: a test that an offer naming a runtime the kernel does not
  have, such as `javascript`, is invalid though its signature and hashes are
  genuine.

### Fixed

- Native `get_task` recovers persisted terminal results after provider shutdown
  and requester restart, validating the signed Quote/Deal/Receipt chain and
  cached result before returning it. Pending or unsigned outcomes still refresh
  from the provider; reading a cached Receipt does not authorize new payments
  or establish live payment-rail confirmation.
- Release software checks report skipped required or requested work as
  incomplete, require the core toolchain, and pass package arguments without
  interpolating them into shell code. Local packaging rebuilds the current
  Cargo version for the native host and rejects mismatched binary headers.
  Release archives exclude macOS metadata and require complete checksums and
  exactly the declared regular-file payload. Image publishing waits for the
  operator-approved draft release.
- Publication secret checks include non-ignored untracked source and the
  candidate's history, with narrow annotations for reviewed public retry
  labels. Social-card builds use licensed checked-in font inputs instead of
  mutable remote font downloads.
- Website CI exercises both real-node playground directions. Runtime-only
  identity checks use the identity API instead of a provider startup message;
  missing-page metadata is excluded from indexing.
- Repository-only onboarding fixtures isolate inherited Froglet settings and
  check that existing unrelated agent configuration remains unchanged.
- Release qualification exercises the freshly built optimized node, including
  the required CI example matrix, without changing signed execution budgets.
  Unexpected example statuses report the verified failure code and budget;
  browser publication tests await compilation completion before checking the
  exact published service.
- Release metadata checks include both npm locks and MCP Registry versions.
  CI audits the root npm distribution dependency tree as well as the nested
  MCP server tree. Compatible dependency updates remove the known advisories
  from those two trees; Wasmtime is updated to the patched `36.0.17` line and
  Wrangler to `4.143.1` with patched Undici. The website cache library is
  reviewed at `4.3.0`, whose separate Vary fixes do not change the disputed
  max-stale/cookie behavior. A clean npm audit is not treated as behavioral
  remediation; the actual deployment boundary remains guarded.

- The Python offline verifier returns an invalid JSON report for malformed
  artifact types in plain artifacts, arrays and wrapped feed entries. List or
  object types previously crashed chain grouping before it emitted a report.
- Payment and integration documentation distinguishes the separate x402 HTTP
  adapter from the current signed Quote/Deal flow, which rejects that rail.

- `froglet-verify` (the Rust library and CLI, its WebAssembly build, and the
  Python verifier) reads a node's `/v1/feed` page as served: it verifies the
  artifact under each entry's `document`. Piping a node's feed into the
  verifier no longer needs `jq` and no longer fails with "item is not an
  artifact document".

## [0.4.6-beta.4] - 2026-09-27

### Fixed

- Explicitly release the Unix identity custody lock when its operation ends,
  even if a concurrently spawned child still holds an inherited descriptor.
  Active operations remain exclusive; identity formats and signatures are unchanged.
- Includes the offline invocation recovery correction from the unpublished beta.3
  candidate, whose release gate exposed the inherited-descriptor race.

## [0.4.6-beta.3] - 2026-09-27

### Fixed

- Native CLI and MCP recover saved invocations before querying current provider
  metadata. Completed signed results survive unpublish, provider shutdown and
  requester restart, with exact input/service/provider checks and no new work.
- Native MCP invocation honors `FROGLET_DAEMON_URL` consistently with CLI and
  publication operations.

## [0.4.6-beta.1] - 2026-09-27

### Added

- Private, invitation and bounded trial admission with persistent request, deal
  and runtime allowances, pause/recovery controls, and isolated builtin workers.
- Durable invitation issue/list/revoke, private token files, and native CLI/MCP
  invocation over HTTPS. Invitations are provider-wide and expire.
- Fixed, operator-approved HTTPS JSON services with schema validation, protected
  upstream credentials and finite request/response/deadline limits.
- Agent-readable service pages, clearer marketplace service descriptions, and
  QR codes for sharing public service links.

### Fixed

- Preserve the host-enabled Wasm ABI during publication. Reject unsupported GPU
  execution instead of advertising capabilities the worker cannot enforce.
- Gate release publication on CI, Linux sandbox enforcement and install checks.
- Bound execution, retained data and public request handling. These are resource
  limits, not monetary caps; base hosting and rejected traffic still cost money.

Payments remain deferred. Protected services use direct HTTPS invitations;
marketplace activation still requires anonymous canaries.

## Earlier 0.4.x beta changes

### Added

- **`froglet-node invoke <service_id> [json_input]` — CLI invocation of
  local services.** The Phase 2 stub is now implemented for services
  published on the LOCAL node: the CLI fetches the canonical service record
  (`GET /v1/provider/services/:id`), builds the same service-addressed
  execution workload as the JS client's `buildServiceAddressedExecution`,
  submits it through `POST /v1/runtime/deals`, and polls the deal to a
  terminal state (`--no-wait` / `--timeout-secs N` / `--json` supported;
  input `-` reads stdin). Requester-spend 402 refusals surface their stable
  `code` and remediation. Remote providers still go through the MCP
  `invoke_service` action — full remote resolution stays JS-side.
  `froglet-node publish` output now advertises the CLI command again.

- **Requester spend caps (buyer safety).** New node-local spend policy bounds
  what a node will pay when it creates deals as a buyer, across every
  settlement rail: `FROGLET_REQUESTER_SPEND_BUDGET_MSAT` (cumulative budget,
  persistently tracked in a spend ledger) and `FROGLET_REQUESTER_MAX_DEAL_MSAT`
  (per-deal cap). Enforcement happens before any money moves; refusals are
  402s with stable codes (`spend_budget_unconfigured`, `spend_cap_exceeded`,
  `spend_budget_exceeded`). New endpoints: `GET /v1/runtime/spend`,
  `POST /v1/runtime/spend/reset`. The runtime now also re-checks the signed
  quote against the caller's `max_price_sats` locally instead of trusting the
  provider to enforce it.

### Changed

- **Dual-mode nodes trust their own provider listener for local
  self-invocation.** The runtime API previously refused deals targeting the
  node's own provider over a loopback URL unless the operator exported
  `FROGLET_RUNTIME_PROVIDER_BASE_URL`. A node that binds a provider listener
  (clearnet or the Tor loopback backend) now records it at startup and uses
  it as the default self-dial target, so a bare `froglet-node` (dual role)
  can invoke its own services — `froglet-node invoke`, MCP `invoke_service`,
  raw `POST /v1/runtime/deals` — with zero configuration. The env var
  remains the override for split runtime/provider deployments, provider-id
  identity checks are unchanged, and runtime-only nodes stay fail-closed
  for loopback provider URLs.

- **BREAKING (behavioral, fail-closed): paid deals are refused until
  `FROGLET_REQUESTER_SPEND_BUDGET_MSAT` is configured.** Buyer nodes that pay
  for services must now set an explicit budget; the node logs a startup
  warning when a buyer wallet is configured without one. Free deals are
  unaffected.

- **`froglet-node` no longer links OpenSSL.** The `oci-registry-client`
  dependency hard-wired reqwest's native-tls backend (the binary's only
  `libssl.so.3` linkage); it is replaced by a small in-repo pull client
  (`src/oci.rs`) — token auth, manifest fetch, blob download — on the
  workspace's rustls-only reqwest. This also collapses the duplicate
  reqwest 0.12 stack the crate pinned. Deployment images only need
  `ca-certificates` now (rustls verifies against the system CA store);
  the MCP image no longer installs `libssl3`.

### Fixed

- **Anonymous ghcr.io pulls for OCI Wasm workloads.** The old OCI client
  required `access_token`/`expires_in`/`issued_at` in the token response,
  but ghcr.io's token endpoint returns only `token`, so anonymous auth
  always failed and the subsequent manifest fetch went out without
  credentials. The new client accepts both token shapes. Docker Hub
  references now also send the correct token-scope service
  (`registry.docker.io`) instead of the submitted host alias.

## [0.4.0] - 2026-05-30

### Added

- **phoenixd Lightning backend — the easy self-custodial rail.**
  `FROGLET_LIGHTNING_MODE=phoenixd` points Froglet at an ACINQ
  [phoenixd](https://phoenix.acinq.co/server) node: a single self-custodial
  binary with automatic liquidity (pay-to-open / splicing), HTTP Basic auth,
  and no channel management. Run it, paste a URL + password, start earning.
- **New settlement method `lightning.prepaid.v1`** (additive; existing methods
  and signing bytes unchanged). phoenixd cannot do hold-invoice escrow, so the
  prepaid model is used: the provider mints an ordinary invoice at deal
  creation, the buyer pays it upfront, and the provider confirms payment before
  executing. The signed receipt carries the payment **preimage** as a
  cryptographic proof of payment (`sha256(preimage) == payment_hash`) — strictly
  stronger than Stripe's attested model.
  - Trade-off vs the LND hold-invoice tier: **no escrow**. The buyer pays
    upfront; on execution failure there is no automatic refund — the signed
    `failed` receipt (`settlement_state=settled`, `execution_state=failed`) is
    the buyer's cryptographic evidence. Use `FROGLET_LIGHTNING_MODE=lnd_rest`
    for pay-on-success escrow.
- Buyer-side prepaid payments: set `FROGLET_LIGHTNING_BUYER_PHOENIXD_URL` +
  `FROGLET_LIGHTNING_BUYER_PHOENIXD_HTTP_PASSWORD` so a node can pay providers'
  prepaid invoices from its own phoenixd. Both sides running phoenixd is a fully
  self-custodial, agent-to-agent Lightning exchange.
- `scripts/setup-payment.sh lightning --mode phoenixd` writes the env snippet
  and probes `GET /getinfo`. Non-loopback (real-funds) phoenixd URLs require an
  explicit `FROGLET_LIGHTNING_PHOENIXD_MAINNET_CONFIRM=1` opt-in.

### Verification

- New end-to-end test `phoenixd_prepaid_full_paid_deal_produces_settled_receipt`
  drives the full HTTP provider+buyer flow against a mock phoenixd and asserts a
  kernel-valid `lightning.prepaid.v1` receipt whose preimage hashes to the
  payment hash. Nine new kernel tests cover the method, including the
  mismatched-preimage rejection and the failed-but-paid (settled) receipt.

## [0.3.0] - 2026-05-30

### Added

- Paid services on the publish path. `settlement.method` now accepts
  `"lightning"` and `"stripe"` in addition to `"none"` (the hosted trial stays
  free). Price a service with `[price] sats = N` plus `currency = "sat"`
  (satoshis, settled over Lightning) or `currency = "usd"` (US cents, settled
  via Stripe).
- Stripe settlement rail using Shared Payment Tokens + manual-capture
  PaymentIntents: buyer-side SPT minting and seller-side reserve → capture →
  release, producing a signed `stripe_mpp.v1` receipt. Stripe settlement proof
  is attested (PaymentIntent reference); Lightning remains the
  preimage-verifiable rail.
- `LightningWallet` trait abstracting the settlement backend (LND REST behind
  it; behavior unchanged).

### Security

- SPT validation now fails closed: a Shared Payment Token missing
  `expires_at`, `currency`, or `maximum_amount` is rejected instead of
  silently passing (previously a payment-bypass risk).
- Stripe `secret_key` / `webhook_secret` are redacted in `Debug` output.

### Notes

- Stripe shared-payment is a preview API; the SPT field shapes are validated
  against a mock and must be confirmed against live Stripe before production
  use (see `src/settlement/stripe.rs::mint_spt`).
- Capture-then-persist is not atomic: a DB failure after a successful Stripe
  capture is logged CRITICAL for manual reconciliation; a durable
  reconciliation queue is a follow-up.

## [0.2.1] - 2026-05-15

Same surface as 0.2.0. Three corrections that made 0.2.0 imperfect:

### Fixed

- npm audit at the repo root and inside `integrations/mcp/froglet`
  both now report 0 vulnerabilities. 0.2.0 was published with two
  unpatched advisories in the MCP integration lockfile (high
  `fast-uri`, moderate `hono`) plus four advisories at the root
  (high `fast-uri`, moderate `hono` / `ip-address` / `express-rate-limit`),
  all transitive through `@modelcontextprotocol/sdk`. `npm audit fix`
  resolves both. The advisories are not reachable from
  `marketplace_publish` or any other Froglet code path, but the clean
  audit posture matters for downstream consumers and for the
  `npx -y froglet-mcp` install story.
- All 16 Froglet version sources now agree at 0.2.1. 0.2.0 shipped
  the publish-engine + CLI + MCP work correctly, but four metadata
  files (`server.json`, `plugins/froglet/.codex-plugin/plugin.json`,
  `plugins/froglet/.claude-plugin/plugin.json`,
  `.claude-plugin/marketplace.json`) were left at 0.1.5. The
  docs-site `plugin-distribution.test.ts` caught it and the fix
  landed on main but did not make it into the 0.2.0 tag.
- The 0.2.0 git tag points at commit `201db5e`, which has CI
  failures on the docs-site test and the npm audit gate. 0.2.1
  points at a tip with all gates green so the tagged commit and the
  published Docker images / binary tarballs match a CI-green state.

### Changed

- No feature changes. The publish engine, CLI subcommands, MCP
  tool, manifest v3, Phase 4 harness, and cleanup script ship at
  byte-identical behavior to 0.2.0; the only differences are in
  metadata files, lockfile dependency-resolution, and the
  `Cargo.toml` / `package.json` version strings.

### Migration

If you pulled `ghcr.io/armanas/froglet-mcp:0.2.0` or installed
`froglet-mcp@0.2.0` from npm (it's not yet published there), repull
or reinstall at 0.2.1. The 0.2.0 Docker images remain available
under their original tags; 0.2.1 supersedes them as the recommended
version.

## [0.2.0] - 2026-05-15

The headline of this release is **agent-grade publish**: an LLM with the
Froglet MCP can turn a one-sentence user intent into a live marketplace
offer in a single `marketplace_publish` tool call. Both the CLI and the
MCP wrap the same Rust publish engine, so the two surfaces can't diverge.

### Added

- `froglet-publish-engine` Rust crate — orchestrates
  build → host → sign → register against the running `froglet-node`
  daemon. Three hosting backends ship in this release: Local (loopback
  only), Tor (auto-onion via the existing hidden-service helper), and
  SelfHosted (operator-supplied URL). Managed and Fly backends are
  deferred to 0.3.
- `froglet-node` CLI subcommands: `init`, `build`, `publish`, `invoke`,
  `whoami`. `init` scaffolds a project (`froglet.toml`,
  `froglet-service.toml`, `handler.py`, `.gitignore`); `publish` reads
  the manifests and calls the engine.
- `marketplace_publish` MCP action in
  `integrations/mcp/froglet/lib/tools.js`. Shells out to
  `froglet-node publish --json` so MCP and CLI can never drift. Input
  mirrors the manifest; output returns `provider_id`, `public_url`,
  `marketplace_offer_url`, `offer_hash`, `invoke_command`.
- Manifest v3: project-level `froglet.toml` + per-service
  `froglet-service.toml`, parsed by `froglet-protocol::manifest` with
  `deny_unknown_fields` and 20 unit tests. v2 manifests still load with
  a deprecation warning rather than an error.
- Phase 4 LLM acceptance matrix harness (`tests/llm_acceptance/`):
  5 prompts × 2 Claude models × 3 hosting backends = 30 cells,
  dual pass-bar (≥27 count AND ≥90% rate). Per-cell JSON transcripts +
  summary.tsv. Stable failure categories (`tool-not-called`,
  `tool-misuse`, `engine-error`, `marketplace-lag`, `llm-loop`).
- 25 regression tests in `tests/llm_acceptance/test_validators.py`
  binding the harness contract — every defect a reviewer caught in the
  initial pass now has a test that would have caught it.
- `scripts/llm_acceptance_cleanup.py` (Phase 4.6) — extracts
  `provider_id`s from a matrix run's cell JSONs and emits idempotent
  SQL suspending each test provider via `provider_enforcements`.
- `ops/hosted_smoke.sh` — single entrypoint to verify hosted surfaces
  (`marketplace.froglet.dev` /healthz + `/v1/providers`,
  `arbiter.froglet.dev` /v1/feed) are reachable and well-shaped.
- Docs: `docs/MANIFEST.md` (v3 spec, both files);
  `docs/PROVIDER_ONBOARDING.md` rewritten to lead with
  `marketplace_publish` / `froglet-node publish`, with the four
  DNS-less paths demoted from "user paths" to "backend implementation
  details."
- Launch packet (`_tmp/launch_packet/{show_hn.md,blog_post.md,faq.md}`)
  rewritten around the one-MCP-call narrative with real evidence
  references.

### Changed

- Bumped workspace versions: `froglet` 0.1.1 → 0.2.0; `froglet-protocol`
  and `froglet-publish-engine` 0.1.0 → 0.2.0; `@froglet/mcp-server`
  0.1.5 → 0.2.0.
- `froglet-node`'s `args[1]` dispatch refactored into a small subcommand
  enum + `CliHandler` trait. No new dependency — does not pull in
  `clap` — but the dispatcher now produces stable exit codes via
  `CliError::exit_code()`.
- `arbiter.froglet.dev` is now a real Froglet provider (it appears in
  `/v1/feed`) rather than a legacy axum-only service. The previous
  `/healthz` route no longer exists; smoke scripts should check
  `/v1/feed` instead.

## [0.1.0-alpha.2] - 2026-05-10 (rolled up from prior Unreleased section)

### Added

- Docs-site and hosted-trial privacy posture note: v0.1.0 has no account,
  email, analytics cookie, or conversion tracking in the public hosted trial
  contract.
- DCO contribution policy, with manual maintainer enforcement for v0.1.0
  unless a DCO bot is installed.

### Changed

- Public claim language now states that `try.froglet.dev` exposes a free
  hosted demo catalog: `demo.add` is the canonical round-trip proof, while
  witness/hash/notarize flows are optional follow-up evidence and Lightning,
  Stripe, and x402 remain local/self-hosted launch adapters.
- `docs/RELEASE.md` now includes a v0.1.0 GitHub release body draft and
  separates scripted release gates from hard manual launch gates such as
  Claude MCP smoke and hosted-trial verification.
- `README.md` hosted-trial paragraph rewritten to match the MVP scope: a
  first-party hosted gateway in front of the reference node, a shared
  session-token pool (authentication only, not per-session identity), five free
  demo services, and explicit removal of the email-claim / account-conversion
  path.
- Docs site `Try In Cloud` page (`docs-site/src/content/docs/learn/cloud-trial.mdx`)
  rewritten to describe the shared session-pool model and remove the stale
  `POST /api/sessions/claim` / `verify` / `resume` endpoints and stale
  hosted-provider implementation details.
- Docs site landing (`docs-site/src/pages/index.astro`) hosted-trial card
  updated to match the session-pool model and drop the GCP wording.
- Docs site learning index (`docs-site/src/content/docs/learn/index.mdx`)
  updates its roadmap copy and stat strip to reference the shared session
  pool and 15-minute session TTL instead of "temporary identity".

### Removed

- Pre-publication launch plans, distribution matrices, historical playground
  plans, first-party hosted runbook stubs, and one-off security evidence docs
  that belonged in private operator/release records rather than the public
  open-source repo.

## [0.1.0-alpha.1] - 2026-04-20

### Added

- Public boundary notes clarifying that first-party hosted deployment,
  monitoring, and rotation runbooks now live in the private services/operator
  workspace.
- `docs-site/wrangler.jsonc` plus `docs-site` Wrangler scripts for the public
  Cloudflare Workers docs deployment path

### Changed

- Moved first-party hosted deployment tooling, hosted-provider specs, operator
  runbooks, and the working launch backlog out of this public repo and into
  a private operator workspace.
- standardized the public docs deployment path on Cloudflare Workers and
  removed the stale GitHub Pages workflow
- enabled GitHub Discussions and linked it from the public README

### Fixed

- release workflow `gh release view/create/download/upload` calls now pass
  `--repo "$GITHUB_REPOSITORY"` so GitHub Actions does not depend on local
  `.git` context when generating notes or publishing release assets

## [0.1.0-alpha.0] - 2026-04-19

### Added

- `scripts/release_gate.sh` — single release-candidate entrypoint combining
  strict checks, docs-site build, docs-site tests, and optional packaging /
  install-smoke / hosted cells, with per-step evidence logs and a
  `summary.tsv`
- `docs/PAYMENT_MATRIX.md` — supported payment rails × verification modes
  matrix with per-cell status and re-run commands
- `docs/IDENTITY_ATTESTATION.md` — normative spec for DNS + OAuth/OIDC
  identity bindings for Froglet keys
- `docs/ARBITER.md` — design stub for the marketplace-layer claims-court
  service
- `froglet-protocol::protocol::identity_attestation` — `IdentityAttestation`
  credential type, validator, and 8 roundtrip tests
- `scripts/cloudflare_dns.sh` — Cloudflare v4 DNS helper (verify / zone /
  list / create / delete / upsert); reads token from macOS Keychain, never
  echoed
- `FROGLET_EGRESS_MODE=strict` — opt-in propagation of the
  same DNS-pinning + SSRF validator used for LLM-controlled URLs to the
  operator-configured `FROGLET_PROVIDER_URL` / `FROGLET_RUNTIME_URL`
  surfaces in the Node MCP and OpenClaw integrations. Off by default;
  local/dev URLs stay on stock `fetch`
- Order-28-style content-shape assertions in `scripts/hosted_smoke.sh`:
  `/health` JSON envelope, `/v1/node/capabilities` key presence,
  `/v1/node/identity` minimum shape, `/v1/openapi.yaml` prefix, docs
  `text/html` + body marker
- `CODE_OF_CONDUCT.md` (Contributor Covenant 2.1), `.github/ISSUE_TEMPLATE/`
  (bug + feature YAML forms + Discussions/security contact links),
  `.github/pull_request_template.md` mirroring the release gate (TODO
  scaffolding; Discussions toggle still pending)
- OCI Wasm workload kind (`compute.wasm.oci.v1`) allowing Wasm modules to be
  referenced by OCI image (`oci_reference` + `oci_digest`) instead of inline hex
  bytes; supports `ghcr.io` and Docker Hub registries with anonymous pulls
- `OciWasmSubmission` and `OciWasmWorkload` structs in `src/wasm.rs`
- `oci-registry-client` dependency for OCI manifest and blob fetching
- OCI Wasm deal execution path with digest verification and sandbox execution
- official Docker assets for the Froglet node, including a starter
  `compose.yaml`
- public OpenClaw plugin with Froglet discovery and provider-surface tools
- checked-in OpenClaw starter config example
- GitHub Actions CI for strict checks and Docker starter validation
- GitHub Actions release workflow for tagged GHCR image publication

### Fixed

- `rustls-webpki` bumped 0.103.10 → 0.103.12 (RUSTSEC-2026-0098 + -0099)
- `cryptography` (Python) bumped 45 → 46.0.7 (3 GHSAs)
- `npm audit fix` in `integrations/mcp/froglet` (hono, @hono/node-server,
  path-to-regexp) and in `docs-site` (vite)
- Added `postgres_mounts` field to four test NodeConfig literals to
  restore `cargo check --all-targets` on `main`
- replaced `todo!()` panic in free OCI Wasm job path with full implementation
- added 50 MB size cap on OCI module downloads to prevent memory exhaustion
- fixed OCI reference parsing to handle `@sha256:` digest syntax alongside `:tag`
- replaced hardcoded registry allowlist with generic `https://{host}` fallback
  for unknown OCI-compliant registries
- extracted shared `fetch_oci_wasm_module` helper to deduplicate OCI pull logic

### Changed

- added `FROGLET_PUBLIC_BASE_URL` so containerized nodes can advertise a
  host-reachable clearnet URL
- tightened OpenClaw output defaults so raw JSON is opt-in via `include_raw`
- expanded OpenClaw tests to cover missing config, 404, invalid JSON, and
  timeout failure paths

### Fixed

- cleaned the warnings-denied Rust build path by removing stale `NodeConfig`
  initializer gaps and the unused-events-query warning

## Alpha Cut Notes

When cutting the first alpha:

1. move the current `Unreleased` notes into `0.1.0-alpha.1`
2. set `Cargo.toml` package version to `0.1.0-alpha.1`
3. push tag `v0.1.0-alpha.1`
4. let `.github/workflows/release.yml` publish the matching GHCR images
