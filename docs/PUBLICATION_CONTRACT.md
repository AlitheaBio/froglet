# Publication contracts

Status: versioned application-layer contract; non-Kernel.

This document defines the public authoring and provider-control shapes used to
turn a service definition into Froglet's existing signed descriptor and offer
artifacts. It does **not** add a Kernel artifact, change Kernel signing bytes,
or alter the verification rules in [`KERNEL.md`](KERNEL.md). The complete HTTP
schema is in [`openapi.yaml`](openapi.yaml).

## Contract versions

| Contract | Current version | Purpose |
|---|---|---|
| Publication intent | `froglet.publication-intent.v1` | Lossless request shared by manifests, agent adapters, the publish engine, and the provider-control API |
| Publication consent | `froglet.publication-consent.v2` | Approval of the exact built package, request, schemas, terms, and preflighted public transport |
| Publication revision | `froglet.publication-revision.v1` | Provider-signed statement binding an existing offer to executable bytes, resolved limits, typed price terms, and successful local verification |
| Service manifest | `froglet-service/v4` | Provider-neutral authoring contract; see [`MANIFEST.md`](MANIFEST.md) |
| Relay reachability lease | Relay ingress v1 (`froglet-relay-auth/v1` + `frame.v1`) | Identity-authenticated live relay session; see [`RELAY.md`](RELAY.md) |
| Download-only file | `froglet.builtin.file_download.v1` | Immutable, bounded file bytes plus signed metadata; enabled only with finite provider file limits |
| Release Bundle | `froglet.release-bundle.v1` | Attested manifest pinning source, binary assets, and role images |

Current authoring adapters emit the publication-intent version explicitly.
The provider-control endpoint still accepts an omitted intent version for
pre-contract direct clients. Unknown fields and unknown explicit versions are
rejected.

## Publication flow and boundaries

```text
froglet-service/v4
        |
        v
PublicationIntent v1 -- provider-private fixture --> local sandbox execution
        |                                              |
        |                                              v
        +------> existing signed descriptor + offer <--+
                            |
                            v
         SignedPublicationRevision v1 (when verified)
                            |
                            v
             external reachability / registration
```

The provider performs local verification and preflights the complete revision
payload before persisting a newly published offer. A failed fixture or invalid
revision therefore returns an error without leaving that offer behind. Local
verification is only one gate: it does not prove public reachability,
marketplace activation, or payment settlement.

For public publication, the agent first performs a read-only plan. Planning
builds the canonical provider-control request but does not POST it, prepare a
hosting backend, or register with a marketplace. Consent v2 discloses and
binds the package digest, build-evidence digest, full canonical request digest,
input/output/data schemas, effective capabilities and limits, payment terms,
and provider-private fixture input hash. Private source, wheel, data, and
fixture bytes are represented by hashes rather than disclosed in the summary.
The approved publish call rebuilds once, requires the resulting consent hash
to match, sends that exact request to the daemon, and requires successful local
verification before any hosting mutation. A relay plan is not approvable until
the daemon reports its provider identity, exact identity-derived HTTPS URL,
and exact configured WSS control endpoint. Planning does not require or open a
live tunnel. Consent binds both endpoints; the activation request must still
equal both approved values after local verification.

The daemon preflight resolves the complete provider definition before that
approval. It validates identifiers, runtime/package/entrypoint combinations,
Wasm bytes and ABI, digest-pinned OCI bindings, locked Python bundle/build
evidence, resolved limits, revision shape, and the deterministic construction
of the private verification workload. Native JSON, CSV, and SQLite snapshots
are decoded and read-only validated through a private, auto-cleaned OS
temporary directory; preflight does not create the operator-visible
`publication-data` directory or write publication database state. The opaque
precondition token also binds the resolved definition digest, so bytes read
from a legacy approved `artifact_path` cannot change between planning and the
provider's approval check. Publish stages the exact in-memory candidate
returned by that check rather than reading the path or native data again.

The same precondition binds the provider's observed identity-backup state and,
when present, the exact encrypted bundle digest. This disclosure contains no
filesystem path, recovery-key location, or operator custody identifier. The
daemon validates the private status and canonical bundle structure during
planning and recomputes it immediately before mutation, so backup replacement,
corruption, loss, or identity rotation invalidates an earlier consent hash.
Backup state is approval-bound operational evidence rather than a Kernel
artifact; a private local proof records it as not required.

