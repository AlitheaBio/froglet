import assert from "node:assert/strict"
import { cp, mkdir, mkdtemp, rm, writeFile } from "node:fs/promises"
import os from "node:os"
import path from "node:path"
import process from "node:process"
import test from "node:test"
import { fileURLToPath } from "node:url"

import { checkApis, runtimeHealthCheck } from "../scripts/doctor.mjs"

const doctorPath = fileURLToPath(new URL("../scripts/doctor.mjs", import.meta.url))
const pluginRoot = fileURLToPath(new URL("..", import.meta.url))

async function stagePlugin(tempDir) {
  const pluginPath = path.join(tempDir, "integrations/openclaw/froglet")
  const sharedPath = path.join(tempDir, "integrations/shared/froglet-lib")
  await mkdir(path.dirname(pluginPath), { recursive: true })
  await mkdir(path.dirname(sharedPath), { recursive: true })
  await cp(pluginRoot, pluginPath, { recursive: true })
  await cp(path.resolve(pluginRoot, "../../shared/froglet-lib"), sharedPath, { recursive: true })
  return pluginPath
}

async function writeConfig(tempDir, fileName, pluginConfig, pluginPath = path.join(tempDir, "plugin")) {
  const configPath = path.join(tempDir, fileName)
  const document = {
    plugins: {
      load: { paths: [pluginPath] },
      entries: {
        froglet: {
          enabled: true,
          config: pluginConfig
        }
      }
    }
  }
  await writeFile(configPath, JSON.stringify(document, null, 2), "utf8")
  return configPath
}

async function runDoctor(args) {
  const child = process.execPath
  const { spawn } = await import("node:child_process")
  return new Promise((resolve, reject) => {
    const proc = spawn(child, [doctorPath, ...args], {
      stdio: ["ignore", "pipe", "pipe"]
    })
    let stdout = ""
    let stderr = ""
    proc.stdout.on("data", (chunk) => {
      stdout += chunk
    })
    proc.stderr.on("data", (chunk) => {
      stderr += chunk
    })
    proc.on("error", reject)
    proc.on("close", (code) => resolve({ code, stdout, stderr }))
  })
}

test("doctor validates the new dual-URL config shape", async () => {
  const tempDir = await mkdtemp(path.join(os.tmpdir(), "froglet-doctor-"))
  try {
    const tokenPath = path.join(tempDir, "froglet.token")
    await writeFile(tokenPath, "froglet-token\n", "utf8")
    const pluginPath = await stagePlugin(tempDir)
    const configPath = await writeConfig(tempDir, "openclaw.json", {
      hostProduct: "openclaw",
      providerUrl: "http://127.0.0.1:8080",
      runtimeUrl: "http://127.0.0.1:8081",
      providerAuthTokenPath: tokenPath,
      runtimeAuthTokenPath: tokenPath,
      requestTimeoutMs: 1000,
      defaultSearchLimit: 10,
      maxSearchLimit: 50
    }, pluginPath)
    const result = await runDoctor(["--config", configPath, "--target", "openclaw"])
    assert.equal(result.code, 0)
    const payload = JSON.parse(result.stdout)
    assert.equal(payload.overall_status, "warning")
  } finally {
    await rm(tempDir, { recursive: true, force: true })
  }
})

test("doctor validates legacy baseUrl/authTokenPath config as fallback", async () => {
  const tempDir = await mkdtemp(path.join(os.tmpdir(), "froglet-doctor-"))
  try {
    const tokenPath = path.join(tempDir, "froglet.token")
    await writeFile(tokenPath, "froglet-token\n", "utf8")
    const pluginPath = await stagePlugin(tempDir)
    const configPath = await writeConfig(tempDir, "openclaw-legacy.json", {
      hostProduct: "openclaw",
      baseUrl: "http://127.0.0.1:8080",
      authTokenPath: tokenPath,
      requestTimeoutMs: 1000,
      defaultSearchLimit: 10,
      maxSearchLimit: 50
    }, pluginPath)
    const result = await runDoctor(["--config", configPath, "--target", "openclaw"])
    assert.equal(result.code, 0)
    const payload = JSON.parse(result.stdout)
    assert.equal(payload.overall_status, "warning")
    assert.equal(payload.checks.find((check) => check.id === "plugin_source").status, "ok")
  } finally {
    await rm(tempDir, { recursive: true, force: true })
  }
})

