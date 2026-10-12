# Plugin And Registry Distribution

Status date: 2026-05-01.

Froglet distribution has two distinct paths:

1. `https://froglet.dev/llms.txt` for the no-install hosted proof.
2. `froglet-mcp` and host plugin wrappers for real local/self-hosted work.

Do not blur those paths. Installed plugins should connect an agent to a local or
self-hosted Froglet node so it can inspect status, discover services, invoke
work, publish artifacts, inspect settlement, and guide installation. The hosted
proof is intentionally HTTP-only and demo-scoped.

## Distribution Order

1. **Official MCP Registry** - publish `froglet-mcp` metadata after the matching
   npm package version exists.
2. **Codex plugin bundle** - ship the repo-local Codex plugin under
   `plugins/froglet/` and list it in `.agents/plugins/marketplace.json`.
3. **Claude Code plugin marketplace** - ship `.claude-plugin/marketplace.json`
   plus the same `plugins/froglet/` bundle.
4. **OpenClaw package** - package the checked-in OpenClaw plugin as an
   actionable local/self-hosted integration.
5. **NemoClaw package** - verify NemoClaw-specific sandbox and HTTPS deltas in a
   real NemoClaw environment before claiming support.
6. **Third-party MCP directories** - submit only after the official MCP Registry
   entry is live, so every directory points at the same npm package, license,
   docs, and local/actionable boundary.

A hosted consumer app directory path is intentionally out of scope for this repo
slice. If that product path returns later, it should be designed as its own app,
not as a repackaging of the local stdio MCP tool.

## Current Artifacts

| Surface | File | Current state |
| --- | --- | --- |
| npm package | `package.json` | `froglet-mcp`, Apache-2.0, stdio binary, `mcpName` set |
| MCP Registry | `server.json` | Active for `io.github.armanas/froglet` at `0.4.0`; latest record verified with `isLatest: true` |
| Codex plugin | `plugins/froglet/.codex-plugin/plugin.json` | Repo-local marketplace/test bundle |
| Codex marketplace | `.agents/plugins/marketplace.json` | Local Codex marketplace entry |
| Claude plugin | `plugins/froglet/.claude-plugin/plugin.json` | Claude Code plugin metadata |
| Claude marketplace | `.claude-plugin/marketplace.json` | GitHub-hosted marketplace entry |
| Shared plugin MCP | `plugins/froglet/.mcp.json` | Starts `npx -y froglet-mcp` with local provider/runtime env |
| Shared plugin skill | `plugins/froglet/skills/froglet/SKILL.md` | Tells agents to check local status, plan install, then execute real actions |
| Claude slash command | `plugins/froglet/commands/froglet.md` | `/froglet` entrypoint for the same local status/install/use-case flow |
| Cursor project MCP | `.cursor/mcp.json` | Project-scoped Cursor config for the published `froglet-mcp` package |
| OpenClaw/NemoClaw | `integrations/openclaw/froglet/` | Source plugin and examples; registry package still pending |

The npm MCP package version and the host-plugin wrapper version are allowed to
move independently. Bump `package.json` and `server.json` only when the
published `froglet-mcp` package or MCP Registry metadata changes. Bump the
plugin manifests and Claude marketplace version when only wrapper metadata,
skills, or commands change.

## MCP Registry Publish

The MCP Registry validates that the package metadata matches `server.json`, so
`package.json` contains:

```json
"mcpName": "io.github.armanas/froglet"
```

The registry metadata lives at repo root in `server.json`. Because npm tarballs
are immutable, publish the npm version first, then publish the registry
metadata.

```bash
npm run check:mcp
npm run test:mcp
npm pack --dry-run
npm publish --tag beta --provenance=false --otp <npm-otp>
mcp-publisher login github
mcp-publisher publish
curl "https://registry.modelcontextprotocol.io/v0.1/servers?search=io.github.armanas/froglet"
```

Expected proof after publish: the registry search result includes
`"name":"io.github.armanas/froglet"` and package identifier `froglet-mcp`.

The current candidate is a prerelease. Publish it explicitly under `beta`; do
not move npm's `latest` tag. GitHub binary/image publication does not publish
this npm package or MCP Registry record. Each needs separate publication and
verification. An unversioned `npx froglet-mcp` selects npm's `latest` package;
use the verified candidate version or `@beta` to exercise the prerelease.
Verify the exact candidate version and `dist-tags.beta`
after publication; `npm view froglet-mcp version` checks `latest`.

Use compact verification commands after publishing:

```bash
npm view froglet-mcp version
node -e 'fetch("https://registry.modelcontextprotocol.io/v0.1/servers?search=io.github.armanas/froglet&limit=20").then(r=>r.json()).then(j=>{const row=j.servers?.find(x=>x.server?.name==="io.github.armanas/froglet"&&x._meta?.["io.modelcontextprotocol.registry/official"]?.isLatest); const s=row?.server; console.log(JSON.stringify({name:s?.name, package:s?.packages?.[0]?.identifier, version:s?.version, isLatest:row?._meta?.["io.modelcontextprotocol.registry/official"]?.isLatest}, null, 2));})'
node -e 'fetch("https://froglet.dev/learn/plugin-distribution/").then(r=>console.log(r.status, r.headers.get("content-type")))'
```

Do not use `curl https://froglet.dev/learn/plugin-distribution/` as a proof
command. That URL is a rendered documentation page, so raw `curl` output is
HTML by design.

## Codex Plugin

Local/repo test path:

```bash
cat .agents/plugins/marketplace.json
cat plugins/froglet/.codex-plugin/plugin.json
```