Preflight cannot promise outcomes that are inherently live or mutating:
verification workload execution and output, concurrent offer compatibility,
publication-directory and database I/O, signing, and external reachability
remain publish-time gates.

The authoring fixture remains provider-private. It is not copied into public
service records, descriptors, offers, or the artifact feed. The response may
include the result hash and a signed publication revision; it never returns the
fixture itself.

## `PublicationIntent` v1

`POST /v1/provider/artifacts/publish` accepts the canonical JSON shape. The
full field table lives in the OpenAPI schema
`ProviderControlPublishArtifactRequest`; this is an illustrative canonical
example:

```json
{
  "schema_version": "froglet.publication-intent.v1",
  "service_id": "echo-json",
  "runtime": "python",
  "package_kind": "inline_source",
  "entrypoint_kind": "handler",
  "entrypoint": "handler",
  "contract_version": "froglet.python.handler_json.v1",
  "inline_source": "def handler(event, context):\n    return event\n",
  "capabilities": [],
  "mode": "sync",
  "price_sats": 0,
  "settlement_method": "none",
  "price_currency": "sat",
  "limits": {
    "max_input_bytes": 16384,
    "max_runtime_ms": 5000,
    "max_memory_bytes": 16777216,
    "max_output_bytes": 16384,
    "fuel_limit": 0
  },
  "verification": {
    "input": {"message": "hello"},
    "expected_output": {"message": "hello"}
  }
}
```

Key invariants:

- Publication mounts default to read-only. Handles contain 1-64 lowercase
  ASCII letters, digits, or underscores. `kind` is one of `postgres`,
  `sqlite`, `object_store`, or `redis`. Authors must not supply `binding`; the
  provider owns binding material. Handles must be unique within one intent.
  The historical `s3` kind and `mount.s3.*` capability namespace are accepted
  only as authoring/read compatibility inputs and normalize to
  `object_store` before a new revision is signed.
- Capabilities are trimmed, lowercased, sorted, and deduplicated before use.
- Omitted limit fields resolve to provider defaults. A requested limit above
  the provider maximum is rejected. `fuel_limit = 0` means no independent fuel
  ceiling within the runtime limit.
- A fixture uses exactly one of `input` or the JSON-string compatibility form
  `input_json`. It may use at most one of `expected_output` and
  `expected_output_json`.
- Fixture execution rejects writable mounts. Python fixtures additionally
  require the node's full Python sandbox; otherwise publication fails closed.
- A paid request requires an explicit `settlement_method`. `stripe` requires
  `price_currency = "usd"`; `lightning` uses `sat` (or the compatibility
  default when currency is omitted). `none` requires every fee to be zero.
- `price_sats` is the legacy whole-unit success amount. If
  `success_fee_msat` is also present, it must equal `price_sats * 1000`.
  The base and success total must be representable by the current whole-unit
  payment APIs.

## Local verification evidence

When a private fixture succeeds, the response `evidence` includes:

```json
{
  "local_verification": {
    "input_hash": "1f64d68ec3c32015b91a00d4f82cfd30f5eb9659407a4ac81e29f5994e135439",
    "result_hash": "8e9f9d26d45557be4c981a02c8a8f86b12f5a3a4fa5b4bfb2ca652f91f5f0b31",
    "expected_output_matched": true
  }
}
```

`input_hash` binds the canonical private fixture input without returning it.
`expected_output_matched` is omitted when the fixture did not specify an
expected output. A mismatch is an error; a successful revision can never carry
`false`.

## `SignedPublicationRevision` v1

The provider returns a revision only when local verification ran successfully.
The revision payload binds:

- provider, service, and offer identities;
- the existing signed offer hash;
- executable binding and package digests;
- immutable non-Kernel build provenance and dependency evidence when produced
  by a current builder;
- runtime and package kind;
- project, summary, starter, and source metadata;
- entrypoint kind/value, execution contract, and sync/async mode;
- author-declared mounts without private bindings and the effective sorted
  capability set mirrored by the signed offer (including
  `mount.<kind>.<read|write>.<handle>` for each mount);
- input and output schemas;
- fully resolved execution limits;
- authoring-level settlement and currency plus the exact offer settlement
  method; and
- local verification evidence.

