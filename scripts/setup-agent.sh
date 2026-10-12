#!/usr/bin/env bash
set -euo pipefail

# Detect whether we're running from a cloned repo (node-based MCP path
# available) or piped via `curl | sh -s -- --target ...` with only the
# froglet-node binary installed. In repo mode we keep the existing
# node-based MCP invocation; in binary-only mode we emit a Docker-based
# config that pulls ghcr.io/armanas/froglet-mcp.
repo_root=""
candidate_root=""
if [[ -n "${BASH_SOURCE[0]:-}" && -f "${BASH_SOURCE[0]}" ]]; then
  if candidate_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." 2>/dev/null && pwd)"; then
    if [[ -f "$candidate_root/integrations/mcp/froglet/server.js" ]]; then
      repo_root="$candidate_root"
    fi
  fi
fi
# Fallback: check the current working directory (useful if the script was
# downloaded into a repo root manually).
if [[ -z "$repo_root" && -f "$PWD/integrations/mcp/froglet/server.js" ]]; then
  repo_root="$PWD"
fi

mcp_image="${FROGLET_MCP_IMAGE:-}"
mcp_docker_network="${FROGLET_MCP_DOCKER_NETWORK:-}"
mcp_mode="${FROGLET_MCP_MODE:-auto}"
node_bin="${FROGLET_NODE_BIN:-}"

provider_url="${FROGLET_PROVIDER_URL:-http://127.0.0.1:8080}"
runtime_url="${FROGLET_RUNTIME_URL:-http://127.0.0.1:8081}"
# Token paths default to the repo-relative `data/runtime/` tree when run
# from inside a cloned repo. In binary-only mode we fall back to
# `~/.froglet/runtime/` since the repo path doesn't exist.
default_token_dir="${repo_root:+$repo_root/data/runtime}"
default_token_dir="${default_token_dir:-$HOME/.froglet/data/runtime}"
provider_token_path="${FROGLET_PROVIDER_AUTH_TOKEN_PATH:-$default_token_dir/froglet-control.token}"
runtime_token_path="${FROGLET_RUNTIME_AUTH_TOKEN_PATH:-$default_token_dir/auth.token}"
request_timeout_ms="${FROGLET_REQUEST_TIMEOUT_MS:-10000}"
default_search_limit="${FROGLET_DEFAULT_SEARCH_LIMIT:-10}"
max_search_limit="${FROGLET_MAX_SEARCH_LIMIT:-50}"
docker_token_mount="/froglet-tokens"
target=""
out_path=""

usage() {
  cat <<'EOF'
Usage:
  scripts/setup-agent.sh --target claude-code|codex|openclaw [--out PATH]

Generates an agent config file that points at the local Froglet provider and
runtime by default.
EOF
}

fail() {
  printf 'error: %s\n' "$*" >&2
  exit 1
}

need_cmd() {
  command -v "$1" >/dev/null 2>&1 || fail "missing required command: $1"
}

quoted_string() {
  local value="$1"
  local newline=$'\n'
  case "$value" in
    *"$newline"*) fail "control characters are not allowed in generated config values" ;;
  esac
  if LC_ALL=C printf '%s' "$value" | grep '[[:cntrl:]]' >/dev/null 2>&1; then
    fail "control characters are not allowed in generated config values"
  fi
  value="${value//\\/\\\\}"
  value="${value//\"/\\\"}"
  printf '"%s"' "$value"
}

json_string() {
  quoted_string "$1"
}

toml_string() {
  quoted_string "$1"
}

require_uint() {
  local name="$1"
  local value="$2"
  [[ "$value" =~ ^[0-9]+$ ]] || fail "$name must be an unsigned integer"
}

