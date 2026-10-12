import { after, before, describe, it } from "node:test"
import assert from "node:assert/strict"
import { copyFile, mkdir, mkdtemp, readFile, rm, stat, writeFile } from "node:fs/promises"
import { execFile as execFileCb } from "node:child_process"
import { join, dirname, resolve } from "node:path"
import { fileURLToPath } from "node:url"
import { tmpdir } from "node:os"
import { promisify } from "node:util"
import { createHash } from "node:crypto"

import { buildToolDefinitions, handleToolCall } from "../lib/tools.js"

const __filename = fileURLToPath(import.meta.url)
const __dirname = dirname(__filename)
const REPO_ROOT = resolve(__dirname, "../../../../")
const execFile = promisify(execFileCb)
const INSTALL_TAG = "v0.1.0-alpha.1"
const INSTALL_MANIFEST_SHA256 = "a".repeat(64)
const INSTALL_BOOTSTRAP = "#!/bin/sh\nset -eu\nprintf 'fixture bootstrap\\n'\n"
const INSTALL_BOOTSTRAP_SHA256 = createHash("sha256").update(INSTALL_BOOTSTRAP).digest("hex")

function immutableReleaseFixture(overrides = {}) {
  return {
    tag_name: INSTALL_TAG,
    immutable: true,
    draft: false,
    assets: [
      {
        name: "release-manifest.json",
        digest: `sha256:${INSTALL_MANIFEST_SHA256}`,
        state: "uploaded"
      },
      {
        name: "agent-bootstrap.sh",
        digest: `sha256:${INSTALL_BOOTSTRAP_SHA256}`,
        state: "uploaded"
      }
    ],
    ...overrides
  }
}

function releaseResponseFor(url, release = immutableReleaseFixture()) {
  return url.includes("?per_page=1") ? [release] : release
}

let tmpDir
let tokenPath
let config

function mockFetch(handler) {
  const original = globalThis.fetch
  globalThis.fetch = handler
  return () => {
    globalThis.fetch = original
  }
}

async function mockMarketplaceJsonRequest(url, options = {}) {
  const fetchOptions = {
    method: options.method ?? "GET",
    headers: {
      "Content-Type": "application/json"
    }
  }
  if (options.jsonBody !== undefined) {
    fetchOptions.body = JSON.stringify(options.jsonBody)
  }
  const response = await fetch(url, fetchOptions)
  const payload = await response.json()
  const expectedStatuses = options.expectedStatuses ?? [200]
  if (!expectedStatuses.includes(response.status)) {
    throw new Error(`${url} failed with ${response.status}: ${JSON.stringify(payload)}`)
  }
  return { status: response.status, payload }
}

async function mockProviderJsonRequest(url, options = {}) {
  assert.equal(options.pin?.pinnedAddress, "93.184.216.34")
  const fetchOptions = {
    method: options.method ?? "GET",
    headers: {
      "Content-Type": "application/json"
    }
  }
  if (options.jsonBody !== undefined) {
    fetchOptions.body = JSON.stringify(options.jsonBody)
  }
  const response = await fetch(url, fetchOptions)
  const payload = await response.json()
  const expectedStatuses = options.expectedStatuses ?? [200]
  if (!expectedStatuses.includes(response.status)) {
    throw new Error(`${url} failed with ${response.status}: ${JSON.stringify(payload)}`)
  }
  return { status: response.status, payload }
}

function providerSearchResponse() {
  return {
    providers: [
      {
        provider_id: "prov-1",
        descriptor_hash: "desc-1",
        transport_endpoints: [
          { uri: "https://provider.example", features: ["quote_http"], priority: 1 }
        ],
        offers: [
          {
            offer_hash: "offer-hash-1",
            service_id: "svc-1",
            offer_id: "svc-1",
            offer_kind: "execution",
            runtime: "python",
            settlement_method: "none",
            base_fee_msat: 0,
            success_fee_msat: 0,
            execution_profile: {
              runtime: "python",
              package_kind: "inline_source",
              contract_version: "froglet.python.handler_json.v1",
              access_handles: ["mount.filesystem.read.workspace"]
            }
          }
        ],
        last_seen_at: "2026-04-09T00:00:00Z"
      }
    ],
    cursor: null,
    has_more: false
  }
}

before(async () => {
  tmpDir = await mkdtemp(join(tmpdir(), "froglet-mcp-test-"))
  tokenPath = join(tmpDir, "token")
  await writeFile(tokenPath, "test-token-abc")
  config = {
    profile: "local",
    providerUrl: "http://127.0.0.1:8080",
    runtimeUrl: "http://127.0.0.1:8081",
    providerAuthTokenPath: tokenPath,
    runtimeAuthTokenPath: tokenPath,
    marketplaceUrl: "https://mkt.example",
    marketplaceArbiterUrl: "https://arbiter.example",
    requestTimeoutMs: 5000,
    defaultSearchLimit: 10,
    maxSearchLimit: 50,
    _deps: {
      client: {
        providerUrl: {
          lookup: async () => [{ address: "93.184.216.34", family: 4 }]
        },
        providerJsonRequest: mockProviderJsonRequest
      },
      marketplace: {
        marketplaceUrl: {
          lookup: async () => [{ address: "93.184.216.34", family: 4 }]
        },
        marketplaceJsonRequest: mockMarketplaceJsonRequest
      },
      install: {
        fetchReleaseMetadata: async (url) => releaseResponseFor(url),
        fetchBootstrap: async () => INSTALL_BOOTSTRAP
      }
    }
  }
})

function exactTextField(text, field) {
  const match = text.match(new RegExp(`^${field}: (.+)$`, "m"))
  assert.ok(match, `missing ${field} in:\n${text}`)
  return match[1]
}

async function approvedInstallGuide(args, testConfig = config) {
  const plan = await handleToolCall(
    "froglet",
    { ...args, action: "plan_install" },
    testConfig
  )
  assert.equal(plan.isError, undefined)
  const planText = plan.content[0].text
  assert.match(planText, /status: approval_required/)
  return handleToolCall(
    "froglet",
    {
      ...args,
      action: "get_install_guide",
      release_tag: exactTextField(planText, "release_tag"),
      install_approval_hash: exactTextField(planText, "install_approval_hash")
    },
    testConfig
  )
}

function withInstallDeps(install) {
  return {
    ...config,
    _deps: {
      ...config._deps,
      install
    }
  }
}

after(async () => {
  if (tmpDir) await rm(tmpDir, { recursive: true, force: true })
})

describe("tool definitions", () => {
  it("returns a single froglet tool with live actions only", () => {
    const tools = buildToolDefinitions(config)
    assert.equal(tools.length, 1)
    assert.equal(tools[0].name, "froglet")
    const actionEnum = tools[0].inputSchema.properties.action.enum
    assert.deepEqual(actionEnum, [
      "discover_services",
      "get_service",
      "invoke_service",
      "list_local_services",
      "get_local_service",
      "publish_artifact",
      "status",
      "get_task",
      "wait_task",
      "run_compute",
      "get_wallet_balance",
      "get_spend_status",
      "reset_spend",
      "list_settlement_activity",
      "get_payment_intent",
      "get_invoice_bundle",
      "plan_install",
      "get_install_guide",
      "plan_use_case",
      "marketplace_register",
      "marketplace_domain_claim",
      "marketplace_domain_complete",
      "marketplace_search",
      "marketplace_provider",
      "marketplace_receipts",
      "marketplace_file_complaint",
      "marketplace_get_complaint",
      "marketplace_publish"
    ])
    assert.match(tools[0].description, /provider_id/)
    assert.match(tools[0].description, /marketplace_publish/)
    assert.match(tools[0].description, /status/)
    assert.equal(tools[0].inputSchema.properties.starter.type, "string")
    assert.deepEqual(tools[0].inputSchema.properties.footprint.enum, [
      "auto",
      "native",
      "docker",
      "binary",
      "source"
    ])
    assert.equal(tools[0].inputSchema.properties.release_tag.type, "string")
    assert.equal(
      tools[0].inputSchema.properties.install_approval_hash.pattern,
      "^[0-9a-f]{64}$"
    )
    assert.match(tools[0].inputSchema.properties.action.description, /two-call flow/)
    assert.doesNotMatch(tools[0].inputSchema.properties.action.description, /one-call/)
    assert.deepEqual(tools[0].inputSchema.properties.network_mode.enum, [
      "clearnet",
      "tor",
      "dual"
    ])
    assert.deepEqual(tools[0].inputSchema.properties.hosting.properties.kind.enum, [
      "local",
      "relay",
      "tor",
      "self"
    ])
    assert.deepEqual(tools[0].inputSchema.properties.mounts.items.properties.kind.enum, [
      "postgres",
      "sqlite",
      "object_store",
      "s3",
      "redis"
    ])
    assert.match(
      tools[0].inputSchema.properties.mounts.items.properties.kind.description,
      /compatibility alias/
    )
    assert.deepEqual(tools[0].inputSchema.properties.workload_profile.enum, [
      "consumer",
      "provider",
      "evidence",
      "payments",
      "batch",
      "gpu"
    ])
    assert.match(
      tools[0].inputSchema.properties.starter.description,
      /example only/
    )
    assert.deepEqual(tools[0].inputSchema.properties.payment_rail.enum, [
      "none",
      "lightning-mock",
      "lightning-lnd-rest",
      "lightning-phoenixd",
      "stripe-test",
      "stripe-live",
      "x402"
    ])
    assert.deepEqual(tools[0].inputSchema.properties.lightning_mode.enum, [
      "mock",
      "lnd_rest",
      "phoenixd"
    ])
    // Regression: marketplace_url must be defined exactly once in the schema
    // source — a duplicate object key silently overwrites the first and the
    // earlier description is never served to agents.
    assert.match(
      tools[0].inputSchema.properties.marketplace_url.description,
      /plan_install/
    )
    assert.match(
      tools[0].inputSchema.properties.marketplace_url.description,
      /marketplace_publish/
    )
  })
})

