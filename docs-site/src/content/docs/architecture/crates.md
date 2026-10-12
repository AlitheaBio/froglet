---
title: Crate Structure
description: The workspace, its crates, and where the kernel, node, verifier, and integrations live.
---

## Workspace

The Cargo workspace has five members: `froglet-protocol`, `froglet-publish-engine`, `froglet-verify`, `froglet-wasm`, and the node crate `froglet` at the repository root. The other directories below are not workspace members.

| Path | What it is |
|------|------------|
| `froglet-protocol/` | The kernel: artifacts, canonical JSON, signing, verification. Builds for wasm32. |
| `froglet-verify/` | The offline verifier: library, CLI, and WebAssembly build. |
| `froglet-wasm/` | The signing half of the kernel for browsers: WebAssembly bindings that sign the five free-deal artifacts, canonicalize JSON, and hash JSON and bytes. It runs the [playground](/open-source/#playground). |
| `src/` and `tests/` | The reference node and author CLI, `froglet-node`, with its tests. |
| `froglet-publish-engine/` | The plan-then-approve publishing pipeline shared by the CLI and the MCP bridge. |
| `python/froglet-verify/` | An independent, zero-dependency Python verifier and conformance runner. |
| `conformance/` | The frozen vectors and the rules a conforming runner must assert. |
| `examples/wasm-services/` | Sample `froglet.wasm.run_json.v1` services in their own workspace, built for wasm32. The [playground](/open-source/#playground)'s editor starts from AssemblyScript ports of them. |
| `integrations/mcp/froglet/` | The JavaScript MCP server for agent hosts. |
| `integrations/openclaw/froglet/` | The OpenClaw and NemoClaw plugin. |
| `integrations/shared/froglet-lib/` | The HTTP client shared by the plugin and the MCP server. |

## froglet-protocol

The single source of truth for kernel types. Independently reimplementable.

| Module | Purpose |
|--------|---------|
| `canonical_json` | RFC 8785 JCS canonicalization |
| `crypto` | secp256k1 BIP340, SHA-256, HMAC |
| `protocol/kernel` | SignedArtifact, all 6 payload types, verify/sign |
| `protocol/chain` | Full artifact-chain validation |
| `protocol/publication` | CuratedList types |
| `protocol/identity_attestation` | DNS and OAuth/OIDC identity attestation wire types |
| `manifest` | Parser and validator for `froglet.toml` and `froglet-service.toml` |
| `publication`, `managed_publication`, `managed_deployment` | Publication authoring, provider-control, and managed-publication contracts |
| `file_download` | The application-level immutable file package; no kernel artifact changes |
| `oci_worker` | The wire contract for isolated OCI execution workers |
| `ExecutionRuntime` | Enum shared across crates |

Its dependencies are listed in [`froglet-protocol/Cargo.toml`](https://github.com/AlitheaBio/froglet/blob/main/froglet-protocol/Cargo.toml).

## froglet (the node crate)

The node framework. Re-exports kernel types from `froglet-protocol`.

| Layer | Modules |
|-------|---------|
| **Core** | `execution` (BuiltinServiceHandler), `identity`, `identity_custody`, `protocol/workload` |
| **Settlement** | `settlement`, `lnd`, `pricing` |
| **Execution** | `sandbox`, `wasm*`, `python_sandbox`, `oci`, `oci_worker`, `confidential` |
| **Transport** | `tls`, `tor`, `nostr`, `relay_tunnel` |
| **Policy** | `provider_policy` (operator policy and allowances), `requester_budget` (requester spend ledger) |
| **Publication and files** | `managed_publication`, `managed_registry`, `file_download` |
| **Runtime** | `api/*`, `cli/*`, `server`, `config`, `state`, `db`, `deals`, `jobs` |

The default public marketplace uses Froglet's public marketplace integration
contract.