Amounts in `price.base_amount_minor` and `price.success_amount_minor` are
satoshis when `currency` is `sat` and US cents when it is `usd`. The exact
Kernel offer method remains separately bound in `offer_settlement_method`.

`revision_hash` is SHA-256 of the canonical JSON payload bytes.
`signer_pubkey` equals `payload.provider_id`. The BIP340 Schnorr signing bytes
are the ASCII domain `froglet-publication-revision-v1\n` followed by those
canonical payload bytes. This signature is intentionally independent from
every Kernel artifact signature domain.

Current signing emits only `object_store`. Verification continues to accept
already-signed v1 revisions that contain the legacy `s3` kind together with
their exact `mount.s3.*` capability. Runtime authorization does not treat the
two capability namespaces as interchangeable: legacy stored workloads need a
legacy grant, while canonical workloads need an `object_store` grant.

### Build evidence v1

Every newly built revision carries `build_evidence` with schema
`froglet.publication-build-evidence.v1`. It binds `source_digest` and
`artifact_digest` (the latter must equal the revision `package_digest`), the
builder identity/version, whether the build was hermetic, and one explicit
dependency state:

- `none`: the package has no separately resolved dependencies;
- `locked`: sorted components include exact role, name, version, and SHA-256;
- `unresolved`: the adapter cannot truthfully prove a dependency lock.

The embedded WAT path records the exact compiler crate version and crates.io
checksum and is hermetic because it invokes no external toolchain. Current
Python inline-source authoring records an exact locked bundle: even the
dependency-free default has a generated lock, while explicit locks contain
hash-pinned local pure-Python wheels. Its components include the lock, declared
CPython compatibility identity, every wheel, and the exact wheel parser crate
version/checksum. Direct provider-control and OCI imports that cannot prove a
dependency lock remain explicitly non-hermetic or `unresolved`. Evidence is
signed as part of the Publication Revision, but it is not inserted into or
allowed to change Kernel artifacts.

### Locked Python bundle v1

`froglet.python-bundle.v1` is provider-private publication material containing
the exact UTF-8 source, `froglet.python-lock.v1`, and any exact wheel bytes.
`artifact_digest` and the publication package digest are SHA-256 of the exact
canonical envelope bytes, not merely the source hash. Public execution is
service-addressed: serialized requests bind that envelope digest as
`source_hash` and do not carry source or bundle bytes.

The builder is resolver-free and network-free. It accepts only local
`<python-tag>-none-any.whl` artifacts whose filename tag, `WHEEL`
`Root-Is-Purelib` and `Tag`, `METADATA` name/version, and declared SHA-256 all
agree. It rejects native-library members, traversal, duplicate paths,
symlinks, and per-member/per-wheel/global size-limit violations. The runtime
re-verifies the bundle, separately attests exact CPython
implementation/version/ABI compatibility, writes each wheel to an
invocation-private temporary directory, prepends those wheel paths, and
executes only the decoded source in the Python sandbox.

### Isolated OCI worker v1

OCI execution crosses a provider-neutral authenticated HTTP seam. The request
contains a digest-only `repository@sha256:<digest>` image, input, explicit
resource/output/time limits, and only declared-and-granted logical mount,
secret, and network capabilities. It contains no host mount bindings, secret
values, provider-native resource identifiers, Docker/Podman socket, or local
engine command. The default adapter is disabled and fails closed.

The normalized result binds the canonical request hash, image digest, and
granted-capability hash and reports duration and status. The node caps the HTTP
body before parsing, rejects redirects and mismatched evidence, and accepts an
authenticated endpoint over HTTPS (or literal loopback HTTP for a sidecar).
Pure shared wire types and validation live in
[`froglet-protocol/src/oci_worker.rs`](../froglet-protocol/src/oci_worker.rs),
while the node adapter lives in [`src/oci_worker.rs`](../src/oci_worker.rs).
The crate location does not make this a Kernel contract: it remains an
execution adapter and does not alter canonical artifact payloads, hashes,
signatures, or state transitions.

## Publication lifecycle API

A verified publication stores each `froglet.publication-revision.v1` document
immutably and keeps lifecycle state separately. The provider-control API is
Bearer-authenticated and exposes:

