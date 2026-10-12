import { existsSync, readdirSync, readFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { afterEach, describe, expect, it } from 'vitest';
import { MATURITY } from '../../data/maturity';
import { COMPILER_ARGS, CONTRACT, MAX_SOURCE_CHARS } from '../playground/compiler';
import { FUNCTIONS } from '../playground/functions';
import { PLAYGROUND_LIMITS } from '../playground/protocol';
import { brokenInternalLinks, docsSite, repoRoot, src } from './route-helpers';

// What the playground section says it is, held to the repository: each claim of "real" or "not real" is tied to the code,
// the tests or the build that show it, so the wording cannot drift from what runs. The behaviour itself is in
// playground-exchange.test.ts and playground-ui.test.ts.

const read = (path: string) => readFileSync(resolve(src, path), 'utf8');
const repo = (path: string) => readFileSync(resolve(repoRoot, path), 'utf8');
const flat = (text: string) => text.replace(/\s+/g, ' ');
const page = read('pages/open-source.astro');
const markup = page.slice(page.indexOf('<div class="play" data-playground hidden>'), page.indexOf('<!-- /playground -->'));
const section = page.slice(page.indexOf('<section class="story-section" id="playground">'), page.indexOf('<section class="story-section" id="vision">'));
const css = read('styles/playground.css');
const cssRules = css.replace(/\/\*[\s\S]*?\*\//g, '');
const playgroundDir = resolve(src, 'scripts/playground');
const modules = readdirSync(playgroundDir).filter((file) => file.endsWith('.ts'));
const moduleSource = (file: string) => readFileSync(resolve(playgroundDir, file), 'utf8');
const functionsDir = resolve(playgroundDir, 'functions');
const nodeExchange = JSON.parse(repo('froglet-wasm/tests/fixtures/node_service_exchange.json'));

afterEach(() => {
  document.body.innerHTML = '';
});

describe('the playground section on the Developers page', () => {
  it('sits second, after the verifier, with a jump link, its stylesheet and its script', () => {
    const order = ['id="verify"', 'id="playground"', 'id="vision"'].map((marker) => page.indexOf(marker));
    expect(order.every((index) => index > 0)).toBe(true);
    expect([...order].sort((a, b) => a - b)).toEqual(order);
    expect(page).toContain('<li><a href="#verify">Verify</a></li><li><a href="#playground">Playground</a></li><li><a href="#vision">Vision</a></li>');
    expect(page).toContain("import '../styles/playground.css'");
    expect(page).toMatch(/import \{ startPlayground \} from '\.\.\/scripts\/playground\/browser';\s*initDevPage\(\);\s*startPlayground\(\);/);
    expect(markup.length).toBeGreaterThan(2000);
    expect(markup).not.toMatch(/style="|<style[\s>]/);
    expect(markup).not.toMatch(/[{}]/);
  });

  it('stays out of sight until its script has run, and says what it needs when there is none', () => {
    expect(markup.startsWith('<div class="play" data-playground hidden>')).toBe(true);
    expect(section).toContain('<noscript><p class="dev-note">The playground needs JavaScript and WebAssembly. The tools below run without a browser.</p></noscript>');
  });

  it('labels every control, gives every button a type, and announces changes', () => {
    document.body.innerHTML = markup;
    const controls = Array.from(document.querySelectorAll<HTMLElement>('input, select, textarea'));
    expect(controls.length).toBeGreaterThanOrEqual(7);
    for (const control of controls) expect(control.closest('label') ?? control.getAttribute('aria-label'), control.outerHTML.slice(0, 70)).toBeTruthy();
    const buttons = Array.from(document.querySelectorAll('button'));
    expect(buttons.length).toBeGreaterThanOrEqual(5);
    for (const button of buttons) expect(button.getAttribute('type'), button.outerHTML.slice(0, 60)).toBe('button');
    for (const region of ['[data-play-bob-log]', '[data-play-outcome]', '[data-play-evidence-out]']) {
      expect(document.querySelector(region)?.getAttribute('role'), region).toBe('status');
      expect(document.querySelector(region)?.getAttribute('aria-live'), region).toBe('polite');
    }
    expect(document.querySelector('[data-play-outcome]')?.getAttribute('tabindex')).toBe('-1');
    expect(document.querySelector('fieldset > legend')?.textContent).toBe('The function to publish');
    expect(document.querySelector('[data-play-steps]')?.getAttribute('aria-label')).toBeTruthy();
    expect(document.querySelectorAll('h1, h2, h3')).toHaveLength(0);
  });

  it('links only to pages and anchors that exist', () => {
    expect(brokenInternalLinks(section)).toEqual([]);
    for (const route of ['/verify-receipt/', '/learn/quickstart/']) expect(section).toContain(`href="${route}"`);
  });
});

describe('what it says is real', () => {
  it('signs with the kernel: froglet-wasm is a thin layer over froglet-protocol that reproduces the vectors', () => {
    expect(section).toContain('come from <code>froglet-protocol</code>, the Rust kernel, compiled to WebAssembly as <code>froglet-wasm</code>');
    const manifest = repo('froglet-wasm/Cargo.toml');
    expect(manifest).toContain('froglet-protocol = { path = "../froglet-protocol"');
    for (const own of ['secp256k1', 'k256', 'sha2', 'serde_jcs']) expect(manifest, own).not.toContain(own);
    const lib = repo('froglet-wasm/src/lib.rs');
    expect(lib).toContain('froglet_protocol::');
    expect(lib).toContain('canonical_json');
    expect(lib).toContain('kernel::sign_artifact');
    // The five artifact types the page names are exactly the ones the crate signs, and its test compares them with the vectors.
    expect(section).toContain('their descriptor, offer, quote, deal, and receipt byte for byte');
    expect(lib).toMatch(/SIGNABLE_TYPES: \[&str; 5\] = \[\s*ARTIFACT_TYPE_DESCRIPTOR,\s*ARTIFACT_TYPE_OFFER,\s*ARTIFACT_TYPE_QUOTE,\s*ARTIFACT_TYPE_DEAL,\s*ARTIFACT_TYPE_RECEIPT,\s*\]/);
    const conformance = repo('froglet-wasm/tests/conformance.rs');
    expect(conformance).toContain('fn signs_every_vector_artifact_byte_for_byte');
    expect(conformance).toContain('assert_eq!(signed, 9)');
    expect(conformance).toContain('conformance/kernel_v1.json');
  });

  it('checks with the verifier: the same generated build as the panel above', () => {
    expect(section).toContain('Every check is <code>froglet-verify</code>, the same WebAssembly build as the panel above.');
    expect(moduleSource('kernel.ts')).toContain("from '../../generated/verifier/froglet_verify.js'");
    expect(moduleSource('kernel.ts')).toContain("from '../receipt-verifier'");
    expect(read('scripts/receipt-verifier.ts')).toContain("from '../generated/verifier/froglet_verify.js'");
    expect(page).toContain('data-dev-verify');
  });

  it('speaks a node\'s provider API, checked against what a real node served', () => {
    expect(section).toContain('A real <code>froglet-node</code> served the same kind of exchange, and a test checks that this page’s responses have the same fields and value types. An opt-in test also has a real node’s requester call this provider.');
    expect(nodeExchange.captured_from).toContain('The responses of a real froglet-node');
    expect(repo('tests/browser_playground_wire.rs')).toContain('fn every_captured_response_round_trips_through_the_nodes_own_types');
    expect(read('scripts/__tests__/playground-exchange.test.ts')).toContain("describe('the playground speaks the wire a real node speaks'");
    // The opt-in test: a real node's runtime is the requester and the shipped provider is what it calls. It is skipped
    // unless asked for, because it needs a built node, and the docs-site README says how to run it.
    const live = read('scripts/__tests__/playground-real-requester.test.ts');
    expect(live).toContain("process.env.FROGLET_PLAYGROUND_REAL_REQUESTER === '1'");
    expect(live).toContain("describe.skipIf(!enabled)('a real froglet-node requester calling the shipped provider'");
    expect(live).toContain("FROGLET_NODE_ROLE: 'runtime'");
    expect(live).toContain('createProvider(');
    expect(live).toContain('/v1/runtime/deals');
    expect(readFileSync(resolve(docsSite, 'README.md'), 'utf8')).toContain('FROGLET_PLAYGROUND_REAL_REQUESTER=1');
    // The routes the provider serves are routes the node has.
    const openapi = repo('docs/openapi.yaml');
    const provider = moduleSource('provider.ts');
    const served: Array<[string, string]> = [
      ['/v1/provider/descriptor', '/v1/provider/descriptor'],
      ['/v1/provider/offers', '/v1/provider/offers'],
      ['/v1/provider/services', '/v1/provider/services'],
      ['/v1/provider/services/{service_id}', '/v1/provider/services/'],
      ['/v1/provider/quotes', '/v1/provider/quotes'],
      ['/v1/provider/deals', '/v1/provider/deals'],
      ['/v1/provider/deals/{deal_id}', '/v1/provider/deals/'],
      ['/v1/artifacts/{artifact_hash}', '/v1/artifacts/'],
    ];
    for (const [documented, implemented] of served) {
      expect(openapi, documented).toContain(`\n  ${documented}:\n`);
      // Written out as a string, or as an escaped pattern for the routes that carry an id.
      expect(provider.includes(implemented) || provider.includes(implemented.replaceAll('/', '\\/')), implemented).toBe(true);
    }
  });

  it('has a real node run what the editor compiles: accepted, run, and named by its hash', () => {
    expect(section).toContain('A real node accepts the same bytes, runs them, and reports the same module hash. An opt-in test checks this.');
    const live = read('scripts/__tests__/playground-real-provider.test.ts');
    expect(live).toContain("process.env.FROGLET_PLAYGROUND_REAL_PROVIDER === '1'");
    expect(live).toContain("describe.skipIf(!enabled)('a real froglet-node running what the editor compiles'");
    expect(live).toContain("FROGLET_NODE_ROLE: 'dual'");
    expect(live).toContain('/v1/provider/artifacts/publish');
    expect(live).toContain('compiler.compile(source)');
    expect(live).toContain('runExchange(');
    expect(live).toContain('artifacts.receipt.payload.executor.module_hash).toBe(moduleHash)');
    expect(live).toContain("expect(onNode.failure).toEqual({ code: 'execution_limit_exceeded'");
    const readme = readFileSync(resolve(docsSite, 'README.md'), 'utf8');
    expect(readme).toContain('FROGLET_PLAYGROUND_REAL_PROVIDER=1');
    expect(readme).toContain('playground-real-provider.test.ts');
  });

  it('runs the function where it can be stopped, with nothing to reach out with, and a memory maximum', () => {
    expect(section).toContain('The function runs in a Web Worker that can import nothing. Bob stops it at his time limit, and adds a memory maximum to the module.');
    const runner = moduleSource('runner.ts');
    expect(runner).toContain('WebAssembly.instantiate(data.bytes, {})');
    expect(runner).toContain('worker.terminate()');
    expect(runner).toContain('setTimeout(');
    expect(moduleSource('wasm-module.ts')).toContain('cappedTo');
    expect(moduleSource('wasm-module.ts')).toContain('a module may not import anything');
    expect(read('scripts/__tests__/playground-runner.test.ts')).toContain('stops a function that never returns at the time limit');
    expect(read('scripts/__tests__/playground-module.test.ts')).toContain('makes growth past the maximum fail');
  });

  it('offers Bob\'s limits as a node does, except the time, which is shorter', () => {
    const profile = nodeExchange.offers.offers.find((offer: any) => offer.payload.offer_id === nodeExchange.service.service.offer_id).payload.execution_profile;
    for (const field of ['fuel_limit', 'max_input_bytes', 'max_memory_bytes', 'max_output_bytes'] as const) expect(PLAYGROUND_LIMITS[field], field).toBe(profile[field]);
    expect(PLAYGROUND_LIMITS.max_runtime_ms).toBeLessThan(profile.max_runtime_ms);
  });
});

describe('what it says is not real', () => {
  it('has no network: nothing in the playground opens a connection, and the one file it loads comes from the site', () => {
    expect(section).toContain('There is no network. Bob and Alice call each other inside this page, each message crosses as JSON text, and nobody else can reach Bob.');
    for (const file of modules) expect(moduleSource(file), file).not.toMatch(/WebSocket|XMLHttpRequest|sendBeacon|EventSource|RTCPeerConnection|navigator\.connection/);
    // Nothing fetches. The only file the page loads is the compiler, which its Worker imports from the site's own copy.
    expect(modules.filter((file) => /\bfetch\(/.test(moduleSource(file)))).toEqual([]);
    expect(moduleSource('browser.ts')).toContain("import compilerUrl from '../../generated/assemblyscript/asc.js?url'");
    expect(moduleSource('browser.ts')).toContain('new URL(compilerUrl, location.href).href');
    expect(modules.filter((file) => /\bimport\(/.test(moduleSource(file)))).toEqual(['compiler.ts']);
    expect(moduleSource('compiler.ts').match(/\bimport\(/g)).toHaveLength(1);
    expect(moduleSource('compiler.ts')).toContain('new Blob([COMPILER_WORKER_SOURCE]');
    expect(moduleSource('runner.ts')).toContain('new Blob([RUNNER_SOURCE]');
    // No script names a host, except the links to the Rust sources on GitHub and the address a descriptor gives Bob, which is
    // in a domain that cannot resolve.
    const hosts = new Set(modules.flatMap((file) => Array.from(moduleSource(file).matchAll(/https?:\/\/([a-z0-9.-]+)/gi), (match) => match[1])));
    expect(Array.from(hosts).sort()).toEqual(['github.com', 'provider.playground.invalid']);
  });

  it('keeps the compiler out of the page: it is a URL the Worker loads when someone compiles, not part of a bundle', () => {
    for (const file of modules) {
      const specifiers = Array.from(moduleSource(file).matchAll(/(?:from|import)\s+['"]([^'"]*assemblyscript[^'"]*)['"]/g), (match) => match[1]);
      for (const specifier of specifiers) expect(specifier, `${file} imports the compiler`).toMatch(/\?url$/);
    }
    // Loading starts when the person shows they mean to use the editor, never when the page opens.
    expect(moduleSource('browser.ts')).not.toContain('preload');
    const ui = moduleSource('ui.ts');
    expect(ui.match(/compiler\.preload\(\)/g)).toHaveLength(1);
    expect(ui).toMatch(/const preload = \(\) => \{\s*if \(preloaded \|\| choice\(\)\.kind !== 'function'\) return;\s*preloaded = true;\s*deps\.compiler\.preload\(\);/);
    for (const trigger of ["bob.addEventListener('focusin', preload)", "editorPanel.addEventListener('pointerenter', preload)", "publishButton.addEventListener('pointerenter', preload)"]) expect(ui, trigger).toContain(trigger);
    expect(section).toContain('which load from this site the first time you compile');
  });

  it('puts text on the page as text: no script writes HTML or runs a string, and the compiler quotes the person\'s own words', () => {
    for (const file of modules) expect(moduleSource(file), file).not.toMatch(/innerHTML|outerHTML|insertAdjacentHTML|document\.write|\beval\(|new Function\(/);
    // The one place a string becomes a program is the Worker's own fixed source, and it is a constant of this repository.
    expect(moduleSource('compiler.ts')).toContain('new Blob([COMPILER_WORKER_SOURCE]');
    expect(read('scripts/__tests__/playground-ui.test.ts')).toContain("shows what the compiler quotes from the source as text, never as markup");
  });

  it('saves nothing: no storage of any kind', () => {
    expect(section).toContain('Nothing is saved. Reload the page and both identities, the function, and the evidence are gone.');
    for (const file of modules) expect(moduleSource(file), file).not.toMatch(/localStorage|sessionStorage|indexedDB|document\.cookie|caches\./);
  });

  it('makes only free deals: no price, no invoice, no settlement', () => {
    expect(section).toContain('Deals are free: no invoice, no payment, no settlement.');
    const provider = moduleSource('provider.ts');
    expect(provider).toContain("settlement_method: 'none'");
    expect(provider).toContain("settlement_state: 'none'");
    expect(provider).toContain('price_schedule: { base_fee_msat: 0, success_fee_msat: 0 }');
    expect(moduleSource('consumer.ts')).toContain('max_price_sats: 0');
    expect(moduleSource('consumer.ts')).not.toContain('invoice_bundle');
  });

  it('cannot count fuel, and does not pretend to', () => {
    expect(section).toContain('Browsers cannot count fuel, so a node’s fuel limit is not enforced here. The time limit and the memory maximum stand in for it.');
    const code = (file: string) => moduleSource(file).replace(/\/\*[\s\S]*?\*\//g, '').replace(/^\s*\/\/.*$/gm, '');
    for (const file of ['runner.ts', 'wasm-module.ts', 'consumer.ts']) expect(code(file), file).not.toMatch(/fuel/i);
    expect(moduleSource('protocol.ts')).toContain('Browsers cannot count fuel, so `fuel_limit` is only what the offer says');
    expect(read('scripts/playground/ui.ts')).toContain('Browsers cannot count fuel, so the fuel limit a node enforces is not enforced here.');
  });

  it('is not a node, and points at the way to run one', () => {
    expect(section).toContain('A tab is not a node. It has no access control, no storage, and no marketplace listing. To serve real callers, <a href="/learn/quickstart/">run <code>froglet-node</code></a>.');
    const provider = moduleSource('provider.ts');
    expect(provider).not.toMatch(/bearer|sqlite|authorization/i);
    expect(provider).not.toContain('linked_identities');
  });

  it('does not run JavaScript, because the kernel has no such runtime, and says so', () => {
    expect(section).toContain('You cannot write JavaScript here. The kernel has no JavaScript runtime, and an offer that names one is invalid, so the editor takes AssemblyScript.');
    // The kernel's runtimes, as its parser knows them. JavaScript is not among them.
    const runtimes = repo('froglet-protocol/src/lib.rs');
    for (const name of ['"any"', '"wasm"', '"python"', '"container"', '"builtin"']) expect(runtimes, name).toContain(`${name} => Ok(Self::`);
    expect(runtimes).not.toMatch(/javascript|"js"|"node"/i);
    // A test in the verifier shows an offer that names one is invalid though its signature and hashes are genuine.
    const verifier = repo('froglet-verify/src/lib.rs');
    expect(verifier).toContain('fn an_offer_naming_a_runtime_the_kernel_lacks_is_invalid_though_its_signature_is_genuine()');
    expect(verifier).toContain('for runtime in ["javascript", "js", "node", "typescript", "assemblyscript"]');
  });

  it('says the two sides are separated by JSON text, which the wire enforces', () => {
    expect(moduleSource('wire.ts')).toContain('JSON.parse(JSON.stringify(request))');
    expect(moduleSource('wire.ts')).toContain('JSON.parse(JSON.stringify(answered))');
  });
});

describe('the functions Bob starts with', () => {
  it('are the Rust services in the repository, ported, and two small ones that have no Rust', () => {
    expect(FUNCTIONS.map((fn) => fn.id)).toEqual(['adder', 'fibonacci', 'echo', 'never-ends']);
    expect(FUNCTIONS.filter((fn) => fn.rust).map((fn) => fn.id)).toEqual(['adder', 'fibonacci']);
    for (const fn of FUNCTIONS.filter((info) => info.rust)) {
      const path = fn.rust!.replace('https://github.com/AlitheaBio/froglet/blob/main/', '');
      expect(existsSync(resolve(repoRoot, path)), path).toBe(true);
    }
    expect(new Set(FUNCTIONS.map((fn) => fn.serviceId)).size).toBe(FUNCTIONS.length);
    for (const fn of FUNCTIONS) {
      expect(fn.serviceId, fn.id).toMatch(/^[a-z0-9][a-z0-9._-]{0,63}$/);
      expect(() => JSON.parse(fn.input), `${fn.id} input`).not.toThrow();
      expect(fn.code.length, fn.id).toBeLessThan(MAX_SOURCE_CHARS);
      expect(existsSync(resolve(functionsDir, `${fn.id}.as`)), `${fn.id}.as`).toBe(true);
    }
  });

  it('are written as a respond function, and leave alloc and run to the contract', () => {
    expect(readdirSync(functionsDir).sort()).toEqual(['adder.as', 'contract.as', 'echo.as', 'fibonacci.as', 'never-ends.as']);
    for (const fn of FUNCTIONS) {
      expect(fn.code, fn.id).toMatch(/^function respond\(request: string\): string \{$/m);
      expect(fn.code, fn.id).not.toMatch(/\bfunction (alloc|run)\b/);
      expect(fn.code, fn.id).not.toMatch(/^export /m);
    }
    expect(CONTRACT).toContain('export function alloc(len: i32): usize');
    expect(CONTRACT).toContain('export function run(pointer: usize, len: i32): i64');
    expect(CONTRACT).toContain('respond(String.UTF8.decodeUnsafe(pointer, len))');
    expect(section).toContain('<pre data-play-contract');
  });

  it('are compiled by the Rust services\' own workspace build, which the tests use as the answer to check them against', () => {
    const build = readFileSync(resolve(docsSite, 'scripts/build-playground.mjs'), 'utf8');
    for (const name of ['adder', 'fibonacci']) {
      expect(build).toContain(`'${name}'`);
      expect(existsSync(resolve(repoRoot, `examples/wasm-services/${name}/src/lib.rs`))).toBe(true);
    }
    expect(build).toContain("'examples/wasm-services/Cargo.toml'");
    expect(repo('examples/wasm-services/README.md')).toContain('cargo build --release --target wasm32-unknown-unknown --manifest-path examples/wasm-services/Cargo.toml');
    expect(repo('examples/README.md')).toContain('wasm-services');
    // The page does not serve the Rust builds. The tests run them next to what the editor compiles, and compare answers.
    expect(moduleSource('browser.ts')).not.toContain('playground/adder');
    const ported = read('scripts/__tests__/playground-compiler.test.ts');
    expect(ported).toContain("describe('the ports answer what the Rust services answer'");
  });

  it('are compiled with options that keep the module a Froglet function, and the page says which compiler', () => {
    expect(COMPILER_ARGS).toEqual(expect.arrayContaining(['--runtime', 'stub', '--use', 'abort=']));
    expect(section).toContain('Compiled in this page by <a href="https://www.assemblyscript.org/"');
    expect(section).toContain('href="/licenses/AssemblyScript-toolchain.txt"');
    expect(existsSync(resolve(docsSite, 'public/licenses/AssemblyScript-toolchain.txt'))).toBe(true);
    const notices = readFileSync(resolve(docsSite, 'public/licenses/AssemblyScript-toolchain.txt'), 'utf8');
    for (const part of ['Apache License, Version 2.0', 'The MIT License', 'The 3-Clause BSD License']) expect(notices, part).toContain(part);
    // The notices name the versions in the bundle, so a compiler upgrade that forgets them fails here.
    const version = (name: string): string => JSON.parse(readFileSync(resolve(docsSite, 'node_modules', name, 'package.json'), 'utf8')).version;
    expect(notices).toContain(`AssemblyScript ${version('assemblyscript')}\n`);
    expect(notices).toContain(`Binaryen ${version('binaryen')}\n`);
    expect(notices).toContain(`long.js ${version('long')}\n`);
    const manifest = JSON.parse(readFileSync(resolve(docsSite, 'package.json'), 'utf8'));
    expect(manifest.devDependencies.assemblyscript, 'pinned to one version, so the compiler is the one the tests ran').toMatch(/^\d+\.\d+\.\d+$/);
  });
});

describe('how the playground is built and kept building', () => {
  it('is built with the site, so a fresh checkout gets it, and the generated files are not committed', () => {
    const scripts = JSON.parse(readFileSync(resolve(docsSite, 'package.json'), 'utf8')).scripts;
    expect(scripts.build).toContain('npm run build:wasm');
    expect(scripts['build:wasm']).toBe('npm run build:verifier && npm run build:playground');
    expect(scripts['build:playground']).toBe('node scripts/build-playground.mjs');
    expect(repo('.gitignore')).toContain('docs-site/src/generated/');
  });

  it('is built before the site tests run in CI, and its crate is checked for wasm32 by the strict checks', () => {
    expect(repo('.github/workflows/ci.yml')).toContain('npm run build:wasm --prefix docs-site');
    const strict = repo('scripts/strict_checks.sh');
    expect(strict).toContain('cargo check -p froglet-wasm --target wasm32-unknown-unknown');
    expect(repo('Cargo.toml')).toMatch(/^members = \[[^\]]*"froglet-wasm"/m);
  });

  it('is not blocked by a content security policy: the site sets none for this page, and a site-wide one would need blob: workers and Wasm', () => {
    // The playground starts Web Workers from blob: URLs and compiles WebAssembly. The Worker sets a policy only on the
    // shared-service pages under /s/ and their QR images; every other route is served from the static assets as they are.
    // If a policy is ever added for the whole site, it must allow `worker-src blob:` and `'wasm-unsafe-eval'`.
    const worker = readFileSync(resolve(src, 'worker.ts'), 'utf8');
    const start = worker.indexOf("url.pathname.startsWith('/s/')");
    const end = worker.indexOf("url.pathname === '/api/shared-service'");
    expect(start).toBeGreaterThan(0);
    expect(end).toBeGreaterThan(start);
    const policies = Array.from(worker.matchAll(/content-security-policy/gi)).map((match) => match.index!);
    expect(policies.length).toBeGreaterThan(0);
    for (const at of policies) expect(at > start && at < end, `a policy at ${at} is outside the /s/ routes`).toBe(true);
    expect(worker).toContain('return env.ASSETS.fetch(request);');
    expect(readFileSync(resolve(docsSite, 'public/_headers'), 'utf8')).not.toMatch(/content-security-policy/i);
  });

  it('is labelled a prototype, with evidence, and says what it has and has not been run against', () => {
    const entry = MATURITY.find((item) => item.id === 'playground');
    expect(entry?.status).toBe('prototype');
    expect(existsSync(resolve(repoRoot, entry!.evidence))).toBe(true);
    expect(entry!.note).toContain('two opt-in tests on one machine: a real node’s requester calls its provider, and a real node runs the modules its editor compiles');
    expect(entry!.note).toContain('not over a network');
    expect(flat(page)).toContain("const entry = pill('playground')");
  });
});

describe('the playground stylesheet', () => {
  it('prefixes every rule with the story shell so it outranks its element styles', () => {
    const parts = (list: string) => {
      const out: string[] = [];
      let depth = 0;
      let current = '';
      for (const char of list) {
        if (char === '(') depth += 1;
        if (char === ')') depth -= 1;
        if (char === ',' && depth === 0) {
          out.push(current.trim());
          current = '';
        } else current += char;
      }
      return [...out, current.trim()];
    };
    const rules = Array.from(cssRules.matchAll(/(^|\})\s*([^{}@]+)\{/g)).map((match) => match[2].trim());
    expect(rules.length).toBeGreaterThan(60);
    expect(rules.filter((selector) => parts(selector).some((part) => !/^(\[data-theme='light'\] )?\.story-main /.test(part)))).toEqual([]);
  });

  it('styles only what the page and its script use', () => {
    const classes = new Set(Array.from(cssRules.matchAll(/\.(play[a-z0-9_-]*)/g)).map((match) => match[1]));
    expect(classes.size).toBeGreaterThan(30);
    const rendered = page + read('scripts/playground/ui.ts');
    expect(Array.from(classes).filter((name) => !new RegExp(`\\b${name}\\b`).test(rendered))).toEqual([]);
  });

  it('uses only colours and fonts from the design tokens, and every token it names exists', () => {
    expect(cssRules).not.toMatch(/#[0-9a-f]{3,8}\b/i);
    expect(cssRules).not.toMatch(/\brgba?\(|\bhsla?\(/);
    const tokens = read('styles/tokens.css');
    const named = new Set(Array.from(cssRules.matchAll(/var\(--([a-z0-9-]+)/g)).map((match) => match[1]));
    expect(named.size).toBeGreaterThan(15);
    expect(Array.from(named).filter((name) => !new RegExp(`--${name}\\s*:`).test(tokens))).toEqual([]);
  });

  it('gives the editor\'s frame a class of its own: play__code is the wire\'s status code, and shared rules turned it into a black bar', () => {
    expect(markup).not.toContain('class="play__code"');
    expect(markup).toContain('<div class="play__frame">');
    expect(read('scripts/playground/ui.ts')).toContain("class: 'play__code', 'data-ok'");
    expect(cssRules).toMatch(/\.story-main \.play__frame \{ display: grid;/);
    // Every rule for the status code sets its colour and nothing else, so it cannot take on a layout.
    const statusRules = Array.from(cssRules.matchAll(/\.play__code[^{]*\{([^}]*)\}/g), (match) => match[1].trim());
    expect(statusRules.length).toBeGreaterThan(0);
    for (const body of statusRules) expect(body).toMatch(/^color: var\(--[a-z0-9-]+\);$/);
  });

  it('keeps elements hidden that the page hides, whatever display they are given', () => {
    expect(cssRules).toMatch(/\.story-main \.play \[hidden\], \.story-main \.play\[hidden\] \{ display: none; \}/);
    // Every element the script or the markup hides has a class or is the play root, so the rule above reaches it.
    expect(markup).toMatch(/<dl class="play__facts" data-play-facts hidden>/);
  });

  it('stacks the two sides before they are squeezed, and the two columns of the truth box on a phone', () => {
    expect(css).toMatch(/@media \(max-width: 1120px\) \{\s*\.story-main \.play__row, \.story-main \.play__bob-grid \{ grid-template-columns: minmax\(0, 1fr\)/);
    expect(css).toMatch(/@media \(max-width: 760px\) \{[^@]*\.story-main \.play__truth-grid \{ grid-template-columns: minmax\(0, 1fr\)/);
    expect(css).toMatch(/@media \(max-width: 420px\) \{[^@]*\.story-main \.play__actions \.btn \{ width: 100%; \}/);
  });

  it('shows focus on every control it styles', () => {
    expect(cssRules).toContain('.story-main .play :is(input, select, textarea):focus-visible { outline: 3px solid var(--frog-400)');
    expect(read('styles/story-pages.css')).toContain('.story-main :is(a, button, summary, textarea, select):focus-visible');
  });
});