require_immutable_image() {
  local name="$1"
  local value="$2"
  [[ "$value" =~ ^[A-Za-z0-9][A-Za-z0-9._:/-]*@sha256:[0-9a-f]{64}$ ]] || \
    fail "$name must be an immutable OCI sha256 digest reference"
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --target)
      [[ $# -ge 2 ]] || fail "--target requires a value"
      target="$2"
      shift 2
      ;;
    --out)
      [[ $# -ge 2 ]] || fail "--out requires a value"
      out_path="$2"
      shift 2
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      fail "unknown argument: $1"
      ;;
  esac
done

[[ -n "$target" ]] || fail "--target is required"
require_uint FROGLET_REQUEST_TIMEOUT_MS "$request_timeout_ms"
require_uint FROGLET_DEFAULT_SEARCH_LIMIT "$default_search_limit"
require_uint FROGLET_MAX_SEARCH_LIMIT "$max_search_limit"

case "$mcp_mode" in
  auto)
    if [[ -n "$node_bin" && -x "$node_bin" ]]; then
      mcp_mode="native"
    elif [[ -n "$repo_root" ]]; then
      mcp_mode="repo"
    else
      mcp_mode="docker"
    fi
    ;;
  native|repo|docker) ;;
  *) fail "FROGLET_MCP_MODE must be auto, native, repo, or docker" ;;
esac

if [[ "$mcp_mode" == "native" ]]; then
  if [[ -z "$node_bin" ]]; then
    node_bin="$(command -v froglet-node 2>/dev/null || true)"
  fi
  [[ -n "$node_bin" && -x "$node_bin" ]] || \
    fail "native MCP mode requires an executable FROGLET_NODE_BIN"
elif [[ "$mcp_mode" == "repo" ]]; then
  [[ -n "$repo_root" ]] || fail "repo MCP mode requires a Froglet checkout"
elif [[ "$target" =~ ^(claude-code|codex)$ ]]; then
  [[ -n "$mcp_image" ]] || \
    fail "Docker MCP mode requires FROGLET_MCP_IMAGE from a verified release manifest"
  require_immutable_image FROGLET_MCP_IMAGE "$mcp_image"
fi

# Output path defaults to the current working directory when not in a repo.
# Repo mode keeps the original repo-root anchoring so existing contributors
# don't see behavior change.
default_out_dir="${repo_root:-$PWD}"

server_path=""
openclaw_plugin_path=""
if [[ -n "$repo_root" ]]; then
  server_path="$repo_root/integrations/mcp/froglet/server.js"
  openclaw_plugin_path="$repo_root/integrations/openclaw/froglet"
fi

provider_token_dir="$(dirname "$provider_token_path")"
runtime_token_dir="$(dirname "$runtime_token_path")"
provider_token_file="$(basename "$provider_token_path")"
runtime_token_file="$(basename "$runtime_token_path")"
docker_provider_token_path="$docker_token_mount/$provider_token_file"
docker_runtime_token_path="$docker_token_mount/$runtime_token_file"
if [[ -z "$repo_root" && "$provider_token_dir" != "$runtime_token_dir" ]]; then
  fail "Docker MCP mode requires provider/runtime token files in the same directory"
fi

server_path_json="$(json_string "$server_path")"
server_path_toml="$(toml_string "$server_path")"
openclaw_plugin_path_json="$(json_string "$openclaw_plugin_path")"
provider_url_json="$(json_string "$provider_url")"
provider_url_toml="$(toml_string "$provider_url")"
runtime_url_json="$(json_string "$runtime_url")"
runtime_url_toml="$(toml_string "$runtime_url")"
provider_token_path_json="$(json_string "$provider_token_path")"
provider_token_path_toml="$(toml_string "$provider_token_path")"
runtime_token_path_json="$(json_string "$runtime_token_path")"
runtime_token_path_toml="$(toml_string "$runtime_token_path")"
request_timeout_ms_json="$(json_string "$request_timeout_ms")"
default_search_limit_json="$(json_string "$default_search_limit")"
max_search_limit_json="$(json_string "$max_search_limit")"
provider_token_dir_json="$(json_string "$provider_token_dir:$docker_token_mount:ro")"
provider_token_dir_toml="$(toml_string "$provider_token_dir:$docker_token_mount:ro")"
docker_provider_token_path_json="$(json_string "$docker_provider_token_path")"
docker_provider_token_path_toml="$(toml_string "$docker_provider_token_path")"
docker_runtime_token_path_json="$(json_string "$docker_runtime_token_path")"
docker_runtime_token_path_toml="$(toml_string "$docker_runtime_token_path")"
mcp_image_json="$(json_string "$mcp_image")"
mcp_image_toml="$(toml_string "$mcp_image")"
node_bin_json="$(json_string "$node_bin")"
node_bin_toml="$(toml_string "$node_bin")"

