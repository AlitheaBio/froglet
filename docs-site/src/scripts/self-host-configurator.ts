export type InstallTarget = 'linux' | 'linux-arm' | 'macos' | 'docker';
export type AgentTarget = 'claude-code' | 'codex' | 'manual';
export type PaymentRail = 'none' | 'lightning-mock' | 'lightning-lnd-rest' | 'stripe-test' | 'stripe-live' | 'x402';

export interface SelfHostConfig {
  install: InstallTarget;
  agent: AgentTarget;
  payment: PaymentRail;
}

export const DEFAULT_SELF_HOST_CONFIG: SelfHostConfig = {
  install: 'linux',
  agent: 'claude-code',
  payment: 'none',
};

const PAYMENT_NOTES: Record<PaymentRail, string> = {
  none: '',
  'lightning-mock': '# configure Lightning mock payment setup locally\nexport FROGLET_PAYMENT_BACKEND=lightning\nexport FROGLET_LIGHTNING_MODE=mock',
  'lightning-lnd-rest': '# After MCP status passes, configure LND REST with your LND URL, macaroon, and TLS cert.',
  'stripe-test': '# configure Stripe test mode with sk_test_... and webhook proof.\nexport FROGLET_PAYMENT_BACKEND=stripe\nexport FROGLET_STRIPE_SECRET_KEY=sk_test_...',
  'stripe-live': '# After MCP status passes, configure Stripe live mode only after a fresh live-payment approval.',
  x402: '# configure x402 with your Base wallet address and facilitator.\nexport FROGLET_PAYMENT_BACKEND=x402\nexport FROGLET_X402_WALLET_ADDRESS=0x...',
};

function getPaymentNote(payment: PaymentRail, cred?: string): string {
  if (payment === 'stripe-test' && cred) {
    return `# configure Stripe test mode with sk_test_... and webhook proof.\nexport FROGLET_PAYMENT_BACKEND=stripe\nexport FROGLET_STRIPE_SECRET_KEY=${cred}`;
  }
  if (payment === 'x402' && cred) {
    return `# configure x402 with your Base wallet address and facilitator.\nexport FROGLET_PAYMENT_BACKEND=x402\nexport FROGLET_X402_WALLET_ADDRESS=${cred}`;
  }
  return PAYMENT_NOTES[payment] || '';
}

export function buildSelfHostScript(config: SelfHostConfig = DEFAULT_SELF_HOST_CONFIG, credential?: string): string {
  const lines: string[] = [];
  const env: string[] = [];

  const note = getPaymentNote(config.payment, credential);
  if (note) {
    lines.push(note);
  }

  if (config.agent !== 'claude-code') {
    env.push(`FROGLET_AGENT_TARGET=${config.agent}`);
  }
  if (config.install === 'docker') {
    env.push('FROGLET_BOOTSTRAP_MODE=docker');
  }

  const bootstrapEnv = `env ${env.length > 0 ? `${env.join(' ')} ` : ''}VERSION="$tag" `;
  lines.push('set -eu');
  lines.push('repo=AlitheaBio/froglet');
  lines.push('metadata="$(mktemp "${TMPDIR:-/tmp}/froglet-release.XXXXXX")"');
  lines.push('bootstrap="$(mktemp "${TMPDIR:-/tmp}/froglet-agent-bootstrap.XXXXXX")"');
  lines.push('tag=v0.4.6-beta.4');
  lines.push("printf '%s' \"$tag\" | grep -Eq '^v[0-9A-Za-z][0-9A-Za-z.+-]*$'");
  lines.push("curl -fsSL --proto '=https' --proto-redir '=https' --tlsv1.2 -H 'Accept: application/vnd.github+json' -H 'X-GitHub-Api-Version: 2026-03-10' \"https://api.github.com/repos/$repo/releases/tags/$tag\" -o \"$metadata\"");
  lines.push('[ "$(sed -n \'s/^  "immutable": \\([a-z]*\\),*$/\\1/p\' "$metadata")" = true ]');
  lines.push('[ "$(sed -n \'s/^  "tag_name": "\\([^\"]*\\)",*$/\\1/p\' "$metadata")" = "$tag" ]');
  lines.push("asset_record=\"$(awk '/^    [{]/ { in_asset=1; name=digest=state=\"\"; next } in_asset && /^      \"name\":/ { v=$0; sub(/^      \"name\": \"/,\"\",v); sub(/\",*$/,\"\",v); name=v } in_asset && /^      \"digest\":/ { v=$0; sub(/^      \"digest\": \"/,\"\",v); sub(/\",*$/,\"\",v); digest=v } in_asset && /^      \"state\":/ { v=$0; sub(/^      \"state\": \"/,\"\",v); sub(/\",*$/,\"\",v); state=v } in_asset && /^    [}],*$/ { if (name==\"agent-bootstrap.sh\") print digest \"|\" state; in_asset=0 }' \"$metadata\")\"");
  lines.push('[ "$(printf \'%s\\n\' "$asset_record" | sed \'/^$/d\' | wc -l | tr -d \' \')" = 1 ]');
  lines.push('bootstrap_digest="${asset_record%%|*}"; asset_state="${asset_record#*|}"');
  lines.push('[ "$asset_state" = uploaded ]');
  lines.push("printf '%s' \"$bootstrap_digest\" | grep -Eq '^sha256:[0-9a-f]{64}$'");
  lines.push('bootstrap_digest="${bootstrap_digest#sha256:}"');
  lines.push('curl -fsSL --proto \'=https\' --proto-redir \'=https\' --tlsv1.2 "https://github.com/$repo/releases/download/$tag/agent-bootstrap.sh" -o "$bootstrap"');
  lines.push('if command -v sha256sum >/dev/null 2>&1; then actual="$(sha256sum "$bootstrap" | awk \'{print $1}\')"; elif command -v shasum >/dev/null 2>&1; then actual="$(shasum -a 256 "$bootstrap" | awk \'{print $1}\')"; else actual="$(openssl dgst -sha256 "$bootstrap" | sed \'s/^.*= //\')"; fi');
  lines.push('[ "$actual" = "$bootstrap_digest" ]');
  lines.push('chmod 0700 "$bootstrap"');
  lines.push(`${bootstrapEnv}sh "$bootstrap" plan`);
  lines.push('# Review the complete plan, then replace the placeholder with its exact approved hash.');
  lines.push(`${bootstrapEnv}sh "$bootstrap" execute '<install_approval_hash>'`);
  lines.push('rm -f "$bootstrap" "$metadata"');

  return lines.join('\n');
}

