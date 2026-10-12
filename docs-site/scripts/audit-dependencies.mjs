import { createHash } from 'node:crypto';
import { spawnSync } from 'node:child_process';
import { existsSync, mkdtempSync, readFileSync, readdirSync, realpathSync, rmSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { dirname, join, relative, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

// Review the observed cache behavior even when npm no longer reports the advisory.
// This is an application applicability review, not a cache-behavior patch. See DEPENDENCY_SECURITY.md.
const ADVISORY = 'https://github.com/advisories/GHSA-ch52-4w7c-c8xp';
const ADVISORY_TITLE = 'http-cache-semantics max-stale handling can disclose cross-user cached responses';
const EXPIRES = Date.parse('2026-11-16T00:00:00Z');
const VERSIONS = { astro: '7.3.5', 'http-cache-semantics': '4.3.0', wrangler: '4.143.1' };
const EFFECTS = new Set(['http-cache-semantics', 'astro', '@astrojs/mdx', '@astrojs/starlight', 'astro-expressive-code']);
const HASHES = {
  astroConfig: '40f6e2207d254b268bfb6f2c742e2f472e626bfd0379e5ced06505f63129f791',
  wranglerConfig: '8fb3b99a0b079cdc2a98f0057c2c2036243d925912abc7ad9da05046dcc14250',
  remoteSource: 'f373fa76e3112446db327c79b34e2bbb1ef1dcad41affb60788adf30edc9588e',
  // Only the generated Wasm filename's content hash is normalized. No JavaScript is omitted.
  bundle: '0c528697ee570a2802b16b25b69f287ba20faf2577e95276047a94cb467e7990',
};
const WORKER_SOURCES = {
  'node_modules/qrcode-generator/dist/qrcode.mjs': 'ea91d7118a5395289170da848b7c6758b996163bfbccf312591ab65a4911b7c0',
  'src/data/service-qr.ts': '9dda63c5219b7441688f36f38e9de951959859eb2eee13f32c52ea0470555a83',
  'src/data/shared-service.ts': '1d5072d27290de60b78f81e9a2073b6afd60fcefe87dec1484bffb942849da15',
  'src/data/service-presentation.ts': '53eddcc40b5d3ca5fdc3adb14cdfbbb4891c431c7f33a8011a9b45bdc7ec3072',
  'src/data/live-snapshot.ts': '11376a70e5a7d39b0a19f2ba1724e60c06ab0183ab72528eae3b970ef0f989da',
  'src/data/file-download.ts': 'aaf23bc51f8e444ac56d08d9738fef80cd9481ea07e75190a0b76f6b53386dfa',
  'src/data/service-link.ts': '094e567764f5715c7c8ddf8bbe8b350ac897d3e40412e921b704e3fa18db3b47',
  'src/data/service-link-page.ts': '3fda670ed4ec7f384cad3b543743793bffc57c598402d7b9ac87d308124b2c2b',
  'src/data/service-link-verifier.ts': 'ecee4c186515e943629a15a281ce8843001e3f7e69bf57c9af3104940abbc3b3',
  'src/generated/verifier/froglet_verify.js': '4c938b45a3d3abdf7aeaf000a5fcca4b70bd678989a06d9f61d557ae8ab1da09',
  'src/data/public-demo-config.ts': '654da53b91039d812c1587c83e1e79a7c4c57c9bf6e1cd20ad79443ce5ace082',
  'src/data/public-demo-proxy.ts': '615669bbded5ba359cd1743112390b0f0300bd84e9d41280765552bc1b05e7e4',
  'src/worker.ts': '1cd1a0c41c9fa4f30b8133026345dd5fcec3303f6329366c12a7ee321da48f35',
};
// Existing non-executable evidence links to tests; any change needs review.
const TEST_REFERENCE_SOURCE = { 'src/data/maturity.ts': '35ddb56fef646cb27bad4e00f01ba42a710376366a953e2d4e79dabcbc5d107f' };
const hash = (value) => createHash('sha256').update(value).digest('hex');

export function evaluateAudit(audit) {
  if (audit?.error || audit?.auditReportVersion !== 2 || !audit.vulnerabilities ||
      typeof audit.vulnerabilities !== 'object' || Array.isArray(audit.vulnerabilities) ||
      !Number.isInteger(audit.metadata?.vulnerabilities?.high) ||
      !Number.isInteger(audit.metadata?.vulnerabilities?.critical)) {
    throw new Error('Invalid npm audit response; dependency audit remains blocking.');
  }
  const rank = { info: 0, low: 1, moderate: 2, high: 3, critical: 4 };
  for (const [name, entry] of Object.entries(audit.vulnerabilities)) {
    if (!entry || entry.name !== name || typeof entry.severity !== 'string' || !Object.hasOwn(rank, entry.severity) || !Array.isArray(entry.via)) {
      throw new Error(`Invalid npm audit finding: ${name}`);
    }
    for (const via of entry.via) {
      const dependency = typeof via === 'string' ? audit.vulnerabilities[via] : via;
      if (!dependency || typeof dependency.severity !== 'string' || !Object.hasOwn(rank, dependency.severity) || rank[dependency.severity] > rank[entry.severity]) {
        throw new Error(`Invalid npm audit dependency severity: ${name}`);
      }
    }
    const maximum = Math.max(...entry.via.map((via) => rank[(typeof via === 'string' ? audit.vulnerabilities[via] : via).severity]));
    if (maximum !== rank[entry.severity]) throw new Error(`Invalid npm audit aggregate severity: ${name}`);
  }
  function covered(name, seen = new Set()) {
    const entry = audit.vulnerabilities[name];
    if (!entry || seen.has(name)) return false;
    if (rank[entry.severity] < rank.high) return true;
    if (!EFFECTS.has(name) || !entry.via.length) return false;
    const next = new Set([...seen, name]);
    return entry.via.every((via) => typeof via === 'string' ? covered(via, next) :
      rank[via.severity] < rank.high || name === 'http-cache-semantics' && via?.name === name && via.dependency === name &&
      via.url === ADVISORY && via.source === 1240991 && via.title === ADVISORY_TITLE &&
      via.severity === 'high' && via.range === '<=4.2.0');
  }
  const exceptedPackages = [];
  for (const [name, entry] of Object.entries(audit.vulnerabilities)) {
    if (entry?.severity !== 'high' && entry?.severity !== 'critical') continue;
    if (entry.severity === 'critical' || !covered(name)) {
      throw new Error(`High/critical advisory not covered by the reviewed website exception: ${name}`);
    }
    exceptedPackages.push(name);
  }
  const highCount = Object.values(audit.vulnerabilities).filter((entry) => entry?.severity === 'high').length;
  const criticalCount = Object.values(audit.vulnerabilities).filter((entry) => entry?.severity === 'critical').length;
  if (highCount !== audit.metadata.vulnerabilities.high || criticalCount !== audit.metadata.vulnerabilities.critical) {
    throw new Error('Invalid npm audit severity counts; dependency audit remains blocking.');
  }
  return { exceptedPackages: exceptedPackages.sort() };
}

export function validatePublishedVersions(versions) {
  if (!Array.isArray(versions) || !versions.includes(VERSIONS['http-cache-semantics']) || versions.some((version) => typeof version !== 'string')) {
    throw new Error('Invalid npm registry version response; exception cannot be applied.');
  }
  const reviewed = VERSIONS['http-cache-semantics'].split('.').map(Number);
  for (const version of versions) {
    const match = /^(0|[1-9]\d*)\.(0|[1-9]\d*)\.(0|[1-9]\d*)(?:-([\da-z-]+(?:\.[\da-z-]+)*))?(?:\+[\da-z-]+(?:\.[\da-z-]+)*)?$/i.exec(version);
    if (!match || match[4]?.split('.').some((part) => /^\d+$/.test(part) && part.length > 1 && part.startsWith('0'))) {
      throw new Error('Invalid npm registry version response; exception cannot be applied.');
    }
    if (!match[4] && (Number(match[1]) > reviewed[0] || Number(match[1]) === reviewed[0] &&
        (Number(match[2]) > reviewed[1] || Number(match[2]) === reviewed[1] && Number(match[3]) > reviewed[2]))) {
      throw new Error('A newer published http-cache-semantics version requires review/update before using this exception.');
    }
  }
}

export function scanProjectSources(siteRoot) {
  const read = (file) => readFileSync(join(siteRoot, file), 'utf8');
  const projectSources = {};
  function walk(directory) {
    for (const entry of readdirSync(join(siteRoot, directory), { withFileTypes: true })) {
      const path = `${directory}/${entry.name}`;
      if (entry.isSymbolicLink()) throw new Error(`Unreviewed production source symlink: ${path}`);
      if (entry.isDirectory() && entry.name !== '__tests__') walk(path);
      else if (entry.isFile() && /\.(?:astro|mdx|[cm]?tsx?|[cm]?jsx?)$/.test(entry.name)) projectSources[path] = read(path);
    }
  }
  walk('src');
  return projectSources;
}

export function loadApplicabilityContext(siteRoot) {
  const read = (file) => readFileSync(join(siteRoot, file), 'utf8');
  return {
    versions: Object.fromEntries(Object.keys(VERSIONS).map((name) => [name, JSON.parse(read(`node_modules/${name}/package.json`)).version])),
    lock: JSON.parse(read('package-lock.json')),
    astroConfig: read('astro.config.mjs'),
    wranglerConfig: read('wrangler.jsonc'),
    remoteSource: read('node_modules/astro/dist/assets/build/remote.js'),
    reviewedSources: Object.fromEntries(Object.keys(WORKER_SOURCES).map((file) => [file, read(file)])),
    projectSources: scanProjectSources(siteRoot),
    now: new Date(),
  };
}

export function validateApplicability(context, { requireBundle = true } = {}) {
  if (!Number.isFinite(+context.now) || +context.now >= EXPIRES) throw new Error('Website advisory applicability exception expired on 2026-10-17 UTC.');
  for (const [name, expected] of Object.entries(VERSIONS)) {
    if (context.versions[name] !== expected || context.lock.packages?.[`node_modules/${name}`]?.version !== expected) {
      throw new Error(`Reviewed dependency version changed: ${name}; applicability review required.`);
    }
  }
  for (const key of ['astroConfig', 'wranglerConfig', 'remoteSource']) {
    if (hash(context[key]) !== HASHES[key]) throw new Error(`Reviewed ${key} changed; applicability review required.`);
  }
  // The exact reviewed Astro config uses default static output and no SSR adapter.
  const wrangler = JSON.parse(context.wranglerConfig);
  if (wrangler.main !== './src/worker.ts' || wrangler.assets?.directory !== './dist' || /\b(?:output|adapter)\s*:/.test(context.astroConfig)) {
    throw new Error('Reviewed static output/custom Worker configuration changed.');
  }
  const consumers = Object.entries(context.lock.packages).flatMap(([path, pkg]) =>
    ['dependencies', 'optionalDependencies', 'peerDependencies', 'devDependencies'].flatMap((field) =>
      Object.entries(pkg[field] ?? {}).filter(([name, spec]) => name === 'http-cache-semantics' ||
        typeof spec === 'string' && spec.startsWith('npm:http-cache-semantics@')).map(([name, spec]) => ({ path, field, name, spec }))));
  if (consumers.length !== 1 || consumers[0].path !== 'node_modules/astro' || consumers[0].field !== 'dependencies' || consumers[0].spec !== '^4.2.0') {
    throw new Error('Affected dependency consumer graph changed; applicability review required.');
  }
  // Deliberately conservative: even a comment/prose mention requires review. Decode
  // module-string escapes so commented/deep/template-literal imports cannot bypass it.
  const forbiddenReference = /astro:assets|astro\/assets|http-cache-semantics/;
  for (const [file, source] of Object.entries(context.projectSources)) {
    const decoded = source.replace(/\\(?:\r\n|[\r\n\u2028\u2029])|\\(?:u\{([\da-f]+)\}|u([\da-f]{4})|x([\da-f]{2})|([0-3][0-7]{0,2}|[4-7][0-7]?)|([\s\S]))/gi,
      (_, point, unicode, byte, octal, identity) => point || unicode || byte
        ? String.fromCodePoint(parseInt(point ?? unicode ?? byte, 16))
        : octal ? String.fromCodePoint(parseInt(octal, 8)) : identity ?? '');
    if (forbiddenReference.test(decoded)) throw new Error(`Unreviewed project asset/cache import or reference: ${file}`);
    if (decoded.includes('__tests__') && hash(source) !== TEST_REFERENCE_SOURCE[file]) {
      throw new Error(`Production source references an excluded test module: ${file}`);
    }
  }
  for (const [file, expected] of Object.entries(WORKER_SOURCES)) {
    if (hash(context.reviewedSources[file] ?? '') !== expected) throw new Error(`Reviewed Worker source changed: ${file}`);
  }
  if (!requireBundle) return;
  const bundle = context.bundle;
  if (!bundle?.sourcesMatch) throw new Error('Missing or mismatched Worker source map proof.');
  const expectedSources = Object.keys(WORKER_SOURCES).sort();
  if (JSON.stringify([...bundle.sources].sort()) !== JSON.stringify(expectedSources)) {
    throw new Error('Worker bundle sources changed; applicability review required.');
  }
  if (/http-cache-semantics|CachePolicy|satisfiesWithoutRevalidation|evaluateRequest|max-stale/.test(bundle.code)) {
    throw new Error('Unreviewed cache-policy code in deployed Worker bundle.');
  }
  const imports = bundle.code.match(/\.\/[a-f0-9]{40}-froglet_verify_bg\.wasm/g) ?? [];
  const normalized = bundle.code.replace(/\.\/[a-f0-9]{40}-froglet_verify_bg\.wasm/g, './froglet_verify_bg.wasm');
  if (imports.length !== 1 || hash(normalized) !== HASHES.bundle) throw new Error('Reviewed Worker bundle changed; applicability review required.');
}

function run(command, args, siteRoot, options = {}) {
  const result = spawnSync(command, args, { cwd: siteRoot, encoding: 'utf8', maxBuffer: 16 * 1024 * 1024, timeout: 45_000, ...options });
  if (result.error || result.signal) throw new Error(`${command} failed: ${result.error?.message ?? result.signal}`);
  return result;
}

export async function inspectWorkerBundle(siteRoot) {
  const directory = mkdtempSync(join(tmpdir(), 'froglet-docs-audit-'));
  try {
    const result = run(process.execPath, ['node_modules/wrangler/bin/wrangler.js', 'deploy', '--config', 'wrangler.jsonc', '--dry-run', '--outdir', directory], siteRoot, {
      env: { ...process.env, CI: '1', WRANGLER_SEND_METRICS: 'false', WRANGLER_WRITE_LOGS: 'false' },
    });
    if (result.status !== 0 || !result.stdout.includes('--dry-run: exiting now.')) throw new Error(`Worker dry-run failed: ${result.stderr || result.stdout}`);
    const code = readFileSync(join(directory, 'worker.js'), 'utf8');
    const map = JSON.parse(readFileSync(join(directory, 'worker.js.map'), 'utf8'));
    if (!Array.isArray(map.sources) || !Array.isArray(map.sourcesContent) || map.sources.length !== map.sourcesContent.length) {
      throw new Error('Invalid Worker source map.');
    }
    const sources = map.sources.map((source) => relative(siteRoot, resolve(directory, source)).replaceAll('\\', '/'));
    const sourcesMatch = map.sources.every((source, index) => readFileSync(resolve(directory, source), 'utf8') === map.sourcesContent[index]);
    const wasmImports = code.match(/\.\/[a-f0-9]{40}-froglet_verify_bg\.wasm/g) ?? [];
    if (wasmImports.length !== 1 || !existsSync(resolve(directory, wasmImports[0]))) throw new Error('Missing generated verifier Wasm in Worker dry-run.');
    return { code, sources, sourcesMatch };
  } finally {
    rmSync(directory, { recursive: true, force: true });
  }
}

async function main() {
  const siteRoot = resolve(dirname(realpathSync(fileURLToPath(import.meta.url))), '..');
  const result = run('npm', ['audit', '--json'], siteRoot);
  if (result.status !== 0 && result.status !== 1) throw new Error(`npm audit failed: ${result.stderr}`);
  const audit = JSON.parse(result.stdout);
  const evaluation = evaluateAudit(audit);
  // A clean audit is registry metadata, not evidence that the observed behavior
  // changed. Keep the application/registry/bundle review mandatory at this pin.
  const context = loadApplicabilityContext(siteRoot);
  validateApplicability(context, { requireBundle: false });
  const registry = run('npm', ['view', 'http-cache-semantics', 'versions', '--json', '--registry=https://registry.npmjs.org'], siteRoot);
  if (registry.status !== 0) throw new Error(`npm registry lookup failed: ${registry.stderr}`);
  validatePublishedVersions(JSON.parse(registry.stdout));
  context.bundle = await inspectWorkerBundle(siteRoot);
  validateApplicability(context);
  console.log(`Raw npm audit: high=${audit.metadata.vulnerabilities.high}, critical=${audit.metadata.vulnerabilities.critical}. Covered advisory packages: ${evaluation.exceptedPackages.join(', ') || 'none'}.`);
  console.log(`Known max-stale/shared-cookie behavior is unchanged in reviewed http-cache-semantics ${VERSIONS['http-cache-semantics']}; this update does not change that behavior. Application review excludes that behavior from the static site/custom Worker deployment bundle and expires 2026-11-16 UTC, even with a clean raw audit.`);
  console.log(`Actual dry-run Worker SHA-256: ${hash(context.bundle.code)}. All other high/critical advisories remain blocking.`);
}

if (process.argv[1] && existsSync(resolve(process.argv[1])) &&
    realpathSync(fileURLToPath(import.meta.url)) === realpathSync(resolve(process.argv[1]))) {
  main().catch((error) => { console.error(error.message); process.exitCode = 1; });
}