docker_network_json_args=""
docker_network_toml_args=""
if [[ -n "$mcp_docker_network" ]]; then
  mcp_docker_network_json="$(json_string "$mcp_docker_network")"
  mcp_docker_network_toml="$(toml_string "$mcp_docker_network")"
  docker_network_json_args="        \"--network\", ${mcp_docker_network_json},
"
  docker_network_toml_args="\"--network\", ${mcp_docker_network_toml}, "
fi

# Generate separately: never truncate an existing agent configuration.
generated_config=""
cleanup_config() { [[ -z "$generated_config" ]] || rm -f "$generated_config"; }
trap cleanup_config EXIT
prepare_output() {
  [[ ! -L "$out_path" ]] || fail "agent config must not be a symlink: $out_path"
  [[ ! -e "$out_path" || -f "$out_path" ]] || fail "agent config must be a regular file: $out_path"
  mkdir -p "$(dirname "$out_path")"
  generated_config="$(mktemp "$(dirname "$out_path")/.froglet-agent-config.XXXXXX")"
  chmod 0600 "$generated_config"
}

case "$target" in
  claude-code)
    out_path="${out_path:-$default_out_dir/.mcp.json}"
    prepare_output
    if [[ "$mcp_mode" == "native" ]]; then
      cat >"$generated_config" <<EOF
{
  "mcpServers": {
    "froglet": {
      "type": "stdio",
      "command": ${node_bin_json},
      "args": ["mcp"],
      "env": {
        "FROGLET_PROVIDER_URL": ${provider_url_json},
        "FROGLET_DAEMON_URL": ${provider_url_json},
        "FROGLET_RUNTIME_URL": ${runtime_url_json},
        "FROGLET_PROVIDER_AUTH_TOKEN_PATH": ${provider_token_path_json},
        "FROGLET_PROVIDER_CONTROL_TOKEN_PATH": ${provider_token_path_json},
        "FROGLET_RUNTIME_AUTH_TOKEN_PATH": ${runtime_token_path_json},
        "FROGLET_DATA_DIR": $(json_string "${FROGLET_DATA_DIR:-$HOME/.froglet/data}")
      }
    }
  }
}
EOF
    elif [[ "$mcp_mode" == "repo" ]]; then
      # Repo mode: use local node server.js
      cat >"$generated_config" <<EOF
{
  "mcpServers": {
    "froglet": {
      "type": "stdio",
      "command": "node",
      "args": [${server_path_json}],
      "env": {
        "FROGLET_PROVIDER_URL": ${provider_url_json},
        "FROGLET_RUNTIME_URL": ${runtime_url_json},
        "FROGLET_PROVIDER_AUTH_TOKEN_PATH": ${provider_token_path_json},
        "FROGLET_RUNTIME_AUTH_TOKEN_PATH": ${runtime_token_path_json},
        "FROGLET_REQUEST_TIMEOUT_MS": ${request_timeout_ms_json},
        "FROGLET_DEFAULT_SEARCH_LIMIT": ${default_search_limit_json},
        "FROGLET_MAX_SEARCH_LIMIT": ${max_search_limit_json}
      }
    }
  }
}
EOF
    else
      # Binary-only mode: use the published MCP Docker image. Token files are
      # mounted read-only so local management actions like publish_artifact
      # work after the agent bootstrap creates ~/.froglet/data/runtime.
      # Provider/runtime URLs point at the local docker compose stack via
      # host.docker.internal by default; override with FROGLET_PROVIDER_URL /
      # FROGLET_RUNTIME_URL for hosted use.
      docker_provider_url="${provider_url/127.0.0.1/host.docker.internal}"
      docker_provider_url="${docker_provider_url/localhost/host.docker.internal}"
      docker_runtime_url="${runtime_url/127.0.0.1/host.docker.internal}"
      docker_runtime_url="${docker_runtime_url/localhost/host.docker.internal}"
      if [[ -n "$mcp_docker_network" ]]; then
        case "$provider_url" in
          http://127.0.0.1:8080|http://localhost:8080) docker_provider_url="http://provider:8080" ;;
        esac
        case "$runtime_url" in
          http://127.0.0.1:8081|http://localhost:8081) docker_runtime_url="http://runtime:8081" ;;
        esac
      fi
      docker_provider_url_json="$(json_string "$docker_provider_url")"
      docker_runtime_url_json="$(json_string "$docker_runtime_url")"
      cat >"$generated_config" <<EOF
{
  "mcpServers": {
    "froglet": {
      "type": "stdio",
      "command": "docker",
      "args": [
        "run", "--rm", "-i",
${docker_network_json_args}
        "--add-host", "host.docker.internal:host-gateway",
        "-v", ${provider_token_dir_json},
        "-e", "FROGLET_PROVIDER_URL",
        "-e", "FROGLET_RUNTIME_URL",
        "-e", "FROGLET_PROVIDER_AUTH_TOKEN_PATH",
        "-e", "FROGLET_RUNTIME_AUTH_TOKEN_PATH",
        "-e", "FROGLET_DAEMON_URL",
        "-e", "FROGLET_PROVIDER_CONTROL_TOKEN_PATH",
        "-e", "FROGLET_REQUEST_TIMEOUT_MS",
        "-e", "FROGLET_DEFAULT_SEARCH_LIMIT",
        "-e", "FROGLET_MAX_SEARCH_LIMIT",
        ${mcp_image_json}
      ],
      "env": {
        "FROGLET_PROVIDER_URL": ${docker_provider_url_json},
        "FROGLET_RUNTIME_URL": ${docker_runtime_url_json},
        "FROGLET_PROVIDER_AUTH_TOKEN_PATH": ${docker_provider_token_path_json},
        "FROGLET_RUNTIME_AUTH_TOKEN_PATH": ${docker_runtime_token_path_json},
        "FROGLET_DAEMON_URL": ${docker_provider_url_json},
        "FROGLET_PROVIDER_CONTROL_TOKEN_PATH": ${docker_provider_token_path_json},
        "FROGLET_REQUEST_TIMEOUT_MS": ${request_timeout_ms_json},
        "FROGLET_DEFAULT_SEARCH_LIMIT": ${default_search_limit_json},
        "FROGLET_MAX_SEARCH_LIMIT": ${max_search_limit_json}
      }
    }
  }
}
EOF
    fi
    if [[ "$mcp_mode" == "repo" ]]; then
      printf 'Activation: restart Claude Code in this repo so it reloads %s\n' "$out_path"
    elif [[ "$mcp_mode" == "native" ]]; then
      printf 'Activation: native Froglet MCP config is installed at %s\n' "$out_path"
    else
      printf 'Activation: move this .mcp.json to your project dir; Claude Code picks it up on next start\n'
      printf 'Note: Docker MCP mode requires docker; %s is mounted read-only for auth tokens\n' "$provider_token_dir"
    fi
    ;;
  codex)
    out_path="${out_path:-$default_out_dir/.codex/config.toml}"
    prepare_output
    if [[ "$mcp_mode" == "native" ]]; then
      cat >"$generated_config" <<EOF
