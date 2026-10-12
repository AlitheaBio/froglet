# Froglet Name Coherence Check

Date: 2026-05-06 (repository location updated 2026-10-12)

This is a lightweight launch-risk check for using `Froglet` as an open source
protocol/project name. It is not legal advice, not attorney trademark
clearance, and not a guarantee that future registry state will remain the same.

## Summary

No obvious software or infrastructure trademark collision was found in this
pass. The visible registry collisions are operational, not launch-blocking:
`froglet` is already taken on PyPI, the `froglet` GitHub user is taken, and the
`froglet` Docker Hub user is taken. The canonical repository moved from
`github.com/armanas/froglet` to `github.com/AlitheaBio/froglet` on 2026-10-12;
GitHub redirects the old URLs. Images from releases published before the move
stay under `ghcr.io/armanas/*`, and later releases publish under
`ghcr.io/alitheabio/*`. The public npm package remains `froglet-mcp`.

The direct `froglet` name should not be used for a Python distribution. Use
`froglet-protocol` or another explicit package name if a public Python package
is published later.

## Registry State

| Surface | 2026-05-06 observation | MVP decision |
|---|---|---|
| GitHub repository | `armanas/froglet` exists and is the canonical public repo. The `froglet` GitHub user exists; `froglet` organization lookup returned 404. | Canonical URLs are under `github.com/AlitheaBio/froglet` since 2026-10-12. Never create a new `armanas/froglet` repository, because that would replace GitHub's redirect. Do not claim a `froglet` GitHub org/user namespace. |
| GHCR | Project docs and release surfaces use `ghcr.io/armanas/*`. | Releases before 2026-10-12 keep their images under `ghcr.io/armanas/*`; later releases publish under `ghcr.io/alitheabio/*`. Docker Hub is not required for MVP. |
| Docker Hub | `froglet` user exists. `armanas` user exists, but the checked `armanas` repository listing returned no public repos. | Do not claim Docker Hub `froglet`; keep GHCR as the supported container registry. |
| npm | `froglet-mcp` exists with latest `0.1.5`. `froglet` returned 404. `@froglet/mcp-server` returned 404 and the local package is marked private. | Public install path remains `npx froglet-mcp`. Do not imply public ownership of `@froglet/*` until that scope is actually secured and published. |
| PyPI | `froglet` exists as an unrelated package: summary `Client for Frog server`, latest `0.3`, homepage `github.com/hslatman/froglet`. `froglet-protocol` and `froglet_protocol` returned 404. | Do not publish as `froglet` on PyPI. Reserve or publish `froglet-protocol` if/when Python packaging becomes public. |
| crates.io | `froglet` and `froglet-protocol` returned 404 through the crates.io API. | Rust crate names appear available as of this check; reserve before relying on them in public instructions. |
| RubyGems | `froglet` and `froglet-protocol` returned 404 through the RubyGems API. | Not part of MVP; no action. |
| Packagist | Search for `froglet` returned one unrelated-looking result, `froget/dev-quotes`, not an exact `froglet/*` protocol package. | Not part of MVP; no action. |

## Trademark And Web Sweep

This pass checked public web results, USPTO-facing search paths, and a
USPTO-data mirror. The public mirror returned no exact `FROGLET` mark in a
software/infrastructure class. The closest visible hits were outside the
project's field:

| Mark | Serial | Classes / goods | Relevance |
|---|---:|---|---|
| `FROGLET INVITATIONAL` | `75370248` | Golf tournaments, golf merchandise, and golf clothing; classes 025, 028, 041. | Contains `FROGLET`, but not software or infrastructure. |
| `FROGLETZ` | `75420589` | Children's clothing and infantwear; class 025. | Similar spelling, but not software or infrastructure. |

The official USPTO TSDR status API returned 401 without an API key, so this
note should not be treated as a full official USPTO clearance report. It is
enough for the current TODO item's stated "avoid a flagrant conflict" goal if
the operator accepts the remaining non-legal risk.

## Launch Decisions

- Keep `Froglet` as the open source protocol/project name for the MVP.
- Keep the repository under `AlitheaBio/froglet`; publish new images under
  `ghcr.io/alitheabio/*` and leave the earlier `ghcr.io/armanas/*` images in place.
- Keep the public MCP package as `froglet-mcp`.
- Do not publish a Python package named `froglet`.
- Do not claim the `froglet` GitHub user, Docker Hub user, PyPI name, or public
  `@froglet/*` npm scope unless those namespaces are secured later.

## Residual Risk

- This is not legal advice and does not replace attorney trademark clearance.
- Registry state can change after 2026-05-06. Reserve desired package names
  before adding public install instructions that depend on them.
- A formal software-class USPTO search or trademark filing remains a human
  business decision if Froglet becomes more than an open source protocol name.

## Source Links

- npm registry: <https://registry.npmjs.org/froglet-mcp>
- npm registry 404 check: <https://registry.npmjs.org/froglet>
- PyPI `froglet`: <https://pypi.org/project/froglet/>
- crates.io `froglet`: <https://crates.io/crates/froglet>
- crates.io `froglet-protocol`: <https://crates.io/crates/froglet-protocol>
- GitHub `AlitheaBio/froglet`: <https://github.com/AlitheaBio/froglet>
- GitHub `froglet` user: <https://github.com/froglet>
- Docker Hub `froglet` user: <https://hub.docker.com/u/froglet>
- Docker Hub `armanas` user: <https://hub.docker.com/u/armanas>
- RubyGems `froglet`: <https://rubygems.org/gems/froglet>
- Packagist search: <https://packagist.org/search/?q=froglet>
- USPTO trademark search: <https://tmsearch.uspto.gov/search/search-information>
- USPTO TSDR status guidance: <https://www.uspto.gov/trademarks/apply/check-status-view-documents>
- USPTO TSDR API catalog: <https://developer.uspto.gov/api-catalog/tsdr-data-api>
- USPTO-data mirror search used for the visible trademark hit list:
  <https://api.markbase.co/search?q=froglet>
