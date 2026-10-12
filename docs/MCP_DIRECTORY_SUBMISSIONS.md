# MCP Directory Submissions

Status date: 2026-05-01.

Use this file when submitting Froglet to third-party MCP directories. Keep every
claim tied to either the official MCP Registry entry, the published npm package,
or a verified local/self-hosted Froglet path.

## Canonical Metadata

| Field | Value |
| --- | --- |
| Name | Froglet |
| Registry name | `io.github.armanas/froglet` |
| npm package | `froglet-mcp` |
| Install command | `npx -y froglet-mcp` |
| Transport | `stdio` |
| Version | `0.1.5` for `froglet-mcp` |
| License | Apache-2.0 |
| Homepage | `https://froglet.dev` |
| Repository | `https://github.com/AlitheaBio/froglet` |
| Issues | `https://github.com/AlitheaBio/froglet/issues` |
| Category | Developer Tools / AI Agents / Compute |
| Tags | `mcp`, `model-context-protocol`, `froglet`, `agents`, `verifiable-compute`, `receipts`, `settlement`, `marketplace`, `wasm` |

Short description:

```text
Run local Froglet verifiable compute, signed receipts, settlement, and service publication.
```

Long description:

```text
Froglet connects agent hosts to a local or self-hosted Froglet node for verifiable compute, signed receipts, service discovery, settlement inspection, artifact publication, marketplace operations, and install/use-case planning. The installed MCP server is local/actionable-first and starts over stdio with `npx -y froglet-mcp`.
```

Boundary statement:

```text
Installed MCP is for local/self-hosted Froglet nodes. The no-install hosted proof lives at https://froglet.dev/llms.txt and is intentionally demo-scoped. Do not claim hosted paid rails, persistent identity, GPU, batch, or marketplace write support unless those paths are separately verified.
```

## Directory Checklist

| Directory | Current action | Status |
| --- | --- | --- |
| Official MCP Registry | Published via `mcp-publisher`; verify latest active record before citing. | Active record is verified at `0.1.5` with `isLatest: true`. |
| MCP.Directory | Submit or claim the GitHub/npm listing; include repo URL, npm package, short description, and contact email if desired. | Prepared, not submitted here. |
| MCPCentral | Use `mcp-publisher` with `--registry https://registry.mcpcentral.io` if the registry accepts the same `server.json` shape. | Prepared, not submitted here. |
| MCP.so | Submit type/name/repo URL/server config using the canonical metadata above. | Prepared, not submitted here. |
| mcpservers.org / awesome-mcp-servers | Submit via mcpservers.org; `wong2/awesome-mcp-servers` redirects submissions there. | Prepared, not submitted here. |
| Smithery | Do not submit the plain stdio npm package until Froglet has either an MCPB bundle or a remote Streamable HTTP/OAuth surface. | Not ready by design. |
| Glama | Verify current claim/submission flow before submitting. | Needs external submission-path verification. |
| PulseMCP | Verify current claim/submission flow before submitting. | Needs external submission-path verification. |

## Verification Commands

```bash
npm view froglet-mcp version dist-tags.latest description license --json
node -e 'fetch("https://registry.modelcontextprotocol.io/v0.1/servers?search=io.github.armanas/froglet").then(r=>r.json()).then(j=>console.log(JSON.stringify(j.servers.map(x=>({version:x.server.version, package:x.server.packages?.[0]?.identifier, isLatest:x._meta?.["io.modelcontextprotocol.registry/official"]?.isLatest})), null, 2)))'
```

The registry search may return old versions as well as the latest version. Treat
only the record with `isLatest: true` as the current official registry proof.