[mcp_servers.froglet]
command = ${node_bin_toml}
args = ["mcp"]
env = { FROGLET_PROVIDER_URL = ${provider_url_toml}, FROGLET_DAEMON_URL = ${provider_url_toml}, FROGLET_RUNTIME_URL = ${runtime_url_toml}, FROGLET_PROVIDER_AUTH_TOKEN_PATH = ${provider_token_path_toml}, FROGLET_PROVIDER_CONTROL_TOKEN_PATH = ${provider_token_path_toml}, FROGLET_RUNTIME_AUTH_TOKEN_PATH = ${runtime_token_path_toml}, FROGLET_DATA_DIR = $(toml_string "${FROGLET_DATA_DIR:-$HOME/.froglet/data}") }
EOF
    elif [[ "$mcp_mode" == "repo" ]]; then
      cat >"$generated_config" <<EOF
[mcp_servers.froglet]
command = "node"
args = [${server_path_toml}]
env = { FROGLET_PROVIDER_URL = ${provider_url_toml}, FROGLET_RUNTIME_URL = ${runtime_url_toml}, FROGLET_PROVIDER_AUTH_TOKEN_PATH = ${provider_token_path_toml}, FROGLET_RUNTIME_AUTH_TOKEN_PATH = ${runtime_token_path_toml}, FROGLET_REQUEST_TIMEOUT_MS = "${request_timeout_ms}", FROGLET_DEFAULT_SEARCH_LIMIT = "${default_search_limit}", FROGLET_MAX_SEARCH_LIMIT = "${max_search_limit}" }
EOF
    else
      docker_provider_url="${provider_url/127.0.0.1/host.docker.internal}"
      docker_provider_url="${docker_provider_url/localhost/host.docker.internal}"
      docker_runtime_url="${runtime_url/127.0.0.1/host.docker.internal}"
      docker_runtime_url="${docker_runtime_url/localhost/host.docker.internal}"
      if [[ -n "$mcp_docker_network" ]]; then
        case "$provider_url" in
          http://127.0.0.1:8080|http://localhost:8080) docker_provider_url="http://provider:8080" ;;
        esac
        case "$runtime_url" in
          http://127.0.0.1:8081|http://localhost:8081) docker_runtime_url="http://runtime:8081" ;;
        esac
      fi
      docker_provider_url_toml="$(toml_string "$docker_provider_url")"
      docker_runtime_url_toml="$(toml_string "$docker_runtime_url")"
      cat >"$generated_config" <<EOF
