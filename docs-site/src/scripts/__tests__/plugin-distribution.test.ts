import { describe, expect, it } from 'vitest';
import { readFileSync } from 'node:fs';
import { dirname, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const here = dirname(fileURLToPath(import.meta.url));
const repoRoot = resolve(here, '../../../..');

function readRepoFile(path: string): string {
  return readFileSync(resolve(repoRoot, path), 'utf8');
}

function readJson(path: string): any {
  return JSON.parse(readRepoFile(path));
}

describe('plugin and registry distribution metadata', () => {
  it('keeps MCP Registry metadata aligned with npm package metadata', () => {
    const pkg = readJson('package.json');
    const server = readJson('server.json');

    expect(pkg.name).toBe('froglet-mcp');
    expect(pkg.mcpName).toBe('io.github.armanas/froglet');
    expect(pkg.license).toBe('Apache-2.0');
    expect(pkg.files).toContain('server.json');
    expect(server.name).toBe(pkg.mcpName);
    expect(server.version).toBe(pkg.version);
    expect(server.packages[0].identifier).toBe(pkg.name);
    expect(server.packages[0].version).toBe(pkg.version);
    expect(server.packages[0].transport.type).toBe('stdio');
  });

  it('packages Codex and Claude plugins around the same local MCP command', () => {
    const codex = readJson('plugins/froglet/.codex-plugin/plugin.json');
    const claude = readJson('plugins/froglet/.claude-plugin/plugin.json');
    const mcp = readJson('plugins/froglet/.mcp.json');
    const cursor = readJson('.cursor/mcp.json');
    const command = readRepoFile('plugins/froglet/commands/froglet.md');

    expect(codex.name).toBe('froglet');
    expect(claude.name).toBe('froglet');
    expect(codex.version).toBe(claude.version);
    expect(codex.license).toBe('Apache-2.0');
    expect(claude.license).toBe('Apache-2.0');
    expect(codex.mcpServers).toBe('./.mcp.json');
    expect(claude.mcpServers).toBe('./.mcp.json');
    expect(mcp.mcpServers.froglet.command).toBe('npx');
    expect(mcp.mcpServers.froglet.args).toEqual(['-y', 'froglet-mcp']);
    expect(mcp.mcpServers.froglet.env.FROGLET_PROFILE).toBe('local');
    expect(mcp.mcpServers.froglet.env.FROGLET_PROVIDER_URL).toBe('http://127.0.0.1:8080');
    expect(mcp.mcpServers.froglet.env.FROGLET_RUNTIME_URL).toBe('http://127.0.0.1:8081');
    expect(mcp.mcpServers.froglet.env.FROGLET_PROVIDER_AUTH_TOKEN_PATH).toContain('froglet-control.token');
    expect(mcp.mcpServers.froglet.env.FROGLET_RUNTIME_AUTH_TOKEN_PATH).toContain('auth.token');
    expect(cursor.mcpServers.froglet.command).toBe('npx');
    expect(cursor.mcpServers.froglet.args).toEqual(['-y', 'froglet-mcp']);
    expect(cursor.mcpServers.froglet.env.FROGLET_PROFILE).toBe('local');
    expect(command).toContain('allowed-tools: mcp__plugin_froglet_froglet__froglet');
    expect(command).toContain('{"action":"status"}');
    expect(command).toContain('plan_install');
    expect(command).toContain('plan_use_case');
  });

  it('publishes repo-local marketplace entries for Codex and Claude Code', () => {
    const codexMarketplace = readJson('.agents/plugins/marketplace.json');
    const claudeMarketplace = readJson('.claude-plugin/marketplace.json');
    const claudePlugin = readJson('plugins/froglet/.claude-plugin/plugin.json');

    expect(codexMarketplace.name).toBe('froglet');
    expect(codexMarketplace.plugins[0].source.path).toBe('./plugins/froglet');
    expect(codexMarketplace.plugins[0].policy.installation).toBe('AVAILABLE');
    expect(codexMarketplace.plugins[0].policy.authentication).toBe('ON_INSTALL');
    expect(claudeMarketplace.name).toBe('froglet');
    expect(claudeMarketplace.plugins[0].source).toBe('./plugins/froglet');
    expect(claudeMarketplace.metadata.version).toBe(claudePlugin.version);
    expect(claudeMarketplace.plugins[0].version).toBe(claudePlugin.version);
  });

  it('documents distribution order and local/actionable boundaries on website and repo docs', () => {
    for (const path of [
      'docs/PLUGIN_DISTRIBUTION.md',
      'docs-site/src/content/docs/learn/plugin-distribution.mdx',
    ]) {
      const text = readRepoFile(path);
      expect(text).toContain('Official MCP Registry');
      expect(text).toContain('Codex plugin');
      expect(text).toContain('Claude Code');
      expect(text).toContain('OpenClaw');
      expect(text).toContain('NemoClaw');
      expect(text).toContain('Third-party MCP directories');
      expect(text).toContain('no-install evaluation: the public beta in a browser');
      expect(text).not.toContain('public free `demo.*` services');
      expect(text).toContain('installed MCP/plugins');
      expect(text).toContain('local/self-hosted provider/runtime actions');
      expect(text).toContain('plan_use_case');
      expect(text).toContain('mcp-publisher publish');
      expect(text).toContain('Do not use `curl https://froglet.dev/learn/plugin-distribution/` as a proof');
      expect(text).toContain('The npm MCP package version and the host-plugin wrapper version');
      expect(text).toContain('MCP_DIRECTORY_SUBMISSIONS.md');
      expect(text).toContain('Cursor');
      expect(text).toContain('NemoClaw');
      expect(text).toContain('node -e');
      expect(text).toContain('claude plugin marketplace add AlitheaBio/froglet --sparse .claude-plugin plugins');
      expect(text).not.toContain('apps.froglet.dev');
      expect(text).not.toContain('chatgpt-app');
    }
  });
});
