import { describe, expect, it, vi, beforeEach, afterEach } from 'vitest';
import { buildSelfHostScript, initSelfHostConfigurator } from '../self-host-configurator';

describe('buildSelfHostScript', () => {
  it('builds a pasteable default script without nested cd commands', () => {
    const script = buildSelfHostScript();

    expect(script).toMatch(/^set -eu$/m);
    expect(script).toContain('froglet-agent-bootstrap.XXXXXX');
    expect(script).toContain('tag=v0.4.6-beta.4');
    expect(script).not.toContain('releases/latest');
    expect(script).toContain('releases/download/$tag/agent-bootstrap.sh');
    expect(script).toContain('/releases/tags/$tag');
    expect(script).toContain('"immutable"');
    expect(script).toContain('"$asset_state" = uploaded');
    expect(script).toContain("--proto-redir '=https'");
    expect(script).toContain('sh "$bootstrap" plan');
    expect(script).toContain('sh "$bootstrap" execute \'<install_approval_hash>\'');
    expect(script).not.toContain('git clone https://github.com/AlitheaBio/froglet.git');
    expect(script.match(/^cd froglet$/gm)).toBeNull();
    expect(script).not.toContain('cd froglet &&');
    expect(script).not.toContain('npm ci --prefix integrations/mcp/froglet');
    expect(script).not.toContain('./scripts/setup-payment.sh');
    expect(script).not.toContain('docker compose up --build -d');
  });

  it('adds the selected agent and payment guidance', () => {
    const script = buildSelfHostScript({
      install: 'linux-arm',
      agent: 'codex',
      payment: 'x402',
    });

    expect(script).toContain(
      'env FROGLET_AGENT_TARGET=codex VERSION="$tag" sh "$bootstrap" plan',
    );
    expect(script).toContain(
      'env FROGLET_AGENT_TARGET=codex VERSION="$tag" sh "$bootstrap" execute \'<install_approval_hash>\'',
    );
    expect(script).not.toMatch(/^FROGLET_AGENT_TARGET=.*curl /m);
    expect(script).toContain('configure x402 with your Base wallet address');
  });

  it('keeps Docker/manual setup on the same no-clone bootstrap', () => {
    const script = buildSelfHostScript({
      install: 'docker',
      agent: 'manual',
      payment: 'stripe-test',
    });

    expect(script).not.toContain('install.sh');
    expect(script).not.toContain('npm ci --prefix integrations/mcp/froglet');
    expect(script).toContain(
      'env FROGLET_AGENT_TARGET=manual FROGLET_BOOTSTRAP_MODE=docker VERSION="$tag" sh "$bootstrap" plan',
    );
    expect(script).not.toMatch(/^FROGLET_(?:AGENT_TARGET|BOOTSTRAP_MODE)=.*curl /m);
    expect(script).toContain('configure Stripe test mode');
  });
});

describe('initSelfHostConfigurator', () => {
  beforeEach(() => {
    vi.useFakeTimers();
    Object.assign(navigator, {
      clipboard: { writeText: vi.fn().mockResolvedValue(undefined) },
    });
  });

  afterEach(() => {
    vi.useRealTimers();
    vi.restoreAllMocks();
    document.body.innerHTML = '';
  });

  it('renders and updates the selected config with aria state', () => {
    document.body.innerHTML = `
      <div id="self-host-card">
        <div class="config-options" data-group="install">
          <button class="config-btn is-active" data-value="linux" aria-pressed="true">Linux</button>
          <button class="config-btn" data-value="docker" aria-pressed="false">Docker</button>
        </div>
        <div class="config-options" data-group="agent">
          <button class="config-btn is-active" data-value="claude-code" aria-pressed="true">Claude Code</button>
          <button class="config-btn" data-value="codex" aria-pressed="false">Codex</button>
        </div>
        <div class="config-options" data-group="payment">
          <button class="config-btn is-active" data-value="none" aria-pressed="true">None</button>
          <button class="config-btn" data-value="stripe-test" aria-pressed="false">Stripe test</button>
        </div>
      </div>
      <pre id="config-output"><code></code></pre>
      <button id="config-copy-btn">Copy</button>
    `;

    initSelfHostConfigurator();

    const dockerButton = document.querySelector<HTMLButtonElement>('[data-value="docker"]');
    const linuxButton = document.querySelector<HTMLButtonElement>('[data-value="linux"]');
    dockerButton?.click();

    expect(dockerButton?.getAttribute('aria-pressed')).toBe('true');
    expect(linuxButton?.getAttribute('aria-pressed')).toBe('false');
    expect(document.querySelector('#config-output code')?.textContent).not.toContain('install.sh');
  });

  it('copies the rendered script', async () => {
    document.body.innerHTML = `
      <div id="self-host-card"></div>
      <pre id="config-output"><code>hello</code></pre>
      <button id="config-copy-btn">Copy</button>
    `;

    initSelfHostConfigurator();
    document.querySelector<HTMLButtonElement>('#config-copy-btn')?.click();
    await vi.advanceTimersByTimeAsync(0);

    expect(navigator.clipboard.writeText).toHaveBeenCalledWith(buildSelfHostScript());
    expect(document.querySelector('#config-copy-btn')?.textContent).toBe('Copied');
  });

  it('updates the output script dynamically when typing in the credentials field', () => {
    document.body.innerHTML = `
      <div id="self-host-card">
        <div class="config-options" data-group="install">
          <button class="config-btn is-active" data-value="linux" aria-pressed="true">Linux</button>
        </div>
        <div class="config-options" data-group="agent">
          <button class="config-btn is-active" data-value="claude-code" aria-pressed="true">Claude Code</button>
        </div>
        <div class="config-options" data-group="payment">
          <button class="config-btn" data-value="none" aria-pressed="false">None</button>
          <button class="config-btn is-active" data-value="stripe-test" aria-pressed="true">Stripe test</button>
        </div>
        <div id="config-payment-input-container" style="display: none;">
          <span id="config-payment-input-label">Credential</span>
          <input id="config-payment-input" type="text" />
        </div>
      </div>
      <pre id="config-output"><code></code></pre>
    `;

    initSelfHostConfigurator();

    const stripeButton = document.querySelector<HTMLButtonElement>('[data-value="stripe-test"]');
    stripeButton?.click();

    const inputContainer = document.querySelector<HTMLElement>('#config-payment-input-container');
    const inputField = document.querySelector<HTMLInputElement>('#config-payment-input');
    const codeBlock = document.querySelector<HTMLElement>('#config-output code');

    expect(inputContainer?.style.display).toBe('block');
    expect(inputField?.placeholder).toBe('sk_test_...');

    if (inputField) {
      inputField.value = 'sk_test_testkey123'; // gitleaks:allow
      inputField.dispatchEvent(new Event('input'));
    }

    expect(codeBlock?.textContent).toContain('export FROGLET_STRIPE_SECRET_KEY=sk_test_testkey123'); // gitleaks:allow
  });
});