[mcp_servers.froglet]
command = "docker"
args = ["run", "--rm", "-i", ${docker_network_toml_args}"--add-host", "host.docker.internal:host-gateway", "-v", ${provider_token_dir_toml}, "-e", "FROGLET_PROVIDER_URL", "-e", "FROGLET_RUNTIME_URL", "-e", "FROGLET_PROVIDER_AUTH_TOKEN_PATH", "-e", "FROGLET_RUNTIME_AUTH_TOKEN_PATH", "-e", "FROGLET_DAEMON_URL", "-e", "FROGLET_PROVIDER_CONTROL_TOKEN_PATH", "-e", "FROGLET_REQUEST_TIMEOUT_MS", "-e", "FROGLET_DEFAULT_SEARCH_LIMIT", "-e", "FROGLET_MAX_SEARCH_LIMIT", ${mcp_image_toml}]
env = { FROGLET_PROVIDER_URL = ${docker_provider_url_toml}, FROGLET_RUNTIME_URL = ${docker_runtime_url_toml}, FROGLET_PROVIDER_AUTH_TOKEN_PATH = ${docker_provider_token_path_toml}, FROGLET_RUNTIME_AUTH_TOKEN_PATH = ${docker_runtime_token_path_toml}, FROGLET_DAEMON_URL = ${docker_provider_url_toml}, FROGLET_PROVIDER_CONTROL_TOKEN_PATH = ${docker_provider_token_path_toml}, FROGLET_REQUEST_TIMEOUT_MS = "${request_timeout_ms}", FROGLET_DEFAULT_SEARCH_LIMIT = "${default_search_limit}", FROGLET_MAX_SEARCH_LIMIT = "${max_search_limit}" }
EOF
    fi
    if [[ "$mcp_mode" == "repo" ]]; then
      printf 'Activation: start Codex from %s so it picks up the project config\n' "$repo_root"
    elif [[ "$mcp_mode" == "native" ]]; then
      printf 'Activation: native Froglet MCP config is installed at %s\n' "$out_path"
    else
      printf 'Activation: place this config.toml at .codex/config.toml in your project dir\n'
    fi
    ;;
  openclaw)
    if [[ -z "$repo_root" ]]; then
      fail "openclaw target requires the froglet repo cloned. The OpenClaw plugin is a local folder (integrations/openclaw/froglet). Clone https://github.com/AlitheaBio/froglet and re-run scripts/setup-agent.sh from the repo root, or use --target claude-code for a Docker-based MCP config."
    fi
    out_path="${out_path:-$repo_root/.froglet/openclaw.config.json}"
    prepare_output
    cat >"$generated_config" <<EOF
{
  "plugins": {
    "load": {
      "paths": [
        ${openclaw_plugin_path_json}
      ]
    },
    "entries": {
      "froglet": {
        "enabled": true,
        "config": {
          "hostProduct": "openclaw",
          "providerUrl": ${provider_url_json},
          "runtimeUrl": ${runtime_url_json},
          "providerAuthTokenPath": ${provider_token_path_json},
          "runtimeAuthTokenPath": ${runtime_token_path_json},
          "requestTimeoutMs": $request_timeout_ms,
          "defaultSearchLimit": $default_search_limit,
          "maxSearchLimit": $max_search_limit
        }
      }
    }
  },
  "agents": {
    "list": [
      {
        "id": "main",
        "tools": {
          "allow": [
            "froglet"
          ]
        }
      }
    ]
  }
}
EOF
    printf 'Verification: node %s --config %s --target openclaw\n' \
      "$repo_root/integrations/openclaw/froglet/scripts/doctor.mjs" \
      "$out_path"
    ;;
  *)
    fail "unsupported target: $target"
    ;;