| Method and path | Meaning |
|---|---|
| `GET /v1/provider/publications` | List bounded verified publication lifecycles |
| `GET /v1/provider/publications/{service_id}` | Read active/selected revision state |
| `GET /v1/provider/publications/{service_id}/revisions` | List immutable revision summaries |
| `GET /v1/provider/publications/{service_id}/revisions/{revision_hash}` | Load and re-verify one signed revision |
| `GET /v1/provider/publications/{service_id}/logs` | List bounded lifecycle operation records |
| `POST /v1/provider/publications/{service_id}/pause` | Clear active projection but retain selected revision |
| `POST /v1/provider/publications/{service_id}/revisions/{revision_hash}/pause` | Compensate only the exact activation token supplied in the JSON body |
| `POST /v1/provider/publications/{service_id}/resume` | Reactivate the selected paused revision |
| `POST /v1/provider/publications/{service_id}/rollback/{revision_hash}` | Activate an exact prior validated revision |
| `POST /v1/provider/publications/{service_id}/unpublish` | Clear active projection and retain history |

The only lifecycle states are `active`, `paused`, and `unpublished`. Paused and
unpublished records have no `active_revision_hash` but retain
`selected_revision_hash`. Resume is forbidden from `unpublished`; an explicit
rollback or a newly verified publication is required. Repeating an already
satisfied transition is idempotent and does not create a second operation log.
Every lifecycle response includes an opaque 64-hex `activation_token`. A real
publish, resume, or rollback activation creates a fresh token; pause,
unpublish, and idempotent calls preserve it. Exact publish compensation must
supply both the selected revision hash and this token, so a delayed request
returns `409` rather than pausing a later activation of the same revision.
Legacy offers with no verified immutable revision return a `409
legacy_unverified` conflict for mutation rather than acquiring invented
history.

Pausing a relay-backed publication withdraws its exact grant and retains the
previously approved relay endpoint in private lifecycle evidence. Resume
validates that endpoint against current configuration, rotates the activation
token, and reconnects with a new exact grant before reporting success. Changed
relay endpoints require new publication approval. An interrupted reconnect
returns partial-state recovery guidance and can be retried with the same
resume action. Older paused records without recovery evidence require explicit
relay activation or republishing.

`POST /v1/publications/{revision_hash}/canary` is the public, concurrency-bounded
external proof endpoint. It accepts `froglet.publication-canary-request.v1`
with the exact revision, offer, fresh challenge, and private hash-bound fixture
input. It executes only the exact active offer and returns a provider-signed
`froglet.publication-canary-result.v1`. The marketplace-owned activation
canary and the later requester-side canary use separate challenges; neither is
substituted by local verification.

## Native MCP adapter

`froglet-node mcp` is the dependency-minimal stdio adapter installed on the
native lane. It delegates manifest loading, planning, consent, publication,
and invocation to the same Rust modules used by the CLI and daemon; it does not
maintain another request or policy model. Its `froglet` tool supports:

- `status`, `local_proof`, and `invoke_service`;
- two-step `marketplace_publish` (first call plans, second supplies the exact
  `consent_hash`); and
- `publication_status`, `publication_logs`, `publication_pause`,
  `publication_resume`, `publication_rollback`, and
  `publication_unpublish`; and
- `managed_operation_status`, `managed_operation_reconcile`, and
  `managed_operation_compensate` for durable provider-neutral deployment work.

Rollback requires a lowercase 64-hex revision hash. Unpublish requires
`confirm_service_id` to equal the exact target before any HTTP request is sent.
Managed reconciliation and compensation require `confirm_operation_id` to
equal the exact 64-hex operation ID because reconciliation can compensate an
external deployment that cannot be proven healthy.
Lifecycle responses are size-bounded and daemon errors are sanitized. The
JavaScript MCP server and OpenClaw plugin remain broader compatibility
adapters; their publication path must still delegate to the canonical module.

## Public provider service pricing

`ProviderServiceRecord` exposes all of the following together:

| Field | Meaning |
|---|---|
| `price_sats` | Legacy whole-unit total; interpret with `price_currency` |
| `base_fee_msat` | Base-fee leg from the linked signed offer |
| `success_fee_msat` | Success-fee leg from the linked signed offer |
| `settlement_method` | Exact method from the linked signed offer |
| `price_currency` | `sat` or `usd`; optional only for compatibility records |

Consumers should prefer the explicit fee legs and settlement method. For a
verified publication revision, the currency-aware `*_amount_minor` fields are
the unambiguous display and approval values.

