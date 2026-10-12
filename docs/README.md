# Docs

Created and maintained by [Armanas Povilionis](https://armanas.dev).

## Start Here

Canonical onboarding lives in
[../docs-site/src/content/docs/docs.mdx](../docs-site/src/content/docs/docs.mdx)
(the `learn/` index remains as a legacy route). Use the repo docs with this
split:

- [../README.md](../README.md): repo overview, component map, verification, and release surfaces
- [../docs-site/src/content/docs/docs.mdx](../docs-site/src/content/docs/docs.mdx): canonical public onboarding path
- `docs/`: specifications, operator references, and integration details once you know which path you need

Public launch entry points:

- [../docs-site/src/content/docs/learn/cloud-trial.mdx](../docs-site/src/content/docs/learn/cloud-trial.mdx): public beta scope and the retired hosted-trial notice
- [../docs-site/src/content/docs/learn/quickstart.mdx](../docs-site/src/content/docs/learn/quickstart.mdx): local self-host quickstart
- [../docs-site/src/content/docs/learn/agents.mdx](../docs-site/src/content/docs/learn/agents.mdx): agent setup
- [../docs-site/src/content/docs/learn/plugin-distribution.mdx](../docs-site/src/content/docs/learn/plugin-distribution.mdx): plugin and registry distribution
- [../docs-site/src/content/docs/learn/payment-rails.mdx](../docs-site/src/content/docs/learn/payment-rails.mdx): Lightning, Stripe, and x402 local/self-hosted setup

## Specifications

- [SPEC.md](SPEC.md): protocol specification umbrella — verification algorithm, chain validation and issue-code registry, settlement-method registry (what each rail proves vs attests), conformance, non-goals
- [KERNEL.md](KERNEL.md): normative kernel specification — signed artifacts, settlement, state machines
- [VERSIONING.md](VERSIONING.md): stability policy — what is frozen, what may change, how `froglet/v2` would happen
- [THREAT_MODEL.md](THREAT_MODEL.md): assets, trust boundaries, key-compromise runbook, accepted risks
- [SERVICE_BINDING.md](SERVICE_BINDING.md): service-binding contract — service_id, offer_kind, product shapes
- [HTTP_SERVICES.md](HTTP_SERVICES.md): bounded HTTP JSON services, invitations, limits and release qualification
- [MANIFEST.md](MANIFEST.md): service manifest format (`froglet-service.toml`)
- [SHAREABLE_SERVICE_LINK.md](SHAREABLE_SERVICE_LINK.md): proposed human/social/agent-readable service URL contract
- [FILE_AND_COMPUTE_SCOPE.md](FILE_AND_COMPUTE_SCOPE.md): paused implementation checkpoint and release gates for files, CPU containers, and GPU jobs; payments deferred
- [openapi.yaml](openapi.yaml): HTTP API reference (OpenAPI)

## Architecture

- [ARCHITECTURE.md](ARCHITECTURE.md): system layering — kernel, adapters, runtime, services
- [AGENT_FIRST_PUBLICATION_PLAN.md](AGENT_FIRST_PUBLICATION_PLAN.md): two-approval implementation/evidence matrix, dependency budget, and remaining external gates
- [PUBLICATION_CONTRACT.md](PUBLICATION_CONTRACT.md): versioned non-Kernel publication, lifecycle/native MCP, reachability, portable hosting, and Release Bundle contracts
- [MARKETPLACE.md](MARKETPLACE.md): marketplace integration and the default public marketplace
- [ARBITER.md](ARBITER.md): MVP complaint and marketplace enforcement boundary
- [RELAY.md](RELAY.md): relay ingress v1 contract (outbound tunnel, zero-DNS public HTTPS)

## Companion Docs

- [ADAPTERS.md](ADAPTERS.md): transport and execution adapter boundaries
- [API_ERRORS.md](API_ERRORS.md): API error reference — status codes and error shapes
- [CONFIDENTIAL.md](CONFIDENTIAL.md): TEE and encrypted execution extension
- [MOUNTS.md](MOUNTS.md): capability-gated data mounts for published services
- [NOSTR.md](NOSTR.md): Nostr linked identity publication
- [RUNTIME.md](RUNTIME.md): bot-facing localhost runtime surface
- [STORAGE_PROFILE.md](STORAGE_PROFILE.md): storage and archival profiles

## Operations

- [CONFIGURATION.md](CONFIGURATION.md): environment-variable configuration reference
- [PROVIDER_ONBOARDING.md](PROVIDER_ONBOARDING.md): publish path and provider onboarding
- [PAYMENT_MATRIX.md](PAYMENT_MATRIX.md): payment rails and verification coverage
- [IDENTITY_ATTESTATION.md](IDENTITY_ATTESTATION.md): optional DNS + OAuth identity bindings
- [IDENTITY_CUSTODY.md](IDENTITY_CUSTODY.md): encrypted backup, fail-closed restore, signed rotation continuity, and KMS/HSM custody adapters
- [DOCKER.md](DOCKER.md): local compose and container deployment
- [GCP_SINGLE_VM.md](GCP_SINGLE_VM.md): single-VM self-host deployment wrapper
- [HOSTED_TRIAL.md](HOSTED_TRIAL.md): retirement record for the former hosted trial
- [NAME_COHERENCE.md](NAME_COHERENCE.md): lightweight launch name and registry-risk note
- [RELEASE.md](RELEASE.md): release process and published image contract
- [FEEDBACK.md](FEEDBACK.md): MVP feedback channel and triage loop

## Integrations

- [PLUGIN_DISTRIBUTION.md](PLUGIN_DISTRIBUTION.md): MCP registry and agent-plugin distribution order
- [MCP_DIRECTORY_SUBMISSIONS.md](MCP_DIRECTORY_SUBMISSIONS.md): MCP directory submission tracking
- [OPENCLAW.md](OPENCLAW.md): OpenClaw (Claude) integration
- [NEMOCLAW.md](NEMOCLAW.md): NemoClaw (local agent) integration
- [ROLE_TOOL_ARCHITECTURE.md](ROLE_TOOL_ARCHITECTURE.md): role and tool architecture
