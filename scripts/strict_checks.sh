#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$repo_root"

qualification_node="${FROGLET_QUALIFICATION_NODE_BIN:-}"
if [[ -n "$qualification_node" && ( ! -f "$qualification_node" || ! -x "$qualification_node" ) ]]; then
  echo "[strict] qualification node must be an executable file" >&2
  exit 1
fi

# A release software gate must run its required toolchain cells. The standalone
# developer matrix retains its optional-tool behavior for partial environments.
if [[ "${FROGLET_REQUIRE_SOFTWARE_CHECKS:-0}" == "1" ]]; then
  missing_tool=0
  for required_tool in cargo rustc rustup node npm python3 openssl; do
    if ! command -v "$required_tool" >/dev/null 2>&1; then
      echo "[strict] required release tool is missing: $required_tool" >&2
      missing_tool=1
    fi
  done
  if [[ "$missing_tool" != "0" ]]; then
    exit 1
  fi
  if ! cargo clippy --version >/dev/null 2>&1; then
    echo "[strict] release checks require cargo-clippy" >&2
    exit 1
  fi
  if ! rustup target list --installed 2>/dev/null | grep -qx wasm32-unknown-unknown; then
    echo "[strict] release checks require target wasm32-unknown-unknown" >&2
    exit 1
  fi
  required_node_major="$(node -p 'process.versions.node.split(".")[0]')" || exit 1
  if [[ ! "$required_node_major" =~ ^[0-9]+$ ]] || [[ "$required_node_major" -lt 18 ]]; then
    echo "[strict] release checks require Node.js 18 or newer" >&2
    exit 1
  fi
fi

strict_rustflags="${RUSTFLAGS:-}"
case " ${strict_rustflags} " in
  *" -D warnings "*) ;;
  *) strict_rustflags="${strict_rustflags:+${strict_rustflags} }-D warnings" ;;
esac

ensure_mcp_dependencies() {
  local package_dir="integrations/mcp/froglet"
  local marker="${package_dir}/node_modules/@modelcontextprotocol/sdk/package.json"

  if [[ -f "$marker" ]]; then
    return
  fi

  if ! command -v npm >/dev/null 2>&1; then
    echo "[strict] MCP checks require npm to install dependencies" >&2
    exit 1
  fi

  echo "[strict] installing MCP server dependencies"
  npm ci --prefix "$package_dir"
}

echo "[strict] cargo fmt --check"
cargo fmt --all --check

echo "[strict] cargo test with compiler warnings denied"
CARGO_INCREMENTAL=0 RUSTFLAGS="$strict_rustflags" cargo test --locked --workspace --all-targets

# The offline verifier must keep building without default features, and for
# wasm32, or the browser/npm distribution silently rots. The same goes for the
# browser signing build the playground uses.
echo "[strict] froglet-protocol builds dependency-light and for wasm32"
CARGO_INCREMENTAL=0 cargo check -p froglet-protocol --no-default-features
if rustup target list --installed 2>/dev/null | grep -q wasm32-unknown-unknown; then
  CARGO_INCREMENTAL=0 cargo check -p froglet-protocol --no-default-features \
    --target wasm32-unknown-unknown
  CARGO_INCREMENTAL=0 cargo check -p froglet-wasm --target wasm32-unknown-unknown
else
  echo "[strict] skipping wasm32 check: target wasm32-unknown-unknown is not installed"
fi

# conformance/kernel_v1.json is frozen: "signed froglet/v1 artifacts verify
# forever" is only true while these bytes never change.
echo "[strict] frozen kernel conformance vectors are unmodified"
if git rev-parse --git-dir >/dev/null 2>&1; then
  git diff --exit-code -- conformance/kernel_v1.json
  git diff --cached --exit-code -- conformance/kernel_v1.json
else
  echo "[strict] skipping frozen-fixture check: not a git checkout"
fi

if cargo clippy --version >/dev/null 2>&1; then
  echo "[strict] cargo clippy -D warnings"
  cargo clippy --locked --workspace --all-targets -- -D warnings
else
  echo "[strict] skipping clippy: cargo-clippy is not installed"
fi