describe("froglet MCP actions", () => {
  it("rejects the removed hosted proof action with a pointer to llms.txt", async () => {
    const result = await handleToolCall(
      "froglet",
      { action: "run_hosted_proof" },
      config
    )
    assert.equal(result.isError, true)
    assert.match(result.content[0].text, /not part of the installed MCP surface/)
    assert.match(result.content[0].text, /froglet.dev\/llms.txt/)
  })

  it("exposes wallet balance through the settlement MCP action", async () => {
    const restore = mockFetch(async (url) => {
      assert.equal(String(url), "http://127.0.0.1:8081/v1/runtime/wallet/balance")
      return new Response(
        JSON.stringify({
          backend: "lightning",
          mode: "mock",
          balance_known: true,
          balance_sats: 4242,
          accepted_payment_methods: ["lightning"],
          reservations: true,
          receipts: true
        })
      )
    })
    try {
      const result = await handleToolCall(
        "froglet",
        { action: "get_wallet_balance" },
        config
      )
      const text = result.content[0].text
      assert.match(text, /backend: lightning/)
      assert.match(text, /balance_sats: 4242/)
      assert.equal(result.isError, undefined)
    } finally {
      restore()
    }
  })

  it("surfaces settlement activity rows to the LLM", async () => {
    const restore = mockFetch(async (url) => {
      assert.ok(
        String(url).startsWith("http://127.0.0.1:8081/v1/runtime/settlement/activity"),
        `unexpected url: ${url}`
      )
      return new Response(
        JSON.stringify({
          items: [
            {
              deal_id: "deal-xyz",
              provider_id: "prov-1",
              status: "succeeded",
              workload_kind: "events.query",
              settlement_method: "none",
              base_fee_msat: 0,
              success_fee_msat: 0,
              has_receipt: true,
              has_result: true,
              created_at: 1,
              updated_at: 2
            }
          ],
          limit: 25
        })
      )
    })
    try {
      const result = await handleToolCall(
        "froglet",
        { action: "list_settlement_activity" },
        config
      )
      const text = result.content[0].text
      assert.match(text, /count: 1/)
      assert.match(text, /deal_id: deal-xyz/)
      assert.match(text, /status: succeeded/)
      assert.equal(result.isError, undefined)
    } finally {
      restore()
    }
  })

  it("publish_artifact forwards the complete canonical publication intent", async () => {
    const restore = mockFetch(async (url, options = {}) => {
      assert.equal(String(url), "http://127.0.0.1:8080/v1/provider/artifacts/publish")
      const body = JSON.parse(options.body)
      assert.equal(body.service_id, "svc-1")
      assert.equal(body.schema_version, "froglet.publication-intent.v1")
      assert.equal(body.project_id, "project-1")
      assert.equal(body.wasm_module_hex, "0061736d01000000")
      assert.equal(body.source_kind, "generated")
      assert.equal(body.starter, "{\"a\":7,\"b\":5}")
      assert.deepEqual(body.input_schema, {
        type: "object",
        required: ["a", "b"]
      })
      assert.deepEqual(body.output_schema, {
        type: "object",
        required: ["sum"]
      })
      assert.deepEqual(body.limits, {
        max_input_bytes: 4096,
        max_runtime_ms: 2500,
        max_memory_bytes: 8388608,
        max_output_bytes: 2048,
        fuel_limit: 50000
      })
      assert.deepEqual(body.verification, {
        input: null,
        expected_output: 42
      })
      assert.equal(body.price_sats, 2)
      assert.equal(body.base_fee_msat, 1000)
      assert.equal(body.success_fee_msat, 2000)
      assert.equal(body.settlement_method, "lightning")
      assert.equal(body.price_currency, "sat")
      return new Response(JSON.stringify({ status: "published" }), {
        status: 201,
        headers: { "Content-Type": "application/json" }
      })
    })
    try {
      const result = await handleToolCall(
        "froglet",
        {
          action: "publish_artifact",
          service_id: "svc-1",
          project_id: "project-1",
          runtime: "wasm",
          package_kind: "inline_module",
          wasm_module_hex: "0061736d01000000",
          source_kind: "generated",
          starter: "{\"a\":7,\"b\":5}",
          input_schema: { type: "object", required: ["a", "b"] },
          output_schema: { type: "object", required: ["sum"] },
          limits: {
            max_input_bytes: 4096,
            max_runtime_ms: 2500,
            max_memory_bytes: 8388608,
            max_output_bytes: 2048,
            fuel_limit: 50000
          },
          verification: { input: null, expected_output: 42 },
          price_sats: 2,
          base_fee_msat: 1000,
          success_fee_msat: 2000,
          settlement_method: "lightning",
          price_currency: "sat"
        },
        config
      )
      assert.equal(result.isError, undefined)
      assert.match(result.content[0].text, /status: published/)
    } finally {
      restore()
    }
  })

  it("publish_artifact demo.add template publishes a free local Python demo", async () => {
    let capturedBody
    const restore = mockFetch(async (url, options = {}) => {
      assert.equal(String(url), "http://127.0.0.1:8080/v1/provider/artifacts/publish")
      capturedBody = JSON.parse(options.body)
      return new Response(
        JSON.stringify({
          status: "published",
          offer: {
            service_id: capturedBody.service_id,
            runtime: capturedBody.runtime,
            package_kind: capturedBody.package_kind,
            publication_state: capturedBody.publication_state,
            offer: { payload: { offer_id: capturedBody.offer_id, price_sats: capturedBody.price_sats } }
          }
        }),
        {
          status: 201,
          headers: { "Content-Type": "application/json" }
        }
      )
    })
    try {
      const result = await handleToolCall(
        "froglet",
        {
          action: "publish_artifact",
          template: "demo.add"
        },
        config
      )
      assert.equal(result.isError, undefined)
      assert.equal(capturedBody.service_id, "demo.add.local")
      assert.equal(capturedBody.runtime, "python")
      assert.equal(capturedBody.package_kind, "inline_source")
      assert.equal(capturedBody.price_sats, 0)
      assert.match(capturedBody.inline_source, /def handler/)
      assert.match(capturedBody.inline_source, /json\.loads\(event\)/)
      assert.match(result.content[0].text, /status: published/)
    } finally {
      restore()
    }
  })

  it("requires an explicit payment choice before returning install commands", async () => {
    // The helper never makes an HTTP call, so no fetch mock is needed.
    const result = await handleToolCall(
      "froglet",
      { action: "get_install_guide" },
      config
    )
    assert.equal(result.isError, undefined)
    const text = result.content[0].text
    assert.match(text, /status: decision_required/)
    assert.match(text, /decision: payment_rail/)
    assert.match(text, /recommended_for_first_install: none/)
    assert.match(text, /lightning-phoenixd/)
  })

  it("withholds executable commands until the exact install plan is approved", async () => {
    const withheld = await handleToolCall(
      "froglet",
      { action: "get_install_guide", payment_rail: "none" },
      config
    )
    assert.equal(withheld.isError, undefined)
    assert.match(withheld.content[0].text, /status: approval_required/)
    assert.doesNotMatch(withheld.content[0].text, /bootstrap_tmp=/)

    const result = await approvedInstallGuide({ payment_rail: "none" })
    assert.equal(result.isError, undefined)
    const text = result.content[0].text
    assert.match(text, /status: approved/)
    assert.match(text, /target_agent: claude-code/)
    assert.match(text, /payment_rail: none/)
    assert.match(text, /run_as: user-host-shell/)
    assert.match(text, /footprint: auto/)
    assert.match(text, new RegExp(`release_tag: ${INSTALL_TAG.replaceAll(".", "\\.")}`))
    assert.match(text, new RegExp(`release_manifest_sha256: ${INSTALL_MANIFEST_SHA256}`))
    assert.match(text, new RegExp(`bootstrap_sha256: ${INSTALL_BOOTSTRAP_SHA256}`))
    assert.match(text, /mktemp .*froglet-agent-bootstrap/)
    assert.match(text, /approved bootstrap SHA-256 mismatch/)
    assert.match(text, new RegExp(`VERSION='${INSTALL_TAG.replaceAll(".", "\\.")}'`))
    assert.match(text, new RegExp(`FROGLET_RELEASE_MANIFEST_SHA256='${INSTALL_MANIFEST_SHA256}'`))
    assert.match(text, /sh "\$bootstrap_tmp"/)
    assert.doesNotMatch(text, /git clone https:\/\/github\.com\/AlitheaBio\/froglet\.git/)
    assert.doesNotMatch(text, /npm ci --prefix integrations\/mcp\/froglet/)
    const command = text
      .split("\n")
      .find((line) => /^\s*1\.\s+/.test(line))
      .replace(/^\s*1\.\s+/, "")
    await execFile("sh", ["-n", "-c", command])
  })

  it("binds agent choice and paid-rail host inputs without embedding secrets", async () => {
    const result = await approvedInstallGuide({ target_agent: "codex", payment_rail: "stripe-test" })
    assert.equal(result.isError, undefined)
    const text = result.content[0].text
    assert.match(text, /target_agent: codex/)
    assert.match(text, /payment_rail: stripe-test/)
    assert.match(text, /FROGLET_AGENT_TARGET='codex'/)
    assert.match(text, /FROGLET_PAYMENT_BACKEND='stripe'/)
    assert.match(text, /FROGLET_STRIPE_SECRET_KEY="\$\{FROGLET_STRIPE_SECRET_KEY:\?set FROGLET_STRIPE_SECRET_KEY/)
    assert.doesNotMatch(text, /<stripe-test-secret-key>|sk_test_fixture_secret/)
    assert.match(text, /verifies and persists the selected stripe-test rail/)
    assert.match(text, /Stripe: set FROGLET_STRIPE_SECRET_KEY/)
  })

  it("plans a local install before commands are executed", async () => {
    const result = await handleToolCall(
      "froglet",
      {
        action: "plan_install",
        target_agent: "codex",
        footprint: "docker",
        role: "provider",
        payment_rail: "lightning-lnd-rest",
        network_mode: "tor",
        marketplace_url: "https://marketplace.froglet.dev",
        use_case: "publish a paid service"
      },
      config
    )
    assert.equal(result.isError, undefined)
    const text = result.content[0].text
    assert.match(text, /status: approval_required/)
    assert.match(text, /^install_approval_hash: [0-9a-f]{64}$/m)
    assert.match(text, new RegExp(`release_tag: ${INSTALL_TAG.replaceAll(".", "\\.")}`))
    assert.match(text, new RegExp(`bootstrap_sha256: ${INSTALL_BOOTSTRAP_SHA256}`))
    assert.match(text, /install_profile:/)
    assert.match(text, /target_agent: codex/)
    assert.match(text, /payment_rail: lightning-lnd-rest/)
    assert.match(text, /lightning_mode: lnd_rest/)
    assert.match(text, /network_mode: tor/)
    assert.match(text, /Questions to ask before running commands:\n  - None/)
    assert.match(text, /FROGLET_NETWORK_MODE='tor'/)
    assert.match(text, /FROGLET_MARKETPLACE_URL='https:\/\/marketplace\.froglet\.dev'/)
    assert.match(text, /FROGLET_AGENT_TARGET='codex'/)
    assert.match(text, /FROGLET_BOOTSTRAP_MODE='docker'/)
    assert.match(text, /FROGLET_PAYMENT_BACKEND='lightning'/)
    assert.match(text, /FROGLET_LIGHTNING_MODE='lnd_rest'/)
    assert.match(text, /FROGLET_LIGHTNING_REST_URL="\$\{FROGLET_LIGHTNING_REST_URL:\?set/)
    assert.match(text, /process_manager_impact: Create the approved Compose file/)
    assert.match(text, /\$HOME\/\.froglet\/data/)
    assert.match(text, /Post-install playbooks:/)
    assert.match(text, /provider-first/)
    assert.match(text, /Safety: This plan is non-mutating/)
  })

  it("resolves immutable release metadata and bootstrap bytes through injected read-only dependencies", async () => {
    const calls = []
    const testConfig = withInstallDeps({
      fetchReleaseMetadata: async (url) => {
        calls.push(["release", url])
        return releaseResponseFor(url)
      },
      fetchBootstrap: async (url) => {
        calls.push(["bootstrap", url])
        return Buffer.from(INSTALL_BOOTSTRAP)
      }
    })
    const plan = await handleToolCall(
      "froglet",
      { action: "plan_install", payment_rail: "none" },
      testConfig
    )
    assert.equal(plan.isError, undefined)
    assert.deepEqual(calls, [
      ["release", "https://api.github.com/repos/armanas/froglet/releases?per_page=1"],
      ["bootstrap", `https://github.com/armanas/froglet/releases/download/${INSTALL_TAG}/agent-bootstrap.sh`]
    ])
    assert.match(plan.content[0].text, /Exact command preview \(not authorized for execution\):/)
  })

  it("rejects stale install approval after profile or bootstrap drift", async () => {
    const plan = await handleToolCall(
      "froglet",
      { action: "plan_install", payment_rail: "none", footprint: "auto" },
      config
    )
    const planText = plan.content[0].text
    const approved = {
      release_tag: exactTextField(planText, "release_tag"),
      install_approval_hash: exactTextField(planText, "install_approval_hash")
    }

    const changedProfile = await handleToolCall(
      "froglet",
      {
        action: "get_install_guide",
        payment_rail: "none",
        footprint: "docker",
        ...approved
      },
      config
    )
    assert.equal(changedProfile.isError, true)
    assert.match(changedProfile.content[0].text, /does not match the current immutable release/)

    const changedBootstrapConfig = withInstallDeps({
      fetchReleaseMetadata: async (url) => releaseResponseFor(url),
      fetchBootstrap: async () => `${INSTALL_BOOTSTRAP}# changed\n`
    })
    const changedBootstrap = await handleToolCall(
      "froglet",
      {
        action: "get_install_guide",
        payment_rail: "none",
        footprint: "auto",
        ...approved
      },
      changedBootstrapConfig
    )
    assert.equal(changedBootstrap.isError, true)
    assert.match(
      changedBootstrap.content[0].text,
      /does not match the immutable GitHub asset digest/
    )
  })

  it("fails closed on mutable releases and malformed trust hashes", async () => {
    const mutable = await handleToolCall(
      "froglet",
      { action: "plan_install", payment_rail: "none" },
      withInstallDeps({
        fetchReleaseMetadata: async (url) => releaseResponseFor(
          url,
          immutableReleaseFixture({ immutable: false })
        ),
        fetchBootstrap: async () => INSTALL_BOOTSTRAP
      })
    )
    assert.equal(mutable.isError, true)
    assert.match(mutable.content[0].text, /is mutable/)

    const malformedManifest = await handleToolCall(
      "froglet",
      { action: "plan_install", payment_rail: "none" },
      withInstallDeps({
        fetchReleaseMetadata: async (url) => releaseResponseFor(
          url,
          immutableReleaseFixture({
            assets: [{ name: "release-manifest.json", digest: "sha256:ABC", state: "uploaded" }]
          })
        ),
        fetchBootstrap: async () => INSTALL_BOOTSTRAP
      })
    )
    assert.equal(malformedManifest.isError, true)
    assert.match(malformedManifest.content[0].text, /64-character lowercase SHA-256/)

    const malformedBootstrap = await handleToolCall(
      "froglet",
      { action: "plan_install", payment_rail: "none" },
      withInstallDeps({
        fetchReleaseMetadata: async (url) => releaseResponseFor(url),
        fetchBootstrap: async () => "echo unapproved-interpreter\n"
      })
    )
    assert.equal(malformedBootstrap.isError, true)
    assert.match(malformedBootstrap.content[0].text, /non-binary #!\/bin\/sh script/)

    let fetched = false
    const malformedApproval = await handleToolCall(
      "froglet",
      {
        action: "get_install_guide",
        payment_rail: "none",
        install_approval_hash: "not-a-hash"
      },
      withInstallDeps({
        fetchReleaseMetadata: async (url) => {
          fetched = true
          return releaseResponseFor(url)
        },
        fetchBootstrap: async () => INSTALL_BOOTSTRAP
      })
    )
    assert.equal(malformedApproval.isError, true)
    assert.equal(fetched, false)
  })

  it("plans post-install use cases without claiming batch or GPU support is complete", async () => {
    const batch = await handleToolCall(
      "froglet",
      {
        action: "plan_use_case",
        use_case: "run a batch queue for long-running model jobs"
      },
      config
    )
    assert.equal(batch.isError, undefined)
    assert.match(batch.content[0].text, /workload_profile: batch/)
    assert.match(batch.content[0].text, /get_task`\/`wait_task/)
    assert.match(batch.content[0].text, /true batch submission and fan-out is not yet implemented/)

    const gpu = await handleToolCall(
      "froglet",
      {
        action: "plan_use_case",
        workload_profile: "gpu",
        use_case: "run CUDA inference"
      },
      config
    )
    assert.equal(gpu.isError, undefined)
    assert.match(gpu.content[0].text, /workload_profile: gpu/)
    assert.match(gpu.content[0].text, /provider advertises GPU capability/)
    assert.match(gpu.content[0].text, /one self-hosted GCP T4 Docker workload has been verified/)
  })

  it("threads expanded install profile choices into the command guide", async () => {
    const result = await approvedInstallGuide({
      target_agent: "manual",
      payment_rail: "none",
      footprint: "docker",
      network_mode: "dual",
      role: "consumer"
    })
    assert.equal(result.isError, undefined)
    const text = result.content[0].text
    assert.match(text, /target_agent: manual/)
    assert.match(text, /payment_rail: none/)
    assert.match(text, /role: consumer/)
    assert.doesNotMatch(text, /npm ci --prefix integrations\/mcp\/froglet/)
    assert.doesNotMatch(text, /setup-agent\.sh --target manual/)
    assert.match(text, /FROGLET_AGENT_TARGET='manual'/)
    assert.match(text, /FROGLET_BOOTSTRAP_MODE='docker'/)
    assert.match(text, /FROGLET_NETWORK_MODE='dual'/)
    assert.match(text, /Manual target selected/)
  })

  it("rejects unknown target_agent or payment_rail with a clear error", async () => {
    const badAgent = await handleToolCall(
      "froglet",
      { action: "get_install_guide", target_agent: "emacs" },
      config
    )
    assert.equal(badAgent.isError, true)
    assert.match(badAgent.content[0].text, /target_agent must be one of/)

    const badRail = await handleToolCall(
      "froglet",
      { action: "get_install_guide", payment_rail: "gold-pieces" },
      config
    )
    assert.equal(badRail.isError, true)
    assert.match(badRail.content[0].text, /payment_rail must be one of/)

    const badNetwork = await handleToolCall(
      "froglet",
      { action: "plan_install", network_mode: "carrier-pigeon" },
      config
    )
    assert.equal(badNetwork.isError, true)
    assert.match(badNetwork.content[0].text, /network_mode must be one of/)
  })

  it("keeps public and compatibility install paths approval-gated", async () => {
    const result = await approvedInstallGuide({ payment_rail: "none" })
    const text = result.content[0].text
    const steps = text
      .split("\n")
      .filter((line) => /^\s*\d+\.\s+/.test(line))
      .map((line) => line.replace(/^\s*\d+\.\s+/, "").trim())
    assert.equal(steps.length, 1, `expected 1 command, got ${steps.length}`)

    assert.match(
      steps[0],
      /github\.com\/armanas\/froglet\/releases\/download\/v0\.1\.0-alpha\.1\/agent-bootstrap\.sh/
    )
    assert.match(steps[0], /approved bootstrap SHA-256 mismatch/)
    assert.match(steps[0], /sh "\$bootstrap_tmp" plan/)
    assert.match(steps[0], /sh "\$bootstrap_tmp" execute "\$native_approval_hash"/)
    assert.doesNotMatch(steps[0], /froglet\.dev\/agent/)

    const readme = await readFile(join(REPO_ROOT, "README.md"), "utf8")
    assert.match(readme, /sh "\$bootstrap" plan/)
    assert.match(readme, /sh "\$bootstrap" execute '<install_approval_hash>'/)
    assert.doesNotMatch(readme, /froglet\.dev\/agent \| bash/)

    const quickstart = await readFile(
      join(REPO_ROOT, "docs-site/src/content/docs/learn/quickstart.mdx"),
      "utf8"
    )
    assert.match(quickstart, /sh "\$bootstrap" plan/)
    assert.match(quickstart, /sh "\$bootstrap" execute '<install_approval_hash>'/)
    assert.doesNotMatch(quickstart, /froglet\.dev\/agent \| bash/)

    const landingPage = await readFile(
      join(REPO_ROOT, "docs-site/src/pages/index.astro"),
      "utf8"
    )
    assert.match(landingPage, /href="\/open-source\/"/)
    // The Developers page sends people to the setup guide, which carries the approval-gated commands checked above,
    // instead of embedding the interactive configurator. The component and its script stay covered below, so an
    // unsafe command cannot come back if the configurator is mounted on a page again.
    const developersPage = await readFile(join(REPO_ROOT, "docs-site/src/pages/open-source.astro"), "utf8")
    assert.match(developersPage, /\['Run it locally', '\/learn\/quickstart\/'\]/)
    assert.doesNotMatch(developersPage, /froglet\.dev\/agent \| bash/)
    assert.doesNotMatch(developersPage, /git clone https:\/\/github\.com\/AlitheaBio\/froglet\.git/)
    const operatorSetup = await readFile(join(REPO_ROOT, "docs-site/src/components/OperatorSetup.astro"), "utf8")
    assert.match(operatorSetup, /initSelfHostConfigurator\(\)/)
    assert.match(operatorSetup, /self-host-card/)

    const configurator = await readFile(
      join(REPO_ROOT, "docs-site/src/scripts/self-host-configurator.ts"),
      "utf8"
    )
    assert.match(configurator, /releases\/download\/\$tag\/agent-bootstrap\.sh/)
    assert.match(configurator, /X-GitHub-Api-Version: 2026-03-10/)
    assert.match(configurator, /\$\{bootstrapEnv\}sh "\$bootstrap" plan/)
    assert.doesNotMatch(configurator, /git clone https:\/\/github\.com\/AlitheaBio\/froglet\.git/)
    assert.doesNotMatch(configurator, /npm ci --prefix integrations\/mcp\/froglet/)
    assert.doesNotMatch(configurator, /docker compose up --build -d/)
    assert.doesNotMatch(configurator, /FROGLET_SETTLEMENT_METHOD/)
  })

  it("keeps the default install command independent of a repo clone", async () => {
    const result = await approvedInstallGuide({ payment_rail: "none" })
    const text = result.content[0].text
    const steps = text
      .split("\n")
      .filter((line) => /^\s*\d+\.\s+/.test(line))
      .map((line) => line.replace(/^\s*\d+\.\s+/, "").trim())

    assert.equal(steps.length, 1)
    assert.doesNotMatch(steps[0], /git clone/)
    assert.match(steps[0], /FROGLET_BOOTSTRAP_MODE='auto'/)
  })

  it("maps native and Docker footprints into the approved bootstrap environment", async () => {
    for (const [footprint, expected] of [
      ["native", "FROGLET_BOOTSTRAP_MODE='native'"],
      ["docker", "FROGLET_BOOTSTRAP_MODE='docker'"]
    ]) {
      const result = await approvedInstallGuide({ payment_rail: "none", footprint })
      const text = result.content[0].text
      assert.match(text, new RegExp(expected))
      assert.match(text, /approved bootstrap SHA-256 mismatch/)
    }
  })

  it("passes every paid no-clone rail through exact host-environment references", async () => {
    const expectations = [
      ["lightning-mock", /FROGLET_PAYMENT_BACKEND='lightning' FROGLET_LIGHTNING_MODE='mock'/],
      ["lightning-lnd-rest", /FROGLET_LIGHTNING_REST_URL="\$\{FROGLET_LIGHTNING_REST_URL:\?set/],
      ["lightning-phoenixd", /FROGLET_LIGHTNING_PHOENIXD_HTTP_PASSWORD="\$\{FROGLET_LIGHTNING_PHOENIXD_HTTP_PASSWORD:\?set/],
      ["stripe-test", /FROGLET_STRIPE_SECRET_KEY="\$\{FROGLET_STRIPE_SECRET_KEY:\?set/],
      ["stripe-live", /FROGLET_STRIPE_LIVE_CONFIRM='fresh'/],
      ["x402", /FROGLET_X402_WALLET_ADDRESS="\$\{FROGLET_X402_WALLET_ADDRESS:\?set/]
    ]
    for (const [paymentRail, expected] of expectations) {
      const result = await approvedInstallGuide({ payment_rail: paymentRail, footprint: "native" })
      const command = result.content[0].text
        .split("\n")
        .find((line) => /^\s*1\.\s+/.test(line))
      assert.match(command, /^\s*1\. bootstrap_tmp=/)
      assert.match(command, expected)
      if (paymentRail === "x402") {
        assert.match(command, /FROGLET_X402_FACILITATOR_URL="\$\{FROGLET_X402_FACILITATOR_URL:\?set/)
      }
      assert.doesNotMatch(command, /<[^>]+>/)
    }
  })

  it("repo-local onboarding helpers work from a clean temp checkout", async () => {
    const checkoutDir = join(tmpDir, "clean-checkout")
    const { stdout } = await execFile("git", ["ls-files", "-z"], {
      cwd: REPO_ROOT,
      encoding: "buffer"
    })
    const paths = stdout
      .toString("utf8")
      .split("\u0000")
      .filter(Boolean)

    for (const relPath of paths) {
      const sourcePath = join(REPO_ROOT, relPath)
      const destPath = join(checkoutDir, relPath)
      await mkdir(dirname(destPath), { recursive: true })
      try {
        await copyFile(sourcePath, destPath)
      } catch (error) {
        if (error?.code === "ENOENT") {
          continue
        }
        throw error
      }
    }

    // The checked-in config is existing user state from setup's perspective.
    // Repository-only setup without the native merger must preserve it.
    // A full qualification run can supply a native binary to other tests.
    // Isolate this fixture's repo-only mode without changing the parent env.
    const setupEnv = Object.fromEntries(
      Object.entries(process.env).filter(([name]) => !name.startsWith("FROGLET_"))
    )
    setupEnv.FROGLET_MCP_MODE = "repo"
    const existingConfigPath = join(checkoutDir, ".mcp.json")
    const existingConfig = JSON.parse(await readFile(existingConfigPath, "utf8"))
    existingConfig.mcpServers.unrelatedFixtureServer = {
      command: "printf", args: ["preserve unrelated server configuration"]
    }
    const originalConfig = `${JSON.stringify(existingConfig, null, 2)}\n`
    await writeFile(existingConfigPath, originalConfig)
    await assert.rejects(execFile("bash", ["scripts/setup-agent.sh", "--target", "claude-code"], {
      cwd: checkoutDir, env: setupEnv, timeout: 10000
    }), /existing agent config requires/)
    assert.equal(await readFile(existingConfigPath, "utf8"), originalConfig)
    const generatedConfig = join(checkoutDir, ".mcp-generated.json")
    await execFile("bash", ["scripts/setup-agent.sh", "--target", "claude-code", "--out", generatedConfig], {
      cwd: checkoutDir, env: setupEnv, timeout: 10000
    })
    assert.equal(await readFile(existingConfigPath, "utf8"), originalConfig)
    await execFile("bash", ["scripts/setup-payment.sh", "lightning"], {
      cwd: checkoutDir, env: setupEnv
    })

    const mcpConfig = JSON.parse(await readFile(generatedConfig, "utf8"))
    assert.equal(mcpConfig.mcpServers.froglet.command, "node")

    const paymentEnv = await readFile(
      join(checkoutDir, ".froglet/payment/lightning.env"),
      "utf8"
    )
    assert.match(paymentEnv, /FROGLET_PAYMENT_BACKEND=lightning/)
    assert.match(paymentEnv, /FROGLET_LIGHTNING_MODE=mock/)

    try {
      await execFile(
        "bash",
        [
          "-lc",
          "set -a && . ./.froglet/payment/lightning.env && export FROGLET_HOST_READABLE_CONTROL_TOKEN=true && set +a && docker compose config >/dev/null"
        ],
        { cwd: checkoutDir, env: setupEnv }
      )
    } catch (error) {
      const message = `${error?.stderr ?? ""}${error?.stdout ?? ""}${error}`
      if (/docker: command not found|Cannot connect to the Docker daemon|docker compose/i.test(message)) {
        return
      }
      throw error
    }
  })

  it("compose config honors selected payment and network env", async () => {
    try {
      const { stdout } = await execFile(
        "bash",
        [
          "-lc",
          "FROGLET_PAYMENT_BACKEND=stripe FROGLET_STRIPE_SECRET_KEY=stripe-test-placeholder FROGLET_NETWORK_MODE=dual FROGLET_MARKETPLACE_URL=https://marketplace.froglet.dev docker compose config"
        ],
        { cwd: REPO_ROOT }
      )
      assert.match(stdout, /FROGLET_PAYMENT_BACKEND: stripe/)
      assert.match(stdout, /FROGLET_NETWORK_MODE: dual/)
      assert.match(stdout, /FROGLET_MARKETPLACE_URL: https:\/\/marketplace\.froglet\.dev/)
    } catch (error) {
      const message = `${error?.stderr ?? ""}${error?.stdout ?? ""}${error}`
      if (/docker: command not found|Cannot connect to the Docker daemon|docker compose/i.test(message)) {
        return
      }
      throw error
    }
  })

  it("setup-payment writes private env snippets and rejects control characters", async () => {
    const dir = await mkdtemp(join(tmpdir(), "froglet-payment-env-"))
    const goodOut = join(dir, "stripe.env")
    await execFile(
      "bash",
      ["scripts/setup-payment.sh", "stripe", "--out", goodOut, "--no-verify"],
      {
        cwd: REPO_ROOT,
        env: {
          ...process.env,
          FROGLET_STRIPE_SECRET_KEY: "sk_test_"
        }
      }
    )

    const fileMode = (await stat(goodOut)).mode & 0o777
    assert.equal(fileMode, 0o600)
    const paymentEnv = await readFile(goodOut, "utf8")
    assert.match(paymentEnv, /FROGLET_STRIPE_SECRET_KEY=sk_test_/)

    const badOut = join(dir, "bad.env")
    await assert.rejects(
      () =>
        execFile(
          "bash",
          ["scripts/setup-payment.sh", "stripe", "--out", badOut, "--no-verify"],
          {
            cwd: REPO_ROOT,
            env: {
              ...process.env,
              FROGLET_STRIPE_SECRET_KEY: "sk_test_",
              FROGLET_STRIPE_API_VERSION: "2026-preview\nFROGLET_PAYMENT_BACKEND=none"
            }
          }
        ),
      /control characters/
    )
  })

  it("marketplace_search routes through invoke_service with marketplace.search", async () => {
    // Natural flow: no provider_url in args (that would trigger the
    // LLM-controlled-URL validator + pinned fetch, which bypasses the
    // global.fetch mock). Discovery resolves the marketplace provider
    // through runtime search, matching how the operator-configured
    // FROGLET_MARKETPLACE_URL is exposed in production.
    let capturedBody
    const restore = mockFetch(async (url, opts) => {
      const urlStr = String(url)
      if (urlStr.endsWith("/v1/runtime/search")) {
        return new Response(
          JSON.stringify({
            providers: [
              {
                provider_id: "prov-mkt",
                descriptor_hash: "desc-mkt",
                transport_endpoints: [
                  { uri: "https://mkt.example", features: ["quote_http"], priority: 1 }
                ],
                offers: [
                  {
                    offer_hash: "marketplace-search-hash",
                    service_id: "marketplace.search",
                    offer_id: "marketplace.search",
                    offer_kind: "builtin",
                    runtime: "builtin"
                  }
                ],
                last_seen_at: "2026-04-16T00:00:00Z"
              }
            ]
          }),
          { status: 200 }
        )
      }
      if (urlStr === "https://mkt.example/v1/provider/services/marketplace.search") {
        return new Response(
          JSON.stringify({
            service: {
              service_id: "marketplace.search",
              offer_id: "marketplace.search",
              provider_id: "prov-mkt",
              runtime: "builtin",
              package_kind: "builtin"
            }
          }),
          { status: 200 }
        )
      }
      if (urlStr.endsWith("/v1/runtime/deals")) {
        capturedBody = JSON.parse(opts.body)
        return new Response(
          JSON.stringify({
            provider_id: "prov-mkt",
            provider_url: "https://mkt.example",
            deal: { status: "succeeded", result: { providers: [{ provider_id: "prov-9" }] } },
            result: { providers: [{ provider_id: "prov-9" }], has_more: false }
          }),
          { status: 200 }
        )
      }
      throw new Error(`unexpected URL: ${urlStr}`)
    })
    try {
      const result = await handleToolCall(
        "froglet",
        {
          action: "marketplace_search",
          offer_kind: "named.v1",
          runtime: "python",
          limit: 25
        },
        config
      )
      assert.equal(result.isError, undefined, result.content?.[0]?.text)
      assert.ok(capturedBody, "expected a runtime deal invocation")
      assert.equal(capturedBody.execution.workload_kind, "marketplace.search")
      assert.equal(capturedBody.execution.builtin_name, "marketplace.search")
      assert.deepEqual(capturedBody.execution.entrypoint, { kind: "builtin", value: "marketplace.search" })
      assert.deepEqual(capturedBody.execution.security, { mode: "standard" })
      assert.equal(capturedBody.execution.module_hash, undefined)
      assert.equal(capturedBody.execution.source_hash, undefined)
      assert.deepEqual(capturedBody.execution.input, {
        offer_kind: "named.v1",
        runtime: "python",
        limit: 25
      })
    } finally {
      restore()
    }
  })

  it("marketplace_register posts a public provider URL to the marketplace API", async () => {
    let capturedBody
    const restore = mockFetch(async (url, opts) => {
      const urlStr = String(url)
      if (urlStr === "https://mkt.example/v1/registrations") {
        capturedBody = JSON.parse(opts.body)
        return new Response(
          JSON.stringify({
            status: "active",
            provider_id: "prov-registered",
            provider_url: "https://1.1.1.1",
            descriptor_hash: "desc-registered",
            offers_seen: 2,
            already_registered: false
          }),
          { status: 201 }
        )
      }
      throw new Error(`unexpected URL: ${urlStr}`)
    })
    try {
      const result = await handleToolCall(
        "froglet",
        {
          action: "marketplace_register",
          provider_url: "https://1.1.1.1"
        },
        config
      )
      assert.equal(result.isError, undefined, result.content?.[0]?.text)
      assert.deepEqual(capturedBody, { provider_url: "https://1.1.1.1", transport: "clearnet" })
      const text = result.content[0].text
      assert.match(text, /status: active/)
      assert.match(text, /provider_id: prov-registered/)
      assert.match(text, /offers_seen: 2/)
    } finally {
      restore()
    }
  })

  it("marketplace_register posts a Tor onion provider URL to the marketplace API", async () => {
    const onionHost = `${"a".repeat(56)}.onion`
    let capturedBody
    const restore = mockFetch(async (url, opts) => {
      const urlStr = String(url)
      if (urlStr === "https://mkt.example/v1/registrations") {
        capturedBody = JSON.parse(opts.body)
        return new Response(
          JSON.stringify({
            status: "active",
            provider_id: "prov-onion",
            provider_url: `http://${onionHost}`,
            transport: "tor",
            descriptor_hash: "desc-onion",
            offers_seen: 1,
            already_registered: false
          }),
          { status: 201 }
        )
      }
      throw new Error(`unexpected URL: ${urlStr}`)
    })
    try {
      const result = await handleToolCall(
        "froglet",
        {
          action: "marketplace_register",
          provider_url: `http://${onionHost}`,
          registration_transport: "tor"
        },
        config
      )
      assert.equal(result.isError, undefined, result.content?.[0]?.text)
      assert.deepEqual(capturedBody, {
        provider_url: `http://${onionHost}`,
        transport: "tor"
      })
      assert.match(result.content[0].text, /transport: tor/)
    } finally {
      restore()
    }
  })

  it("marketplace_domain_claim creates a managed subdomain claim", async () => {
    let capturedBody
    const seen = []
    const restore = mockFetch(async (url, opts) => {
      const urlStr = String(url)
      seen.push(urlStr)
      if (urlStr === "http://127.0.0.1:8080/v1/provider/domain-claims/sign") {
        assert.equal(opts.headers.Authorization, "Bearer test-token-abc")
        assert.deepEqual(JSON.parse(opts.body), {
          signing_message:
            "froglet-provider-domain-claim-intent-v1\n" +
            `provider_id:${"a".repeat(64)}\n` +
            "slug:demo-1\n" +
            "hostname:demo-1.providers.froglet.dev\n" +
            "public_ip:8.8.8.8"
        })
        return new Response(
          JSON.stringify({
            provider_id: "a".repeat(64),
            signature: "f".repeat(128)
          }),
          { status: 200 }
        )
      }
      if (urlStr === "https://mkt.example/v1/provider-domains/claims") {
        capturedBody = JSON.parse(opts.body)
        return new Response(
          JSON.stringify({
            status: "pending_signature",
            claim_id: "claim-1",
            provider_id: capturedBody.provider_id,
            hostname: "demo-1.providers.froglet.dev",
            public_ip: capturedBody.public_ip,
            signing_message: "froglet-provider-domain-claim-v1\nclaim_id:claim-1",
            expires_at: "2026-05-07T12:00:00Z"
          }),
          { status: 200 }
        )
      }
      throw new Error(`unexpected URL: ${urlStr}`)
    })
    try {
      const result = await handleToolCall(
        "froglet",
        {
          action: "marketplace_domain_claim",
          provider_id: "a".repeat(64),
          requested_slug: "demo-1",
          public_ip: "8.8.8.8"
        },
        config
      )
      assert.equal(result.isError, undefined, result.content?.[0]?.text)
      assert.deepEqual(capturedBody, {
        provider_id: "a".repeat(64),
        requested_slug: "demo-1",
        public_ip: "8.8.8.8",
        intent_signature: "f".repeat(128)
      })
      assert.deepEqual(seen, [
        "http://127.0.0.1:8080/v1/provider/domain-claims/sign",
        "https://mkt.example/v1/provider-domains/claims"
      ])
      assert.match(result.content[0].text, /claim_id: claim-1/)
      assert.match(result.content[0].text, /hostname: demo-1\.providers\.froglet\.dev/)
    } finally {
      restore()
    }
  })

  it("marketplace_domain_complete signs locally and completes the claim", async () => {
    const seen = []
    const restore = mockFetch(async (url, opts = {}) => {
      const urlStr = String(url)
      seen.push(urlStr)
      if (urlStr === "http://127.0.0.1:8080/v1/provider/domain-claims/sign") {
        assert.equal(opts.headers.Authorization, "Bearer test-token-abc")
        assert.deepEqual(JSON.parse(opts.body), {
          signing_message: "froglet-provider-domain-claim-v1\nclaim_id:claim-1"
        })
        return new Response(
          JSON.stringify({
            provider_id: "a".repeat(64),
            signature: "b".repeat(128)
          }),
          { status: 200 }
        )
      }
      if (urlStr === "https://mkt.example/v1/provider-domains/claims/claim-1/complete") {
        assert.deepEqual(JSON.parse(opts.body), {
          signature: "b".repeat(128)
        })
        return new Response(
          JSON.stringify({
            status: "pending_operator_dns",
            provider_id: "a".repeat(64),
            hostname: "demo-1.providers.froglet.dev",
            public_ip: "8.8.8.8",
            dns_required: true
          }),
          { status: 200 }
        )
      }
      throw new Error(`unexpected URL: ${urlStr}`)
    })
    try {
      const result = await handleToolCall(
        "froglet",
        {
          action: "marketplace_domain_complete",
          claim_id: "claim-1",
          signing_message: "froglet-provider-domain-claim-v1\nclaim_id:claim-1"
        },
        config
      )
      assert.equal(result.isError, undefined, result.content?.[0]?.text)
      assert.deepEqual(seen, [
        "http://127.0.0.1:8080/v1/provider/domain-claims/sign",
        "https://mkt.example/v1/provider-domains/claims/claim-1/complete"
      ])
      assert.match(result.content[0].text, /dns_required: true/)
    } finally {
      restore()
    }
  })

  it("legacy marketplace_stake returns a local disabled-action error", async () => {
    const result = await handleToolCall(
      "froglet",
      { action: "marketplace_stake", amount_msat: 1000 },
      config
    )
    assert.equal(result.isError, true)
    assert.match(result.content[0].text, /marketplace_stake is not available/)
  })

  it("legacy marketplace_topup returns a local disabled-action error", async () => {
    const result = await handleToolCall(
      "froglet",
      {
        action: "marketplace_topup",
        marketplace_provider_id: "prov-1",
        amount_msat: 1000
      },
      config
    )
    assert.equal(result.isError, true)
    assert.match(result.content[0].text, /marketplace_topup is not available/)
  })

  it("marketplace_file_complaint posts a public complaint to the arbiter", async () => {
    let capturedBody
    const restore = mockFetch(async (url, opts) => {
      const urlStr = String(url)
      if (urlStr === "https://arbiter.example/v1/complaints") {
        capturedBody = JSON.parse(opts.body)
        return new Response(
          JSON.stringify({
            complaint_id: "cmp-1",
            provider_id: "prov-9",
            deal_id: "deal-9",
            receipt_hash: "receipt-9",
            complainant_id: "requester-1",
            reason: "receipt did not verify",
            evidence: [{ artifact_hash: "receipt-9" }],
            status: "open",
            created_at: 1770000000
          }),
          { status: 201 }
        )
      }
      throw new Error(`unexpected URL: ${urlStr}`)
    })
    try {
      const result = await handleToolCall(
        "froglet",
        {
          action: "marketplace_file_complaint",
          marketplace_provider_id: "prov-9",
          deal_id: "deal-9",
          receipt_hash: "receipt-9",
          complainant_id: "requester-1",
          reason: "receipt did not verify",
          evidence: [{ artifact_hash: "receipt-9" }]
        },
        config
      )
      assert.equal(result.isError, undefined, result.content?.[0]?.text)
      assert.deepEqual(capturedBody, {
        provider_id: "prov-9",
        deal_id: "deal-9",
        receipt_hash: "receipt-9",
        complainant_id: "requester-1",
        reason: "receipt did not verify",
        evidence: [{ artifact_hash: "receipt-9" }]
      })
      assert.match(result.content[0].text, /complaint_id: cmp-1/)
      assert.match(result.content[0].text, /status: open/)
    } finally {
      restore()
    }
  })

  it("marketplace_get_complaint reads complaint status from the arbiter", async () => {
    const restore = mockFetch(async (url) => {
      const urlStr = String(url)
      if (urlStr === "https://arbiter.example/v1/complaints/cmp-1") {
        return new Response(
          JSON.stringify({
            complaint: {
              complaint_id: "cmp-1",
              provider_id: "prov-9",
              deal_id: "deal-9",
              reason: "receipt did not verify",
              evidence: [],
              status: "upheld",
              created_at: 1770000000
            },
            verdicts: [
              {
                verdict_id: "vrd-1",
                complaint_id: "cmp-1",
                provider_id: "prov-9",
                verdict: "upheld",
                remedy: "suspend_provider",
                reason: "bad receipt",
                operator_id: "operator",
                created_at: 1770000001
              }
            ]
          }),
          { status: 200 }
        )
      }
      throw new Error(`unexpected URL: ${urlStr}`)
    })
    try {
      const result = await handleToolCall(
        "froglet",
        {
          action: "marketplace_get_complaint",
          complaint_id: "cmp-1"
        },
        config
      )
      assert.equal(result.isError, undefined, result.content?.[0]?.text)
      assert.match(result.content[0].text, /status: upheld/)
      assert.match(result.content[0].text, /latest_remedy: suspend_provider/)
    } finally {
      restore()
    }
  })

  it("marketplace_file_complaint degrades to arbiter_unavailable on a dark endpoint 404", async () => {
    const restore = mockFetch(async () => new Response(null, { status: 404 }))
    try {
      const result = await handleToolCall(
        "froglet",
        {
          action: "marketplace_file_complaint",
          marketplace_provider_id: "prov-9",
          deal_id: "deal-9",
          reason: "receipt did not verify"
        },
        config
      )
      assert.equal(result.isError, undefined, result.content?.[0]?.text)
      assert.match(result.content[0].text, /status: arbiter_unavailable/)
      assert.match(result.content[0].text, /NOT recorded/)
    } finally {
      restore()
    }
  })

  it("marketplace_get_complaint degrades to arbiter_unavailable on connection failure", async () => {
    const restore = mockFetch(async () => {
      throw new TypeError("fetch failed")
    })
    try {
      const result = await handleToolCall(
        "froglet",
        { action: "marketplace_get_complaint", complaint_id: "cmp-1" },
        config
      )
      assert.equal(result.isError, undefined, result.content?.[0]?.text)
      assert.match(result.content[0].text, /status: arbiter_unavailable/)
    } finally {
      restore()
    }
  })

  it("marketplace_get_complaint keeps a live arbiter's JSON 404 as a normal error", async () => {
    const restore = mockFetch(
      async () =>
        new Response(JSON.stringify({ error: "complaint not found" }), { status: 404 })
    )
    try {
      const result = await handleToolCall(
        "froglet",
        { action: "marketplace_get_complaint", complaint_id: "cmp-missing" },
        config
      )
      assert.equal(result.isError, true)
      assert.match(result.content[0].text, /404/)
      assert.doesNotMatch(result.content[0].text, /arbiter_unavailable/)
    } finally {
      restore()
    }
  })

  it("returns isError when get_payment_intent is called without a deal_id", async () => {
    const result = await handleToolCall(
      "froglet",
      { action: "get_payment_intent" },
      config
    )
    assert.equal(result.isError, true)
    assert.match(result.content[0].text, /deal_id is required/)
  })

  it("formats dual-health status output", async () => {
    const restore = mockFetch(async (url) => {
      const urlStr = String(url)
      if (urlStr === "http://127.0.0.1:8080/health") {
        return new Response(JSON.stringify({ healthy: true }))
      }
      if (urlStr === "http://127.0.0.1:8080/v1/node/capabilities") {
        return new Response(JSON.stringify({ compute_offer_ids: ["execute.compute"] }))
      }
      if (urlStr === "http://127.0.0.1:8080/v1/node/identity") {
        return new Response(JSON.stringify({ node_id: "node-1", discovery: { mode: "marketplace" } }))
      }
      if (urlStr === "http://127.0.0.1:8081/health") {
        return new Response(JSON.stringify({ status: "ok" }))
      }
      throw new Error(`unexpected path: ${urlStr}`)
    })
    try {
      const result = await handleToolCall("froglet", { action: "status" }, config)
      const text = result.content[0].text
      assert.match(text, /healthy: true/)
      assert.match(text, /provider_healthy: true/)
      assert.match(text, /runtime_healthy: true/)
      assert.match(text, /node_id: node-1/)
      assert.equal(result.isError, undefined)
    } finally {
      restore()
    }
  })

  it("discovers marketplace providers and renders flattened services", async () => {
    const restore = mockFetch(async (url, opts) => {
      assert.equal(String(url), "http://127.0.0.1:8081/v1/runtime/search")
      assert.deepEqual(JSON.parse(opts.body), { limit: 5, include_inactive: false })
      return new Response(JSON.stringify(providerSearchResponse()))
    })
    try {
      const result = await handleToolCall(
        "froglet",
        { action: "discover_services", limit: 5, query: "svc-1" },
        config
      )
      const text = result.content[0].text
      assert.match(text, /providers: 1/)
      assert.match(text, /services: 1/)
      assert.match(text, /service_id: svc-1/)
      assert.match(text, /provider_id: prov-1/)
    } finally {
      restore()
    }
  })

  it("invokes services through runtime deals after provider lookup and public service fetch", async () => {
    let runtimeDealBody = null
    const restore = mockFetch(async (url, opts) => {
      const urlStr = String(url)
      if (urlStr === "http://127.0.0.1:8081/v1/runtime/providers/prov-1") {
        return new Response(
          JSON.stringify({
            provider: {
              provider_id: "prov-1",
              transport_endpoints: [
                { uri: "https://provider.example", features: ["quote_http"], priority: 1 }
              ]
            }
          })
        )
      }
      if (urlStr === "https://provider.example/v1/provider/services/svc-1") {
        return new Response(
          JSON.stringify({
            service: {
              service_id: "svc-1",
              offer_id: "svc-1",
              provider_id: "prov-1",
              runtime: "python",
              package_kind: "inline_source",
              binding_hash: "feedface"
            }
          })
        )
      }
      if (urlStr === "http://127.0.0.1:8081/v1/runtime/deals") {
        runtimeDealBody = JSON.parse(opts.body)
        return new Response(
          JSON.stringify({
            provider_id: "prov-1",
            provider_url: "https://provider.example",
            quote: { hash: "quote-hash" },
            deal: { deal_id: "deal-1", status: "succeeded", result: "world" }
          })
        )
      }
      throw new Error(`unexpected URL: ${urlStr}`)
    })
    try {
      const result = await handleToolCall(
        "froglet",
        {
          action: "invoke_service",
          provider_id: "prov-1",
          service_id: "svc-1",
          input: { text: "hello" }
        },
        config
      )
      const text = result.content[0].text
      assert.match(text, /status: succeeded/)
      assert.match(text, /result: "world"/)
      assert.equal(runtimeDealBody.offer_id, "svc-1")
      assert.equal(runtimeDealBody.kind, "execution")
      assert.equal(runtimeDealBody.execution.security.service_id, "svc-1")
    } finally {
      restore()
    }
  })

  it("rejects invoke_service when provider_id discovery advertises a private URL", async () => {
    const restore = mockFetch(async (url) => {
      const urlStr = String(url)
      if (urlStr === "http://127.0.0.1:8081/v1/runtime/providers/prov-private") {
        return new Response(
          JSON.stringify({
            provider: {
              provider_id: "prov-private",
              transport_endpoints: [
                { uri: "https://[::ffff:7f00:1]", features: ["quote_http"], priority: 1 }
              ]
            }
          })
        )
      }
      throw new Error(`unexpected URL: ${urlStr}`)
    })
    try {
      const result = await handleToolCall(
        "froglet",
        {
          action: "invoke_service",
          provider_id: "prov-private",
          service_id: "svc-private"
        },
        config
      )
      assert.equal(result.isError, true)
      assert.match(result.content[0].text, /local or private address/)
    } finally {
      restore()
    }
  })

  it("invokes local services with the operator-configured provider fallback", async () => {
    let runtimeDealBody = null
    const restore = mockFetch(async (url, opts) => {
      const urlStr = String(url)
      if (urlStr === "http://127.0.0.1:8081/v1/runtime/providers/local-provider") {
        return new Response(JSON.stringify({ error: "provider not found" }), { status: 404 })
      }
      if (urlStr === "http://127.0.0.1:8080/v1/node/identity") {
        return new Response(JSON.stringify({ node_id: "local-provider" }))
      }
      if (urlStr === "http://127.0.0.1:8080/v1/provider/services/demo.add.local") {
        return new Response(
          JSON.stringify({
            service: {
              service_id: "demo.add.local",
              offer_id: "demo.add.local",
              provider_id: "local-provider",
              runtime: "python",
              package_kind: "inline_source",
              binding_hash: "feedface"
            }
          })
        )
      }
      if (urlStr === "http://127.0.0.1:8081/v1/runtime/deals") {
        runtimeDealBody = JSON.parse(opts.body)
        return new Response(
          JSON.stringify({
            provider_id: "local-provider",
            provider_url: "http://127.0.0.1:8080",
            quote: { hash: "quote-hash" },
            deal: { deal_id: "deal-local", status: "succeeded", result: { sum: 12 } }
          })
        )
      }
      throw new Error(`unexpected URL: ${urlStr}`)
    })
    try {
      const result = await handleToolCall(
        "froglet",
        {
          action: "invoke_service",
          provider_id: "local-provider",
          service_id: "demo.add.local",
          input: { a: 7, b: 5 }
        },
        config
      )
      const text = result.content[0].text
      assert.match(text, /status: succeeded/)
      assert.match(text, /result: {"sum":12}/)
      assert.deepEqual(runtimeDealBody.provider, {
        provider_id: "local-provider",
        provider_url: "http://127.0.0.1:8080"
      })
      assert.equal(runtimeDealBody.offer_id, "demo.add.local")
      assert.equal(runtimeDealBody.kind, "execution")
    } finally {
      restore()
    }
  })

  it("runs compute through runtime deals", async () => {
    let runtimeDealBody = null
    const restore = mockFetch(async (url, opts) => {
      const urlStr = String(url)
      if (urlStr === "http://127.0.0.1:8081/v1/runtime/providers/prov-1") {
        return new Response(JSON.stringify({ error: "provider not found" }), { status: 404 })
      }
      if (urlStr === "http://127.0.0.1:8080/v1/node/identity") {
        return new Response(JSON.stringify({ node_id: "prov-1" }))
      }
      if (urlStr === "http://127.0.0.1:8081/v1/runtime/deals") {
        runtimeDealBody = JSON.parse(opts.body)
        return new Response(
          JSON.stringify({
            provider_id: "prov-1",
            provider_url: "https://provider.example",
            quote: { hash: "quote-hash" },
            deal: { deal_id: "deal-2", status: "succeeded", result: { ok: true } }
          })
        )
      }
      throw new Error(`unexpected URL: ${urlStr}`)
    })
    try {
      // Use a public https literal so the LLM-controlled provider_url
      // validator passes without real DNS; the runtime call is intercepted by
      // mockFetch, so no outbound traffic reaches 1.1.1.1.
      const result = await handleToolCall(
        "froglet",
        {
          action: "run_compute",
          provider_id: "prov-1",
          provider_url: "https://1.1.1.1",
          runtime: "wasm",
          package_kind: "inline_module",
          wasm_module_hex: "0061736d01000000"
        },
        config
      )
      const text = result.content[0].text
      assert.match(text, /status: succeeded/)
      assert.match(text, /result: {"ok":true}/)
      assert.equal(runtimeDealBody.kind, "wasm")
      assert.equal(runtimeDealBody.offer_id, "execute.compute")
    } finally {
      restore()
    }
  })

  it("runs compute through runtime deals with the operator-configured provider fallback", async () => {
    let runtimeDealBody = null
    const restore = mockFetch(async (url, opts) => {
      const urlStr = String(url)
      if (urlStr === "http://127.0.0.1:8081/v1/runtime/providers/prov-1") {
        return new Response(JSON.stringify({ error: "provider not found" }), { status: 404 })
      }
      if (urlStr === "http://127.0.0.1:8080/v1/node/identity") {
        return new Response(JSON.stringify({ node_id: "prov-1" }))
      }
      if (urlStr === "http://127.0.0.1:8081/v1/runtime/deals") {
        runtimeDealBody = JSON.parse(opts.body)
        return new Response(
          JSON.stringify({
            provider_url: "http://127.0.0.1:8080",
            quote: { hash: "quote-hash" },
            deal: { deal_id: "deal-2b", status: "succeeded", result: { ok: true } }
          })
        )
      }
      throw new Error(`unexpected URL: ${urlStr}`)
    })
    try {
      const result = await handleToolCall(
        "froglet",
        {
          action: "run_compute",
          provider_id: "prov-1",
          runtime: "wasm",
          package_kind: "inline_module",
          wasm_module_hex: "0061736d01000000"
        },
        config
      )
      const text = result.content[0].text
      assert.match(text, /status: succeeded/)
      assert.match(text, /result: {"ok":true}/)
      assert.deepEqual(runtimeDealBody.provider, {
        provider_id: "prov-1",
        provider_url: "http://127.0.0.1:8080"
      })
      assert.equal(runtimeDealBody.kind, "wasm")
      assert.equal(runtimeDealBody.offer_id, "execute.compute")
    } finally {
      restore()
    }
  })

  it("reads runtime deals first and falls back to provider jobs on a shared API base", async () => {
    const sharedSurfaceConfig = { ...config, runtimeUrl: config.providerUrl }
    const restore = mockFetch(async (url) => {
      const urlStr = String(url)
      if (urlStr === "http://127.0.0.1:8080/v1/runtime/deals/task-1") {
        return new Response(JSON.stringify({ error: "deal not found" }), { status: 404 })
      }
      if (urlStr === "http://127.0.0.1:8080/v1/node/jobs/task-1") {
        return new Response(JSON.stringify({ task: { task_id: "task-1", state: "completed", result: 42 } }))
      }
      throw new Error(`unexpected URL: ${urlStr}`)
    })
    try {
      const result = await handleToolCall(
        "froglet",
        { action: "get_task", task_id: "task-1" },
        sharedSurfaceConfig
      )
      const text = result.content[0].text
      assert.match(text, /task_id: task-1/)
      assert.match(text, /result: 42/)
    } finally {
      restore()
    }
  })

  it("reports job not found without provider fallback on split deployments", async () => {
    const restore = mockFetch(async (url) => {
      const urlStr = String(url)
      if (urlStr === "http://127.0.0.1:8081/v1/runtime/deals/task-missing") {
        return new Response(JSON.stringify({ error: "deal not found" }), { status: 404 })
      }
      throw new Error(`unexpected URL: ${urlStr}`)
    })
    try {
      const result = await handleToolCall(
        "froglet",
        { action: "get_task", task_id: "task-missing" },
        config
      )
      assert.equal(result.isError, true)
      assert.match(result.content[0].text, /job not found/)
    } finally {
      restore()
    }
  })
})

describe("error handling", () => {
  it("returns isError on removed and unknown actions", async () => {
    const removed = await handleToolCall("froglet", { action: "create_project" }, config)
    assert.equal(removed.isError, true)
    assert.match(removed.content[0].text, /not available/)

    const unknown = await handleToolCall("froglet", { action: "explode" }, config)
    assert.equal(unknown.isError, true)
    assert.match(unknown.content[0].text, /Unknown Froglet action/)
  })

  it("returns isError on unknown tool and fetch failures", async () => {
    const unknownTool = await handleToolCall("missing", {}, config)
    assert.equal(unknownTool.isError, true)
    assert.match(unknownTool.content[0].text, /Unknown tool/)

    const restore = mockFetch(async () => {
      throw new Error("connection refused")
    })
    try {
      const result = await handleToolCall("froglet", { action: "list_local_services" }, config)
      assert.equal(result.isError, true)
      assert.match(result.content[0].text, /connection refused/)
    } finally {
      restore()
    }
  })
})
