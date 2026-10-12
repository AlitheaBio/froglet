import { existsSync, readFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { describe, expect, it } from 'vitest';
import { MATURITY } from '../../data/maturity';
import { brokenInternalLinks, docsSite, repoRoot, src } from './route-helpers';

const read = (path: string) => readFileSync(resolve(src, path), 'utf8');
const page = read('pages/managed.astro');
const css = read('styles/organizations.css');
const cssRules = css.replace(/\/\*[\s\S]*?\*\//g, '');

/** Width and height from a PNG's IHDR chunk. */
function pngSize(path: string): { width: number; height: number } {
  const bytes = readFileSync(resolve(docsSite, path));
  return { width: bytes.readUInt32BE(16), height: bytes.readUInt32BE(20) };
}

describe('For organizations page', () => {
  it('is built on the shared story layout with its own stylesheet, agent metadata and social card', () => {
    expect(page).toContain("import StoryLayout from '../layouts/StoryLayout.astro'");
    expect(page).toContain("import '../styles/organizations.css'");
    expect(page).toContain('<AgentMeta slot="head" route="managed" />');
    expect(page).toContain("src: 'https://froglet.dev/og/organizations.png'");
    expect(page).not.toMatch(/<style[\s>]/);
    expect(page).not.toMatch(/style="/);
    expect(pngSize('public/og/organizations.png')).toEqual({ width: 1200, height: 630 });
  });

  it('replaces the stale "Cloud-hosted Froglet, coming soon" card that contradicted the page', () => {
    const generator = readFileSync(resolve(docsSite, 'scripts/generate-og.mjs'), 'utf8');
    expect(generator).not.toContain('Cloud-hosted Froglet');
    expect(generator).not.toContain("slug: 'managed'");
    expect(generator).toContain("slug: 'organizations'");
    expect(existsSync(resolve(docsSite, 'public/og/managed.png'))).toBe(false);
  });

  it('has one heading, and every hero link points at a section that exists', () => {
    expect(page.match(/<h1[\s>]/g)).toHaveLength(1);
    for (const anchor of page.match(/href="#[a-z-]+"/g) ?? []) {
      const id = anchor.slice(7, -1);
      expect(page, `#${id} has no target`).toContain(`id="${id}"`);
    }
    for (const id of ['controls', 'exchange', 'pilot', 'questions', 'direction']) expect(page).toContain(`id="${id}"`);
  });

  it('answers each agreement question with both what works and what does not', () => {
    const asks = page.match(/^\s+ask: '/gm) ?? [];
    const todays = page.match(/^\s+today: '/gm) ?? [];
    const gaps = page.match(/^\s+gap: '/gm) ?? [];
    expect(asks.length).toBeGreaterThanOrEqual(7);
    expect(todays).toHaveLength(asks.length);
    expect(gaps).toHaveLength(asks.length);
  });

  it('builds every maturity label from the maturity data instead of writing it in the page', () => {
    expect(page).toMatch(/import \{[^}]*MATURITY[^}]*\} from '\.\.\/data\/maturity'/);
    expect(page).toContain('statusLabel(entry.status)');
    expect(page).toContain('MATURITY_LADDER[entry.status].meaning');
    expect(page).not.toMatch(/data-status="(spec|prototype|beta|production)"/);
    const ids = Array.from(page.matchAll(/maturity: \[([^\]]*)\]/g)).flatMap((match) => Array.from(match[1].matchAll(/'([a-z0-9-]+)'/g)).map((id) => id[1]));
    expect(ids.length).toBeGreaterThan(0);
    const known = new Set(MATURITY.map((entry) => entry.id));
    expect(ids.filter((id) => !known.has(id))).toEqual([]);
  });

  it('never presents the beta as production-ready, certified, or commercially supported', () => {
    const banned = /production[- ]ready|enterprise[- ]grade|\bcertified\b|contact sales|book a demo|request a demo|fully managed|guaranteed uptime/i;
    expect(page.match(banned)).toBeNull();
  });

  it('says what a signed record does not prove, in the row and in the answer', () => {
    expect(page).toContain('not that the work was correct or who the legal signatory is');
    expect(page).toContain('identify a legal signatory');
  });

  it('resolves every internal link, including anchors, to something that exists', () => {
    expect(page.match(/href="\/|href: '\//g)?.length).toBeGreaterThan(5);
    expect(brokenInternalLinks(page)).toEqual([]);
  });

  it('points GitHub links at files that exist in the repository', () => {
    const paths = Array.from(page.matchAll(/https:\/\/github\.com\/AlitheaBio\/froglet\/blob\/main\/([^"'#\s)]+)/g)).map((match) => match[1]);
    for (const path of paths) expect(existsSync(resolve(repoRoot, path)), path).toBe(true);
  });
});

describe('claims on the page match the repository', () => {
  const repo = (path: string) => readFileSync(resolve(repoRoot, path), 'utf8');

  it('states the invitation limits the node enforces', () => {
    const policy = repo('src/provider_policy.rs');
    expect(policy).toContain('expires_at > now.saturating_add(30 * 86400)');
    expect(policy).toContain('(2..=10_000).contains(&max_requests)');
    expect(page).toContain('lasts up to 30 days and allows 2 to 10,000 requests; a normal call uses two');
    expect(repo('docs/HTTP_SERVICES.md')).toContain('A normal invocation consumes two invitation requests');
  });

  it('states the catalog limits the preparer enforces', () => {
    const prepare = repo('src/cli/prepare.rs');
    expect(prepare).toContain('const MAX_BYTES: usize = 16 * 1024 * 1024;');
    expect(prepare).toContain('const MAX_ROWS: usize = 100_000;');
    expect(page).toContain('16 MiB and 100,000 rows per table');
  });

  it('says services are open by default, because the access mode defaults to open', () => {
    expect(repo('src/provider_policy.rs')).toMatch(/#\[default\]\s*Open,/);
    expect(repo('docs/CONFIGURATION.md')).toMatch(/`FROGLET_PROVIDER_ACCESS_MODE` \| `open`/);
    expect(page).toContain('Services are open by default');
    expect(page).toContain('No. A new service is open to anyone who can reach it');
  });

  it('says allowances are provider-wide counters that neither cap cloud costs nor stop admitted work', () => {
    const configuration = repo('docs/CONFIGURATION.md');
    expect(configuration).toContain('Limits apply to this database, not an entire cloud account');
    expect(configuration).toContain('Already admitted work can finish after a limit');
    expect(configuration).toContain('not a monetary cap on a cloud bill');
    expect(page).toContain('provider-wide counters, not currency budgets or a cap on cloud costs');
    expect(page).toContain('Deals already admitted still finish');
  });

  it('says a protected node will not start without finite allowances', () => {
    expect(repo('src/provider_policy.rs')).toContain('protected providers require FROGLET_PROVIDER_MAX_TOTAL_DEALS');
    expect(page).toContain('A protected node will not start without finite allowances');
  });

  it('says an assistant pays nothing by default', () => {
    expect(repo('src/cli/mcp.rs')).toContain('defaults to 0 (free only). Paid calls also require a buyer wallet and runtime cumulative spend budget');
    expect(repo('docs/CONFIGURATION.md')).toContain('when unset, paid deals are refused');
    expect(page).toContain('An assistant pays nothing by default');
  });

  it('discloses that the relay can see requests and results', () => {
    expect(repo('docs/RELAY.md')).toContain('The relay terminates TLS and sees request/response plaintext.');
    // Both places a reader might look: the confidentiality row and the reviewer question.
    expect(page).toContain('publishes through a Froglet relay that terminates TLS and can see requests and results');
    expect(page).toContain('the relay terminates TLS and can see requests and results');
  });

  it('discloses what the threat model accepts: no key revocation and keys that are not encrypted at rest', () => {
    const threats = repo('docs/THREAT_MODEL.md');
    expect(threats).toContain('no protocol-level key revocation');
    expect(threats).toContain('Seeds are plaintext-on-disk');
    expect(page).toContain('no protocol-level key revocation');
    expect(page).toContain('not encrypted at rest');
  });

  it('describes the credential policy file the node enforces', () => {
    expect(repo('src/config.rs')).toContain('HTTP operation credential policy must be owned by this user with mode 0600');
    expect(page).toContain('owned by its user with mode 0600');
  });

  it('quotes the security policy timelines and supported versions', () => {
    const policy = repo('SECURITY.md');
    expect(policy).toContain('Acknowledgement within 48 hours');
    expect(policy).toContain('An assessment within 7 days');
    expect(policy).toContain('within 90 days');
    expect(policy).toContain('Only the latest release on the `main` branch is actively supported');
    expect(page).toContain('acknowledgement within 48 hours, an assessment within 7 days, and a fix or documented mitigation within 90 days');
    expect(page).toContain('Only the latest release receives security fixes');
  });

  it('says receipts are public because the feed is not gated by access mode', () => {
    const api = repo('src/api/mod.rs');
    const feed = api.slice(api.indexOf('pub async fn get_feed('), api.indexOf('async fn current_public_feed_artifacts('));
    const limit = api.slice(api.indexOf('async fn public_request_limit('), api.indexOf('async fn admit_new_provider_work('));
    const admission = api.slice(api.indexOf('async fn admit_new_provider_work('), api.indexOf('fn is_new_provider_work('));
    const newWork = api.slice(api.indexOf('fn is_new_provider_work('), api.indexOf('async fn provider_access_allowed('));
    expect(feed.length).toBeGreaterThan(500);
    expect(feed).not.toContain('provider_access_allowed');
    expect(repo('src/api/http_catalog.rs')).toContain('.route("/v1/feed", get(super::get_feed))');
    expect(newWork).toMatch(/\*method == axum::http::Method::POST\s*&&/);
    expect(newWork).not.toContain('/v1/feed');
    // Admission moved into one helper shared with A2A; the feed remains outside it.
    expect(limit).toMatch(/if is_new_provider_work\(request\.method\(\), path\)\s*&& let Err\(\(status, body\)\) =\s*admit_new_provider_work\(state\.as_ref\(\), request\.headers\(\), path\)\.await/);
    expect(admission).toMatch(/if !operator && !provider_access_allowed\(state, headers, provider_path\)\.await \{\s*return Err\(\(\s*StatusCode::FORBIDDEN/);
    expect(admission).toContain('provider_storage_admission(state)?;');
    expect(admission).toContain('with_read_conn(crate::provider_policy::require_not_paused)');
    expect(api).toContain('`None` means the normal local/Tor feed may expose every receipt.');
    expect(page).toContain('Signed receipts appear on the provider’s public feed, even for invite-only services');
    // Both the disclosure row and its FAQ answer must include diagnostics and their precise experimental scope.
    expect(page.match(/Receipts generated by this node name the requester’s public key/g)).toHaveLength(2);
    expect(page.match(/but omit successful result bodies/g)).toHaveLength(2);
    expect(page.match(/In the experimental Python runtime, error output from a failing program can become public failure diagnostics and reveal data derived from its input/g)).toHaveLength(2);
    expect(page).not.toContain('but not the result');
    expect(page).not.toContain('not the result.');
  });

  it('keeps A2A new work behind the same admission controls while preserving exact existing recovery', () => {
    const api = repo('src/api/mod.rs');
    const providerRoutes = api.slice(api.indexOf('fn provider_routes('), api.indexOf('fn publication_canary_routes('));
    expect(providerRoutes).toContain('.merge(a2a::routes())');
    const a2a = repo('src/api/a2a.rs');
    const quote = a2a.slice(a2a.indexOf('async fn quote_operation('), a2a.indexOf('async fn submit_operation('));
    const submit = a2a.slice(a2a.indexOf('async fn submit_operation('), a2a.indexOf('async fn task_operation('));
    expect(quote).toContain('admit_new_provider_work(&state, headers, "/v1/provider/quotes").await?;');
    expect(submit).toMatch(/if !permitted\(client, &existing\)\s*\|\| existing\.quote\.hash != request\.quote\.hash\s*\|\| existing\.spec != request\.spec/);
    expect(submit).toMatch(/replay_record\(state\.clone\(\), existing\)\.await\?\s*\} else \{[\s\S]*admit_new_provider_work\(&state, headers, "\/v1\/provider\/deals"\)\.await\?;\s*create_deal_record\(state\.clone\(\), request\)\.await\?\.0/);
  });

  it('says the deal is signed by the requester and the rest of the chain by the provider', () => {
    const kernel = repo('docs/KERNEL.md');
    expect(kernel).toContain('`requester_id`: Froglet application identity of the requester; MUST equal `signer`');
    expect(kernel).toContain('- `descriptor`\n- `offer`\n- `quote`\n- `deal`\n- `invoice_bundle`\n- `receipt`');
    expect(page).toContain("actor: 'Partner', title: 'Agree'");
  });
});

describe('organizations stylesheet', () => {
  it('labels every stacked cell so the table still reads as a table on a phone', () => {
    for (const label of ['Works today', 'Not there yet', 'What you do', 'Done when']) {
      expect(page, label).toContain(`data-label="${label}"`);
    }
    expect(css).toContain('content: attr(data-label)');
  });

  it('gives both tables a caption and column headers for assistive technology', () => {
    expect(page.match(/<caption class="sr-only">/g)).toHaveLength(2);
    expect(page.match(/<th scope="col">/g)?.length).toBeGreaterThanOrEqual(6);
    expect(page.match(/<th scope="row">/g)?.length).toBeGreaterThanOrEqual(2);
  });

  it('prefixes every rule with the story shell so it outranks its element styles', () => {
    const rules = Array.from(cssRules.matchAll(/(^|\})\s*([^{}@]+)\{/g)).map((match) => match[2].trim());
    // Split a selector list on its top-level commas only: `:is(th, td)` is one selector.
    const parts = (list: string) => {
      const out: string[] = [];
      let depth = 0;
      let current = '';
      for (const char of list) {
        if (char === '(') depth += 1;
        if (char === ')') depth -= 1;
        if (char === ',' && depth === 0) { out.push(current.trim()); current = ''; } else current += char;
      }
      return [...out, current.trim()];
    };
    const loose = rules.filter((selector) => parts(selector).some((part) => !/^(\[data-theme='light'\] )?\.story-main /.test(part)));
    expect(loose).toEqual([]);
  });

  it('never hides a table cell with display:none, which would drop it from the stacked layout', () => {
    const rules = Array.from(cssRules.matchAll(/([^{}]*org-matrix[^{}]*)\{([^}]*)\}/g));
    expect(rules.length).toBeGreaterThan(0);
    for (const [, selector, body] of rules) expect(body, selector.trim()).not.toMatch(/display\s*:\s*none/);
  });
});