## Relay reachability lease v1

The current Reachability Lease is the authenticated live session defined by
[`RELAY.md`](RELAY.md), not a separate JSON resource. It has no `lease_id`,
expiry field, or renewal endpoint.

The node sends a `hello` containing `provider_id` and the `frame.v1`
capability, signs the relay's challenge using the
`froglet-relay-auth/v1` domain, and receives a `ready` frame containing
`public_url`, `heartbeat_secs`, and `max_body_bytes`. While that session is
live, the relay routes the provider's deterministic identity-derived HTTPS
hostname to request/response `frame.v1` messages. A newly authenticated
session for the same identity replaces the prior session; disconnect or
liveness eviction removes the route and public requests return
`provider_offline`.

This lease is a non-Kernel transport fact. The relay sees plaintext after TLS
termination. Existing Froglet artifact signatures continue to provide
integrity, but the live relay session does not create an artifact signature or
payload-confidentiality guarantee. The node implementation is
[`src/relay_tunnel.rs`](../src/relay_tunnel.rs); the matching service contract
is exercised in the
sibling relay implementation (private `froglet-services` repository, not open source)
and its end-to-end tests.

Froglet ships without a hard-coded production relay pair. An operator or
installer may explicitly configure `FROGLET_RELAY_URL` together with
`FROGLET_RELAY_PUBLIC_SUFFIX`; until then relay planning is disabled. Literal
loopback WS remains allowed for local tests; public relay control endpoints
require WSS. Configuration derives a stable identity-bound public URL locally
and leaves status `reserved`; it opens no WSS connection.

Before Froglet can return an approvable relay consent, the publication planner
reads `/v1/node/capabilities` and requires `enabled=true`, a plannable status
(`reserved`, `starting`, `up`, or `down`), a provider identity, the exact
credential-free HTTPS `url`, and the exact credential-free WSS `control_url`.
Disabled or incomplete metadata fails before approval or publication mutation.
Changing either URL changes the consent hash.

After exact approval, the daemon persists and locally verifies the Publication
Revision before relay activation. Activation compare-and-persists a durable
grant over transport, service ID, revision hash, activation token, public URL,
and control URL. Only then may the supervisor dial WSS; it must receive the
same public URL in `ready`. Multiple publications can share the tunnel, while
each pause or unpublish removes only its own scope. Resume, rollback, identity
rotation, or either endpoint changing requires fresh authorization. Startup
deletes stale grants before any reconnect.

Relay-origin catalog, quote, deal, canary, and artifact access is filtered by
that exact grant set. The relay overwrites its fixed origin marker and the node
linearizes each admitted request against revocation, so local-only or revoked
offers are not exposed through the relay. This authorization is provider-local
operational state and does not change Kernel artifacts.

The public sequence is: exact local verification, exact transport activation,
an independent requester-side canary, then marketplace registration. The
registration POST is exact and idempotent; if a response is lost or a success
body is unreadable, the engine replays that identical POST once. If the second
result is still ambiguous, it returns structured `MarketplaceStateUnknown`
evidence rather than claiming failure or success, attempts exact local pause
and relay withdrawal, and reports the marketplace's 60-second candidate
visibility bound. Once registration has returned exact `active`, failure to
observe the eventually consistent marketplace projection is a warning with a
status URL, not a false rollback of the live publication.

The same read-only capabilities preflight binds the daemon's exact 64-hex
`provider_id` into every public Relay, Tor, or self-hosted consent. Tor plans
also require `enabled=true` and bind the exact credential-free
`http://<56-char-v3>.onion` origin advertised as `tor.url` or `tor.onion_url`;
v2, path-bearing, query-bearing, credential-bearing, and non-HTTP endpoints
fail before approval. Relay URLs and self-hosted URLs are likewise root origins,
not path prefixes. Self-hosted plans require a credential-free public HTTPS
origin and reject localhost, `.local`, `.internal`, `.onion`, and local/private
numeric IP literals without performing DNS resolution. Rebuilding a plan after
identity rotation or endpoint change therefore changes the consent hash and
rejects a stale approval before provider-control publication. Tor is
non-default and materially different from relay activation: an advertised Tor
hidden service is already live before approval, so consent explicitly reports
that approval does not open it. Private Local plans remain
identity-preflight-free. Managed deployment currently stops at the explicit
provider-neutral adapter seam; any future preparable managed adapter must bind
the daemon identity before it can return an approvable public plan.