function setGroupValue(group: HTMLElement, value: string): void {
  group.querySelectorAll<HTMLButtonElement>('.config-btn').forEach((button) => {
    const isActive = button.dataset.value === value;
    button.classList.toggle('is-active', isActive);
    button.setAttribute('aria-pressed', String(isActive));
  });
}

export function initSelfHostConfigurator(root: Document | HTMLElement = document): void {
  const card = root.querySelector<HTMLElement>('#self-host-card');
  const output = root.querySelector<HTMLElement>('#config-output');
  const copyButton = root.querySelector<HTMLButtonElement>('#config-copy-btn');
  const inputContainer = root.querySelector<HTMLElement>('#config-payment-input-container');
  const inputLabel = root.querySelector<HTMLElement>('#config-payment-input-label');
  const inputField = root.querySelector<HTMLInputElement>('#config-payment-input');

  if (!card || !output) return;

  const state: SelfHostConfig = { ...DEFAULT_SELF_HOST_CONFIG };

  function updateInputVisibility(): void {
    if (!inputContainer || !inputLabel || !inputField) return;

    if (state.payment === 'stripe-test') {
      inputContainer.style.display = 'block';
      inputLabel.textContent = 'Stripe Secret Key';
      inputField.placeholder = 'sk_test_...';
    } else if (state.payment === 'x402') {
      inputContainer.style.display = 'block';
      inputLabel.textContent = 'Base Wallet Address';
      inputField.placeholder = '0x...';
    } else {
      inputContainer.style.display = 'none';
      inputField.value = '';
    }
  }

  function render(): void {
    const code = output?.querySelector('code');
    if (code) {
      const cred = inputField?.value?.trim() || '';
      code.textContent = buildSelfHostScript(state, cred);
    }
  }

  card.querySelectorAll<HTMLElement>('.config-options').forEach((group) => {
    const groupName = group.dataset.group as keyof SelfHostConfig | undefined;
    if (!groupName) return;

    setGroupValue(group, state[groupName]);

    group.querySelectorAll<HTMLButtonElement>('.config-btn').forEach((button) => {
      button.addEventListener('click', (event) => {
        const nextValue = button.dataset.value;
        if (!nextValue) return;

        event.preventDefault();
        button.blur();
        state[groupName] = nextValue as SelfHostConfig[typeof groupName];
        setGroupValue(group, nextValue);
        updateInputVisibility();
        render();
      });
    });
  });

  inputField?.addEventListener('input', () => {
    render();
  });

  copyButton?.addEventListener('click', async () => {
    const code = output.querySelector('code');
    const originalLabel = copyButton.textContent || 'Copy';

    try {
      await navigator.clipboard.writeText(code?.textContent || '');
      copyButton.textContent = 'Copied';
    } catch {
      copyButton.textContent = 'Failed';
    }

    setTimeout(() => {
      copyButton.textContent = originalLabel;
    }, 1500);
  });

  render();
}
