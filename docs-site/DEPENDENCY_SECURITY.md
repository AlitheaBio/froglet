# Website dependency applicability review — 2026-10-04

The reviewed dependency is now `http-cache-semantics` **4.3.0**, selected for its
Vary-wildcard correction. The installed source matches the published package
at [upstream commit b1d4bd6](https://github.com/kornelski/http-cache-semantics/commit/b1d4bd682fbab0252985de45219f4e7497c0067c).
The current raw npm audit reports zero findings because the reported range for
[GHSA-ch52-4w7c-c8xp](https://github.com/advisories/GHSA-ch52-4w7c-c8xp)
ends at 4.2.0. That metadata does **not** demonstrate remediation of the
previously observed max-stale/shared-cookie behavior: the same synthetic
comparison still reproduces it on the published 4.3.0 package.

The maintainer disputes the advisory's interpretation and recommends explicit
`Cache-Control: private` for private data. [Proposal 58](https://github.com/kornelski/http-cache-semantics/pull/58)
and [cumulative proposal 60](https://github.com/kornelski/http-cache-semantics/pull/60)
were closed without merging on October 4. Froglet does not describe 4.3.0 as a
patch for that reported issue, and this application review does not resolve the
upstream standards dispute.

The October 3 review used 4.2.0, when the raw audit reported that advisory and
its known Astro dependency effects. The previous guard correctly refused the
newly published 4.3.0 before this review. This update retains a temporary
**application applicability review** of the observed behavior. All other
high/critical findings, audit/registry errors, or failed applicability checks
block the job. The review expires at **2026-11-16 00:00 UTC** (extended on
9 October from 17 October; see below), or earlier when a
stable version newer than the reviewed 4.3.0 is published. A clean raw audit
still requires the version, source, configuration, registry, and actual Worker
bundle checks below.

## Reviewed boundaries and evidence

- The exact lockfile consumer is `astro` **7.3.5**, requiring
  `http-cache-semantics` `^4.2.0`. The guard requires the installed and locked
  versions to match, and rejects additional normal, optional, peer, development,
  or aliased consumers.
- Astro's reviewed
  [`assets/build/remote.ts`](https://github.com/withastro/astro/blob/astro%407.3.5/packages/astro/src/assets/build/remote.ts)
  uses `storable()` and `timeToLive()` to calculate build-image expiry. It
  creates its own empty/conditional request headers and does not invoke
  `evaluateRequest()` or `satisfiesWithoutRevalidation()`. The installed
  compiled file's exact SHA-256 is guarded. This does not certify all remote
  image caching behavior; that caller also owns expiry/error fallbacks.
- Project production sources have no `astro:assets`, `astro/assets`, or
  `http-cache-semantics` imports. The guard scans executable source variants
  outside tests, rejects source symlinks and production references into test
  modules, decodes module-string escapes and line continuations, and
  conservatively rejects even prose/comment references until reviewed.
  Existing test-evidence links in `maturity.ts` are admitted only at the exact
  reviewed file hash; this does not allow importing a module from tests.
- The reviewed Astro configuration uses its default static output with no
  SSR adapter. `wrangler.jsonc` uploads that static `dist` and the separate
  `src/worker.ts` entrypoint. Both configuration files are pinned by hash.
- A fresh Wrangler **4.143.1** production dry-run emitted a Worker containing
  `qrcode-generator`, Froglet's service-page/data code, and generated verifier
  glue. Its source map matched all 11 current source files exactly. Neither
  Astro nor this cache library nor the observed reuse methods appear in
  the deployment JavaScript. All source hashes, the source set, and JavaScript
  bundle bytes are checked again on every guard run. Only the content
  hash in the generated verifier Wasm filename is normalized; JavaScript
  changes remain blocking. The separate Wasm asset must exist.
- The local preview emulator contains a bundled older cache-policy copy,
  but its cache caller computes TTL, discards request Cache-Control, and
  rejects stored Set-Cookie responses. It does not call the observed
  reuse method. Miniflare is not uploaded as part of this custom Worker;
  this review makes no claim about Cloudflare's internal cache implementation.
- A synthetic, network-free comparison of the exact published 4.2.0 and
  4.3.0 package sources reproduced the same cookie behavior: `storable()` was
  true, `maxAge()` was zero, ordinary reuse was rejected, and a request with
  `max-stale=99999` accepted reuse while retaining a fixture-only Set-Cookie
  header. A `Cache-Control: private` control was not storable. The 4.3.0
  source SHA-256 was
  `ede1cc404a492fa348eb9d97a3007a0d72aa717bd22cd86a56bd0824c19729ca`.
  Separate Vary-wildcard controls with `Vary: * ` and `Vary: accept, *` changed
  from accepting reuse in 4.2.0 to rejecting it in 4.3.0. This verifies the
  selected Vary correction; it does **not** establish a patch for the cookie
  behavior or settle the disputed advisory. No real credentials or requests
  were used.

On October 5, the shared-service renderer's availability wording was reviewed
independently and its source/Worker fingerprints refreshed. It displays
“Availability not confirmed” while preserving the existing machine status,
lease checks and registration behavior. The fresh production dry-run source
map matched the same 11 files; all other Worker sources, configurations,
dependency versions and the reviewed Astro caller were unchanged. The Worker
contained no affected cache-policy code. The renderer SHA-256 is
`3fda670ed4ec7f384cad3b543743793bffc57c598402d7b9ac87d308124b2c2b`;
the normalized Worker SHA-256 is
`dcef1268fbdb11a85e474b37cd4eca78ddb9e667a24b731bbf3f089dcb2f7834`.
This copy change does not remediate the dependency behavior or extend the
October 17 expiry. All rejection and mandatory audit/registry/bundle controls
remain in place.

Run the full audit with the reviewed guard after the website build:

```sh
npm run build --prefix docs-site
node docs-site/scripts/audit-dependencies.mjs
```

The helper fetches the full npm audit, checks the official npm version list,
and runs `wrangler deploy --dry-run` locally to inspect its actual output.
These checks run **even when the raw audit contains no findings**. It never
publishes or changes dependencies. Its output separates the raw high/critical
counts from the application applicability review of the observed behavior.
A subprocess regression runs the actual guard with clean synthetic npm metadata
and checks its real Worker fingerprint; another rejects a future stable version
under the same clean-audit condition. The CLI resolves its actual script and
project paths before running; regressions cover symbolic-link invocation both
with and without `--preserve-symlinks-main`, and importing it from Node stdin.

Changing versions, configuration, reviewed Worker code/dependencies, bundle
output, or adding asset/cache imports invalidates this review. Reassess
before introducing SSR, private remote image data, an incoming-header cache,
or another affected consumer. The CI helper deliberately fails closed instead
of automatically broadening the review. The historical advisory allowlist is
unchanged: changed advisory identity, range, severity, or dependency effects
remain blocking; a clean raw audit cannot disable the separate application
review.


### 6 October 2026 public-demo transport review

The static website now imports a fixed-provider public-demo transport into the
custom Worker. The new route accepts only free pure-Wasm jobs and the selected
synthetic catalog, caps request and response bytes, refuses redirects and other
runtimes, and uses the edge rate-limit binding. It forwards no caller credentials
or cookies and sets every API response to `no-store`. The browser signing key
stays in the browser; operator/runtime routes are not exposed.

Eight route-boundary tests, twelve ingress/accounting fixtures, six real-node
client tests, and five actual released-Linux HTTPS canaries were run. The actual
Wrangler source map matches all thirteen imported sources. No additional package
or affected cache-policy code entered the Worker. The configuration, changed
Worker source, two new sources, and complete normalized bundle are rebound to
their reviewed bytes. Every existing rejection and registry check remains.

This is an application-source/configuration review, not remediation of the
disputed library behavior. The existing expiry of 17 October 2026 remains.

The guarded publication found a newly reviewed high-severity source-map-js
advisory (GHSA-68fv-2mgg-jv7q). The compatible patch was applied from 1.2.1 to 1.2.2.
The lockfile comparison confirmed this was the only package-version change.
It is remediation for that indexed-source-map issue, not for the separately
tracked cache-library behavior above. The guard was retained throughout.

The final-main publication gate subsequently detected GHSA-wq5f-xc86-pv6w in
Sharp's prebuilt librsvg dependency, indexed on 6 October 2026. Sharp 0.35.5
ships patched librsvg 2.63.2. Both the direct website dependency and Miniflare's
exact transitive Sharp pin are resolved to that patch through a targeted npm
override. Wrangler remains 4.143.1; no advisory exception is added or widened.
The regression loads Sharp from both actual resolution paths, checks the native
library versions, and decodes a controlled SVG through each. This patch does not
remediate the separately tracked cache-library behavior or change its expiry.

### 7 October 2026 bounded public-read adapter review

The custom Worker now has fixed-provider GET adapters for public profile and
inventory reads. The reviewed route path validates the provider, request shape,
response DTO and body size, refuses redirects, forwards no caller credentials,
and sets responses to `no-store`. A single one-second deadline bounds admission,
fetch and body handling, including a rate limiter or cancellation that does not
settle. The existing public-demo wire contract is unchanged. The new enable flag
uses the existing 120-requests-per-60-seconds binding; no resource is allocated.
The independent source review and 39 route tests included actual elapsed-time
regressions for stalled admission, fetch and cancellation.

A fresh local Wrangler 4.143.1 production dry-run matched the same thirteen
imported sources, with every source-map `sourcesContent` entry equal to its
actual source file. Only the three reviewed adapter/entrypoint sources and the
enable-flag configuration changed. The complete Worker contained no affected
cache-policy code, and dependency versions, Astro configuration and the reviewed
Astro remote-image caller remained unchanged. The new SHA-256 pins are:

| Reviewed bytes | SHA-256 |
| --- | --- |
| `wrangler.jsonc` | `8fb3b99a0b079cdc2a98f0057c2c2036243d925912abc7ad9da05046dcc14250` |
| `src/data/public-demo-config.ts` | `b9daa65e5561a90e7c94a7ffa72a118b50c9950594297260f69ca8efefe359a3` |
| `src/data/public-demo-proxy.ts` | `0fa95be304dad55de880ddd80c91b366fe865c52babe48cb86e6909431ed215c` |
| `src/worker.ts` | `1cd1a0c41c9fa4f30b8133026345dd5fcec3303f6329366c12a7ee321da48f35` |
| Complete normalized Worker | `55c8a8546819bed2b7f2bd999f7fcaccb136379c91c2b75cb113ae735ed6ea7a` |

The raw dry-run Worker SHA-256 was
`656731246306939d8f7382af21d9aea4014ce4126a65ccbb1f48118f421969fd`.
Normalization still replaces only the single generated verifier Wasm filename's
content hash; it omits no JavaScript. The separate verifier Wasm asset existed
and had SHA-256
`508be94e2582ea708ad73e14bfb3a23b1c060039af5b1dcc72aee0d9a5dd445f`.
This source/configuration requalification does not remediate the known
`http-cache-semantics` 4.3.0 max-stale/shared-cookie behavior. The 17 October
expiry, newer-version refusal, complete graph and byte checks, mandatory
audit/registry/bundle review even with a clean audit, and all existing rejection
controls are unchanged; no advisory exception is added or widened.


### 7 October 2026 published-tool bridge source review

The reviewed Worker adds a disabled-by-default policy/transport bridge for
three fixed C7 publications: `marketplace-provider`, `marketplace-search`,
and `marketplace-receipts`. The shared public-profile parser requires the
complete three-service set and nine exact public fields per profile; it rejects
extra fields, mismatched provider identities, invalid digests and an authoring
filename in place of the runtime `run` entrypoint. Requests remain bound to
the fixed provider origin, installed Wasm bytes, one approved HTTP-operation
capability, free terms and the declared resource limits. Native admission
remains authoritative for signatures and actual workload commitments; the
browser independently verifies its received evidence with the existing Rust
verifier. No cryptographic dependency was added to the Worker.

A single absolute 15-second transport deadline now covers admission, request
body, fetch and response body for both original and named demo routes. Late
admission cannot initiate forwarding, and cancellation acknowledgement cannot
extend the response deadline. Native-shaped public metadata is restricted to
pinned interfaces, the declared read schemas, typed public build/local
verification evidence and bounded public starters. Mocked private-field
reproductions demonstrated policy gaps before these checks; they were not
observations of private data returned by a real native node.

The reviewed configuration is still the exact prior `wrangler.jsonc`: the
published-service enable flag and public-profile JSON are absent, so the three
named tools remain disabled. Later production activation requires actual
verified publication pins in
`FROGLET_PUBLIC_DEMO_PUBLISHED_SERVICES_JSON`, the explicit
`FROGLET_PUBLIC_DEMO_PUBLISHED_SERVICES_ENABLED` flag, and a fresh review of
the exact configuration and emitted Worker before deployment. The strict
Worker parser does not waive the guard's configuration hash: any configuration
change remains blocking until requalified. No placeholder pins are admitted to
production by this source review.

An isolated physical copy of the complete current source and dependency
installation emitted a fresh Wrangler 4.143.1 production dry-run with the same
thirteen sources. Every source-map entry matched its actual file, the
complete JavaScript contained no affected cache-policy code, and the separate
verifier Wasm asset was unchanged. The minimal guard delta updates only these
two source hashes and the complete normalized Worker hash:

| Reviewed bytes | SHA-256 |
| --- | --- |
| `src/data/public-demo-config.ts` | `654da53b91039d812c1587c83e1e79a7c4c57c9bf6e1cd20ad79443ce5ace082` |
| `src/data/public-demo-proxy.ts` | `31b58b9ea1beab8bad8e0ee196023f87f2b71e83b383ada9563f3d17d490dc7c` |
| Complete normalized Worker | `404affed5f2a42211d483a2b3830e8b64c760a5cbcb72f12a7d4379115db2d30` |

The raw Worker SHA-256 is
`12adeda965afb00e75526fefc8870d6d9bd0fd2a5670633002a5d4044cd3463f`.
Normalization still changes only the one generated verifier Wasm filename; no
JavaScript is omitted. Dependency versions, consumer graph, static-output and
configuration pins, advisory identity checks and expiry are unchanged. This
is source/configuration qualification, not native publication, live canary,
public execution or cloud-capacity qualification. The observed
`http-cache-semantics` 4.3.0 behavior remains unremediated, and the application
review still expires at **2026-10-17 00:00 UTC**.

The exact isolated source passed **931 website tests** with **29 native/remote
opt-in tests skipped**, including all **72 deployment-guard tests**. The static
build produced 39 pages using the twelve separately hash-bound generated
inputs; no Rust rebuild or native execution was performed in this qualification.
An actual mandatory guard run against the official npm registry also passed.
The raw audit reported zero high/critical findings, and the source, registry
and fresh bundle gates still ran. All source files remained unchanged across
validation except the three explicit guard fingerprints and this documentation
update. These checks add no advisory allowance or publication claim.

### 9 October 2026 artifact-read route and expiry extension

Commit `505731e` lets the public demo Worker proxy exact
`GET /v1/artifacts/<64-hex hash>` reads and rejects a provider reply whose
`hash` differs from the requested one. A fresh Wrangler 4.143.1 dry-run built
with the previous `public-demo-proxy.ts` reproduced the reviewed normalized
Worker hash `404affed…` exactly; the current source differs from it by only
those three lines of the artifact-read route, and the complete JavaScript
still contains no affected cache-policy code. The guard delta updates only
these two fingerprints:

| Reviewed bytes | SHA-256 |
| --- | --- |
| `src/data/public-demo-proxy.ts` | `615669bbded5ba359cd1743112390b0f0300bd84e9d41280765552bc1b05e7e4` |
| Complete normalized Worker | `0c528697ee570a2802b16b25b69f287ba20faf2577e95276047a94cb467e7990` |

The application review was extended, with owner approval, from
**2026-10-17** to **2026-11-16 00:00 UTC**. No stable `http-cache-semantics`
release newer than 4.3.0 exists on the npm registry as of this review, and
the observed max-stale/shared-cookie behavior remains unremediated. Versions,
consumer graph, configuration pins and advisory identity checks are unchanged.

### 12 October 2026 repository-link configuration requalification

The repository moved from `armanas/froglet` to `AlitheaBio/froglet`. The only
change to `astro.config.mjs` is the Starlight GitHub social link `href`; the
default static output, the absence of an `output` or `adapter` key, and every
other line are byte-identical to the reviewed configuration. No Worker source,
dependency version, `wrangler.jsonc` or remote-image caller changed, so the
complete normalized Worker pin is unchanged. The guard delta updates only this
fingerprint:

| Reviewed bytes | SHA-256 |
| --- | --- |
| `astro.config.mjs` | `40f6e2207d254b268bfb6f2c742e2f472e626bfd0379e5ced06505f63129f791` |

The **2026-11-16 00:00 UTC** expiry and all rejection controls are unchanged.