esac

merge_binary="${FROGLET_CONFIG_MERGER:-$node_bin}"
# Do not invoke a random PATH binary as a configuration writer. Old releases
# treated unknown commands as daemon startup. Bootstrap supplies its verified
# current binary explicitly; repository-only setup can safely create a new file.
if [[ "$target" =~ ^(claude-code|codex)$ && -n "$merge_binary" && -x "$merge_binary" ]]; then
  merge_args=(configure-agent --target "$target" --generated "$generated_config" --output "$out_path")
  if [[ -n "${FROGLET_AGENT_CONFIG_EXPECTED_SHA256:-}" ]]; then
    merge_args+=(--expected-sha256 "$FROGLET_AGENT_CONFIG_EXPECTED_SHA256")
  fi
  if [[ -n "${FROGLET_AGENT_CONFIG_RECEIPT:-}" ]]; then
    merge_args+=(--receipt "$FROGLET_AGENT_CONFIG_RECEIPT")
  fi
  "$merge_binary" "${merge_args[@]}"
  [[ -f "$out_path" && ! -L "$out_path" ]] || fail "configuration merger did not produce a regular configuration file"
else
  # Compatibility bootstrap without a native merger may create a new file,
  # but must never erase an existing client configuration.
  [[ ! -e "$out_path" ]] || fail "existing agent config requires the current froglet-node configuration merger; original file unchanged: $out_path"
  [[ "${FROGLET_AGENT_CONFIG_EXPECTED_SHA256:-missing}" == missing ]] || fail "agent config changed after approval"
  # A hard link provides no-clobber creation even if another writer races us.
  ln "$generated_config" "$out_path" || fail "agent config appeared during setup; original file unchanged"
fi
printf 'Configured %s at %s\n' "$target" "$out_path"

if [[ "$mcp_mode" == "repo" && "$target" =~ ^(claude-code|codex)$ ]]; then
  mcp_dependency_marker="$repo_root/integrations/mcp/froglet/node_modules/@modelcontextprotocol/sdk/package.json"
  if [[ ! -f "$mcp_dependency_marker" ]]; then
    printf 'Dependency: run npm ci --prefix %s before starting the local MCP server.\n' \
      "$repo_root/integrations/mcp/froglet"
  fi
fi

if [[ "$provider_token_path" == "$repo_root/data/runtime/froglet-control.token" ]]; then
  printf 'Compose-backed usage: start docker compose with FROGLET_HOST_READABLE_CONTROL_TOKEN=true so %s is readable on the host.\n' "$provider_token_path"
fi
