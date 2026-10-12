import { existsSync, readdirSync, readFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { describe, expect, it } from 'vitest';
import { repoRoot, src } from './route-helpers';
import { SUPPORT_MATRIX, SUPPORT_STATUSES } from '../../data/support-matrix';

// The developer page shows the project, its tools and its status. The step-by-step and reference material it used
// to repeat lives in these three docs pages, so they must state it accurately.

const doc = (path: string) => readFileSync(resolve(src, 'content/docs', path), 'utf8');
const repo = (path: string) => readFileSync(resolve(repoRoot, path), 'utf8');
/** Documents wrap their lines, so quoted sentences are compared with whitespace collapsed. */
const flat = (text: string) => text.replace(/\s+/g, ' ');
/** The text of one `## Heading` section, up to the next one. */
const section = (text: string, heading: string) => text.split(`\n${heading}\n`)[1]?.split('\n## ')[0] ?? '';
const fixture = JSON.parse(repo('conformance/kernel_v1.json'));

const kernel = doc('spec/kernel.md');
const conformance = doc('spec/conformance.md');
const crates = doc('architecture/crates.md');

describe('The kernel summary states how one artifact is signed', () => {
  it('gives the steps the conformance rules and the kernel source give', () => {
    const rules = repo('conformance/README.md');
    expect(rules).toContain('`payload_hash` — `SHA256(JCS(payload))`');
    expect(rules).toContain('`artifact_hash` — `SHA256(canonical signing bytes)`');
    expect(flat(rules)).toContain('BIP-340 signature over the 32-byte `SHA256(canonical signing bytes)` digest');
    expect(flat(repo('froglet-protocol/src/protocol/kernel.rs'))).toContain('schema_version, artifact_type, signer, created_at, payload_hash, payload');
    const steps = section(kernel, '## How one artifact is signed');
    expect(steps).toMatch(/^1\. `payload_hash` is the SHA-256 of the payload in RFC 8785 canonical JSON\./m);
    expect(steps).toContain('`[schema_version, artifact_type, signer, created_at, payload_hash, payload]`');
    expect(steps).toContain('BIP-340 Schnorr over that 32-byte digest');
    expect(steps).toContain('x-only secp256k1 public key');
  });

  it('says the vectors record the exact signing bytes in hex, and they do', () => {
    expect(kernel).toContain('record the exact signing bytes in hex');
    for (const name of ['descriptor', 'offer', 'quote', 'deal', 'invoice_bundle', 'receipt']) {
      expect(fixture.artifacts[name].canonical_signing_bytes_hex, name).toMatch(/^[0-9a-f]+$/);
    }
  });
});

describe('The conformance guide says how to run the vectors', () => {
  const run = section(conformance, '## Run the vectors');

  it('has the section the developer page links to', () => {
    expect(run.length).toBeGreaterThan(400);
  });

  it('shows the commands the README documents, and the files they run exist', () => {
    expect(run).toContain('cargo run -p froglet-verify -- conformance/kernel_v1.json');
    expect(repo('README.md')).toContain('cargo run -p froglet-verify -- conformance/kernel_v1.json');
    expect(run).toContain('cargo test -p froglet --test kernel_conformance_vectors');
    expect(repo('conformance/README.md')).toContain('cargo test -p froglet --test kernel_conformance_vectors');
    expect(existsSync(resolve(repoRoot, 'tests/kernel_conformance_vectors.rs'))).toBe(true);
    expect(run).toContain('PYTHONPATH=python/froglet-verify python3 -m froglet_verify.conformance conformance/kernel_v1.json');
    expect(repo('python/froglet-verify/froglet_verify/conformance.py')).toContain('python -m froglet_verify.conformance');
  });

  it('describes the verifier the way its own source does', () => {
    const cli = repo('froglet-verify/src/bin/froglet-verify.rs');
    expect(cli).toContain('Exit codes: 0 = nothing invalid, 1 = at least one invalid finding,');
    expect(cli).toContain('2 = usage or input error.');
    expect(cli).toContain('--now <unix-seconds>');
    expect(cli).toContain('--json');
    expect(flat(run)).toContain('Exit code 0 means nothing was invalid, 1 that something was, and 2 a usage error.');
    expect(flat(run)).toContain('`--json` prints a machine-readable report and `--now <unix>` turns on expiry checks.');
  });

  it('accepts a node’s feed as served, with the pipeline the README shows and its paging caveat', () => {
    const pipeline = "curl -s 'http://127.0.0.1:8080/v1/feed?limit=50' | cargo run -q -p froglet-verify -- -";
    expect(run).toContain(pipeline);
    expect(repo('README.md')).toContain(pipeline);
    expect(flat(run)).toContain('The feed is paged, so the verdict covers the page you fetched');
    // The claim is tested, on a page a real node served and against the node's own route.
    expect(repo('froglet-verify/tests/feed_page.rs')).toContain('fn cli_verifies_the_served_page_from_stdin');
    expect(repo('src/api/mod.rs')).toContain('async fn public_feed_page_verifies_offline_with_froglet_verify');
  });
});

describe('The crate structure page matches the workspace', () => {
  /** True when a module named in the docs exists under the crate's `src`. */
  const moduleExists = (root: string, token: string): boolean => {
    if (token.endsWith('/*')) return existsSync(resolve(root, token.slice(0, -2)));
    if (token.endsWith('*')) return readdirSync(root).some((name) => name.startsWith(token.slice(0, -1)));
    return [`${token}.rs`, `${token}/mod.rs`].some((candidate) => existsSync(resolve(root, candidate)));
  };
  /** The backticked names in one column of every table row of a section. */
  const namesIn = (text: string, column: number) => text.split('\n')
    .filter((line) => /^\| /.test(line) && !/^\|[-| ]+\|$/.test(line))
    .flatMap((line) => Array.from((line.split('|')[column] ?? '').matchAll(/`([^`]+)`/g)).map((match) => match[1]));

  it('names the five workspace members, and says the other directories are not members', () => {
    const members = (repo('Cargo.toml').match(/^members = \[([^\]]*)\]/m)?.[1].match(/"([^"]+)"/g) ?? []).map((name) => name.replaceAll('"', ''));
    expect([...members].sort()).toEqual(['.', 'froglet-protocol', 'froglet-publish-engine', 'froglet-verify', 'froglet-wasm']);
    expect(crates).toContain('The Cargo workspace has five members: `froglet-protocol`, `froglet-publish-engine`, `froglet-verify`, `froglet-wasm`, and the node crate `froglet` at the repository root.');
    expect(crates).toContain('The other directories below are not workspace members.');
  });

  it('lists workspace paths that all exist', () => {
    const paths = namesIn(section(crates, '## Workspace'), 1);
    expect(paths).toHaveLength(12);
    for (const path of paths) expect(existsSync(resolve(repoRoot, path)), path).toBe(true);
    expect(existsSync(resolve(repoRoot, 'froglet-protocol/Cargo.toml'))).toBe(true);
    expect(crates).toContain('froglet-protocol/Cargo.toml');
  });

  it('names only protocol modules that exist', () => {
    const modules = namesIn(section(crates, '## froglet-protocol'), 1).filter((name) => !name.includes('.toml'));
    expect(modules.length).toBeGreaterThanOrEqual(12);
    for (const name of modules) {
      if (name === 'ExecutionRuntime') expect(repo('froglet-protocol/src/lib.rs')).toContain('pub enum ExecutionRuntime');
      else expect(moduleExists(resolve(repoRoot, 'froglet-protocol/src'), name), name).toBe(true);
    }
  });

  it('names only node modules that exist', () => {
    const modules = namesIn(section(crates, '## froglet (the node crate)'), 2);
    expect(modules.length).toBeGreaterThanOrEqual(28);
    for (const name of modules) expect(moduleExists(resolve(repoRoot, 'src'), name), name).toBe(true);
  });

  it('carries no line counts, which go stale, and no claim of two crates', () => {
    expect(crates).not.toMatch(/\d[\d,]* lines/);
    expect(crates).not.toMatch(/two public crates/i);
  });
});

describe('The MCP and A2A guide matches the implemented source profile', () => {
  const interoperability = doc('learn/agent-interoperability.mdx');
  const nativeMcp = repo('src/cli/mcp.rs');
  const invoke = repo('src/cli/invoke.rs');
  const a2a = repo('src/api/a2a.rs');
  const config = repo('src/a2a_config.rs');
  const demo = repo('examples/a2a_compute_demo.py');

  it('is reachable from both documentation hubs under the same name', () => {
    expect(interoperability).toMatch(/^title: MCP and A2A$/m);
    for (const path of ['docs.mdx', 'learn/index.mdx']) {
      expect(doc(path), path).toContain('href="/learn/agent-interoperability/"');
      expect(doc(path), path).toContain('<strong>MCP and A2A</strong>');
    }
  });

  it('documents native actions, exact input/retry controls and the actual inline bounds', () => {
    for (const action of ['run_compute', 'get_task']) {
      expect(nativeMcp).toContain(`"${action}"`);
      expect(interoperability).toContain(`\`${action}\``);
    }
    const hexKiB = Number(invoke.match(/MAX_INLINE_WASM_HEX_BYTES: usize = (\d+) \* 1024/)?.[1]);
    const inputKiB = Number(invoke.match(/MAX_INLINE_WASM_INPUT_BYTES: usize = (\d+) \* 1024/)?.[1]);
    const keyBytes = Number(invoke.match(/MAX_IDEMPOTENCY_KEY_BYTES: usize = (\d+)/)?.[1]);
    expect(hexKiB).toBeGreaterThan(0);
    expect(inputKiB).toBeGreaterThan(0);
    expect(keyBytes).toBeGreaterThan(0);
    expect(flat(interoperability)).toContain(`${hexKiB / 2} KiB, encoded as at most ${hexKiB} KiB of hex`);
    expect(flat(interoperability)).toContain(`canonical JSON input limit is ${inputKiB} KiB`);
    expect(interoperability).toContain(`1–${keyBytes} UTF-8 bytes`);
    expect(interoperability).toContain('`input` is required even when its');
    expect(flat(interoperability)).toContain('value is `null`');
    expect(interoperability).toContain('`froglet.wasm.run_json.v1` ABI');
    expect(flat(interoperability)).toContain('Changed work under an existing key is refused');
    expect(flat(interoperability)).toContain('does not compile source or accept OCI packages or other execution runtimes');
  });

  it('gives a read-only recovery template using the new call’s reference', () => {
    const examples = Array.from(interoperability.matchAll(/```json\n([\s\S]*?)\n```/g))
      .map((match) => JSON.parse(match[1]));
    expect(examples).toHaveLength(1);
    expect(examples[0]).toEqual({
      action: 'get_task',
      task_id: '<deal_id returned by your run_compute call>',
      response_format: 'compact',
    });
    expect(flat(interoperability)).toContain('does not submit new work or authorize payment');
    expect(flat(interoperability)).toContain('A durable `submission_pending` reference records local intent');
    expect(flat(interoperability)).toContain('It does not prove that Bob accepted the work');
    expect(flat(interoperability)).toContain('the runtime does not automatically resubmit those intents');
    expect(flat(interoperability)).toContain('Calls default to `max_price_sats: 0`');
    expect(flat(interoperability)).toContain('Supplying a ceiling alone does not grant spending authority');
  });

  it('states the configured, private, polling A2A boundary without widening qualification', () => {
    const extension = a2a.match(/const EXTENSION: &str = "([^"]+)"/)?.[1];
    const prefix = a2a.match(/const PREFIX: &str = "([^"]+)"/)?.[1];
    expect(extension).toBeTruthy();
    expect(prefix).toBeTruthy();
    expect(interoperability).toContain(extension!);
    expect(interoperability).toContain(`\`${prefix}\``);
    expect(interoperability).toContain('A2A 1.0 HTTP+JSON');
    expect(interoperability).toContain('A2A is disabled by default');
    expect(config).toContain('FROGLET_A2A_CONFIG_PATH');
    expect(interoperability).toContain('`FROGLET_A2A_CONFIG_PATH`');
    expect(flat(interoperability)).toContain('exact allowed signed Offer hashes');
    expect(flat(interoperability)).toContain('file must be operator-owned, regular, at most 64 KiB and mode `0600`');
    expect(flat(interoperability)).toContain('Streaming, push notifications, conversational continuation, provider cancellation and relay A2A ingress are outside this profile');
    expect(flat(interoperability)).toContain('Production paid settlement, public-network operation and full A2A TCK conformance are not qualified by this demo');
    expect(flat(interoperability)).toContain('Signed evidence does not independently establish output correctness or usefulness');
  });

  it('points to the repeatable public source example and distinguishes release support', () => {
    expect(interoperability).toContain('python3 examples/a2a_compute_demo.py --serve');
    expect(interoperability).toContain('rustup target add wasm32-unknown-unknown');
    expect(interoperability).toContain('Python 3.10+, Cargo/Rust');
    expect(demo).toContain('parser.add_argument("--serve"');
    expect(demo).toContain('"FROGLET_PAYMENT_BACKEND": "none"');
    expect(demo).toContain('"max_price_sats": 0');
    expect(interoperability).toContain('`{"a":6,"b":7}`');
    expect(interoperability).toContain('`{"sum":13,"product":42}`');
    expect(flat(interoperability)).toContain('Support in an installed release depends on its exact binary');
    expect(flat(interoperability)).toContain('Source tests do not establish that an older immutable release includes these actions');
    expect(interoperability).not.toMatch(/_tmp\/|private_work\//);
  });
});