The plugin does not vendor Froglet code. It starts the published MCP server in
local mode:

```json
{
  "mcpServers": {
    "froglet": {
      "type": "stdio",
      "command": "npx",
      "args": ["-y", "froglet-mcp"],
      "env": {
        "FROGLET_PROFILE": "local",
        "FROGLET_PROVIDER_URL": "http://127.0.0.1:8080",
        "FROGLET_RUNTIME_URL": "http://127.0.0.1:8081",
        "FROGLET_PROVIDER_AUTH_TOKEN_PATH": "data/runtime/froglet-control.token",
        "FROGLET_RUNTIME_AUTH_TOKEN_PATH": "data/runtime/auth.token"
      }
    }
  }
}
```

Operational rule: the first task should call `status`. If the local node is not
running or tokens are missing, call `plan_install`; present its immutable
release, hashes, filesystem/process impact, exact command preview, and approval
hash. Only after explicit approval should the agent pass the exact returned
`release_tag` and `install_approval_hash` to `get_install_guide` and run its
host-shell command.
After local health is verified, call `plan_use_case` before implementing the
user's first workflow, especially for batch or GPU requests where unsupported
boundaries must be named before execution. Claude Code also exposes `/froglet`,
which follows the same local-first flow.

Current Codex host status: Codex CLI 0.114.0 exposes `codex mcp`, but no public
`plugin install` or `plugin validate` command. The repo-local plugin metadata is
therefore packaging-ready, while clean Codex plugin installation remains blocked
on a supported Codex host workflow. Direct MCP verification is available through
`scripts/setup-agent.sh --target codex` or the shared `froglet-mcp` stdio config.

## Cursor MCP

The repository includes a project-scoped Cursor config at `.cursor/mcp.json`.
It uses the published package:

```json
{
  "mcpServers": {
    "froglet": {
      "type": "stdio",
      "command": "npx",
      "args": ["-y", "froglet-mcp"]
    }
  }
}
```

Cursor host verification requires Cursor or `cursor-agent` to be installed. If
neither binary is available, the supported evidence is limited to config
inspection, MCP server tests, and direct MCP stdio smoke.

## Claude Code Plugin Marketplace

Claude Code can consume a GitHub-hosted marketplace from the repository root.
Use sparse checkout so Claude only fetches plugin metadata and the plugin
bundle:

```bash
claude plugin marketplace add AlitheaBio/froglet --sparse .claude-plugin plugins
claude plugin install froglet@froglet
```

Local test path:

```bash
claude plugin validate .
claude plugin marketplace add . --scope local
claude plugin install froglet@froglet --scope local
```

Expected proof: the plugin installs, starts the bundled MCP server, reports
local `status`, then performs one real local action such as `list_local_services`
or `invoke_service` after the provider/runtime and token paths are available.

## OpenClaw And NemoClaw

OpenClaw/NemoClaw remain source-plugin paths until their registry/package story
is verified. The checked-in plugin is:

```text
integrations/openclaw/froglet/
```

Validation commands:

```bash
node --check integrations/openclaw/froglet/index.js
node --check integrations/openclaw/froglet/scripts/doctor.mjs
node --test integrations/openclaw/froglet/test/plugin.test.js \
  integrations/openclaw/froglet/test/config-profiles.test.mjs \
  integrations/openclaw/froglet/test/doctor.test.mjs \
  integrations/openclaw/froglet/test/froglet-client.test.mjs
```

OpenClaw can use loopback host URLs. NemoClaw must be verified separately
because the plugin usually runs inside a sandbox and reaches the host over HTTPS.
Do not claim NemoClaw distribution is complete until a real NemoClaw environment
installs the package and runs at least one local Froglet action.

Current OpenClaw status: source-level plugin tests, doctor runtime checks,
direct source-plugin `froglet` tool invocation, and OpenClaw `2026.5.5`
Gateway-mediated invocation pass against a local Froglet provider/runtime stack.
The 2026-05-06 GCP proof covered `plugins install --link`, `config validate`,
`plugins inspect froglet --runtime --json`, Gateway startup, `tools.catalog`,
and HTTP `/tools/invoke` for `status` plus `list_local_services`. Current
OpenClaw requires Node.js `22.14.0+`; the verified Gateway run supplied
`FROGLET_PROVIDER_AUTH_TOKEN_PATH` and `FROGLET_RUNTIME_AUTH_TOKEN_PATH` in the
Gateway environment.

NemoClaw status: source-level shared plugin tests pass, but real NemoClaw
verification requires a NemoClaw/OpenShell environment, staged plugin source,
staged token, HTTPS host reachability from the sandbox, and one successful
Froglet tool action.

## Third-Party MCP Directories

Reusable metadata and directory-specific submission notes live in
[`docs/MCP_DIRECTORY_SUBMISSIONS.md`](MCP_DIRECTORY_SUBMISSIONS.md). The official
MCP Registry is active for `io.github.armanas/froglet`; third-party listings
remain unverified until each public directory shows Froglet pointing at
`froglet-mcp`, Apache-2.0, `https://github.com/AlitheaBio/froglet`, and the
local/self-hosted MCP boundary.

## Boundary

Every distribution surface must preserve this boundary:

- no-install evaluation: the public beta in a browser (`https://froglet.dev/services/`)
  and offline receipt verification, with no local secrets; the older `try.froglet.dev` trial is retired
- installed MCP/plugins: local/self-hosted provider/runtime actions, paid rails,
  persistent identity, service publication, marketplace write flows, long jobs,
  batch, GPU, and production use cases
