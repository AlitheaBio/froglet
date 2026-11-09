import { access, readFile, stat } from "node:fs/promises"
import { constants as fsConstants } from "node:fs"
import path from "node:path"
import process from "node:process"
import { parseArgs } from "node:util"
import { fileURLToPath } from "node:url"

import {
  ABSOLUTE_MAX_SEARCH_LIMIT,
  MAX_TIMEOUT_MS,
  MIN_SEARCH_LIMIT,
  MIN_TIMEOUT_MS,
  normalizeBaseUrl,
  requestJson
} from "../lib/shared.js"
import { readPluginConfig } from "../lib/config.js"

const LOOPBACK_HTTP_HOSTS = new Set(["127.0.0.1", "localhost", "::1", "[::1]"])

function usage() {
  return [
    "Usage:",
    "  node scripts/doctor.mjs --config <path> --target openclaw|nemoclaw [--check-runtime]",
    "",
    "This validates the Froglet plugin section inside a complete OpenClaw or NemoClaw config."
  ].join("\n")
}

function buildCheck(id, status, summary, details = {}) {
  return { id, status, summary, details }
}

function overallStatus(checks) {
  if (checks.some((check) => check.status === "error")) {
    return "error"
  }
  if (checks.some((check) => check.status === "warning")) {
    return "warning"
  }
  return "ok"
}

export function runtimeHealthCheck(status) {
  return buildCheck(
    "runtime_health",
    status.healthy === true ? "ok" : "error",
    status.healthy === true
      ? "Froglet provider and runtime APIs are healthy"
      : "Froglet provider or runtime API reported unhealthy",
    {
      healthy: status.healthy,
      node_id: status.node_id,
      provider_healthy: status.provider_healthy,
      runtime_healthy: status.runtime_healthy,
      compute_offers: status.compute_offers,
      components: status.components
    }
  )
}

function validateInteger(name, value, minimum, maximum) {
  if (!Number.isInteger(value)) {
    throw new Error(`${name} must be an integer`)
  }
  if (value < minimum || value > maximum) {
    throw new Error(`${name} must be between ${minimum} and ${maximum}`)
  }
}

function validateBaseUrl(name, value) {
  const normalized = normalizeBaseUrl(value, name)
  const parsed = new URL(normalized)
  if (parsed.protocol === "http:" && !LOOPBACK_HTTP_HOSTS.has(parsed.hostname)) {
    throw new Error(`${name} must use https:// or loopback http://`)
  }
  return normalized
}

async function pathReadable(targetPath) {
  try {
    await access(targetPath, fsConstants.R_OK)
    return true
  } catch {
    return false
  }
}

export async function checkPluginSource(pluginPath) {
  try {
    if (!(await stat(pluginPath)).isDirectory()) {
      return buildCheck("plugin_source", "error", `Plugin path is not a directory: ${pluginPath}`)
    }
  } catch {
    return buildCheck("plugin_source", "error", `Plugin directory is missing or unreadable: ${pluginPath}`)
  }

  const sharedPath = path.resolve(pluginPath, "../../shared/froglet-lib")
  const requiredFiles = [
    ...["index.js", "package.json", "openclaw.plugin.json", "lib/config.js", "lib/shared.js", "lib/froglet-tool.js"]
      .map((file) => path.join(pluginPath, file)),
    ...["package.json", "shared.js", "tool-contract.js", "tool-dispatch.js", "froglet-client.js", "summarize.js", "marketplace-publish.js", "url-safety.js"]
      .map((file) => path.join(sharedPath, file))
  ]
  const missingFiles = (await Promise.all(requiredFiles.map(async (file) => {
    try {
      return (await stat(file)).isFile() && await pathReadable(file) ? null : file
    } catch {
      return file
    }
  }))).filter((file) => file !== null)

  return buildCheck(
    "plugin_source",
    missingFiles.length === 0 ? "ok" : "error",
    missingFiles.length === 0
      ? `Plugin files and shared dependencies are present: ${pluginPath}`
      : `Plugin staging is incomplete: ${missingFiles.length} required file(s) missing or unreadable`,
    { plugin_path: pluginPath, shared_path: sharedPath, missing_files: missingFiles }
  )
}

