# Froglet TODO / Roadmap Notes

This file preserves roadmap order references used by code comments, tests, and
design stubs. It is not the kernel specification. `docs/KERNEL.md` remains the
authoritative source for canonical artifact payloads, hashing, signing bytes,
state transitions, and settlement bindings.

## Current checkpoint — 4 October 2026

The immutable [v0.4.7-beta.1 prerelease](https://github.com/AlitheaBio/froglet/releases/tag/v0.4.7-beta.1) is published from
`d70cb50017c7c526d1f7ad4faf278da07e8248a4`. Its asset digests, Release Bundle,
attestations and OCI distribution are verified. The downloaded Mac binary
`c1ad573f68e8a84d241f8739c3844c52770f7474affc7bbab1cb3cbf8d50ec30`
passed a separate deterministic localhost replay: seven verified chains and
six saved offline terminal task reads. See [the exact release scope](docs/RELEASE.md#versioning).
A separate native-MCP Mac-to-Fly proof passed six bounded free cases, provider
restart persistence and offline task recovery on one isolated host, now removed.
Available hosted/production compute, load, fresh agent/OS qualification,
npm/MCP registry publication, paid operation and independent human trials remain open. The old agent/replay
failures and original demo pins below are historical evidence, not promoted
by this pass.

<a id="current-checkpoint--1-october-2026"></a>

## Historical checkpoint — 1 October 2026

Launch preparation on 2 October keeps this free scope and its qualification
limits. The [production handoff](docs/RELEASE.md#production-launch-preparation--2-october-2026)
separates the software gate, immutable distribution, advertised agent/OS cells,
public execution, operations and first-user evidence. The companion source pin
must advance only after the matching public immutable release exists. Payments
and compute-environment expansion remain deferred.

The user has resumed work on a free native MCP workflow for selected-data
sharing and bounded Wasm computation, with optional A2A between explicitly
configured counterparties. The scope is one useful exchange between a data
owner and an independent recipient. More features are not a substitute for
qualification or evidence that this exchange solves a real problem.

| Area | Current state | Next evidence |
|---|---|---|
| Native MCP and optional A2A | Final local candidate is pinned at binary SHA-256 `584e3887e9b12d9f433bef2a70487a6ebee65976cd2c146b603bd2b16d76b166`. | [Exact local qualification and remaining boundaries](docs/RELEASE.md#october-candidate-local-qualification). The immutable public release remains unpublished; an older release does not inherit the new actions. |
| Installed agent host | Actual Codex CLI `0.154.0` on the current Mac completed four native tool calls; the extracted Mac demo bundle also passed outside the checkout. | Claude Code is logged out; its actual MCP session remains unrun. Clean OS/agent cells and first-time-human installation remain unrun. The bundle's Python setup helper and ad-hoc macOS signing do not qualify no-runtime setup, Gatekeeper downloads or every supported target. |
| Captured generated program | Deterministic replay of Alice's retained new Wasm passed four frozen answers and independent verification of all seven admitted chains: six successes and one separate looping-program failure. Exact completed data/compute retries recovered cached results after Bob stopped and Alice restarted; `get_task` returned HTTP 502 while Bob was stopped. | [Local finalization evidence](docs/RELEASE.md#local-finalization--captured-program-replay). The earlier actual-agent qualification remains failed and preserved. A fresh Codex repeat still requires explicit approval for possible project-trust registration; no replay substitutes for that gate. |
| Reproducible demo handoff | Deterministic allowlisted packaging with exact binary/module pins; extracted Mac/Python3.13 and Linux arm64/Python3.11 demos and optional TLS checks passed without build tools. | Local candidates only; public assets, Linux x86_64, downloaded Gatekeeper, fresh machines and first-time-human installation remain open. |
| Linux arm64 runtime | Read-only production snapshot passed 625 library and nine MCP tests, plus the two-node terminology scenario; six chains were independently verified. | Docker development build only. Thirteen ignored tests, Python isolation, fresh Linux installation, release assets and Linux agent-host qualification remain outside this evidence. |
| Selected-data research profile | Final pinned terminology flow produced six Deals and four matching oracle cases at the same 2,000 ms budget. A pinned public GO catalog was locally published and retrieved in 100- and 40-row pages with two verified receipts. | [Profile and exact metadata checks](docs/RESEARCH_PROFILE.md). Scientific interpretation, independent data-owner usefulness and demand remain unmeasured. |
| Local capacity | Independent requesters completed tiny workflows without errors through concurrency eight in this short sweep. At 16/32, six selected-data tasks failed at the four-process limit with verified failure chains; 248 Bob Deals match 248 Alice intents, zero pending. | Shared-identity collisions fail before Deal admission but limit identical tuple Quotes to about one per second; 83 Bob Deals match 83 Alice intents, zero pending. [Recorded limits and counts](docs/RELEASE.md#october-candidate-local-qualification). Historical baseline timings remain invalid for performance comparison. |
| Public free exchange | New MCP/A2A profile's public compute HTTPS and capacity remain unqualified. GCP authentication blocks public compute deployment. | Two independent machines, trusted HTTPS, exact requester/Offer authorization, bounded resource/concurrency limits, overload refusal, and lost-response/restart recovery. Local capacity does not establish public production capacity. |
| First-user usefulness | The synthetic terminology checker proves a bounded consistency audit; it does not prove semantic truth, demand, or lower operating cost. | [Five-participant task and real catalog comparison kit](docs/AGENT_PUBLISH_ACCEPTANCE.md#first-user-trial-kit--1-october-2026). Real catalog, participants and independent recipients still need to be supplied; do not invent observations. |
| Public website | Restored and updated on 1 October, current Worker `857f8336-ed78-48cb-8f08-959f0ff8bff6`. Homepage and both affected routes returned HTTP 200 and matched the final build; new scenario rows were rendered live. The hosted compute trial remains unavailable. | Local response/screenshot evidence: `_tmp/improvement-2026-10-01/continuation/site/`. Website publication does not qualify public compute or publish the Node candidate. |
| File downloads, general CPU jobs, GPU and payments | Existing work is preserved. No new release claim for these paths. | Keep F1 separate. C1/G1 expansion, new rails, purchases, paid flows, and automatic allowance refill remain deferred. |

Qualification commands and historical checkpoints below remain useful. The
selected profile's acceptance record must identify the candidate binary and
source, distinguish local from public execution, and report every skipped or
unavailable gate. Preparing a trial kit does not establish participant results
or authorize invitations.

The local finalization checks passed the default matrix: 1,008 Rust tests,
199 Node tests, 177 core Python tests, 228 verifier tests and 90 conformance
checks. Eight Rust tests were ignored in this Mac run; OS-specific and live
external gates remain separate. The final replay recorder corrections also
passed eleven focused tests and actual two-node execution. The replay and
reviewable source checkpoint remain unpublished local evidence, not a stable
release or a newly qualified hosted service.

<a id="current-checkpoint--28-september-2026"></a>

## Historical paused checkpoint — 28 September 2026

Product development was paused while the next direction was decided. The work
below was preserved; the October checkpoint above defines the resumed scope.

| Area | State | Where to resume |
|---|---|---|
| Public website | Temporary “Coming back soon” page is live. The redesigned full site is preserved. | [Deployment and exact rollback](docs-site/README.md#temporary-pause-2026-09-28). `npm run deploy` keeps maintenance; `npm run deploy:site` explicitly ends it. |
| File downloads (F1) | Implemented and exercised locally, capped at 8 MiB; not released or deployed. | [Implementation checkpoint and evidence](docs/FILE_AND_COMPUTE_SCOPE.md#10-implementation-checkpoint--2026-09-28). Node, share UI and companion relay must be qualified together. |
| CPU containers (C1) | Recovery/quarantine foundations exist. General jobs are incomplete. | Controlled pulls with hard disk limits, batch adapter, status/cancel/results and Linux isolation/deadline qualification. |
| GPU (G1) | Execution disabled in the current worker; not a rental capability. | Finish C1 first, then qualify actual NVIDIA hardware and exclusive-device cleanup. |
| Payments | Deferred by the operator. | No new rails, purchases, paid flow, or automatic allowance refill. |

The restart notes recorded at that checkpoint were:

1. Choose the next user-facing outcome. The smallest prepared candidate is F1
   download-only sharing; do not bundle CPU/GPU or payment work into its release.
2. Use matching `froglet` and `froglet-services` checkpoints. The companion
   relay carries the download headers/8 MiB transfer coverage; its OCI worker
   contains the persisted quarantine work. Those changes are not deployed.
3. Rerun the relevant checks, then qualify the selected outcome end to end in an
   isolated environment. Keep the website pause until a relaunch is requested.

Validation entrypoints: `./scripts/strict_checks.sh`, `npm test --prefix docs-site`,
`npm run build --prefix docs-site`, and the companion relay/OCI-worker tests.
Historical test evidence is recorded in the scope document; hardware-gated tests
must not be counted as passing when skipped. Generated builds, test ledgers,
screenshots and raw logs remain ignored; do not discard them as part of Git cleanup.

### Saved checkpoints

Both repositories used the local branch `arma/paused-development-checkpoint`
during cleanup. These commits were not pushed or deployed during that pass.
They are historical recovery points, not a requirement to reset the resumed work.

| Repository | Commit | Preserved work |
|---|---|---|
| `froglet` | `3d1012e` | Website redesign, illustrations and reviewed documentation |
| `froglet` | `b44a0ec` | Live maintenance source, rollback instructions and deployment defaults |
| `froglet` | `c1f057c` | Unreleased F1 file sharing and C1 safety foundations |
| `froglet-services` | `ca1c4d1` | Bounded relay file transport |
| `froglet-services` | `62d2345` | Worker recovery/quarantine, including the relay commit |

Cleanup verification, 28 September 2026:

- Main strict matrix passed: 936 Rust tests (8 ignored), Clippy, native/wasm
  verifier builds, frozen Kernel fixture check, secret scan, all 185 Node
  integration tests, 145 Python runtime/setup tests, 221 Python verifier tests,
  and 90 conformance checks. Log: `_tmp/repo-cleanup/strict-checks.log`.
- Site: 199 tests and the 37-page build passed. Default deployment dry run
  bundled only the maintenance worker. Logs: `_tmp/repo-cleanup/site-*.log`.
- Companion relay/OCI worker: 34 tests passed, actual-container test ignored;
  formatting, Clippy and scans of the changed service directories passed.
  Logs: `/private/tmp/froglet-cleanup-{companion,worker,relay}-*.log`.

This is source/test checkpointing, not new hardware or production qualification.
The existing rendered-site and local transfer evidence remains in the dated
review and scope documents. No source, test ledger, runtime data or evidence
was deleted; generated files and local caches stay outside Git.

## Order 44 - Batch Fan-Out

Status: not implemented; boundary is enforced in agent-facing truth strings
(`integrations/shared/froglet-lib/tool-dispatch.js` `plan_use_case` batch
profile, `docs/HOSTED_TRIAL.md`, `docs/llms/try.froglet.dev.txt`).

Multi-item batch fan-out, retries, and paid async settlement across the
runtime deal flow. Until this lands, batch workloads use the existing async
task-status primitives one task at a time, and agent surfaces must not claim
batch support.

## Order 45 - GPU Scheduling and Routing

Status: disabled in the current isolated-worker path; not release-qualified.

Earlier single-node Docker/T4 smoke evidence belongs to the previous execution
path. It does not qualify the current worker for GPU rental or isolation.
The current node refuses GPU execution and the worker reports it unsupported. Follow the staged C1/G1 gates
in [the file and compute scope](docs/FILE_AND_COMPUTE_SCOPE.md), including actual
device allocation, finite occupancy, cancellation, cleanup and safe reuse.

## Order 70 - Strict Egress Pin Propagation

Status: implemented with regression coverage in
`integrations/shared/froglet-lib/test/egress-mode.test.mjs`.

Track strict egress behavior through the shared Froglet JavaScript client:
`frogletRequest`, `frogletRequestWithStatus`, and `frogletPublicRequest` must
resolve operator pins when `FROGLET_EGRESS_MODE=strict`, cache the resolved pin,
and honor caller-supplied pins over cached lookups.

Remaining work: keep this coverage aligned as additional agent integrations
delegate requests through `integrations/shared/froglet-lib/`.

## Order 81 - Identity Attestation Issuing Service

Status: protocol payload and validator exist in
`froglet-protocol/src/protocol/identity_attestation.rs`; service issuance
flows remain to be built.

Implement the marketplace attestation service that issues DNS and OAuth/OIDC
identity attestations:

- DNS flow: create a challenge, verify the DNS evidence, issue the signed
  `identity_attestation/v1` artifact, and schedule re-verification.
- OAuth/OIDC flow: verify provider identity evidence, bind it to the Froglet
  subject key, issue the signed artifact, and schedule re-verification.
- Revocation and expiry handling: consumers must reject expired attestations;
  failed re-verification should invalidate the attestation before expiry.

Out of scope for this order: W3C Verifiable Credentials and proof-of-personhood.

## Arbiter Mechanism Design

Status: stubbed in `docs/ARBITER.md`.

The arbiter design still needs data-backed economic parameters before it should
be promoted from stub to implementation spec. Open parameters include deposit
tiers, stake floors, fee split, appeal multiplier, adjudicator throughput, and
grief-filing cost assumptions.

Implementation should update `docs/ARBITER.md` with concrete values and tests
once those parameters are chosen.
