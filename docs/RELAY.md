# Relay Ingress (v1 contract)

Status: implemented non-Kernel transport contract. The **node-side tunnel
client** lives in this repo (`src/relay_tunnel.rs`, configured via
`FROGLET_RELAY_URL` / `FROGLET_RELAY_PUBLIC_SUFFIX`, tested in
`tests/relay_tunnel.rs`). The matching relay service and cross-side tests live
in the
sibling services workspace (private `froglet-services` repository, not open source).
This document is the v1 interface both sides implement.

Relay ingress gives a froglet provider a public HTTPS address without DNS
setup, TLS certificates, port forwarding, or NAT traversal: the node dials
**out** to the relay and holds a persistent tunnel; the relay terminates TLS
for `https://<label>.relay.froglet.dev` and forwards requests down the tunnel.
This is the enterprise-friendly reachability path (outbound-only, works behind
corporate firewalls) and complements — not replaces — clearnet self-hosting
and Tor hidden services.

**Layering:** relay ingress is a transport adapter. It does not touch the
kernel: signed artifacts simply advertise the relay URL as their
`provider_url`, exactly as clearnet or onion URLs are advertised today.
`provider_resolution` already classifies `https://*.relay.froglet.dev` as
public clearnet HTTPS; no consumer-side changes are required.

## Reachability lease semantics

The v1 Reachability Lease is the authenticated live tunnel itself. It is not a
separate stored JSON record and has no lease identifier, expiry field, or
renewal endpoint. A successful `ready` frame activates the identity-derived
route while the WebSocket session remains live. A newer authenticated session
for the same provider identity replaces the previous one. Disconnect or
liveness eviction removes the route, after which the provider hostname returns
`503 {"error":"provider_offline"}` until the node reconnects.