test("doctor rejects a readable file as a plugin and exits nonzero", async () => {
  const tempDir = await mkdtemp(path.join(os.tmpdir(), "froglet-doctor-"))
  try {
    const tokenPath = path.join(tempDir, "froglet.token")
    await writeFile(tokenPath, "froglet-token\n", "utf8")
    await writeFile(path.join(tempDir, "plugin"), "not a plugin directory\n", "utf8")
    const configPath = await writeConfig(tempDir, "invalid-plugin.json", {
      hostProduct: "nemoclaw",
      baseUrl: "https://node.example",
      authTokenPath: tokenPath
    })
    const result = await runDoctor(["--config", configPath, "--target", "nemoclaw"])
    assert.equal(result.code, 1)
    const payload = JSON.parse(result.stdout)
    assert.equal(payload.overall_status, "error")
    assert.equal(payload.checks.find((check) => check.id === "plugin_source").status, "error")
    assert.equal(payload.checks.find((check) => check.id === "runtime_health").status, "warning")
  } finally {
    await rm(tempDir, { recursive: true, force: true })
  }
})

test("doctor rejects missing credentials and keeps skipped-runtime status explicit", async () => {
  const tempDir = await mkdtemp(path.join(os.tmpdir(), "froglet-doctor-"))
  try {
    const pluginPath = await stagePlugin(tempDir)
    const configPath = await writeConfig(tempDir, "missing-token.json", {
      hostProduct: "nemoclaw",
      baseUrl: "https://node.example",
      authTokenPath: path.join(tempDir, "missing.token")
    }, pluginPath)
    const result = await runDoctor(["--config", configPath, "--target", "nemoclaw", "--check-runtime"])
    assert.equal(result.code, 1)
    const payload = JSON.parse(result.stdout)
    assert.equal(payload.checks.find((check) => check.id === "auth_token").status, "error")
    const runtimeCheck = payload.checks.find((check) => check.id === "runtime_health")
    assert.equal(runtimeCheck.status, "warning")
    assert.equal(runtimeCheck.summary, "Runtime check skipped")
    const uploadCommands = runtimeCheck.details.manual_commands.filter((command) => command.startsWith("openshell sandbox upload"))
    assert.equal(uploadCommands.length, 3)
    assert.deepEqual(uploadCommands, [
      `openshell sandbox upload <sandbox-name> /absolute/path/to/froglet/integrations/openclaw/froglet '${path.dirname(pluginPath)}'`,
      `openshell sandbox upload <sandbox-name> /absolute/path/to/froglet/integrations/shared/froglet-lib '${path.resolve(pluginPath, "../../shared")}'`,
      `openshell sandbox upload <sandbox-name> /absolute/path/to/froglet-control.token '${path.join(tempDir, "missing.token")}'`
    ])
  } finally {
    await rm(tempDir, { recursive: true, force: true })
  }
})

test("doctor checkApis probes provider and runtime health", async () => {
  const tempDir = await mkdtemp(path.join(os.tmpdir(), "froglet-doctor-"))
  const previousFetch = global.fetch
  try {
    const tokenPath = path.join(tempDir, "froglet.token")
    await writeFile(tokenPath, "froglet-token\n", "utf8")
    global.fetch = async (url, options = {}) => {
      const urlStr = String(url)
      const auth = options.headers?.Authorization
      if (urlStr === "http://127.0.0.1:8080/health") {
        return new Response(JSON.stringify({ healthy: true }))
      }
      if (urlStr === "http://127.0.0.1:8080/v1/node/capabilities" && auth === "Bearer froglet-token") {
        return new Response(JSON.stringify({ compute_offer_ids: ["execute.compute"] }))
      }
      if (urlStr === "http://127.0.0.1:8080/v1/node/identity" && auth === "Bearer froglet-token") {
        return new Response(JSON.stringify({ node_id: "node-1" }))
      }
      if (urlStr === "http://127.0.0.1:8081/health") {
        return new Response(JSON.stringify({ status: "ok" }))
      }
      throw new Error(`unexpected URL ${urlStr}`)
    }

    const status = await checkApis(
      "http://127.0.0.1:8080",
      tokenPath,
      "http://127.0.0.1:8081",
      1000
    )
    assert.equal(status.node_id, "node-1")
    assert.equal(status.healthy, true)
    assert.equal(status.provider_healthy, true)
    assert.equal(status.runtime_healthy, true)
  } finally {
    global.fetch = previousFetch
    await rm(tempDir, { recursive: true, force: true })
  }
})

test("doctor marks an HTTP 200 unhealthy payload as an error", () => {
  const check = runtimeHealthCheck({
    healthy: false,
    provider_healthy: true,
    runtime_healthy: false,
    components: {}
  })
  assert.equal(check.status, "error")
  assert.equal(check.details.healthy, false)
})
