# File sharing and bounded container compute

Status: implementation paused at the operator's request; F1 implemented locally, C1 partial, G1 disabled. No deployment has been made for this scope. See [restart tasks](../TODO.md#current-checkpoint--28-september-2026).
Version: implementation checkpoint 0.2, 2026-09-28.
Scope: the Froglet Node, native agent integration, share page, and companion relay/OCI worker.

## 1. Outcome and boundaries

Deliver three independently useful releases, in this order:

1. **Files:** Bob selects a file, reviews what becomes public, and sends Alice a link or QR. Alice downloads the exact approved bytes without installing Froglet.
2. **CPU jobs:** Bob provides a configured machine and finite capacity. Alice requests an allowed container image and a bounded job, then retrieves its status and result or cancels it.
3. **GPU jobs:** the same job flow can reserve one whole supported GPU exclusively for one job, with enforced deadlines and verified release of the device.

Payments remain deferred. These releases use zero-fee terms and existing private/invitation access, or an explicitly enabled finite public allowance. They must not require a wallet, introduce credits, or imply a purchase flow. Implementation was subsequently requested; infrastructure purchases, deployments and increased production limits remain outside this change.

The first compute release targets one Linux provider and one dedicated worker host. It does not add a cluster scheduler, cloud autoscaling, an interactive shell, a VM rental product, Kubernetes, distributed training, or indefinite sessions. File sharing is download-only: uploads, synchronization, shared editing, folder traversal, and general object storage are outside this scope. An explicitly prepared archive can be shared as one file.

Keep the [Kernel](KERNEL.md) unchanged. New job, transfer, and worker state belongs in application adapters and operational storage. Reuse the existing Publication Revision, service identity, access controls, usage accounting patterns, and share URL. Any change that genuinely requires different canonical artifacts or state transitions needs separate interoperability discussion.

## 2. Verified starting point

This table records the starting point before implementation, not the current feature status or a deployment claim. See the checkpoint below.