The versioned surfaces are the `froglet-relay-auth/v1` signature domain,
`frame.v1` capability, and `x-froglet-relay: v1` forwarding header. This lease
is operational reachability state, not a signed Froglet artifact. See
[`PUBLICATION_CONTRACT.md`](PUBLICATION_CONTRACT.md#relay-reachability-lease-v1)
for its relationship to publication evidence.

## 1. Addressing

Each provider gets a stable hostname derived from its identity key:

```
label     = lowercase RFC 4648 base32, no padding, of the 32-byte provider pubkey
hostname  = <label>.relay.froglet.dev          (label is 52 chars, fits the 63-char DNS limit)
public_url = https://<label>.relay.froglet.dev
```

The label is deterministic and reversible: anyone can recompute the expected
hostname from a `provider_id` and vice versa, so a relay URL self-certifies
which identity it claims to front (the tunnel auth in § 3 proves the claim).

## 2. Transport

- v1 tunnel transport is **WebSocket over TLS (WSS)** to
  `wss://relay.froglet.dev/v1/tunnel`. WSS traverses corporate proxies and is
  trivially implementable on both sides; QUIC/HTTP-3 is an explicit future
  optimization, negotiated via the `capabilities` field (§ 3), not a v1
  requirement.
- One tunnel connection per provider. Reconnect with exponential backoff plus
  jitter (initial 1s, cap 60s). The relay treats a new authenticated tunnel
  for the same identity as a replacement and closes the old one.
- Heartbeat: WebSocket ping/pong every 30s from the relay; three missed pongs
  evict the tunnel and the hostname stops resolving to a backend (returns 503
  with a JSON body `{"error": "provider_offline"}`).

## 3. Authentication (challenge–response, provider identity key)

```
client → relay   {"type": "hello", "provider_id": "<64-hex pubkey>",
                  "capabilities": ["frame.v1"]}
relay  → client  {"type": "challenge", "challenge": "<32-byte hex nonce>"}
client → relay   {"type": "auth", "signature": "<hex Schnorr signature>"}
relay  → client  {"type": "ready", "public_url": "https://<label>.relay.froglet.dev",
                  "heartbeat_secs": 30, "max_body_bytes": 10485760}
```

The signature is over the domain-separated message
`"froglet-relay-auth/v1" || challenge_bytes || pubkey_bytes` using the node's
identity key (the same key and Schnorr scheme as `froglet-node sign-message`).
Challenges are single-use and expire after 60s. Registration is permissionless
— possession of the key is the only requirement — with relay-side quotas
(§ 6).

## 4. Request framing (`frame.v1`)

HTTP requests arriving at the public hostname are forwarded as JSON text
frames; responses return the same `id`:

```
relay  → client  {"id": "<opaque>", "type": "request", "method": "GET",
                  "path": "/v1/feed", "query": "limit=10",
                  "headers": {"accept": "application/json"},
                  "body_b64": ""}
client → relay   {"id": "<opaque>", "type": "response", "status": 200,
                  "headers": {"content-type": "application/json"},
                  "body_b64": "<base64>"}
```

- Bodies are capped at `max_body_bytes` (10 MiB in relay v1); oversized
  requests get a relay-generated 413. This is a transport ceiling, not an
  endpoint allowance: the node's default API body limit is lower and special
  endpoints may enforce their own limits, so forwarded requests can still
  receive a node-generated 413 below 10 MiB.
- Header forwarding is allowlist-based in both directions (content-type,
  accept, authorization, content-length, plus `x-froglet-*`). The relay adds
  `x-froglet-relay: v1`, without forwarding client IP headers or cookies.
- Per-request timeout: 60s from frame dispatch to response frame; the relay
  answers 504 on expiry and discards late responses by `id`.
- Streaming responses and WebSocket pass-through are out of scope for
  `frame.v1`; they are future capability strings.

## 5. Node-side behavior (this repo)

- Config (naming follows existing `FROGLET_TOR_*` patterns):
  - `FROGLET_RELAY_URL` — relay endpoint, e.g. `wss://relay.froglet.dev/v1/tunnel`;
    configure it together with the suffix. This exact control endpoint is
    included in consent and in the durable grant.
  - `FROGLET_RELAY_PUBLIC_SUFFIX` — DNS-only public suffix, e.g.
    `relay.froglet.dev`, used for deterministic endpoint planning.
- Froglet has no hard-coded production relay. Both values are empty until an
  operator or installer supplies a verified pair. Configuration alone remains
  dormant (`status=reserved`): it derives the exact identity-bound HTTPS URL
  locally but opens no WSS connection. Public control endpoints require WSS;
  only literal loopback WS is accepted for local tests.
- The tunnel client forwards decoded requests to the local provider listener
  (same loopback backend the Tor hidden service uses).
- `/v1/node/capabilities` advertises the locally planned HTTPS `url`, the exact
  configured WSS `control_url`, and status. On `ready`, the returned URL must
  equal the planned URL before status becomes `up`.
- On eviction or disconnect, the planned URL remains visible and status moves
  to `down`. The supervisor reconnects only while at least one matching durable
  grant remains. With no grants, status is `reserved` and no socket is open.

### Publication readiness

Selecting relay hosting is not sufficient to activate it. The first
`marketplace_publish` call is read-only and checks
`/v1/node/capabilities`. It returns a consent hash only when all of the
following are true:

1. relay endpoint planning is configured;
2. status is `reserved`, `starting`, `up`, or `down`;
3. the daemon reports its provider identity;
4. the daemon reports an exact credential-free HTTPS origin and exact
   credential-free WSS control endpoint (loopback WS is test-only); and
5. the consent summary binds both endpoints and discloses outbound WSS, TLS
   termination, plaintext visibility, and operator quotas.

After approval, the node first persists and locally verifies the exact
Publication Revision. Only then does the provider-control activation endpoint
compare the service ID, revision hash, activation token, public URL, and WSS
control URL against current lifecycle and configuration, persist the grant,
and allow the supervisor to dial. It waits for a `ready` frame with the exact
approved public URL. A disabled, URL-less, or changed relay fails closed; the
engine does not fabricate an endpoint or reuse approval for another control
server.

The grant is durable non-Kernel authorization scoped to one exact publication
instance. Multiple services may share one tunnel. Pausing one removes only its
grant and relay-visible offers; the tunnel remains while another grant exists.
Pause, unpublish, resume, rollback, identity rotation, and endpoint drift all
invalidate stale grants. Startup reconciles both approved URLs before any
tunnel task can reconnect.

Relay-origin request admission is also grant-scoped. The relay removes any
caller-supplied `x-froglet-relay`, writes the fixed `x-froglet-relay: v1`
marker, and the node holds a read-side admission guard through the complete
response. It exposes only exact granted services, offers, revisions, and
deal-linked artifacts. Lifecycle/grant mutation holds the corresponding write
guard, so a revoked service cannot be newly admitted after the revocation
commit. Direct local and Tor access retain their existing scopes.

Once the exact relay endpoint is live, the publish engine runs an independent
requester-side canary before marketplace registration. Registration and later
marketplace projection remain separate gates from reachability.

## 6. Trust boundary and limits

- **The relay terminates TLS and sees request/response plaintext.** It is
  marketplace-operated infrastructure in the same trust class as the
  marketplace index. Kernel signatures on artifacts, offers, deals, and
  receipts protect *integrity* end-to-end regardless; payload
  *confidentiality* from the relay operator is not provided in v1. Providers
  handling sensitive payloads should use clearnet self-hosting (their own
  TLS) or negotiate end-to-end encryption at the application layer; an
  e2e-encrypted tunnel mode is a candidate v2 capability.
- Relay-side quotas are operator policy (per-identity connection rate,
  request rate, bandwidth); quota errors surface as HTTP 429 with a JSON body
  including a `retry_after_secs` hint.
- The relay never originates requests down the tunnel other than forwarding
  public traffic; the node must still treat every forwarded request as
  untrusted public input.

## 7. Future-version questions

1. Hostname reuse after long offline periods — reserve labels indefinitely
   (they are identity-derived, so yes by default) vs. quota-expire mappings.
2. Whether the relay should verify the fronted node serves a signed
   `/v1/feed` matching the tunnel identity before routing (cheap
   anti-confusion check, adds a startup probe).
3. Multi-relay federation and `relay.<region>.froglet.dev` naming.
4. Frame encoding: JSON is v1 for debuggability; CBOR is a candidate
   `frame.v2` if profiling shows overhead.

### Bounded file downloads

File delivery adds `origin`, `range`, `access-control-request-method` and
`access-control-request-headers` to the request allowlist. Responses additionally
allow `content-disposition`, `content-length`, `etag`, `cache-control`,
`x-content-type-options`, the CORS allow-origin/methods/headers/expose-headers
fields, and `vary`. Redirect locations and cookies remain excluded. The relay
rejects duplicate or inconsistent response lengths; HEAD may declare a file
length but must have no response bytes.

The node replaces the internal `x-froglet-relay-response-limit` value with the
negotiated limit, so a caller cannot raise it. A file larger than that limit is
refused before file allowance reservation. F1 files are at most 8 MiB; relay v1
continues to buffer bounded base64 frames. It does not stream or resume them and
this feature does not raise the configured global relay allowance.
