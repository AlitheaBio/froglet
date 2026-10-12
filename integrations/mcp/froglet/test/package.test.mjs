import assert from "node:assert/strict"
import { readFile } from "node:fs/promises"
import { dirname, join, resolve } from "node:path"
import test from "node:test"
import { fileURLToPath } from "node:url"

const __filename = fileURLToPath(import.meta.url)
const __dirname = dirname(__filename)
const REPO_ROOT = resolve(__dirname, "../../../../")

async function readRootPackage() {
  return JSON.parse(await readFile(join(REPO_ROOT, "package.json"), "utf8"))
}

async function readRegistryServer() {
  return JSON.parse(await readFile(join(REPO_ROOT, "server.json"), "utf8"))
}

test("root npm package publishes froglet-mcp, not the private repo-local package", async () => {
  const manifest = await readRootPackage()
  assert.equal(manifest.name, "froglet-mcp")
  assert.equal(manifest.mcpName, "io.github.armanas/froglet")
  assert.equal(manifest.private, undefined)
  assert.equal(manifest.license, "Apache-2.0")
  assert.equal(manifest.bin?.["froglet-mcp"], "integrations/mcp/froglet/server.js")
  assert.equal(manifest.publishConfig?.access, "public")
  assert.equal(manifest.publishConfig?.provenance, undefined)
})

test("root npm package includes MCP server and shared client without broad repo contents", async () => {
  const manifest = await readRootPackage()
  const files = manifest.files ?? []
  assert.ok(files.includes("integrations/mcp/froglet/server.js"))
  assert.ok(files.includes("integrations/mcp/froglet/package.json"))
  assert.ok(files.includes("integrations/mcp/froglet/lib/**/*.js"))
  assert.ok(files.includes("integrations/shared/froglet-lib/**/*.js"))
  assert.ok(files.includes("server.json"))
  assert.ok(files.includes("LICENSE"))
  assert.ok(files.includes("NOTICE"))
  assert.ok(files.includes("README.md"))
  assert.equal(files.some((entry) => entry.includes("docs-site")), false)
  assert.equal(files.some((entry) => entry.includes("private_work")), false)
  assert.equal(files.some((entry) => entry === "target" || entry.startsWith("target/")), false)
})

test("MCP Registry server.json matches npm package metadata", async () => {
  const manifest = await readRootPackage()
  const server = await readRegistryServer()
  assert.equal(server.name, manifest.mcpName)
  assert.equal(server.version, manifest.version)
  assert.equal(server.repository?.url, "https://github.com/AlitheaBio/froglet")
  assert.equal(server.repository?.source, "github")
  assert.equal(server.packages?.[0]?.registryType, "npm")
  assert.equal(server.packages?.[0]?.identifier, manifest.name)
  assert.equal(server.packages?.[0]?.version, manifest.version)
  assert.equal(server.packages?.[0]?.transport?.type, "stdio")
})