echo "[strict] installer and release helper shell syntax"
sh -n scripts/install.sh
bash -n scripts/gitleaks_gate.sh
bash -n scripts/agent-bootstrap.sh
bash -n scripts/setup-agent.sh
bash -n scripts/setup-payment.sh
bash -n scripts/fresh_host_quickstart_smoke.sh
bash -n scripts/fly_provider_smoke.sh
sh -n docs-site/public/agent
bash -n scripts/deploy_gcp_single_vm.sh
bash -n scripts/gpu_smoke.sh
bash -n scripts/package_release_assets.sh
bash -n scripts/verify_release_assets.sh
bash -n scripts/smoke_install_from_assets.sh
bash -n scripts/release_gate.sh

if [[ "${FROGLET_SKIP_GITLEAKS:-0}" != "1" ]]; then
  echo "[strict] gitleaks publication gate"
  ./scripts/gitleaks_gate.sh
fi

if command -v node >/dev/null 2>&1; then
  node_major=$(node -e 'process.stdout.write(String(process.versions.node.split(".")[0]))')
  if [ "$node_major" -ge 18 ] 2>/dev/null; then
    ensure_mcp_dependencies

    echo "[strict] OpenClaw plugin checks"
    node --check integrations/openclaw/froglet/index.js
    node --check integrations/openclaw/froglet/scripts/doctor.mjs
    node --test integrations/openclaw/froglet/test/plugin.test.js \
      integrations/openclaw/froglet/test/*.test.mjs
    python3 -W error integrations/openclaw/froglet/test/openshell-qualification-backend.test.py -v

    echo "[strict] MCP server checks"
    node --check integrations/mcp/froglet/server.js
    node --test integrations/mcp/froglet/test/*.test.mjs

    echo "[strict] shared froglet-lib checks"
    node --check integrations/shared/froglet-lib/froglet-client.js
    node --check integrations/shared/froglet-lib/url-safety.js
    node --test integrations/shared/froglet-lib/test/*.test.mjs

    if [[ "${FROGLET_RUN_COMPOSE_SMOKE:-0}" == "1" ]]; then
      if ! command -v docker >/dev/null 2>&1; then
        echo "[strict] compose smoke requested but docker is not installed" >&2
        exit 1
      fi

      echo "[strict] compose-backed bot-surface smoke"
      docker compose -f compose.yaml -f compose.ci.yaml down --remove-orphans
      docker compose -f compose.yaml -f compose.ci.yaml up --build -d --wait
      trap 'docker compose -f compose.yaml -f compose.ci.yaml down --remove-orphans' EXIT

      node integrations/openclaw/froglet/test/compose-smoke.mjs
      node integrations/mcp/froglet/test/compose-smoke.mjs
    fi
  else
    echo "[strict] skipping Node integration checks: node $node_major < 18"
  fi
else
  echo "[strict] skipping Node integration checks: node is not installed"
fi

if rustup target list --installed 2>/dev/null | grep -q wasm32-unknown-unknown; then
  echo "[strict] ontology checker and independent oracle unit checks"
  CARGO_INCREMENTAL=0 RUSTFLAGS="$strict_rustflags" \
    cargo test --locked --manifest-path examples/wasm-services/Cargo.toml -p ontology-check
  python3 -W error -m unittest examples.test_a2a_compute_demo examples.test_immunopeptidomics_demo examples.test_research_profile examples.test_go_subset_catalog examples.test_qualify_ontology_capacity examples.test_generated_program_replay -v

  if [[ -z "$qualification_node" && "${FROGLET_REQUIRE_SOFTWARE_CHECKS:-0}" == 1 ]]; then
    # Standalone release CI has no package gate to supply its candidate.
    # Build its native optimized executable explicitly instead of qualifying
    # a debug executable against the same signed execution deadline.
    compiler_details="$(rustc -vV)"
    native_target=""
    while IFS= read -r line; do
      [[ "$line" != host:\ * ]] || native_target="${line#host: }"
    done <<<"$compiler_details"
    if [[ ! "$native_target" =~ ^[A-Za-z0-9_-]+$ ]]; then
      echo "[strict] could not resolve native Rust compiler target" >&2
      exit 1
    fi
    echo "[strict] build current native optimized qualification node ($native_target)"
    CARGO_INCREMENTAL=0 RUSTFLAGS="$strict_rustflags" \
      cargo build --locked --release -p froglet --bin froglet-node \
        --manifest-path "$repo_root/Cargo.toml" --target "$native_target" \
        --target-dir "$repo_root/target"
    qualification_node="$repo_root/target/$native_target/release/froglet-node"
    [[ -f "$qualification_node" && -x "$qualification_node" ]] || {
      echo "[strict] native qualification build did not produce its executable" >&2
      exit 1
    }
  fi
  if [[ -n "$qualification_node" ]]; then
    echo "[strict] build locked Wasm samples for explicit qualification node: $qualification_node"
    CARGO_INCREMENTAL=0 RUSTFLAGS="$strict_rustflags" \
      cargo build --locked --release --target wasm32-unknown-unknown \
        --manifest-path "$repo_root/examples/wasm-services/Cargo.toml" \
        --target-dir "$repo_root/examples/wasm-services/target"
    wasm_output="$repo_root/examples/wasm-services/target/wasm32-unknown-unknown/release"
    [[ -f "$wasm_output/adder.wasm" && -f "$wasm_output/ontology_check.wasm" ]] || {
      echo "[strict] locked Wasm sample build did not produce required modules" >&2
      exit 1
    }
  fi
  run_local_demo() {
    local example="$1"
    local module="$2"
    shift 2
    if [[ -n "$qualification_node" ]]; then
      CARGO_INCREMENTAL=0 RUSTFLAGS="$strict_rustflags" \
        python3 -W error "$example" --binary "$qualification_node" --module "$module" "$@"
    else
      CARGO_INCREMENTAL=0 RUSTFLAGS="$strict_rustflags" \
        python3 -W error "$example" "$@"
    fi
  }
  echo "[strict] local native MCP and A2A operator demo"
  run_local_demo examples/a2a_compute_demo.py "${wasm_output:-}/adder.wasm"
  echo "[strict] selected terminology retrieval and disposable Wasm mapping check"
  run_local_demo examples/a2a_compute_demo.py "${wasm_output:-}/ontology_check.wasm" --scenario ontology --no-build
  if command -v openssl >/dev/null 2>&1; then
    echo "[strict] reviewed invitation and scoped A2A setup over local TLS"
    run_local_demo examples/a2a_counterparty_demo.py "${wasm_output:-}/ontology_check.wasm" --no-build
  else
    echo "[strict] skipping local TLS invitation/A2A qualification: OpenSSL is not installed"
  fi
else
  echo "[strict] skipping local MCP/A2A demo: target wasm32-unknown-unknown is not installed"
fi

echo "[strict] core python-backed runtime tests with warnings as errors"
python3 -W error -m unittest \
  python.tests.test_protocol \
  python.tests.test_runtime \
  python.tests.test_jobs \
  python.tests.test_payments \
  python.tests.test_sandbox \
  python.tests.test_acceptance \
  python.tests.test_pentest \
  python.tests.test_security \
  python.tests.test_privacy \
  python.tests.test_hardening \
  python.tests.test_install_script \
  python.tests.test_gitleaks_gate \
  python.tests.test_release_gate \
  python.tests.test_release_metadata \
  python.tests.test_public_demo_proxy \
  python.tests.test_fly_provider_smoke \
  python.tests.test_setup_scripts \
  python.tests.test_package_demo_bundle \
  python.tests.test_conformance_vectors -v

echo "[strict] standalone python verifier package"
(
  cd python/froglet-verify
  python3 -W error -m unittest discover -s tests -v
)
echo "[strict] python verifier reproduces every conformance fixture"
PYTHONPATH="python/froglet-verify" python3 -m froglet_verify.conformance conformance/

if [[ "${FROGLET_RUN_TOR_INTEGRATION:-0}" == "1" ]]; then
  echo "[strict] tor integration"
  python3 -W error -m unittest -v python.tests.test_tor_integration
fi

if [[ "${FROGLET_RUN_LINUX_SANDBOX_TESTS:-0}" == "1" ]]; then
  # Require real Landlock ABI v3+ and seccomp enforcement. These run without
  # privilege escalation under NO_NEW_PRIVS; an unsupported runner must fail
  # qualification rather than silently skipping the sandbox checks.
  echo "[strict] linux sandbox tests (landlock + seccomp)"
  CARGO_INCREMENTAL=0 RUSTFLAGS="$strict_rustflags" \
    cargo test --locked --workspace --all-targets -- --ignored \
      python_sandbox::tests:: \
      service_addressed_python_execution_runs_from_redacted_service_record
fi

if [[ "${FROGLET_RUN_LND_REGTEST:-0}" == "1" ]]; then
  echo "[strict] lnd regtest integration"
  python3 -W error -m unittest -v python.tests.test_lnd_regtest
fi