function getPluginPath(document) {
  const paths = document?.plugins?.load?.paths
  if (!Array.isArray(paths) || paths.length === 0) {
    throw new Error("plugins.load.paths must contain at least one plugin path")
  }
  const pluginPath = String(paths[0] ?? "").trim()
  if (!path.isAbsolute(pluginPath)) {
    throw new Error("plugins.load.paths[0] must be an absolute filesystem path")
  }
  return pluginPath
}

function getPluginConfig(document) {
  const entry = document?.plugins?.entries?.froglet
  if (!entry || entry.enabled !== true || typeof entry.config !== "object") {
    throw new Error("plugins.entries.froglet.enabled=true with a config object is required")
  }
  return readPluginConfig({ config: entry.config })
}

export async function checkApis(
  providerUrl,
  providerAuthTokenPath,
  runtimeUrl,
  requestTimeoutMs
) {
  const providerToken = (await readFile(providerAuthTokenPath, "utf8")).trim()
  const providerAuthHeaders = { Authorization: `Bearer ${providerToken}` }

  const [providerHealth, capabilities, identity, runtimeHealth] = await Promise.all([
    requestJson(`${providerUrl}/health`, { method: "GET", timeoutMs: requestTimeoutMs }),
    requestJson(`${providerUrl}/v1/node/capabilities`, {
      method: "GET",
      timeoutMs: requestTimeoutMs,
      headers: providerAuthHeaders
    }),
    requestJson(`${providerUrl}/v1/node/identity`, {
      method: "GET",
      timeoutMs: requestTimeoutMs,
      headers: providerAuthHeaders
    }),
    requestJson(`${runtimeUrl}/health`, { method: "GET", timeoutMs: requestTimeoutMs })
  ])

  const providerHealthy = providerHealth?.healthy === true || providerHealth?.status === "ok"
  const runtimeHealthy = runtimeHealth?.healthy === true || runtimeHealth?.status === "ok"
  return {
    healthy: providerHealthy && runtimeHealthy,
    node_id: identity?.node_id ?? identity?.id,
    provider_healthy: providerHealthy,
    runtime_healthy: runtimeHealthy,
    compute_offers: capabilities?.compute_offer_ids ?? [],
    components: {
      provider: { healthy: providerHealthy, health: providerHealth },
      runtime: { healthy: runtimeHealthy, health: runtimeHealth }
    }
  }
}

function shellQuote(value) {
  return `'${String(value).replace(/'/g, "'\\''")}'`
}

function manualCommands(target, config, pluginPath) {
  const providerUrl = config.providerUrl
  const runtimeUrl = config.runtimeUrl
  const tokenPath = config.providerAuthTokenPath
  if (target === "nemoclaw") {
    return [
      "nemoclaw <sandbox-name> status",
      "openshell sandbox get <sandbox-name>",
      `openshell sandbox upload <sandbox-name> /absolute/path/to/froglet/integrations/openclaw/froglet ${shellQuote(path.dirname(pluginPath))}`,
      `openshell sandbox upload <sandbox-name> /absolute/path/to/froglet/integrations/shared/froglet-lib ${shellQuote(path.resolve(pluginPath, "../../shared"))}`,
      `openshell sandbox upload <sandbox-name> /absolute/path/to/froglet-control.token ${shellQuote(tokenPath)}`,
      "nemoclaw <sandbox-name> connect",
      `TOKEN=$(cat ${tokenPath})`,
      `curl -H "Authorization: Bearer $TOKEN" ${providerUrl}/health`,
      `curl -H "Authorization: Bearer $TOKEN" ${providerUrl}/v1/node/identity`,
      `curl ${runtimeUrl}/health`
    ]
  }
  return [
    `TOKEN=$(cat ${tokenPath})`,
    `curl ${providerUrl}/health`,
    `curl -H "Authorization: Bearer $TOKEN" ${providerUrl}/v1/node/identity`,
    `curl ${runtimeUrl}/health`
  ]
}