describe('The shared support matrix preserves implementation and qualification boundaries', () => {
  const entry = (id: string) => {
    const found = SUPPORT_MATRIX.find(row => row.id === id);
    if (!found) throw new Error(`Missing support row ${id}`);
    return found;
  };

  it('uses one rendered data source in the guide and developer page', () => {
    expect(doc('learn/agent-interoperability.mdx')).toContain('<SupportMatrix />');
    expect(readFileSync(resolve(src, 'pages/open-source.astro'), 'utf8')).toContain('<SupportMatrix id="developer-support-matrix" />');
    const component = readFileSync(resolve(src, 'components/SupportMatrix.astro'), 'utf8');
    expect(component).toContain('SUPPORT_MATRIX.map');
    expect(component).toContain('SUPPORT_STATUSES[row.status]');
    expect(component).toContain('Source reference:');
    expect(component).not.toContain('github.com/AlitheaBio/froglet/blob/main');
    expect(new Set(SUPPORT_MATRIX.map(row => row.id)).size).toBe(SUPPORT_MATRIX.length);
  });

  it('gives every claim an existing evidence path and a separate scenario and limit', () => {
    for (const row of SUPPORT_MATRIX) {
      expect(row.status in SUPPORT_STATUSES, row.id).toBe(true);
      expect(existsSync(resolve(repoRoot, row.evidence)), row.evidence).toBe(true);
      expect(row.scenario.length, row.id).toBeGreaterThan(20);
      expect(row.boundary.length, row.id).toBeGreaterThan(20);
    }
  });

  it('keeps x402 outside priced signed Deal admission, matching the actual filter and refusal', () => {
    const provider = repo('src/api/mod.rs');
    expect(provider).toContain('.filter(|method| method != "x402_usdc")');
    expect(provider).toContain('priced Kernel deals require a configured Lightning or Stripe settlement backend');
    expect(entry('x402').status).toBe('experimental');
    expect(entry('x402').boundary).toContain('priced Quote/Deal admission rejects this rail');
    expect(repo('docs/PAYMENT_MATRIX.md')).not.toContain('Operators\n  can still issue x402 offers');
  });

  it('does not promote the disabled worker, manual updates or removed marketplace methods', () => {
    expect(entry('gpu').status).toBe('disabled');
    expect(repo('docs/FILE_AND_COMPUTE_SCOPE.md')).toMatch(/GPU.*disabled|disabled.*GPU/is);
    expect(repo('src/cli/prepare.rs')).toContain('"automatic_publication":false');
    expect(entry('updates').boundary).toContain('No automatic republishing');
    expect(repo('integrations/shared/froglet-lib/tool-dispatch.js')).toContain('marketplace_stake is not available');
    expect(entry('spend').boundary).toContain('marketplace top-up are unavailable');
    expect(entry('tor').boundary).toContain('Advanced self-hosted path');
  });

  it('distinguishes ordinary A2A discovery from execution authority and refuses cancellation', () => {
    const provider = repo('src/api/a2a.rs');
    expect(provider).toContain('TASK_NOT_CANCELABLE');
    expect(provider).toContain('"streaming":false');
    expect(entry('a2a-extra').status).toBe('unsupported');
    expect(entry('open-buy').status).toBe('unsupported');
    expect(entry('a2a').boundary).toContain('exact requester and Offer scope');
  });

  it('retains the failed fresh-session qualification when describing new program evidence', () => {
    const guide = flat(doc('learn/agent-interoperability.mdx'));
    expect(guide).toContain('fresh-session qualification remains incomplete');
    expect(guide).toContain('persistent Codex configuration changed');
    expect(guide).toContain('do not silently turn the earlier fresh-session attempt into a pass');
    expect(guide).toContain('all seven new signed execution chains');
    expect(guide).toContain('returned HTTP 502 while he was offline');
    expect(entry('recovery').boundary).toContain('get_task revalidates saved terminal receipts offline');
    expect(entry('recovery').boundary).toContain('Unsigned or unresolved tasks still need Bob for refresh');
    expect(entry('public-compute').status).toBe('unqualified');
    expect(entry('batch').status).toBe('planned');
  });
});