| Area | Existing implementation | Gap addressed here |
|---|---|---|
| File preparation | [Native preparation](../src/cli/prepare.rs) selects CSV, JSON, or SQLite data into a snapshot; it also accepts Wasm. | An arbitrary regular file is not yet a downloadable publication. |
| Sharing | [Service links](SHAREABLE_SERVICE_LINK.md), branded QR codes, public metadata, and lifecycle controls exist. | A file download action and resource-specific job guidance are missing. |
| Relay | [Relay v1](RELAY.md) forwards bounded, fully buffered requests/responses; its response-header allowlist is narrow. | Download headers are missing; streaming and range/resume are unsupported. |
| CPU containers | The [node adapter](../src/oci_worker.rs) calls a separate authenticated worker. The reference worker (private `froglet-services` repository, not open source) enforces rootless execution and finite limits. | The worker uses `--pull=never`; images must already exist. Its input and successful output are JSON. |
| Jobs | [Job storage](../src/jobs.rs) and [routes](../src/api/http_execution.rs) provide creation, polling, request hashes, and idempotency. | No cancellation route exists there. Recovery can requeue running jobs; external containers require reconciliation before retry. |
| Admission | [Protected access and cumulative allowances](CONFIGURATION.md#provider-access-pause-controls-and-usage-allowances) persist across restarts. | Pull bytes, file transfers, GPU occupancy, and retained outputs need explicit accounting. |
| GPU | Configuration can retain inventory labels, but the executor refuses GPU capabilities. | Device discovery, assignment, execution, and release must be proven before advertising support. |

Rootless containers are an existing defense layer, not evidence of safe arbitrary multi-tenant code execution. The first compute release admits provider-approved images and callers. Unrestricted customer images remain a separate qualification gate.

## 3. Shared contract

### Provider, requester, and marketplace

The provider chooses the exact resource, access policy, allowed images, finite limits, retention, and transport. The agent presents the resolved plan before publication; standing image policy can authorize later matching jobs without a new approval for every call. Material changes invalidate the old plan.

The requester sees what will be downloaded or executed, the selected immutable revision/image, maximum resource use, result retention, and any access requirement. A retry must recover the original operation or report a conflict; it must not silently start additional work.

The marketplace lists and describes resources. It does not pull images, run customer workloads, or download full files just to render a listing or social preview. It distinguishes `reachable`, `requires access`, `busy`, `paused`, `exhausted`, and `offline`; reaching an endpoint is not execution evidence. Private/invitation services must not be run anonymously by an admission probe.

### Admission and cost protection

- New resource types are disabled until the provider supplies all required finite ceilings. An absent limit does not mean unlimited. Existing installations and the previously approved relay allowance are not changed automatically.
- Reserve worst-case permitted use transactionally **before** a download, pull, or execution. Scope every operation to its provider, resource revision, requester or transfer authorization, and idempotency key. Reusing a key with different inputs fails.
- Enforce both per-operation limits and persistent aggregate limits. Account separately for request count, CPU core-time, runtime, GPU occupied time, registry download bytes, transfer bytes, and storage occupancy where applicable.
- Failed attempts and disconnects do not replenish cumulative allowances. Replays do not create new reservations, but retransmitting bytes consumes additional transfer allowance. Any explicit reconciliation of unused reservations must be conservative and auditable; unknown use stays reserved.
- Disk and concurrency reservations may be released only after deletion or termination is verified. These occupancy reservations are distinct from cumulative usage, which does not reset on restart, redeploy, cache eviction, or date rollover.
- Account for all relevant paths: metadata/polling, authentication failures, downloads, image resolution and pulls, execution, results, logs, and cleanup. Keep control/recovery capacity separately bounded so exhaustion does not prevent shutdown.
- A worker enforces deadlines locally even if the caller, node, or relay disconnects. No automatic retry may exceed the original deadline or reserve a second execution without explicit authorization.
- Application allowances do not cap a cloud bill. Deployment qualification must record fixed hosting costs, infrastructure rate/concurrency limits, disk/log limits, and the operator's stop procedure. No autoscaling or automatic allowance refill is introduced.

### Privacy and evidence

Links, QR codes, previews, manifests, and logs must contain no access token, local source path, registry credential, worker credential, or host device identifier. A share link grants discovery, not access. Existing invitations are provider-scoped; do not describe them as service-scoped. A derived transfer/job authorization must narrow access to its exact resource and caller.

Public metadata may survive in caches after unpublication. The publication plan must explicitly disclose filename/title, size, media type, image/model labels, and content fingerprints that would become public. File bytes and job inputs/results are not preview material.

Relay traffic is readable by the relay operator after TLS termination. These releases do not add end-to-end confidentiality or anonymous onboarding. First-party branded pages target HTTPS relay services; Tor compatibility is independently tested and must not be silently downgraded to clearnet.

## 4. Release F1: download-only files

### User journey

1. Bob asks his agent to share an exact regular file. Preparation reads it into a provider-owned immutable snapshot, calculates its size and digest, and creates a private draft.
2. The plan shows public metadata, who may download, the snapshot version, expiry, maximum file/transfer/storage sizes, and the provider-online requirement. Source edits never silently change the published file.
3. Publication uses the existing lifecycle and exact consent machinery. It returns the existing share URL and logo QR. The page shows a **Download file** action, filename, size, version, access requirement, and current availability.
4. Alice downloads through a browser or agent. An invitation can be entered separately on the page; it is submitted only to the verified provider endpoint, never in a URL, QR, analytics event, or persistent browser storage.
5. Pause, unpublish, expiry, or revocation stops new transfer admission. An already admitted F1 transfer may finish within its reserved bytes and deadline; the UI must disclose this. Operator abort closes active transfers. Bytes already received cannot be revoked.

### Required implementation

- Bind the snapshot digest, length, and approved metadata through an application file contract covered by the Publication Revision. Validate it using the existing signing machinery; do not invent a separate unsigned source of truth or change Kernel signing bytes.
- Accept regular files only; reject directories, symlinks, device files, sockets, and FIFOs. Open safely against path replacement, copy from the opened handle within size bounds, and hash the completed snapshot. Downloads never reopen the original source path.
- Snapshot storage has a provider-owned byte quota. Preparation cannot fill the disk before approval. Failed staging is cleaned up; active snapshots cannot be evicted as disposable cache.
- Add service/revision-addressed metadata and download handlers. No user-provided filesystem path or arbitrary proxy URL is accepted. Relay grant checks must cover these routes without exposing operator APIs or unrelated publications.
- Metadata/HEAD requests never trigger a full file transfer. Browser download admission reserves count and bytes before opening the response. HTTP requests cannot bypass transfer admission merely because the file is free.
- Return exact bytes with a sanitized attachment filename, correct length, safe media type, `nosniff`, and a version validator. HTML/SVG and other active content download as attachments; they are not rendered inside the Froglet origin.
- Extend both relay header allowlists only for the required download headers. Scope browser CORS to approved share origins; do not expose credentials through redirects. Protected responses use `no-store` and cannot be served from a public CDN cache.
- F1 caps files at **8 MiB** and permits bounded buffering. Validate the entire frame and node/edge limits including encoding overhead; reduce the effective cap if any layer is lower. Do not bypass limits with base64-in-JSON downloads or increase the global relay body cap.
- F1 has no resume/range support. Reject range requests explicitly and do not advertise `Accept-Ranges`. A failed retry is a new transfer reservation unless no bytes can have been sent.
- A successful browser download is an operational transfer record, not automatically a signed Deal/Receipt. Report provider bytes sent separately from requester checksum verification. Native signed interactions must retain the existing evidence semantics.

### Optional follow-up F2: larger, resumable files

F2 is separately shippable and is not required for F1. Add a negotiated relay streaming capability while retaining v1 behavior for existing clients. Specify bounded chunks, backpressure, total/idle deadlines, cancellation, and per-stream/per-provider memory and byte admission. Support one HTTP byte range per request, exact length/validator checks, and version-bound resume; reject multipart ranges. Reserve and charge retransmitted bytes. Old relay/node pairs must refuse oversized transfers with a clear limit rather than buffer them or silently fall back.

F2 must pass a slow-reader memory-bound test and interrupted resume against the same snapshot before raising the file ceiling. A source update cannot splice two versions into one resumed download. No streaming endpoint may become an unrestricted TCP or HTTP tunnel.

## 5. Release C1: bounded CPU container jobs

### Product scope

Support finite batch commands in provider-approved OCI images. The requester may select an allowed digest, input, and arguments within the provider policy. Initially support public registries and explicit immutable digests; a setup helper may resolve a tag, but the provider approves the resulting digest. A mutable tag is never resolved again at execution time.

The initial implementation has one worker and a small bounded queue. It supports JSON-in/JSON-out images through the current adapter and a versioned batch-command adapter for ordinary tools that use arguments, stdin, exit codes, and output files. This adapter is necessary before claiming general container jobs; arbitrary existing images must not be assumed to implement Froglet's JSON convention.

No interactive terminal, incoming container ports, privileged mode, host namespaces, engine socket, host filesystem mounts, or caller-supplied secrets are exposed. Network is off for jobs in C1. Registry fetching is performed by the worker's controlled image-preparation path, independently of container network access.

### Image preparation and execution

- Image resolution/pull is authenticated and admitted before registry traffic. Policy restricts registry/repository/digest and architecture. Validate redirects and registry-provided fetch locations, block private/link-local/metadata targets, and do not forward credentials across origins.
- Pull and extraction have finite deadlines, byte limits, concurrency, and a hard storage quota. Manifest size estimates alone are insufficient for an expanded-image bound. Refuse automatic pulling on a worker whose image-store growth cannot be bounded.
- Deduplicate preparation of an identical digest on the same worker. A cache miss can wait only within a bounded preparation deadline. Verify the selected platform manifest and layer digests; run the exact resulting image, never a changed tag or architecture fallback.
- Evict only worker-owned unused cache entries. Never prune the operator's other images, active layers, publication snapshots, results under retention, or accounting records. Pull failure and interrupted extraction are recovered before admitting more storage.
- Bind image digest, platform, entrypoint/argument array, input digest, permitted output paths, and resource ceilings to the request hash and exact admission. Use argument arrays, not shell interpolation. Code in the image remains untrusted.
- Retain rootless execution, a non-root container user, dropped capabilities, read-only root filesystem, bounded scratch, no new privileges, finite PID/CPU/memory/swap/output/runtime limits, and no host-engine socket on the Froglet Node.
- Permit only explicit per-job scratch and result directories. Validate output files without following symlinks or special files, enforce aggregate size, and return bounded metadata plus download references. Results reuse the F1 delivery path and respect its file ceiling until F2 is qualified. They remain job-private: reuse must not create a public publication, preview, or listing for a result.
- Keep stdout/stderr and exit status separate from a JSON result. Do not publish logs automatically. Bound and redact operator diagnostics; untrusted job output is visible only to its authorized requester and provider. C1 injects no provider/registry credentials into jobs.

### Jobs, cancellation, and recovery

Extend existing job persistence and request hashing rather than add a queue service. The external worker needs durable execution identity, status, and cancellation/reconciliation support; the current synchronous worker call cannot prove that a disconnected process has stopped.

Operational states are `queued`, `preparing`, `running`, `stopping`, and terminal `succeeded`, `failed`, `canceled`, or `timed_out`. Track cleanup separately as `pending`, `verified`, or `quarantined`; terminal user-facing failure is not evidence that a device or slot is reusable. These are application job states, not new Kernel Deal states.

- Persist the job, authorization, exact request hash, reservations, and worker execution identity before dispatch. Worker dispatch is idempotent for that identity and refuses conflicting bytes.
- Status, logs, result download, and cancellation are bound to the authorized requester/provider and original operation. Knowing a job ID is insufficient. Resolve cancellation/completion races transactionally.
- Cancellation stops preparation or signals the running workload, escalates termination within a fixed grace period, and verifies container/process termination and cleanup. Report `stopping` until verified; quarantine uncertain capacity.
- Node/worker restart reconciles durable container identity and deadline before retry. Do not apply the current generic running-to-queued recovery blindly to external containers. If execution may have happened, report an indeterminate failure and retain the reservation; never promise exactly-once external side effects.
- A local supervisor enforces the persisted deadline independently of the request handler and worker API process. Killing that process without restarting it must not leave an indefinitely running container. Worker restart finds and terminates expired/orphaned containers before accepting new work. A host reboot must not auto-restart old workload containers.
- Publication pause stops new admissions; it does not implicitly cancel admitted jobs. Provide separate requester cancellation and provider stop-all actions. Queue expiry consumes no execution but does not erase incurred preparation cost.

Remote compute continues through signed Quote/Deal admission and existing result recovery. Keep owner-only `/v1/node/jobs` behind runtime authentication; it is not a public shortcut around protected admission. Extend provider/requester application adapters for status/cancel/results, with request-bound authorization, rather than forwarding the local runtime bearer token.

Map operational outcomes to existing Kernel semantics. For example, cancellation before execution can have `execution_state=not_started`; killing an executing workload is an execution failure and must not fabricate a successful result. Fixtures must verify every mapping against the existing conformance rules before release.

## 6. Release G1: one exclusive GPU per job

G1 depends on C1 cancellation, persistent accounting, and recovery evidence. Target one documented Linux/NVIDIA driver/runtime combination on actual hardware first. Other vendors and host platforms remain unsupported until separately qualified.

- Discover hardware on the worker and verify device availability and a small execution probe. Operator-supplied inventory labels alone cannot enable a capability or marketplace badge.
- Reserve one entire physical GPU transactionally before launch. Use one job per device; no fractional allocation, MIG, MPS, time slicing, multi-GPU jobs, or distributed scheduling in G1.
- Pass only the selected device to the container, never all host GPUs. Bind the device allocation to the durable job identity and worker authorization. Do not expose host-specific device identifiers in public metadata.
- Publish detected model and total usable memory. Whole-device allocation is not enforcement of an arbitrary requested VRAM quota; do not claim a smaller hard memory partition or guaranteed throughput.
- Bound wall-clock occupancy and cumulative GPU occupied time alongside C1 CPU, RAM, disk, network, and output limits. Count device initialization and cleanup occupancy, not only kernel execution time. Reserve a cleanup margin and stop execution before the occupied-time deadline; a cleanup overrun quarantines capacity and is recorded rather than hidden or automatically granted a fresh budget.
- On cancellation, timeout, crash, or disconnect, stop the job, verify there are no surviving job processes or device contexts, and run the qualified health/reset check before reuse. Uncertain cleanup quarantines the GPU; it must not be handed to another requester automatically.
- A driver reset/failure reports an execution failure without silently rerunning on CPU or another GPU. Health, availability, and usage reports must distinguish declared limits from observed values.
- Establish and test the cross-job data-isolation boundary for the supported stack. A passed arithmetic demo alone does not qualify unrelated untrusted GPU renters. Until that gate passes, limit G1 to provider-approved images and trusted/invited requesters on a dedicated worker.

## 7. Proposed first-release limits

These are conservative **new-feature profile proposals**, not changes to any current deployment. Providers must explicitly approve finite aggregate budgets appropriate to their hardware. Policy is the minimum of the request, publication, provider, worker, and transport ceilings; reject incompatible plans before exposure.

| Resource | Proposed initial ceiling |
|---|---|
| F1 file | 8 MiB; one concurrent transfer per provider; finite total downloads, egress bytes, and snapshot storage required |
| CPU queue | Four waiting jobs; one executing job per worker; maximum queue age five minutes |
| CPU job | One CPU core, 512 MiB RAM, 64 PIDs, five minutes execution, 64 MiB scratch |
| Job input/results | 1 MiB input; 1 MiB each stdout/stderr; 8 MiB total result files; 24-hour result retention within a finite storage quota |
| Image preparation | One pull at a time; two-minute deadline; 512 MiB fetched bytes, 1 GiB expanded image, 2 GiB total worker image cache under an enforced storage quota |
| GPU job | One exclusive GPU; five-minute occupied-time allowance including cleanup; explicit finite cumulative GPU budget |

If a GPU image needs larger image, RAM, or scratch allowances than this CPU starter profile, G1 qualification chooses and records an explicit tested profile; it must not silently enlarge C1 limits. No stage ships with unlimited pulls, unlimited result retention, or a daily automatic budget reset.

## 8. Delivery boundaries and acceptance gates

Each release includes the native agent action, CLI equivalent, provider status/stop controls, documentation, and the share-page/marketplace presentation for that resource. A link must explain what Alice can do and why an action is unavailable. Social previews must not trigger execution, downloads, or GPU allocation.

| Gate | Required evidence |
|---|---|
| F1-1: exact content | Browser and independent agent download a known fixture through the real relay; byte count and digest match the approved snapshot. Source modification does not change it. |
| F1-2: access/lifecycle | Missing/expired/revoked access, paused/unpublished revisions, wrong-service grants, and expired transfers cannot start downloads. An admitted transfer obeys the documented finish/abort rule. |
| F1-3: filesystem/privacy | Symlink/path-replacement and special-file fixtures fail; no source paths/tokens/file bytes appear in public metadata, QR, previews, or caches. Active content downloads as an attachment. |
| F1-4: exhaustion | Concurrent requests, disconnects, retries, maximum-size files, and restart cannot exceed reserved transfer/storage ceilings. Mobile and desktop download flows are inspected live. |
| F2-1: streaming, if shipped | A slow reader keeps memory within a measured fixed bound; interruption/resume verifies one version; range abuse and old-capability clients fail safely. |
| C1-1: real worker | On the declared Linux/rootless engine, pull an approved public image into an empty cache and run both a JSON fixture and an ordinary batch-command fixture. Preserve digest and result evidence. |
| C1-2: admission | Disallowed images/registries, changed tags, wrong architecture, private registry targets, oversized layers, exhausted disk/runtime budgets, and duplicate concurrent submissions cannot bypass limits. |
| C1-3: termination | Exercise CPU/memory/PID/output/scratch exhaustion, hung pulls, cancellation during each phase, node and worker death including no restart, and host reboot. Prove deadline enforcement and termination or quarantine before capacity reuse. |
| C1-4: recovery/isolation | Retry and crash injection do not duplicate an uncertain execution; requesters cannot inspect/cancel another job; jobs cannot access host files, engine sockets, credentials, other results, or network. |
| G1-1: actual device | Execute a known GPU operation on the declared hardware and prove it used the assigned device. Concurrent jobs cannot share it or access another device. |
| G1-2: failure/reuse | Kill client/node/worker and cancel or exceed deadline during GPU work. Prove bounded occupancy, cleanup/reset, healthy reuse, and quarantine on ambiguous failure. |
| G1-3: honest capability | Missing device, incompatible driver/runtime, failed health check, and unavailable capacity yield accurate unavailable/busy states, never CPU fallback or fabricated GPU success. |
| Common: compatibility | Existing data/function publication, invitation, QR, relay-v1, signed evidence, and Kernel conformance tests remain passing; old clients receive explicit unsupported-capability responses. |

Start with focused unit/contract tests, then real worker/relay integration and live UI checks. Run each touched repository's required release checks before deployment. Fake runners cannot satisfy C1 or G1 hardware/isolation gates. Record source revision, release digest, effective limits, platform, test transcript, and residual limitations; keep secrets out of that record.

F1 is complete only when an independent recipient can download within the declared limits. C1 is complete only when an allowed image can be safely prepared, executed, canceled, and reconciled on a real worker. G1 is complete only after actual device allocation and cleanup are proven. “Implemented locally” and “deployed and externally testable” remain separate statuses.

## 9. Implementation map and remaining decisions

| Layer | Work |
|---|---|
| `froglet` | File preparation/publication adapter; transfer admission; job lifecycle extensions; requester ownership; finite usage counters; worker reconciliation; truthful capabilities. |
| `froglet-services` | OCI image preparation and durable worker operations; isolated CPU/GPU execution; relay download headers and optional negotiated streaming; marketplace capability projection. |
| `docs-site` | Resource-specific download/job actions, access entry, live availability, result expiry, and existing QR/preview reuse. |
| Native MCP/CLI | Extend the existing preparation/publication/status/invocation surfaces; add only the missing download/job-status/cancel actions. No separate orchestration product. |

Before implementation, define additive application schemas and endpoint/authentication mappings in the existing publication/OpenAPI/worker contracts. Avoid selecting new dependencies before checking the existing HTTP, SQLite, hashing, lifecycle, and container-engine support. No new queue, identity service, scheduler, or billing database is required for this scope.

The specification chooses provider-approved images, public-registry pulls, private/invited compute, capped F1 downloads, a single worker, and exclusive G1 devices. Hardware selection and finite production budgets remain operator decisions before deployment. Do not block F1 on GPU hardware or expand C1 into unrestricted multi-tenant rental. A later request for arbitrary customer images, large streaming transfers, private registries, or broader GPU stacks must explicitly expand scope and pass its own qualification.

## 10. Implementation checkpoint — 2026-09-28

This checkpoint deliberately separates software checks from release qualification.
Payments, production infrastructure and existing production allowances are unchanged.

Preserved companion checkpoints on `arma/paused-development-checkpoint`:
`ca1c4d1` (relay file transport) and `62d2345` (worker recovery/quarantine).
Use the latter to include both changes. They remain local, unreleased commits.

| Scope | Current local implementation | Remaining release work |
|---|---|---|
| F1 files | Canonical application package; immutable snapshot preparation/publication; signed metadata; GET/HEAD and exact relay grants; persisted global and revision transfer ceilings; invitation, pause, expiry and operator abort; bounded storage; browser checksum/save action; CLI/native MCP verified download. | Production rollout not performed. Local browser save, independent transfer checks and regression gates are recorded below. |
| C1 safety foundations | Node restart fails uncertain external jobs or holds their deals without redispatch. Worker crash markers and an exclusive journal lease prevent reuse after uncertain cleanup; cleanup runs after all outcomes and client disconnects. Podman has conmon timeout flags and no automatic container restart. | Controlled pulls with enforced expanded-image storage quota, batch adapter, durable operation status/cancel/reconciliation, private retained results, finite aggregate compute/pull budgets, and real Linux failure/isolation qualification. These remain unfinished, not implied by the existing JSON worker. |
| G1 GPU | Existing refusal remains enabled; inventory labels cannot enable execution. | C1 completion plus actual NVIDIA hardware discovery, exclusive allocation, accounting, deadline, cleanup/reset and cross-job isolation tests. |
| F2 streaming | Not included. | Separate future work; F1 remains buffered and capped at 8 MiB. |

Observed transfer evidence: a synthetic four-byte HTML fixture and an 8,388,608-byte
fixture passed through the real local companion relay and the node tunnel. An
independent Python recipient checked length and SHA-256; the 8 MiB digest was
`7014ae0f2fc0fee42a440b97859207efb72ffee09d4864f7433f1bf756a17aca`.
No production traffic or user file was used. The browser consumed the real relay
response, verified its size/hash, and showed **Save verified file**. Desktop and
390-pixel mobile layouts, expiry and offline disabling were inspected live.
The browser automation timed out while waiting for its download event, but the
saved fixture was subsequently read from Downloads: four bytes, SHA-256
`b0ce1a82db7de32dcb040d8b810b05752736534cac3341ce0ee526480b0ed5c3`,
matching the approved source. Browser disk saving is therefore verified. JavaScript regression tests cover no
preview transfer, checksum mismatch, short/oversized bodies, invitation clearing
and refusal of non-provider destinations.

Worker unit/integration tests use fake engines and cannot establish container or
GPU isolation. The local host is macOS; its reachable Docker engine reports Linux
and cgroup v2 but does not report rootless isolation. Podman is installed but its
engine is unreachable. No qualifying Linux/rootless worker or NVIDIA device was
provided for C1/G1 acceptance. Do not enable or advertise the unfinished stages.

Operator instructions for F1 are in [configuration](CONFIGURATION.md#download-only-file-publications).
The companion worker README explains its persisted quarantine and current manual
reconciliation boundary. Kernel artifacts, signing bytes and conformance vectors
are unchanged.

### Completed local verification

- `./scripts/strict_checks.sh`: passed, including 936 Rust workspace tests
  (8 explicit opt-in/fixture tests ignored), Clippy with warnings denied,
  native/wasm verifier builds, the unchanged Kernel fixture check, secret scan,
  OpenClaw/MCP/shared client tests, Python runtime/setup tests and conformance.
- Full Python discovery: 172 tests, passed with 3 opt-in skips, including the
  five-minute soak and stress cases.
- Site: 199 tests passed; static production build passed. The final metadata
  validation tightening also passed the 21 focused file/service-link tests.
- Companion relay and OCI worker: 34 tests passed, with the actual-container
  test explicitly ignored; warnings-denied Clippy passed. Relay integration
  covers 8 MiB GET, HEAD with a declared file length and no body, download/CORS
  headers, and rejection of a mismatched content length.
- Real local relay/node transfer and browser checks are described above. Finite
  local test ledgers were preserved when exhausted. No production allowance
  was reset or increased. Temporary task servers were stopped.

The existing website redesign was preserved. One stale MCP setup test was
updated to follow the homepage link to `/open-source/` and its `OperatorSetup`
component. The built route was inspected live and its generated setup command
was visible; no installation command was executed.

### Exact change inventory

Froglet files changed for this implementation (the pre-existing redesign is excluded):

- [docs-site/public/service-share.js](../docs-site/public/service-share.js)
- [docs-site/src/data/file-download.ts](../docs-site/src/data/file-download.ts)
- [docs-site/src/data/service-link-page.ts](../docs-site/src/data/service-link-page.ts)
- [docs-site/src/data/service-link.ts](../docs-site/src/data/service-link.ts)
- [docs-site/src/scripts/__tests__/file-download.test.ts](../docs-site/src/scripts/__tests__/file-download.test.ts)
- [docs-site/src/scripts/__tests__/service-link.test.ts](../docs-site/src/scripts/__tests__/service-link.test.ts)
- [docs/CONFIGURATION.md](../docs/CONFIGURATION.md)
- [docs/FILE_AND_COMPUTE_SCOPE.md](../docs/FILE_AND_COMPUTE_SCOPE.md)
- [docs/PUBLICATION_CONTRACT.md](../docs/PUBLICATION_CONTRACT.md)
- [docs/RELAY.md](../docs/RELAY.md)
- [docs/openapi.yaml](../docs/openapi.yaml)
- [froglet-protocol/src/file_download.rs](../froglet-protocol/src/file_download.rs)
- [froglet-protocol/src/lib.rs](../froglet-protocol/src/lib.rs)
- [froglet-protocol/src/manifest.rs](../froglet-protocol/src/manifest.rs)
- [froglet-protocol/src/publication.rs](../froglet-protocol/src/publication.rs)
- [froglet-publish-engine/src/builder.rs](../froglet-publish-engine/src/builder.rs)
- [froglet-publish-engine/src/lib.rs](../froglet-publish-engine/src/lib.rs)
- [integrations/mcp/froglet/test/server.test.mjs](../integrations/mcp/froglet/test/server.test.mjs)
- [src/api/http_catalog.rs](../src/api/http_catalog.rs)
- [src/api/http_files.rs](../src/api/http_files.rs)
- [src/api/http_files_tests.rs](../src/api/http_files_tests.rs)
- [src/api/mod.rs](../src/api/mod.rs)
- [src/bin/froglet-node.rs](../src/bin/froglet-node.rs)
- [src/builtin_worker.rs](../src/builtin_worker.rs)
- [src/builtins/safe_fetch.rs](../src/builtins/safe_fetch.rs)
- [src/cli/download.rs](../src/cli/download.rs)
- [src/cli/mcp.rs](../src/cli/mcp.rs)
- [src/cli/mod.rs](../src/cli/mod.rs)
- [src/cli/prepare.rs](../src/cli/prepare.rs)
- [src/cli/safeguards.rs](../src/cli/safeguards.rs)
- [src/cli/service_link.rs](../src/cli/service_link.rs)
- [src/config.rs](../src/config.rs)
- [src/db.rs](../src/db.rs)
- [src/file_download.rs](../src/file_download.rs)
- [src/lib.rs](../src/lib.rs)
- [src/provider_policy.rs](../src/provider_policy.rs)
- [src/relay_tunnel.rs](../src/relay_tunnel.rs)

Companion `froglet-services` files:

- `services/oci-worker/README.md`
- `services/oci-worker/src/config.rs`
- `services/oci-worker/src/lib.rs`
- `services/oci-worker/src/runner.rs`
- `services/oci-worker/src/journal.rs`
- `services/oci-worker/src/tests.rs`
- `services/relay/src/lib.rs`
- `services/relay/tests/relay_e2e.rs`