export async function main() {
  const parsed = parseArgs({
    allowPositionals: false,
    options: {
      config: { type: "string" },
      target: { type: "string" },
      "check-runtime": { type: "boolean", default: false },
      help: { type: "boolean", default: false }
    }
  })

  if (parsed.values.help) {
    process.stdout.write(`${usage()}\n`)
    process.exit(0)
  }

  const configPath = parsed.values.config
  const target = parsed.values.target
  if (!configPath || !target || (target !== "openclaw" && target !== "nemoclaw")) {
    process.stderr.write(`${usage()}\n`)
    process.exit(1)
  }

  const document = JSON.parse(await readFile(configPath, "utf8"))
  const pluginPath = getPluginPath(document)
  const pluginConfig = getPluginConfig(document)

  if (pluginConfig.hostProduct !== target) {
    throw new Error(`doctor target ${target} requires hostProduct=${target}`)
  }

  validateBaseUrl("providerUrl", pluginConfig.providerUrl)
  validateBaseUrl("runtimeUrl", pluginConfig.runtimeUrl)
  validateInteger("requestTimeoutMs", pluginConfig.requestTimeoutMs, MIN_TIMEOUT_MS, MAX_TIMEOUT_MS)
  validateInteger("defaultSearchLimit", pluginConfig.defaultSearchLimit, MIN_SEARCH_LIMIT, ABSOLUTE_MAX_SEARCH_LIMIT)
  validateInteger("maxSearchLimit", pluginConfig.maxSearchLimit, MIN_SEARCH_LIMIT, ABSOLUTE_MAX_SEARCH_LIMIT)
  if (pluginConfig.defaultSearchLimit > pluginConfig.maxSearchLimit) {
    throw new Error("defaultSearchLimit must be less than or equal to maxSearchLimit")
  }
  if (!path.isAbsolute(pluginConfig.providerAuthTokenPath)) {
    throw new Error("providerAuthTokenPath must be an absolute filesystem path")
  }
  if (!path.isAbsolute(pluginConfig.runtimeAuthTokenPath)) {
    throw new Error("runtimeAuthTokenPath must be an absolute filesystem path")
  }

  const checks = []
  checks.push(buildCheck("config", "ok", "plugin config parsed", { hostProduct: pluginConfig.hostProduct }))

  checks.push(await checkPluginSource(pluginPath))

  const tokenReadable = await pathReadable(pluginConfig.providerAuthTokenPath)
  const runtimeTokenReadable = await pathReadable(pluginConfig.runtimeAuthTokenPath)
  checks.push(
    buildCheck(
      "auth_token",
      tokenReadable && runtimeTokenReadable ? "ok" : "error",
      tokenReadable && runtimeTokenReadable
        ? `Auth token is readable: ${pluginConfig.providerAuthTokenPath}`
        : `Auth token is missing or unreadable: ${pluginConfig.providerAuthTokenPath} / ${pluginConfig.runtimeAuthTokenPath}`
    )
  )

  if (parsed.values["check-runtime"] && tokenReadable && runtimeTokenReadable) {
    try {
      const status = await checkApis(
        pluginConfig.providerUrl,
        pluginConfig.providerAuthTokenPath,
        pluginConfig.runtimeUrl,
        pluginConfig.requestTimeoutMs
      )
      checks.push(runtimeHealthCheck(status))
    } catch (error) {
      checks.push(buildCheck("runtime_health", "error", `Runtime check failed: ${error.message}`))
    }
  } else {
    checks.push(
      buildCheck(
        "runtime_health",
        "warning",
        "Runtime check skipped",
        { manual_commands: manualCommands(target, pluginConfig, pluginPath) }
      )
    )
  }

  const status = overallStatus(checks)
  process.stdout.write(
    `${JSON.stringify(
      {
        target,
        overall_status: status,
        checks
      },
      null,
      2
    )}\n`
  )
  if (status === "error") {
    process.exitCode = 1
  }
}

if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  main().catch((error) => {
    process.stderr.write(`${error.message}\n`)
    process.exit(1)
  })
}
