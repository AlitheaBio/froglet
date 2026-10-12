# Release

This repo now has a tagged release path for the public Froglet node, tagged
Docker images in GHCR, the MCP image, and the checked-in docs deployment
configuration.

New tags cut by the current workflow also publish the non-Kernel
[`froglet.release-bundle.v1`](PUBLICATION_CONTRACT.md#release-bundle-v1)
manifest that binds source, binary checksums, and immutable role-image digests.

Maintained by [Armanas Povilionis-Muradian](https://armanas.dev).

## Versioning

The current immutable public prerelease is [v0.4.7-beta.1](https://github.com/AlitheaBio/froglet/releases/tag/v0.4.7-beta.1),
built from source revision `d70cb50017c7c526d1f7ad4faf278da07e8248a4`.
On 4 October, all eight official assets, the complete Release Bundle, both
workflow attestations and all four anonymous OCI image references were verified.
The exact-source main CI passed all six required jobs; optional coverage was
skipped. The release workflow also passed.

The downloaded Apple Silicon binary has SHA-256
`c1ad573f68e8a84d241f8739c3844c52770f7474affc7bbab1cb3cbf8d50ec30`.
Its deterministic localhost replay passed independent verification of seven
signed chains, four frozen arithmetic answers, a signed execution-budget failure
and all six saved terminal task reads after Bob stopped and Alice restarted.
It starts no fresh agent session or compiler. The original failed agent attempt
and earlier failed replays remain failed and preserved.

A separate 4 October cross-host qualification used that actual released Mac
binary through native MCP and the released provider image
`ghcr.io/armanas/froglet-provider@sha256:f6ce86bad406de2faae0175ea8ae50d76a364cd489f9f18d3f07a3a6bcb64163`
on one isolated Fly machine with a finite free invitation. Six signed chains
covered four frozen arithmetic cases, retrieval of nine synthetic rows/four
fields and an expected fuel failure. All six direct provider records survived
an actual restart, and all six saved tasks remained readable after the provider
stopped. Independent review found no findings in this bounded scenario. The
temporary host and private test state were removed. Remote A2A was not tested
by this run; its configured-counterparty evidence remains local.

The remote record has SHA-256
`a19e67a833bbd94949459c25709385d7cb6a6234926a30627298cd8c9d6d07b3`.
Its custody interruption remains recorded. Four original response contents
were recovered under their complete pre-deletion hashes; three are explicitly
derived recoveries from later responses and recorded listener prefixes. Later
local controls are separately labelled and do not establish remote
pending-admission behavior.

These checks qualify distribution, the published Mac localhost replay and only
that finite free cross-host scenario. They do not qualify an available hosted
compute service, production operation or load, a fresh Claude/Codex journey,
a clean operating system or Gatekeeper download, npm/MCP registry publication,
payments or independent human usefulness. The last observed hosted deployment
was `v0.1.0-beta.23` with public `v0.4.6-beta.4`; this pass has not upgraded it.
Its upgrade remains a separate gate.
Future releases need a new tag and verified immutable bundle, followed by an
exact services pin, locked dependencies and their own deployment qualification.

Dated preparation and qualification records below retain their original scope;
they are historical evidence, not the current release-availability summary.

## Production launch preparation — 2 October 2026

The current launch scope is free selected-data sharing and bounded Wasm
computation through native MCP, with optional A2A between configured
counterparties. This is an unpublished source candidate. Preparing a clean
checkpoint and passing the local software gate does not promote it to a
production deployment or a qualified stable release.

The current binary and local workflow evidence are recorded below. Keep that
evidence attached to its exact source and executable: changing runtime inputs
requires rebuilding and requalifying the candidate. Do not relabel the failed
fresh Bob/Alice agent attempt as passing because the deterministic replay passed.

| Launch gate | Evidence required | Current boundary |
|---|---|---|
| Public source checkpoint | Clean reviewed checkout, locked dependencies, frozen Kernel fixtures and a complete public-source secret scan | Local preparation; no public tag or immutable Release Bundle is implied |
| Software and website | Required toolchain present, successful release-gate logs and targeted checks for later edits | Required skipped steps are incomplete, not a PASS; optional/live checks remain separate |
| Immutable distribution | Exact tag/source, all target archives and checksums, bootstrap and manifest trust, workflow attestations and immutable published assets | Local packaging does not satisfy this gate |
| Advertised OS and agent hosts | Actual installation and tool execution for every claimed host, including signed-in Claude and clean-machine cells below | Current Mac Codex and Linux arm64 development evidence do not cover all cells |
| Public free execution | Independent requester/provider hosts, trusted HTTPS, exact Offer/requester authorization, independently verified receipts and finite allowances | New compute profile remains unqualified; GCP is the chosen host, not a protocol dependency |
| Operational safeguards | Effective target configuration, overload/refusal tests, durable accounting, owner-control authentication and ingress boundary, backup restore and guarded rollback | Local fixtures and historical hosted evidence do not establish current production behavior |
| First-user usefulness | Independent catalog owners and recipients complete the declared task with recorded observations | Trial kit is prepared; participant results remain absent |
| Website and release copy | Deployed routes inspected against the exact released candidate and its qualified scope | Current support matrix and Alithea Bio attribution published on 3 October; six pages and the logo match the prepared build. The compute candidate remains unpublished |

### Local verification completed on 2 October

The candidate-bound software gate passed all six selected steps: publication
secret scan, native package build/verification, strict checks, website build,
website tests and local installation smoke. It passed 1,008 Rust tests (eight
ignored), 199 Node tests, 228 core Python tests, 228 offline-verifier tests,
44 example tests, 90 conformance checks and 666 website tests, including all
14 opted-in real-node cases. The companion's 47 operator/provenance/local
HTTP fixture tests passed separately; its Rust runtime was not rebuilt.

The freshly built Apple Silicon executable has SHA-256
`64cbbeb3925c2a329146957968446277e02cb8e835ed7ad8638722477ba83d3c`.
Its archive contains exactly the executable and LICENSE without macOS metadata,
and the archived executable matches that digest. Installation smoke used the
current Mac and a local file/manifest fixture; it does not qualify public
download attestations, notarization, Gatekeeper or another OS.

The retained requester-generated program passed a separate deterministic replay
against that exact executable: seven independently verified admitted chains
(six successes and one execution-budget failure), four frozen-answer cases,
selected-data privacy, retry/restart recovery and tampering controls. The 136
recorded production build inputs remained unchanged. `get_task` still returns
HTTP 502 while its provider is offline; exact completed data/compute retries
recover cached results. The original failed fresh-agent attempt stays failed.
No new Codex or Claude session was used; the earlier actual Codex result remains
attached to executable `584e3887…` below.

Full local logs, source hashes, retained failed gate runs and checkpoint records
are under ignored `_tmp/launch-prep-2026-10-02/`. These are local qualification
records, not published release assets or production-host evidence.

### Website publication and cloud selection — 3 October

The prepared documentation site and restored Alithea Bio footer were published
to the existing `froglet-docs` Worker at version
`1a07cc84-eeb2-4c1f-8951-471ac6676401`. Six public pages and the original logo
returned HTTP 200 and matched the prepared build bytes. The prior full-site
version `857f8336-ed78-48cb-8f08-959f0ff8bff6` is the rollback target. Build,
secret-scan, deployment and HTTP evidence are retained under ignored
`_tmp/launch-prep-2026-10-02/site-publication-2026-10-03-*`. Website publication
does not publish the Node candidate or qualify public computation.

At the user's request, the active GCP CLI project was changed to
`froglet-prod-eu`; a read-only metadata check returned `ACTIVE`. Fly
authentication was also confirmed. GCP reported a separate Application Default
Credentials quota-project mismatch, which was not changed. These checks do not
establish deployment permissions, provider availability, operational safeguards
or current hosted compute qualification. The earlier authentication blocker is
historical; the public execution and operational gates above remain open.

### Launch continuation — 3 October

The next source candidate is `0.4.7-beta.1`; its tag was checked unused. Do not
reuse or replace the immutable beta.4 tag. Cargo, npm distribution, nested MCP
and registry metadata now use the candidate version consistently. Wasmtime is
updated to the patched `36.0.17` line. Fresh Cargo, root npm and nested MCP npm
audits report no vulnerabilities. The website's remaining cache-library advisory
is **unresolved**, with a narrowly reviewed, expiring applicability policy:
[`docs-site/DEPENDENCY_SECURITY.md`](../docs-site/DEPENDENCY_SECURITY.md).
The policy verifies the actual custom Worker bundle and fails on other high or
critical findings, reviewed-source/configuration drift, a newer cache-library
release, or expiry on 17 October. It does not label the affected package fixed.

On 4 October, the guard blocked newly published `http-cache-semantics` 4.3.0
until review. Its tarball preserves the reported max-stale/cookie behavior;
upstream closed the proposed fixes unmerged and disputes the advisory. The
selected 4.3.0 update includes separate Vary matching fixes. npm audit reports
zero findings for that version, which is not evidence that the reproduced
behavior was repaired. The deployment review retains its finite expiry and
source/configuration/bundle boundaries, including when the package audit is
clean. Fresh exact-source CI and publication remain required. See the updated
[dependency review](../docs-site/DEPENDENCY_SECURITY.md) for the current proof.

Authenticated read-only inspection of `froglet-production` in
`froglet-prod-eu/europe-west4-b` found all five containers running. The four
Froglet service containers reference beta.23 configuration and immutable image
digests; the provider advertises public beta.4. CPU, memory, process and log
limits were inspected, and application ports are bound to host loopback. The
public provider proxy exposes bearer-protected owner-control routes; loopback
container ports do not make those proxied routes network-private.
Effective provider usage reports private admission and zero quote/deal/runtime
allowance. Public website, relay and marketplace health routes, marketplace
stats and provider capabilities returned HTTP 200 with the normal HTTP client.
This audits the existing deployment; it does not deploy the new candidate.
Evidence is retained in ignored `_tmp/launch-continuation-2026-10-03/`.

An isolated Fly smoke used the verified immutable beta.4 provider digest, after
checking Release Bundle provenance and Linux amd64 image metadata. It passed
HTTPS health, version/identity/origin and anonymous owner-control/quote/deal
refusals. The created app was destroyed and its absence checked independently.
This is reachability/refusal evidence for the existing release, not new compute,
effective authenticated allowances, marketplace admission or paid execution.

The new terminal-cache read path passed 43 targeted Rust tests: nine invocation,
twelve native computation and twenty-two runtime-route cases. Successful and
failed tasks survive requester restart and provider shutdown; signed rejected
and canceled outcomes, historical identity rotation, authentication, unknown
IDs, selected data, tampering and unsigned/pending refusal are covered. Only
read-only task retrieval changes; payment mutation paths keep provider refresh.
The optimized Apple Silicon build has SHA-256
`1e6621c461d7f368328467ad15f7e1cc6280997bf438238ed62974c0a96a8c8f`.
A fresh deterministic replay against that exact binary exported seven complete
signed chains (six successes and one execution-limit failure), matched four
previously frozen arithmetic answers, and recovered all six Alice terminal tasks
with Bob stopped and Alice restarted. It includes the selected-data task and
the signed failure, with no new admitted work during reads or exact retries.
No fresh LLM session or compiler was used. The new-candidate software gate then
passed all six selected steps: public-source/history secret scanning, locked
native packaging, strict checks, website build/tests and packaged installation.
The packaged executable retains the exact SHA-256 above. The strict run passed
1,004 workspace Rust tests plus seven ontology tests, 199 JavaScript integration
tests, 250 core Python tests, 46 example checks and 228 independent-verifier tests.
Eight environment-specific Rust cases remained ignored. All 732 website tests
passed, including fourteen opted-in real-node cases and sixty-six audit-guard
cases. A fresh applicability check accepted only the documented unresolved
website advisory and verified the actual Worker bundle. The updated recovery
guide and expanded developer matrix were inspected in the rendered local build.
Evidence is in `_tmp/launch-continuation-2026-10-03/software-gate-02/`, with the
first aborted packaging attempt retained separately. This is local software
qualification; exact-source public CI, immutable distribution and new cloud
compute qualification remain open. The earlier beta.4-source candidate's
results do not qualify this binary.

Run the software gate from the candidate checkout with installed Rust 1.91,
Clippy, `wasm32-unknown-unknown`, Python, OpenSSL and Node/npm prerequisites. Website
builds also require the versions accepted by the locked Astro dependencies.
The gate sets `FROGLET_REQUIRE_SOFTWARE_CHECKS=1`, so the strict matrix requires
its core tools rather than silently removing required test cells:

```sh
./scripts/release_gate.sh --evidence-dir _tmp/launch-prep/software-gate
```

For a host-compatible package/install check, add `--install-smoke --version`
with the candidate's chosen tag and the current `--platform`/`--arch`. This
requires the tag to match `Cargo.toml` and creates local artifacts only. The
gate builds and verifies the native optimized executable before passing that
exact binary to the local computation examples and opted-in browser/node
checks. Required standalone CI builds its own optimized node; developer-only
checks retain their existing default. The signed execution budgets are
unchanged. Confirm the package version and changelog before
cutting a real release; keep new changes under `Unreleased` until that decision.

The companion repository has its own
[`LAUNCH/README.md`](https://github.com/armanas/froglet-services/blob/main/LAUNCH/README.md) and exact source
pin. Publish and verify the public immutable release first, then update the
companion pin and regenerate its lockfile against that exact source before
building the matching hosted release. Do not point it at `main`, copy a local
binary into an old hosted release, or invent a source revision to unblock CI.
The companion's current pin remains independent of this launch candidate.

Payments, general CPU containers, GPU rental, confidential execution, managed
hosting and automatic batch scheduling remain outside this launch scope. Their
historical or experimental paths do not become production-qualified by passing
the free workflow gate. The historical v0.1.0 campaign/paid-launch checklist
below is retained for reference and is not the current free-profile cut plan.

## Effortless publishing qualification

**Current checkpoint, 1 October 2026:** source work has resumed on free native
MCP selected-data exchange and bounded Wasm computation, with optional configured
A2A transport. Its [local qualification](AGENT_INTEROPERABILITY_PLAN.md#qualification-results--1-october-2026)
does not establish a published release, clean-machine agent qualification,
public compute HTTPS/capacity, real payments, or first-user usefulness. Those results must name
the exact candidate and retain their own evidence.

The public documentation website was restored on 1 October after the September
pause, at Worker version `7dd4a560-b18d-478f-a7a1-bf1b6cd73eb2`. Seven checked
routes returned HTTP 200 and matched the local build bytes. Local evidence is
`_tmp/improvement-2026-10-01/site-http-qualification.json` (ignored, not a release
artifact); it explicitly records `remote_compute_verified: false`.

The continued qualification descriptions were deployed at Worker version
`857f8336-ed78-48cb-8f08-959f0ff8bff6`. The homepage and both affected guide/open-source
routes returned HTTP 200 and matched the final build bytes. The new scenario
rows were inspected in the live browser; the screenshot and response evidence
are in `_tmp/improvement-2026-10-01/continuation/site/`. The previous restored
version remains the rollback target. This is a documentation update only.

The hosted compute trial remains unavailable, and GCP authentication blocks new
public compute deployment. The source candidate remains unpublished. The final
actual Codex host and extracted bundle passed on the current Mac; corrected
local capacity sweeps have recorded outcomes below. Historical HTTP-baseline
timings must not be used for performance comparison. The earlier
relay/marketplace beta deployment in `froglet-prod-eu` and its September evidence
below are historical; they do not establish current availability or qualification
of the new MCP/A2A profile. The [September pause and rollback
record](../docs-site/README.md#temporary-pause-2026-09-28) is preserved.
Froglet is **not a qualified stable release**.
The command `scripts/release_gate.sh` covers software checks; a PASS
with skipped platform or external work does not satisfy the stable product gate
below.

Platform and participant qualification remain open. On 1 October the user
authorized preparation of the next free qualification work; access to a real
catalog, first-time participants and independent recipient hosts remains an
external dependency. The [trial kit](AGENT_PUBLISH_ACCEPTANCE.md#first-user-trial-kit--1-october-2026)
contains a repeatable task and empty capture record. No participants, invitations,
or trial results are implied by its existence. Payments and GPU expansion remain
deferred.

The supported first-use journey is free, native publishing and consumption with
Codex and Claude Code. JSON, explicitly typed CSV, SQLite selections, and small
Wasm functions are in scope. Python remains advanced Linux functionality.
Windows, managed always-on hosting, arbitrary applications, broader agent
qualification, and advanced payments are outside this release.

### October candidate local qualification

The final local candidate binary has SHA-256
`584e3887e9b12d9f433bef2a70487a6ebee65976cd2c146b603bd2b16d76b166`.
Its 136 recorded production build inputs remained unchanged. This identifies a
locally packaged, unpublished candidate; it is not an immutable public release
or an external build attestation.

- Actual authenticated Codex CLI `0.154.0` on the current Apple Silicon Mac
  completed `status`, selected-table `invoke_service`, `run_compute` and
  read-only `get_task` through native MCP. Independent evidence checks matched
  the returned five rows, oracle result, result hashes and recovery references.
  The native invocation PATH excluded Cargo, Rustup, Node and npm. This is the
  existing Mac and agent account, not a fresh OS or a first-time-human session.
- The extracted Mac demo bundle ran outside the checkout with checked member
  digests and matched scenario results. Its setup helper requires Python 3.10+;
  the native binary/MCP server needs no language runtime. The binary is ad-hoc
  signed, without Developer ID/notarization or downloaded Gatekeeper
  qualification. It does not qualify Linux bundles or the public installer.
- The pinned research-profile terminology flow produced six signed execution
  Deals and four independently matching oracle cases, preserving its 2,000 ms
  runtime budget and retry/restart behavior. Namespace, version, unit, provenance
  and mapping-policy mismatches were declined as exact declaration differences.
- The pinned public GO catalog adaptation was prepared, locally published and
  retrieved through native MCP/configured A2A in 100- and 40-row pages. All 140
  selected `id`, `name`, `term_uri` rows matched the pinned adaptation, with two
  verified receipts and distinct requester/provider identities. This is a local
  reuse of public data; it is not public service publication, scientific mapping
  evaluation or independent research-demand evidence.

The continued Linux arm64 check uses a read-only snapshot of the same 136
production inputs in a pinned Rust 1.91.0 Debian Bookworm Docker image. Its
separate development binary has SHA-256
`99fd4a7ddb229064bac322b87307888e5f9233cd497f4ee88cfea86250542570`.
The run passed 625 library tests and nine native MCP integration tests; 13
library tests were ignored. Twelve ignored tests concern Python OS sandbox or
service execution, so this check does not qualify Python isolation. The full
two-node terminology scenario then passed. Independent offline review verified
all six signed chains, exact selected rows, program/input/result hashes, profile,
four oracle cases and the execution-limit failure. This is a development build
inside the current machine's Docker VM, not a clean-host install, a Linux agent
session or an immutable release. Local evidence is
`_tmp/improvement-2026-10-01/continuation/linux-arm64/independent-review.json`.
The initial demo invocation used an unsupported output flag and was refused
before execution; that log is retained beside the corrected successful run.

The local demo handoff is now reproducible with
`scripts/package_demo_bundle.py`. Caller-supplied binary and Wasm digests,
an explicit local candidate label and an allowlisted payload produce deterministic
archives; each launcher verifies member bytes and permissions before starting
its own disposable Nodes. This integrity manifest is not an attestation or an
immutable Release Bundle. The ontology and optional counterparty checks passed
from extracted bundles outside the checkout on the current Mac (Python 3.13,
system LibreSSL) and in a minimal Linux arm64 runtime (Python 3.11.16), without
Cargo, Rustup, Rust, Node or npm. The setup helpers still require Python;
OpenSSL is an additional prerequisite for the separate TLS check.

Standalone testing exposed missing subject/authority key identifiers in the
local TLS certificate fixture under Python 3.13 strict verification. The fixture
in `examples/a2a_counterparty_demo.py` now generates the identifiers for the CA
and server. TLS verification remains enabled; the failing archive and log are
retained. Twenty packaging tests and 52 setup tests passed. This fixture and
packaging change does not alter the native binary or Kernel artifacts.

The installed Claude Code CLI was also checked through its documented read-only
authentication command. It is logged out, so no Claude native MCP run occurred.
The actual Codex qualification above does not substitute for that host. Sign-in
and a new actual tool trace are prerequisites for closing the Claude gate.

The final capacity sweeps use one shared M4 machine, one provider, finite free
jobs and a private pinned copy of this binary. Report source and executable
fingerprints stayed unchanged during each run. Latencies include every attempted
workflow, including failures; concurrency counts client workflows, not TCP
connections or simultaneous executing programs.

| Requester setup | Concurrent tiny workflows | Complete / attempted | Recorded boundary |
|---|---|---|---|
| Independent requester identities | 1 / 2 / 4 / 8 | 2/2 · 4/4 · 8/8 · 16/16 | No workflow errors in this two-iteration sweep. |
| Independent requester identities | 16 / 32 | 29/32 · 61/64 | Six admitted selected-data tasks failed at four process slots. All six terminal failure chains were recovered and independently verified as `capacity_exhausted`. |
| One shared requester identity | 1 / 2 / 4 / 8 / 16 / 32 | 2/2 · 4/4 · 7/8 · 7/16 · 7/32 · 13/64 | 86 workflow errors retained. Identical requester/workload/terms Quote collisions were refused with a clear pre-Deal 503 after bounded attempts. |

Independent mode retained 248 provider Deals and 248 requester intents: 242
succeeded and six failed, with zero pending submissions or separate jobs.
Shared mode retained 83 provider Deals and 83 succeeded requester intents, with
zero pending submissions or separate jobs. The single-use Quote contract limits
one identical requester/workload/terms tuple to about one Quote per Unix second;
safe refusal does not remove that throughput constraint. Both runs preserved
usage across restart, recovered the same warmed result and refused new work at
the configured cumulative Deal limit.

The corrected conventional HTTP/Python baseline performs the same selected-table
policy audit and uses plain HTTP connections. It omits requester-supplied code,
signed authorization/receipts, sandbox resource contracts, durable tasks and
recovery. These timings do not prove performance superiority, developer-time or
maintenance-cost savings, useful scientific output, demand, or public capacity.
The historical unsuffixed reports retain diagnostics; their per-call unused TLS
context overhead invalidates their HTTP performance comparison.

Local evidence below is ignored and is not published release material:

- `_tmp/improvement-2026-10-01/agent-install/final-after-quote-fix/host-qualification.json`
- `_tmp/improvement-2026-10-01/agent-install/final-after-quote-fix/demo-bundle.json`
- `_tmp/research-profile-2026-10-01/final-profile-qualification.json`
- `_tmp/research-profile-2026-10-01/ontology-report-final.json`
- `_tmp/research-profile-2026-10-01/go-catalog-runtime-report.json`
- `_tmp/ontology-capacity-2026-10-01/final-shared-identity-report.json`
- `_tmp/ontology-capacity-2026-10-01/final-independent-identities-report.json`
- `_tmp/ontology-capacity-2026-10-01/independent-statistics-verification.json`
- `_tmp/ontology-capacity-2026-10-01/independent-failure-chain-verification.json`

The required matrix exited successfully in
`/private/tmp/froglet-improvement-strict-20261001.log`: 1,007 Rust tests passed
(including seven ontology checker tests), eight ignored; 199 Node, 29 example
Python, 155 core Python and 226 verifier tests passed, with 90 conformance checks
reproduced. Later Python setup-edge and capacity failure-reader changes received
their own targeted checks: 52 full setup tests, 12 focused tests, 31 independent
reviewer probes and nine capacity helper tests. The full matrix preceded those
last Python changes; it is not claimed to cover one unchanged source snapshot
across every stage. The final candidate's recorded production build inputs were
unaffected by those harness changes.

### Local finalization — captured-program replay

The deterministic replay uses the same pinned Mac binary above and Alice's
retained 81,222-byte Wasm, SHA-256
`73d2a6842b5c3372d0ff430432a6c781384a25ce7a5e2ef9686d10de2825a615`.
No Codex/Claude session, compiler or agent settings operation was started.
The [public runner and runbook](../examples/README.md#replay-an-already-generated-program-without-an-llm)
take explicit binary/program pins and original/held-out answers. They depend on
three public Python files and the standard library, with no ignored incubation
code dependency.

Independent offline verification covered every admitted Deal: six successes
and one failure, including an explicit facilitator owner canary, selected-data
retrieval, four captured-program executions and a separate 75-byte looping
program. All four arithmetic outputs match answers frozen before the actual
agent sessions. The signed publication binds the exact nine-row, four-field
snapshot; nineteen excluded private values are absent. This checks selected-data
exchange and arithmetic on those inputs, not scientific truth or universal
program correctness.

After Bob restarted, all six Alice tasks recovered their original results and
signed references. After Bob stopped and Alice restarted, exact completed
`run_compute` and `invoke_service` retries returned cached verified results,
with provider/requester counts unchanged. **`get_task` currently refreshes
provider state and returned HTTP 502 with Bob stopped.** It does not provide
offline status recovery by task ID. Unknown remote admission can still remain
unresolved; no universal exactly-once guarantee is implied.

Receipt, result and publication tamper controls were rejected. The independent
report is `_tmp/local-finalization-2026-10-01/replay/independent-verification.json`,
SHA-256 `5df50cd4cb6b7a21b9791cc19d7fbe7c0048b7595a2b36337e462b57fd072f33`.
It binds final `run-03/evidence.json`, SHA-256
`dc243699ff6b597f53ac7112cc003b59f2eab9fdc0b22f767ef00e6681b4672f`.
The complete chains were exported before private Node state was removed. An
independent root check also verified all seven chains and rejected a changed
Receipt in each.

The earlier actual Bob/Alice Codex attempt remains **failed**: its recorder lost
the complete execution-chain export, and persistent Codex configuration changed
for an unresolved reason. All seventy retained attempt files remain unchanged.
Two subsequent deterministic recorder failures remain preserved separately:
failed-task reads were incorrectly treated as failed tool calls, and provider
database IDs were incorrectly conflated with signed Deal hashes. The corrected
replay qualifies its new executions, not the missing earlier evidence. The
guarded fresh Codex repeat still awaits explicit project-trust approval.

The default validation matrix subsequently completed with 1,008 Rust tests
passing and eight ignored, 199 Node tests, 38 example Python tests, 177 core
Python tests, 228 verifier tests and 90 conformance checks. Final replay recorder
changes received eleven focused tests and the live qualification above. The
matrix includes formatting, warnings-denied tests and Clippy, protocol/browser
builds, frozen Kernel bytes, shell syntax and tracked-tree/history secret scans.
Its final log is `_tmp/local-finalization-2026-10-01/strict-checks-final.log`.
Negative fixture diagnostics inside verifier tests are expected; all selected
suites exited successfully. Earlier sandbox-permission and interrupted-script
logs are retained. Linux syscall isolation, live payments, public hosts, fresh
operating systems and actual Claude remain unqualified by this Mac run.

The local release checkpoint packages the reviewed source and exact candidate
pins with explicit gates. It does not create a Git commit/tag, publish release
assets, change the companion service's source pin, or claim an immutable Release
Bundle. Companion release pins must follow a separately published immutable
Froglet revision. Website source/preview and conference packages have their own
rendered checks; local edits do not replace the recorded public deployment above.

### Beta implementation and verification evidence

- Configuration changes merge only Froglet, preserve TOML comments, back up
  changed files, compare approved fingerprints, and compensate only matching
  writes. Reconnection reuses the installed immutable release. Unknown occupied
  installation paths fail closed and retain their contents.
- Preparation writes a selected snapshot and runs an example. Private source
  selections, fingerprints, and a recoverable preparation journal are separate
  from signed artifacts. Source edits never republish automatically.
- Native remote invocation uses the existing requester runtime and endpoint
  validation with a zero-price ceiling. Uncertain calls retain their retry key;
  subsequent calls reconcile the runtime's durable intent.
- Publication output separates local proof, public reachability, marketplace
  activation, and requester execution. The read-only local status interface has
  its own loopback listener and short-lived browser credentials, outside relay
  routes. An initialized MCP call is distinguished from a shell probe; an ended
  observed process reports disconnection without claiming current attachment.
- Sharing pages are for the first-party relay lane. Advanced hosting keeps its
  own endpoint and does not receive a first-party relay link.

Local qualification includes four native catalog/function executions and receipt
verification, deduplicated invocations, selected-data exclusion, source change
rejection, update, pause/resume, exact rollback, source deletion, and restart
identity/service continuity. Installer tests use isolated service-manager and
release fixtures; they are not real clean-machine or reboot evidence. Website
fixture states are not real public relay availability. No Kernel signing or
hashing change is part of this work.

On 2026-09-25, private digest-pinned binary installation, health, repeated
installation, and identity-retention smokes passed on fresh Linux x86_64 and
arm64 VMs without a checkout or toolchain. All 11 Linux Python isolation tests
passed on an arm64 VM with Landlock ABI v3 support. A Debian 12 arm64 VM exposed
its older ABI v2; Python fails closed there with an explicit requirement for v3.
The earlier uncommitted macOS arm64 candidate passed packaging and a pinned local
installer smoke. From this Mac, a separate requester runtime made a free call
through the candidate HTTPS relay and verified receipt
`ac7169e32b2356fbfc9e7e99c1c08d6f035bbf3947e1c79932c21c0e4b6ab13a`.
Those earlier checks did not prove actual Codex/Claude Code connection on clean
machines, immutable public release installation, or the five-participant gate.

On 2026-09-25, the immutable `v0.4.5` Mac asset passed a no-checkout native
installer smoke in an isolated home, including local JSON publication,
invocation, and cleanup. Hosted CI, strict checks, audits, and website tests
passed. The website, relay, marketplace, indexer, marketplace API, and node are
deployed as public beta. Before and after the digest-pinned hosted
`v0.1.0-beta.19` upgrade, an independent Linux x86_64 requester running the
immutable `v0.4.5` asset made free calls by explicit relay endpoint and by
marketplace discovery. All four calls succeeded with verified receipts and
only the three approved snapshot fields. The test publication was removed;
the relay denied it immediately and the indexed offer disappeared after its
health lease expired. An EU pre-upgrade snapshot is ready; a separate earlier
snapshot was restored to a temporary disk and checked. This is public-path
evidence, not the six clean agent/platform cells or five-user trial.

On 2026-09-25, `scripts/release_gate.sh --compose --package-assets
--install-smoke --version v0.4.1-rc.1 --platform darwin --arch arm64`
passed its secret scan, strict checks (including Compose), full docs build and
tests, asset verification, and installer smoke. Evidence is in
`_tmp/release_gate/20260925T085434Z/` locally; this ignored directory is not a
published attestation. A candidate-origin docs build also confirmed the
publishing prompt points to the guide in the same build. The corrected guide
is now live at `froglet.dev/learn/share-services/`.

An isolated macOS agent probe on 2026-09-25 did not satisfy the agent-connection
gate: Claude Code's CLI required login, and Codex discovered the native MCP
tool but its non-interactive approval policy blocked execution. Do not count a
discovered tool, a CLI health check, or an attempted call as an agent execution.

### Nonpayment beta qualification (2026-09-27)

The immutable `v0.4.6-beta.1` bundle and every downloaded asset were verified
against GitHub's recorded digests, its release-workflow attestation, and source
`71955ef96804fc40eb0e0c9610c1fd4ea4e770f8`. The release workflow passed Linux
strict checks, real Landlock/seccomp tests, native-worker address-space denial,
Docker Compose, packaged installation, and extended tests. Hosted CI also passed
the real rootless Podman worker test against a digest-pinned JSON echo container.

A fresh Mac requester using the released native binary called a separate Linux
provider over trusted public HTTPS. The fixed operation read Froglet's own health
JSON, with no paid upstream. All three admitted deals returned verified signed
receipts; the first receipt was
`24e8c3292ba0926dc6700267bed0cd4722d63ab70035bed55123ff318339b12b`.
Anonymous, expired, revoked, paused, and exhausted invitations were refused.
Revoked replay recovered the same deal and receipt. Pausing and cumulative
exhaustion survived provider restarts. The provider admitted exactly three deals
and reserved 15,000 ms, then remained exhausted after another restart. The
temporary ingress was removed and the provider stopped after qualification.

This qualifies the direct HTTPS invitation path through those released assets;
it does not qualify protected marketplace listing activation, live payment
settlement, a paid model API, or the clean-agent and participant gates below.
Payment rails remain deferred. See [HTTP services](HTTP_SERVICES.md) for operator
setup, explicit finite allowances, and the current product boundaries.

### Protected-listing qualification

The v0.4.6-beta.2 provider/client supports invitation-only relay
admission when paired with hosted v0.1.0-beta.21. Registration checks signed
metadata and the exact active revision without execution; the listing discloses
that boundary. The real local relay integration test uses zero quote/deal/runtime
allowance and verifies all three counters remain zero, anonymous execution is
refused, and disconnected listings expire from discovery. Kernel artifacts and
payment rails are unchanged. Hosted release/deployment qualification must be
recorded separately from these local checks.

A fresh native-MCP Codex local publication/invocation passed; see
[agent evidence](AGENT_PUBLISH_ACCEPTANCE.md#local-publication-and-invocation--2026-09-27).
The clean-machine and human gates below remain open.

The immutable beta.2 Release Bundle, all eight asset digests, and its exact-source
workflow attestation passed verification. A mandatory-attestation installation of
the released Mac binary then published an isolated local catalog and made one
call. The receipt was
`17691175b179cebabfb4d711bc007551b5c30e4e453c693ea7f92690705a16a6`;
usage was one quote, one deal, and 10,000 ms reserved. This proves the released
Mac executable on the current host, not a clean installation or public listing.

On 2026-09-27, the released beta.2 provider and separate requester passed the
production HTTPS invitation-listing drill against hosted beta.21. Metadata-only
admission used zero quote/deal/runtime allowance; anonymous execution was refused.
One invited call returned verified receipt
`fa67171171ea8ef57e7df5846efaf5a0b056dd04df71060d3aa35f8450f34254`.
Replay after revocation returned the same receipt with one quote, one deal and
10,000 ms reserved. Live rendered inspection confirmed invitation instructions and
QR sharing without embedding access credentials. The synthetic publication was
unpublished, its processes stopped, and its feed suspended with history retained.

The subsequent recovery correction is verified in the `v0.4.6-beta.3` candidate: native CLI and MCP recovered that receipt by share URL from the same
requester ledger after process restart and provider unpublish, without supplying
an invitation. This does not extend the earlier beta.2 executable's capabilities.

The beta.3 tagged release gate exposed a pre-existing Unix custody-lock race
before publication. A concurrently spawned child can retain the lock's open file
description until exec, briefly extending a completed operation's lock lifetime.
The beta.4 correction releases the lock explicitly at the guard boundary. A
regression retains a duplicate descriptor and checks both release and continued
exclusion while a replacement operation holds the lock. This changes no identity
material, canonical artifact, signature or recovery-journal format. Beta.3 remains
an unpublished candidate; beta.4 requires its own immutable release evidence.

### Required platform and external evidence

| Target | Codex on a clean machine | Claude Code on a clean machine |
|---|---|---|
| macOS Apple Silicon | Pending | Pending |
| Linux x86_64 | Pending | Pending |
| Linux arm64 | Pending | Pending |

For every cell, record OS and agent versions, independently trusted candidate
digest, actual installation approval, merged configuration diff, native service
health, an actual agent tool call, and verified catalog execution. No source
checkout, developer toolchain, or npm dependency is allowed. Exercise existing
unrelated settings, repeated install, interrupted setup, configuration drift,
occupied ports, reboot, failed upgrade, rollback, and uninstall with retained
identity and services. Refusal of an unknown partial installation is safe but
does not by itself qualify automated recovery after a hard crash.

On two independent machines, exercise trusted HTTPS, the real relay and
marketplace, exact publication approval, admission, the recipient's signed-offer
checks and free call, and receipt verification. Then test source update, changed
schema, deletion, pause/resume, rollback, unpublish, network loss, sleep/wake,
and reconnect. Confirm that stale approval, unexpected paid offers, and repeated
uncertain operations cannot produce misleading success or duplicate execution.

### Five-participant acceptance session

Recruit five first-time users with small catalogs and a separate-machine
recipient. Access to participants and test hosts remains a scheduling dependency;
no invitations have been sent by this implementation pass. Use the
[session procedure and evidence template](AGENT_PUBLISH_ACCEPTANCE.md#first-user-trial-kit--1-october-2026).
Provide this task:

> Make this catalog usable by another agent. Share only the fields the recipient
> needs, inspect a local result, approve publication, and give the link to the
> recipient's agent. Then explain what is public and what happens when your
> computer sleeps.

Start timing at the first publishing prompt. Stop only after the recipient's
independent call and receipt verification. Count installation and publication
approvals within the 15 minutes. The facilitator may observe but must not repair
configuration, run commands, explain hidden steps, or supply missing manifests.
Record participant ID, platform/agent, source format, elapsed time, interventions,
result/receipt evidence, privacy explanation, sleep explanation, and every blocker.
Pass only if at least four of five finish within 15 minutes without developer
intervention and explain both consequences correctly. Fix blockers and repeat
with fresh participants when needed; views or impressions do not count as trials.

This is a usability gate. Assess product value separately using the participant's
real task, their existing CSV/API workflow, independently checked outputs, actual
setup/operation effort, and observed voluntary reuse. An unassisted first call,
valid signatures, or a faster synthetic calculation does not establish demand,
scientific validity, or an operating-cost advantage.

UI qualification must also include desktop/mobile rendering, keyboard operation,
screen-reader announcements, contrast, zoom, stale/offline states, copying when
clipboard permission is absent, and actual downloaded file contents. DOM roles
and screenshots alone are not a complete screen-reader qualification.

### Stable release sequence (pending)

The historical beta sequence above does not qualify the resumed candidate.
Before a stable label, complete
every clean machine and agent cell, recovery and UI qualification, and the
five-participant acceptance gate described above. Cut a new immutable release
for any code changes made to close those gaps, update the hosted source pin, and repeat
the external journey through those final assets. Keep the stable claim closed
until the checks and human gate pass.

The 2026-09-24 services candidate audit reported `RUSTSEC-2026-0097` for
transitive `rand 0.10.0` and a yanked `chacha20 0.10.0`. The hosted beta.19
lockfile uses `rand 0.10.3` and `chacha20 0.10.2`; locked tests, Clippy, and
`cargo audit` passed before publication. Its source pin names the exact public
`v0.4.5` revision above.

The supported stabilization scope is the signed agreement/receipt chain,
offline verification (including browser WASM), local node and agent flows,
and the marketplace's derived catalog. Payment adapters remain subject to
the evidence limits in [PAYMENT_MATRIX.md](PAYMENT_MATRIX.md). Local mocks
and database tests do not establish live payment, relay, cloud lifecycle,
or Linux container isolation behavior.

Broker/enterprise purchasing, batch fan-out, GPU scheduling, richer trust
ranking, identity-attestation issuance, decentralized arbitration, TEE,
and selective disclosure are outside this candidate's supported scope.
They must not be advertised as completed features. A signed receipt proves
the signer's statement and artifact linkage, not independent execution
quality or external settlement finality.

Use semver with a new version for every immutable release. GitHub marks
`v0.4.5` as a prerelease even though its version has no prerelease suffix;
future beta versions can use an explicit suffix such as `v0.4.6-beta.1`.
The Git tag must be prefixed with `v`.

`Cargo.toml` and the Git tag must match exactly apart from that leading `v`.
The release workflow checks this and fails if they diverge.

Repository or organization **immutable releases must be enabled before a tag
is created**. The GitHub Actions token cannot read this administration API:
its attempt on `v0.4.1` returned HTTP 403, so that tag has no release assets.
An operator with repository administration access must first run the read-only
`GET /repos/{owner}/{repo}/immutable-releases` check with API version
`2026-03-10` and require `enabled=true`. After the exact source commit is on
`main`, the operator creates a **draft** GitHub release using an unused tag
and `--target` set to that commit. Creating the draft does not create a Git tag.
For a beta, also set `--prerelease --latest=false`. Verify the draft's prerelease
status and exact target before tagging; the workflow does not derive that
status from the tag suffix. After publication, check that the stable `latest`
release is unchanged.
Then create and push that exact tag at the same source commit to trigger the
workflow. The workflow requires the existing draft and verifies that the tag
resolves to its source commit; it cannot create a release on its own. It then
uploads every binary, checksum, manifest, and attestation asset and publishes
only after the bundle is complete. The final job requires the published
release to report `immutable: true`. Immutable releases are not rebuildable
or overwritable. See GitHub's
[immutable release model](https://docs.github.com/en/code-security/concepts/supply-chain-security/immutable-releases)
and [release API](https://docs.github.com/en/rest/releases/releases).

## Published Images

Pushing a matching tag triggers
[../.github/workflows/release.yml](../.github/workflows/release.yml), which
publishes the role-specific images:

- `ghcr.io/armanas/froglet-provider:<version>`
- `ghcr.io/armanas/froglet-provider:<sha-tag>`
- `ghcr.io/armanas/froglet-runtime:<version>`
- `ghcr.io/armanas/froglet-runtime:<sha-tag>`
- `ghcr.io/armanas/froglet-dual:<version>`
- `ghcr.io/armanas/froglet-dual:<sha-tag>`
- `ghcr.io/armanas/froglet-mcp:<version>`
- `ghcr.io/armanas/froglet-mcp:<sha-tag>`

The tags are discovery aliases. Agents and bootstrap scripts consume the
matching `@sha256:` references from the verified Release Bundle rather than
persisting mutable tags.

If the repository remains private, the package visibility still has to be
changed to public in GitHub package settings before anonymous pulls work.

## Published Docs

`docs-site/` is configured for Cloudflare Workers via
[`docs-site/wrangler.jsonc`](../docs-site/wrangler.jsonc). Production deploys
run either through `npm --prefix docs-site run deploy` when Cloudflare
credentials are present locally, or through Cloudflare Workers Builds with:

- Build command: `npm run build`
- Deploy command: `npx wrangler deploy`

The build requires Rust with the `wasm32-unknown-unknown` target. It compiles
the offline verifier and uses the exact `wasm-bindgen` CLI version from
`Cargo.lock` before building Astro. Running `astro build` alone is not a
complete website build. Use `npm ci` and `npm run build`; see
[`docs-site/README.md`](../docs-site/README.md) for local Worker verification.

The repo no longer uses GitHub Pages for docs deployment. The intended public
shape is the apex `https://froglet.dev`; `docs.froglet.dev` previously mirrored
the same deployment and is no longer advertised as a separate launch surface.
The public host should only be treated as live after the Cloudflare deployment
and direct route checks for `/`, `/learn/quickstart/`, and `/learn/cloud-trial/`
pass.

## Published Binaries

The same tagged workflow also publishes GitHub release assets for:

- `froglet-node-<tag>-linux-x86_64.tar.gz`
- `froglet-node-<tag>-linux-arm64.tar.gz`
- `froglet-node-<tag>-darwin-arm64.tar.gz`
- `agent-bootstrap.sh`
- `SHA256SUMS`
- `release-manifest.json` (`froglet.release-bundle.v1`)
- `release-manifest.intoto.jsonl` (offline GitHub artifact attestation bundle)
- `agent-bootstrap.intoto.jsonl` (separate offline bootstrap attestation bundle)

The one-line installer at [../scripts/install.sh](../scripts/install.sh)
downloads from those release assets. By default it installs the latest tagged
`froglet-node` release into `~/.local/bin`. Use `VERSION=<tag>` to pin a
release and `INSTALL_DIR=/path` to override the destination.

Before trusting manifest values, the dependency-free installer queries the
official GitHub Releases API for the requested tag, requires
`immutable: true`, requires exactly one manifest and bootstrap asset, and
verifies both downloads against their API `sha256:` digests. It
then verifies the platform archive against the digest in the trusted manifest.
If GitHub CLI is already installed, the installer additionally verifies the
offline artifact attestation against the expected repository, release
workflow, tag ref, and constrained source revision. A caller may instead
provide an independently trusted `FROGLET_RELEASE_MANIFEST_SHA256` pin; there
is no unverified bypass. Set `FROGLET_GH_ATTESTATION_MODE=required` when `gh`
provenance verification must be a hard gate, or `off` for a deliberately
dependency-minimal host that still enforces immutable-release digest trust. See
[`scripts/release_manifest.py`](../scripts/release_manifest.py) for the exact
closed v1 field set and validation rules.

The public first-hop block resolves immutable release metadata and verifies the
uploaded bootstrap API digest before any bootstrap bytes execute. The bootstrap
then provides a native two-call contract. `plan` writes only temporary files,
requires its own bytes to match both the manifest and GitHub asset digest, and
verifies the unique release-manifest and exact target-platform binary digests,
and binds the current bootstrap plus the source-revision install, lifecycle,
agent-setup, and payment-setup script bytes. It returns a canonical approval
hash covering the profile, persistent paths, service-manager impact, and exact
execution command. `execute` recomputes the same contract before creating a
persistent directory or changing a service and installs the exact approved
binary only when the hash matches. This removes `gh`, Python, Node.js, `jq`,
Docker, and cloud CLIs from the native lane. HTTPS/GitHub availability remains
an external dependency, but mutable branch bytes are no longer an executable
first hop.

The optional JavaScript MCP/OpenClaw compatibility surface provides a stronger
pre-execution contract for agents. `plan_install` reads GitHub release metadata
and the tag-specific bootstrap in memory, requires a published immutable
release, and returns an approval hash that binds the release tag, manifest and
bootstrap SHA-256 values, install profile, persistent paths, process-manager
impact, and exact command. `get_install_guide` returns executable commands only
when that same contract is recomputed from the exact `release_tag` and matching
`install_approval_hash`; its default command verifies the temporary bootstrap
file before passing `VERSION` and `FROGLET_RELEASE_MANIFEST_SHA256` to it.

The public release surface covered directly by the tag workflow is the tracked
protocol docs in this repo, reference node binaries, tagged container images,
supported integrations, and validation assets. The public docs host and the
first-party hosted node are separate deploy steps outside the tag workflow.

## Release Candidate Gate

This is the current release gate for the public Froglet repo. It has one
entrypoint, [`scripts/release_gate.sh`](../scripts/release_gate.sh), which
runs selected line items in sequence, writes per-step evidence logs into
`_tmp/release_gate/<UTC-timestamp>/`, and prints a pass/fail summary at the
end. Required software steps are `secrets`, `strict`, `docs-build` and
`docs-test`. A failed step exits `1`; a skipped required step makes the
software gate `INCOMPLETE` and exits `2`. Skipping an explicitly requested
package or install check also makes it incomplete. An unrequested optional
package or install smoke remains recorded as `SKIP`. Exit `0` proves only the selected
local software scope, not the external launch gates above. Unknown skip IDs
and malformed packaging arguments are rejected before executing steps.

### Running the gate

```bash
# Minimum gate (covered end-to-end from this repo, no external deps):
./scripts/release_gate.sh

# Full local gate, including the compose-backed OpenClaw+MCP smoke:
./scripts/release_gate.sh --compose

# Native package check on a Linux x86_64 host (use the matching Cargo version):
./scripts/release_gate.sh \
  --package-assets \
  --version v0.4.6-beta.4 \
  --platform linux \
  --arch x86_64

# Native package + installer smoke on Apple Silicon macOS:
./scripts/release_gate.sh \
  --install-smoke \
  --version v0.4.6-beta.4 \
  --platform darwin \
  --arch arm64

```

Every step writes to `_tmp/release_gate/<ts>/<step>.log`, and the summary is
also dumped to `_tmp/release_gate/<ts>/summary.tsv` for CI ingestion.
The secret scanner preserves separate fresh run directories inside its evidence
root. Local gate packaging rebuilds the native release executable from this
checkout and requires the tag to match `Cargo.toml`; it refuses cross-target
labels. The release workflow builds each target on its native runner. Standalone
archive structure and checksum checks alone do not prove source provenance.

First-party hosted smoke for `ai.froglet.dev` is intentionally outside this
scripted public-repo gate and is maintained separately from the public repo
checks. Launch still requires separate hosted evidence in the manual gates
below.

### Gate steps

| Step id | Status today | Validation | Underlying command | Notes |
| --- | --- | --- | --- | --- |
| `secrets` | Ready | Publication secret scan | `./scripts/gitleaks_gate.sh` | Tracked and non-ignored untracked public source, candidate `HEAD`, and the selected GitHub-visible refs. |
| `strict` | Ready | Repo validation matrix | `./scripts/strict_checks.sh` | Rust, Python, OpenClaw, MCP, release helper syntax. Also gates compose/LND/Tor integrations via env flags set by the gate. |
| `docs-build` | Ready | Docs build | `npm --prefix docs-site run build` | Pre-publish docs-site build |
| `docs-test` | Ready | Docs-site unit tests | `npm --prefix docs-site test` | Vitest suite under `docs-site/src/**/__tests__/` |
| `package` | Ready (opt-in) | Release asset packaging + verification | `scripts/package_release_assets.sh` + `scripts/verify_release_assets.sh` | Requires `--version`, `--platform`, `--arch` |
| `install-smoke` | Ready (opt-in) | Installer-path smoke from packaged assets | `scripts/smoke_install_from_assets.sh` | Implies `--package-assets`; packaged target must match the current host |

<a id="hard-launch-gates-outside-the-script"></a>

### Historical v0.1.0 launch gates outside the script

This section preserves the earlier paid-MVP launch plan. The current free
candidate uses the [October launch gates](#production-launch-preparation--2-october-2026)
above and the required platform/external evidence. Neither plan can be
satisfied by a local software-gate exit code alone. The earlier `v0.1.0`
plan required:

- Live Claude MCP smoke. Claude Code or Claude Desktop must load the generated
  Froglet MCP config and complete the expected tool smoke. This is a hard
  blocker, not a nice-to-have.
- First-party hosted trial smoke. `try.froglet.dev` must mint a session,
  expose the documented free demo catalog, and complete the canonical
  `demo.add` flow with a receipt.
- Hosted upstream guard smoke. Direct public session/demo writes to
  `ai.froglet.dev` must remain outside contract and reject as documented in
  [HOSTED_TRIAL.md](HOSTED_TRIAL.md).
- Distribution smoke for every launch channel named in the release notes.
  Record direct evidence links in the release PR or release notes.

That paid MVP launch gate required one live crypto rail and one live fiat
rail. Lightning and Stripe are the selected blockers; x402 is desirable but
must not block launch if its hosted proof is not ready. Do not claim any
hosted paid rail until the public transcript for that rail exists.

Confidential/TEE execution must remain framed as experimental for v0.1.0
unless a real attestation backend is proven and documented. The current launch
copy must not imply production TEE guarantees from a mock or limited backend.

### Cut steps

1. Update `Cargo.toml` package version.
2. Move the relevant `Unreleased` notes in [../CHANGELOG.md](../CHANGELOG.md)
   into a concrete version section.
3. Run the release gate with the release-cut flags. If you include
   `--install-smoke`, use the current host target so the packaged binary can
   execute locally:
   ```bash
   ./scripts/release_gate.sh \
     --install-smoke \
     --version v0.1.0-alpha.1 \
     --platform darwin \
     --arch arm64
   ```
4. Run the first-party hosted smoke separately when the hosted stack is part
   of the cut.
5. Commit the version/changelog update (attach the gate evidence directory path
   in the PR description).
6. Push the release tag, for example:

```bash
git tag v0.1.0-alpha.1
git push origin v0.1.0-alpha.1
```

## Historical v0.1.0 GitHub Release Body Draft

This is an archived draft, not copy for the current free candidate. Use it as
the release body only for that `v0.1.0` scope after replacing evidence
placeholders with links to the final release gate, workflow, and hosted smoke
results.

````md
# Froglet v0.1.0

Froglet v0.1.0 is the first public release of the reference Froglet node and
bot-facing integration surface. It ships the signed kernel artifacts, the
`froglet-node` binary, container images, local agent setup, and a constrained
free hosted demo catalog with one canonical end-to-end proof.

## What ships

- `froglet-node` binaries for Linux x86_64, Linux arm64, and macOS arm64
- `SHA256SUMS` for release asset verification
- GHCR images:
  - `ghcr.io/armanas/froglet-provider:0.1.0`
  - `ghcr.io/armanas/froglet-runtime:0.1.0`
  - `ghcr.io/armanas/froglet-dual:0.1.0`
  - `ghcr.io/armanas/froglet-mcp:0.1.0`
- Docker Compose starter configuration
- OpenClaw/NemoClaw plugin under `integrations/openclaw/froglet/`
- MCP server under `integrations/mcp/froglet/`
- public docs at `froglet.dev`
- free hosted trial at `try.froglet.dev`

## Install

Use the transparent dependency-free shell block in the public
[Quickstart](../docs-site/src/content/docs/learn/quickstart.mdx). It resolves
the immutable release metadata, verifies the uploaded `agent-bootstrap.sh`
asset against GitHub's API digest before execution, then runs the separate
`plan`/approved-`execute` contract. For a pinned release, set `tag` to the
desired normalized tag before the metadata request instead of resolving
`/releases/latest`; do not replace the release-asset URL with a mutable raw
branch URL.

## Verification

Release evidence:

- default release gate: `<link-to-release-gate-summary>`
- compose OpenClaw/MCP smoke: `<link-to-compose-smoke-evidence>`
- Claude MCP smoke: `<link-to-claude-smoke-evidence>`
- hosted trial smoke: `<link-to-hosted-trial-curl-transcript>`
- release workflow: `<link-to-github-actions-run>`
- checksums: `<link-to-SHA256SUMS>`

## Hosted trial scope

The hosted trial is free-only. `demo.add` is the canonical discover -> deal ->
result -> receipt proof through `try.froglet.dev`; witness/hash/notarize demos
can provide stronger evidence for URLs or content hashes. It does not prove
paid rails, persistent identity, hosted account recovery, service publication,
marketplace depth, or general hosted runtime access.

## Payment rails

Local and self-hosted payment setup is documented for Lightning, Stripe, and
x402. The MVP public paid-service claim requires hosted evidence for:

- hosted Lightning: launch blocker
- hosted Stripe: launch blocker
- hosted x402: desirable, not launch-blocking unless verified before launch

## Confidential and TEE scope

Confidential routes and artifacts are experimental in v0.1.0. The launch does
not claim production TEE guarantees unless a real backend has been separately
proven and documented; mock or limited attestation remains explicitly limited.

## Known limits

- no hosted paid settlement
- no persistent hosted user identity
- no PyPI, npm registry, Homebrew, or OS package-manager distribution
- marketplace and hosted-provider claims depend on the linked live smoke
  evidence above
````

## Release Notes Template

Use the matching changelog section as the release body. For the first alpha,
the release notes should call out:

- published `SHA256SUMS` for release asset verification
- tagged provider, runtime, dual-role fallback, and MCP images in GHCR
- downloadable `froglet-node` binaries
- official site at `froglet.dev` if the docs deployment is live at cut time
- public OpenClaw integration
- reference discovery
- reference operator image
- local/self-hosted payment adapters: Lightning, Stripe, and x402
- hosted Lightning and Stripe claims only if live transcripts exist; hosted
  x402 remains desirable but non-blocking
- Claude MCP smoke evidence, because it is a hard launch blocker
- confidential/TEE scope as experimental unless a real backend is proven and
  documented
- any intentionally deferred layers, especially external broker and closed higher-layer
  services