## Release Bundle v1

`release-manifest.json` is a deterministic JSON document with
`schema = "froglet.release-bundle.v1"`. Its exact generator and validator are
[`scripts/release_manifest.py`](../scripts/release_manifest.py), and the
installer verification path is [`scripts/install.sh`](../scripts/install.sh).
Every v1 field is required:

| Field group | Fields |
|---|---|
| Release source | `release`, `source_repository`, `source_revision`, `source_ref`, `attestation_signer_workflow` |
| Agent bootstrap | `agent_bootstrap_asset`, `agent_bootstrap_sha256` |
| Role images | `image_provider`, `image_runtime`, `image_dual`, `image_mcp` |
| Optional-location mirrors | `image_provider_mirrors`, `image_runtime_mirrors`, `image_dual_mirrors`, `image_mcp_mirrors` |
| Linux x86_64 binary | `binary_froglet_node_linux_x86_64_asset`, `binary_froglet_node_linux_x86_64_sha256` |
| Linux arm64 binary | `binary_froglet_node_linux_arm64_asset`, `binary_froglet_node_linux_arm64_sha256` |
| macOS arm64 binary | `binary_froglet_node_darwin_arm64_asset`, `binary_froglet_node_darwin_arm64_sha256` |

All role images are immutable `@sha256:` references. Every mirror must carry
the same digest as its canonical role image, mirror entries must be unique,
and a mirror must not repeat the canonical reference. The bootstrap asset is
exactly `agent-bootstrap.sh`; it and every tag-derived binary asset have their
own lowercase SHA-256.
`release` is a normalized `v`-prefixed tag, `source_revision` is a 40- or
64-character lowercase hex digest, `source_ref` is the matching tag ref, and
`attestation_signer_workflow` is that repository's release workflow. Unknown
or missing fields are rejected.

```json
{
  "schema": "froglet.release-bundle.v1",
  "release": "v0.4.0",
  "source_repository": "armanas/froglet",
  "source_revision": "1111111111111111111111111111111111111111",
  "source_ref": "refs/tags/v0.4.0",
  "attestation_signer_workflow": "armanas/froglet/.github/workflows/release.yml",
  "agent_bootstrap_asset": "agent-bootstrap.sh",
  "agent_bootstrap_sha256": "9999999999999999999999999999999999999999999999999999999999999999",
  "image_provider": "ghcr.io/armanas/froglet-provider@sha256:2222222222222222222222222222222222222222222222222222222222222222",
  "image_provider_mirrors": [],
  "image_runtime": "ghcr.io/armanas/froglet-runtime@sha256:3333333333333333333333333333333333333333333333333333333333333333",
  "image_runtime_mirrors": [],
  "image_dual": "ghcr.io/armanas/froglet-dual@sha256:8888888888888888888888888888888888888888888888888888888888888888",
  "image_dual_mirrors": [],
  "image_mcp": "ghcr.io/armanas/froglet-mcp@sha256:4444444444444444444444444444444444444444444444444444444444444444",
  "image_mcp_mirrors": [],
  "binary_froglet_node_linux_x86_64_asset": "froglet-node-v0.4.0-linux-x86_64.tar.gz",
  "binary_froglet_node_linux_x86_64_sha256": "5555555555555555555555555555555555555555555555555555555555555555",
  "binary_froglet_node_linux_arm64_asset": "froglet-node-v0.4.0-linux-arm64.tar.gz",
  "binary_froglet_node_linux_arm64_sha256": "6666666666666666666666666666666666666666666666666666666666666666",
  "binary_froglet_node_darwin_arm64_asset": "froglet-node-v0.4.0-darwin-arm64.tar.gz",
  "binary_froglet_node_darwin_arm64_sha256": "7777777777777777777777777777777777777777777777777777777777777777"
}
```

