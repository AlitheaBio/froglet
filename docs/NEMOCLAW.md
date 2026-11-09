# NemoClaw

NemoClaw uses the same `froglet` plugin contract as OpenClaw.

The node model is the same too: a Froglet node can publish local services and
invoke remote services. NemoClaw only changes where the plugin runs and how it
reaches the Froglet control API.

Named services, data services, and open-ended compute are all the same Froglet
primitive at the deal layer.

Current implementation note:

- the checked-in execution profiles are current reference implementations
- broader interpreted/container compute is part of the same Froglet primitive

Remote service listing still goes through Froglet discovery. If discovery is
misconfigured or unhealthy, `discover_services` returns a structured error.

## Deployment

In NemoClaw the plugin runs inside the sandbox and talks to the host-side
Froglet control API over HTTPS.

Use the checked-in configs:

- [../integrations/openclaw/froglet/examples/openclaw.config.nemoclaw.example.json](../integrations/openclaw/froglet/examples/openclaw.config.nemoclaw.example.json)
- [../integrations/openclaw/froglet/examples/openclaw.config.nemoclaw.hosted.example.json](../integrations/openclaw/froglet/examples/openclaw.config.nemoclaw.hosted.example.json)

Supported plugin keys are the same as OpenClaw:

- `hostProduct`
- `baseUrl`
- `providerUrl`
- `runtimeUrl`
- `marketplaceUrl`
- `marketplaceArbiterUrl`
- `authTokenPath`
- `providerAuthTokenPath`
- `runtimeAuthTokenPath`
- `requestTimeoutMs`
- `defaultSearchLimit`
- `maxSearchLimit`

## Native Staging

Use documented OpenShell/NemoClaw commands only.

Stage the plugin and its shared library, preserving the repository's relative
directory layout. The plugin imports `integrations/shared/froglet-lib`; uploading
only the plugin directory leaves those imports unresolved. For a directory
upload, OpenShell retains the source directory's basename inside the destination,
so pass its parent directory as the destination:

```bash
openshell sandbox upload my-node \
  /absolute/path/to/froglet/integrations/openclaw/froglet \
  /sandbox/froglet/integrations/openclaw
openshell sandbox upload my-node \
  /absolute/path/to/froglet/integrations/shared/froglet-lib \
  /sandbox/froglet/integrations/shared
```

Stage the Froglet control token. Unlike a directory upload, this unfiltered
single-file upload names the destination file:

```bash
openshell sandbox upload my-node \
  /absolute/path/to/froglet-control.token \
  /sandbox/.openclaw/froglet-control.token
```

If the host uses a private CA, stage that CA as well.

The token in this example is readable inside the sandbox. This staging flow
does not implement credential substitution outside the agent workload.

## Verification

Full NemoClaw-managed onboarding and plugin execution are not yet verified.
The [standalone OpenShell qualification](OPENSHELL_QUALIFICATION.md) is a separate
target: it uses OpenShell 0.1.7, while
[NemoClaw 0.0.142 pins OpenShell 0.0.116](https://github.com/NVIDIA/NemoClaw/blob/v0.0.142/nemoclaw-blueprint/blueprint.yaml#L7).
Do not treat a standalone OpenShell result as verification of that NemoClaw stack.

The plugin doctor checks the staged entrypoint, manifest and shared dependency
files without executing plugin code. An error produces a nonzero exit status;
a skipped runtime check remains a warning. Passing these checks does not verify
OpenShell policy enforcement or replace a real tool invocation in the sandbox.

```bash
nemoclaw my-node status
openshell sandbox get my-node
nemoclaw my-node connect
```

Inside the sandbox:

```bash
TOKEN=$(cat /sandbox/.openclaw/froglet-control.token)
curl -H "Authorization: Bearer $TOKEN" \
  https://node.example/health
```

Minimum completion evidence:

```bash
openshell sandbox upload my-node \
  /absolute/path/to/froglet/integrations/openclaw/froglet \
  /sandbox/froglet/integrations/openclaw
openshell sandbox upload my-node \
  /absolute/path/to/froglet/integrations/shared/froglet-lib \
  /sandbox/froglet/integrations/shared
openshell sandbox upload my-node \
  /absolute/path/to/froglet-control.token \
  /sandbox/.openclaw/froglet-control.token
nemoclaw my-node connect
```

Then run the `froglet` tool from inside NemoClaw with `action=status`,
`action=plan_install`, and one real local action once the host Froglet node is
reachable over HTTPS.

The bot-facing tool contract is identical to OpenClaw: one tool named
`froglet`.

The live tool contract is the same as OpenClaw:

- `summary` is metadata only
- agent-grade publication goes through `marketplace_publish`; lower-level local
  artifact publication still goes through `publish_artifact`
- service metadata exposes `offer_kind` and `resource_kind`; direct compute
  still goes through `run_compute` rather than service listing
- `run_compute` should include `provider_id` or `provider_url` because direct
  compute is provider-targeted rather than service-discovered
- project authoring, log tailing, and node restart actions are not part of the
  current public API surface
