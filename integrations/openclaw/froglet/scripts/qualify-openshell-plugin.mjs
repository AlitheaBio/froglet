import assert from "node:assert/strict"
import register from "../index.js"

// Runs the actual registered plugin action, with real HTTP transport. This is a
// plugin/runtime boundary check, not an OpenClaw gateway or model-inference test.
const mode = process.argv[2]
assert.ok(["baseline", "restricted"].includes(mode), "expected baseline or restricted")
let tool
register({
  config: {
    hostProduct: "openclaw",
    baseUrl: "https://host.openshell.internal:19443",
    providerAuthTokenPath: "/sandbox/provider.token",
    runtimeAuthTokenPath: "/sandbox/runtime.token",
    requestTimeoutMs: 10000
  },
  registerTool: value => { tool = value },
  logger: { info() {} }
})
assert.equal(tool?.name, "froglet")
const results = []
for (const action of ["list_local_services", "get_spend_status"]) {
  const result = await tool.execute(`qualification-${mode}-${action}`, { action })
  const text = result.content?.filter(item => item.type === "text")
    .map(item => item.text).join("\n") ?? ""
  results.push({ action, isError: result.isError === true, text })
}
console.log(JSON.stringify({ mode, registered_tool: tool.name, results }, null, 2))
assert.equal(results[0].isError, false, "service listing must succeed")
assert.match(results[0].text, /services: \d+/)
if (mode === "baseline") {
  assert.equal(results[1].isError, false, "spend read must succeed under baseline policy")
  assert.match(results[1].text, /spend_budget_msat:/)
} else {
  assert.equal(results[1].isError, true, "spend read must be denied under restricted policy")
  assert.match(results[1].text, /403/)
  assert.match(results[1].text, /policy_denied/)
}