The JSON document does not contain an inline signature. The tagged release
workflow publishes separate `release-manifest.intoto.jsonl` and
`agent-bootstrap.intoto.jsonl` GitHub artifact attestations. Those are
separately verifiable provenance evidence; the dependency-free path does not
claim to verify an attestation. It requires the official GitHub release record
to be immutable and verifies both the manifest and bootstrap downloads against
their unique release-asset API `sha256:` digests. The running bootstrap also
requires its own bytes to match the bootstrap digest in the verified manifest.
If `gh` is available (or explicitly required), the installer additionally
verifies the manifest's offline attestation against the trusted repository,
workflow, tag ref, and source revision. An
explicit caller-trusted `FROGLET_RELEASE_MANIFEST_SHA256` remains an alternate
trust pin. The installer then verifies the selected binary asset digest. This
supply-chain contract is non-Kernel and does not change Froglet artifact hashes
or signing bytes.

## Service manifest v4 and hosting portability

New authoring uses `schema_version = "froglet-service/v4"`. A v4 managed
deployment selects an abstract target and an operator-defined profile:

```toml
schema_version = "froglet-service/v4"
service_id = "echo-json"
runtime = "python"
package_kind = "inline_source"
entrypoint = "handler.py"

[hosting]
default = "managed"

[hosting.managed]
target = "regional-container"
profile = "small-public"

[settlement]
method = "none"
```

`target` and `profile` are provider-neutral adapter inputs. They do not encode
AWS, GCP, Azure, Fly, or Kubernetes semantics into the manifest contract. Both
are required and non-empty for a v4 managed manifest. The public publish engine
validates this neutral authoring shape but does not provision infrastructure;
that responsibility belongs to the sibling operator's
Managed Deployment contract (private `froglet-services` repository, not open source)
which defines provider-neutral desired state, adapter capabilities,
mutation-free planning, and normalized results. Its Lightsail compatibility
and generic SSH + OCI adapters implement the same initial lifecycle subset
behind that contract; wiring a managed publication to an operator deployment
and credentialed live canaries remain separate operations gates.

`froglet-service/v3` remains readable for compatibility. A v3
`hosting.default = "fly"` manifest emits a deprecation warning. V4 rejects
`fly`; migrate it to `managed` and place provider-specific selection behind
the configured target/profile adapter. V2 remains a legacy compatibility
shape with missing-section warnings.

## Verification responsibility

| Claim | Evidence required |
|---|---|
| Local execution works | `LocalVerificationEvidence` |
| Offer and executable are immutably bound | Verified `SignedPublicationRevision` |
| Public endpoint is reachable | External probe of the advertised endpoint |
| Marketplace listing is active | Exact offer lookup by `offer_hash` |
| Payment works | Completed rail-specific payment and receipt evidence |

None of these claims implies the next one.

## Download-only file contract

A file publication uses existing `builtin` runtime/package fields and
`data_source.format = "file"`. `content_base64` transports the application
package to the authenticated publication endpoint; it is never a catalog or
file-download response. Native preparation builds this package from one regular
file. The package layout is ASCII `FROGLET-FILE-V1\n`, a four-byte big-endian
metadata length, canonical JCS JSON metadata, then the raw file bytes.
Metadata is capped at 4096 bytes and raw content at 8 MiB. Unknown metadata
fields and noncanonical encodings are rejected.

Metadata contains `filename`, `media_type`, `size_bytes`, `sha256`, `expires_at`,
`max_downloads`, and `max_transfer_bytes`. The filename is a portable ASCII
basename; media type has no parameters. SHA-256 describes the raw bytes, while
the publication's existing module/binding digest covers the complete package.
The service output schema is `{ "type": "object", "const": <metadata> }` and
is included in the provider-signed Publication Revision. This adds no Kernel
artifact fields or signing rules.

The only execution input is `{ "action": "describe" }`; its result is metadata.
Publication verifies this fixture locally. File bytes instead use the separately
admitted `GET /v1/provider/services/{service_id}/files/{revision}/download`
endpoint. A relay grant must match the service and revision in one grant.
The route requires current activation, valid signed metadata, free settlement,
provider access, unexpired metadata, available storage and a transactionally
reserved global plus snapshot transfer allowance. HEAD performs access checks
without file reservation. OPTIONS supports the first-party share-page origin.

Responses are attachments with `application/octet-stream`, `nosniff`, `no-store`,
length, SHA-256 and ETag. Range requests are refused. Pause, unpublish and
invitation revocation stop new admissions. Already admitted responses may finish;
an explicit owner abort interrupts active node streams without refunding usage.
File download is an operational transfer, not a fabricated signed execution
receipt. See [configuration](CONFIGURATION.md#download-only-file-publications)
for limits, preparation and native recipient commands.
