import assert from "node:assert/strict"
import { cp, mkdir, mkdtemp, readFile, rm } from "node:fs/promises"
import { execFile } from "node:child_process"
import os from "node:os"
import path from "node:path"
import process from "node:process"
import test from "node:test"
import { fileURLToPath } from "node:url"
import { promisify } from "node:util"

import register from "../index.js"
import { checkPluginSource } from "../scripts/doctor.mjs"

const pluginRoot = fileURLToPath(new URL("..", import.meta.url))
const execFileAsync = promisify(execFile)
const exampleFiles = [
  "openclaw.config.example.json",
  "openclaw.config.nemoclaw.example.json",
  "openclaw.config.nemoclaw.hosted.example.json"
]

function buildToolList(config) {
  const tools = new Map()
  register({
    config,
    registerTool(definition, options) {
      tools.set(definition.name, { definition, options: options ?? {} })
    },
    logger: { info() {} }
  })
  return [...tools.keys()].sort()
}

function extractFrogletConfig(document) {
  return document.plugins.entries.froglet.config
}

test("checked-in examples parse and register the single froglet tool", async () => {
  for (const fileName of exampleFiles) {
    const document = JSON.parse(
      await readFile(path.join(pluginRoot, "examples", fileName), "utf8")
    )
    const pluginConfig = extractFrogletConfig(document)
    assert.deepEqual(
      buildToolList(pluginConfig),
      ["froglet"],
      `${fileName} must register only the froglet tool`
    )
  }
})

test("documented OpenShell directory uploads preserve the plugin and shared-library layout", async () => {
  const stagingRoot = await mkdtemp(path.join(os.tmpdir(), "froglet-staging-"))
  try {
    const repoRoot = path.resolve(pluginRoot, "../../..")
    const documentation = await readFile(path.join(repoRoot, "docs/NEMOCLAW.md"), "utf8")
    const directoryUploads = [...documentation.matchAll(
      /openshell sandbox upload my-node \\\n  \/absolute\/path\/to\/froglet\/(integrations\/(?:openclaw\/froglet|shared\/froglet-lib)) \\\n  (\/sandbox\/[^\s]+)/g
    )]
    assert.equal(directoryUploads.length, 4)
    assert.deepEqual(directoryUploads.slice(0, 2).map((match) => match.slice(1)),
      directoryUploads.slice(2).map((match) => match.slice(1)))
    async function uploadDirectory(match) {
      const source = path.join(repoRoot, match[1])
      const destination = path.join(stagingRoot, match[2].slice(1))
      // OpenShell extracts a directory archive into DEST, retaining its basename.
      const uploadedPath = path.join(destination, path.basename(source))
      await mkdir(destination, { recursive: true })
      await cp(source, uploadedPath, { recursive: true })
      return uploadedPath
    }
    const stagedPlugin = path.join(stagingRoot, "sandbox/froglet/integrations/openclaw/froglet")
    assert.equal(await uploadDirectory(directoryUploads[0]), stagedPlugin)
    const importAndRegister = `
      import { pathToFileURL } from 'node:url';
      const { default: register } = await import(pathToFileURL(process.argv[1]).href);
      const names = [];
      register({
        config: { hostProduct: 'nemoclaw', baseUrl: 'https://node.example', authTokenPath: '/sandbox/froglet.token' },
        registerTool(tool) { names.push(tool.name); },
        logger: { info() {} }
      });
      process.stdout.write(JSON.stringify(names));
    `
    const args = ["--input-type=module", "-e", importAndRegister, path.join(stagedPlugin, "index.js")]

    await assert.rejects(execFileAsync(process.execPath, args), (error) => {
      assert.match(error.stderr, /ERR_MODULE_NOT_FOUND/)
      assert.match(error.stderr, /shared\/froglet-lib/)
      return true
    })
    const incomplete = await checkPluginSource(stagedPlugin)
    assert.equal(incomplete.status, "error")
    assert.ok(incomplete.details.missing_files.some((file) => file.includes("shared/froglet-lib")))

    const stagedShared = path.join(stagingRoot, "sandbox/froglet/integrations/shared/froglet-lib")
    assert.equal(await uploadDirectory(directoryUploads[1]), stagedShared)
    const complete = await checkPluginSource(stagedPlugin)
    assert.equal(complete.status, "ok")
    const result = await execFileAsync(process.execPath, args)
    assert.deepEqual(JSON.parse(result.stdout), ["froglet"])
  } finally {
    await rm(stagingRoot, { recursive: true, force: true })
  }
})
