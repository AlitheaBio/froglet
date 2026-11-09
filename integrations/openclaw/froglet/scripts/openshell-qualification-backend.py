#!/usr/bin/env python3
"""Prepare a disposable real Froglet/TLS endpoint for the documented macOS test.

Requires the pinned OpenShell 0.1.7 CLI and gateway on PATH, Docker Desktop,
OpenSSL, and a built Froglet node. No inference provider or production token is
used. This helper does not start OpenShell, install software, or change Docker
settings. See docs/OPENSHELL_QUALIFICATION.md for the complete test and cleanup.
"""
from __future__ import annotations

import argparse
import atexit
import http.server
import json
import os
from pathlib import Path
import re
import shlex
import shutil
import signal
import ssl
import subprocess
import sys
import time
import urllib.error
import urllib.request


def wait_for_provider(node, timeout=30):
    """Wait independently for the dual-role provider listener and healthy API."""
    deadline = time.monotonic() + timeout
    opener = urllib.request.build_opener(urllib.request.ProxyHandler({}))
    while time.monotonic() < deadline:
        if node.process.poll() is not None:
            raise RuntimeError(f"Froglet exited before provider readiness; inspect {node.log}")
        match = re.search(r"Local API Gateway: (http://127\.0\.0\.1:\d+)",
                          node.log.read_text(encoding="utf-8", errors="replace"))
        if match:
            provider = match.group(1)
            try:
                with opener.open(provider + "/health", timeout=2) as response:
                    if json.load(response).get("status") == "ok":
                        return provider
            except (urllib.error.URLError, TimeoutError, json.JSONDecodeError, OSError):
                pass
        time.sleep(0.1)
    raise RuntimeError(f"Froglet provider readiness timed out; inspect {node.log}")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--work-dir", type=Path, required=True)
    parser.add_argument("--repo", type=Path, default=Path(__file__).resolve().parents[4])
    parser.add_argument("--binary", type=Path)
    args = parser.parse_args()
    if sys.platform != "darwin":
        parser.error("This recipe is qualified on macOS/Docker Desktop only; Linux is not yet verified")
    root = args.work_dir.resolve()
    root.mkdir(mode=0o700, parents=True, exist_ok=True)
    root.chmod(0o700)
    if (root / "tls").exists() or (root / "test-node").exists():
        parser.error("Use a new work directory; existing private state is never overwritten")
    repo = args.repo.resolve()
    binary = (args.binary or repo / "target/release/froglet-node").resolve()
    if not binary.is_file():
        parser.error("Build froglet-node first or supply --binary")
    for command, expected in [("openshell", "openshell 0.1.7"),
                              ("openshell-gateway", "openshell-gateway 0.1.7")]:
        version = subprocess.check_output([command, "--version"], text=True).strip()
        if version != expected:
            parser.error(f"Expected {expected}; found {version}")
    docker_host = subprocess.check_output(
        ["docker", "context", "inspect", "--format", "{{.Endpoints.docker.Host}}"],
        text=True).strip()
    if os.environ.get("DOCKER_HOST"):
        docker_host = os.environ["DOCKER_HOST"]
    if not docker_host.startswith("unix://"):
        parser.error("Only a local Unix Docker socket is accepted; remote engines are outside this test")
    subprocess.run(["docker", "info", "--format", "{{.ServerVersion}}"], check=True,
                   stdout=subprocess.DEVNULL)
    for directory in ["tls", "config", "state", "data", "stage", "evidence", "image"]:
        (root / directory).mkdir(mode=0o700)
    environment = {key: value for key, value in os.environ.items()
                   if not key.startswith("OPENSHELL_")}
    environment.update({
        "XDG_CONFIG_HOME": str(root / "config"),
        "XDG_STATE_HOME": str(root / "state"),
        "XDG_DATA_HOME": str(root / "data"),
        "OPENSHELL_LOCAL_TLS_DIR": str(root / "tls/gateway"),
    })
    (root / "env.sh").write_text("\n".join(
        f"export {key}={shlex.quote(environment[key])}"
        for key in ["XDG_CONFIG_HOME", "XDG_STATE_HOME", "XDG_DATA_HOME", "OPENSHELL_LOCAL_TLS_DIR"]
    ) + "\n")

    def checked(argv):
        subprocess.run(argv, check=True, env=environment, stdout=subprocess.DEVNULL,
                       stderr=subprocess.PIPE)

    checked(["openshell-gateway", "generate-certs", "--output-dir", str(root / "tls/gateway"),
             "--server-san", "host.docker.internal", "--server-san", "localhost",
             "--server-san", "127.0.0.1"])
    (root / "config/ca.cnf").write_text("""[req]
distinguished_name=dn
x509_extensions=extensions
prompt=no
[dn]
CN=Froglet disposable qualification CA
[extensions]
basicConstraints=critical,CA:TRUE
keyUsage=critical,digitalSignature,keyCertSign
""")
    (root / "config/leaf.cnf").write_text("""basicConstraints=critical,CA:FALSE
keyUsage=critical,digitalSignature,keyEncipherment
extendedKeyUsage=serverAuth
subjectAltName=DNS:host.openshell.internal,DNS:host.docker.internal,DNS:localhost,IP:127.0.0.1
""")
    checked(["openssl", "req", "-x509", "-newkey", "rsa:2048", "-nodes", "-days", "2",
             "-config", str(root / "config/ca.cnf"), "-keyout", str(root / "tls/backend-ca.key"),
             "-out", str(root / "tls/backend-ca.crt")])
    checked(["openssl", "req", "-new", "-newkey", "rsa:2048", "-nodes",
             "-subj", "/CN=Froglet qualification endpoint", "-keyout", str(root / "tls/backend.key"),
             "-out", str(root / "tls/backend.csr")])
    checked(["openssl", "x509", "-req", "-in", str(root / "tls/backend.csr"),
             "-CA", str(root / "tls/backend-ca.crt"), "-CAkey", str(root / "tls/backend-ca.key"),
             "-CAcreateserial", "-out", str(root / "tls/backend.crt"), "-days", "2",
             "-extfile", str(root / "config/leaf.cnf")])
    for key in (root / "tls").glob("*.key"):
        key.chmod(0o600)
    # json.dumps quotes paths as TOML basic strings without shell interpolation.
    quote = lambda path: json.dumps(str(path))
    gateway_tls = root / "tls/gateway"
    (root / "config/gateway.toml").write_text(f"""[openshell]
version = 2
[openshell.gateway]
name = "froglet-qualification"
bind_address = "127.0.0.1:18670"
compute_driver = "docker"
guest_tls_ca = {quote(gateway_tls / 'ca.crt')}
[openshell.gateway.tls]
cert_path = {quote(gateway_tls / 'server/tls.crt')}
key_path = {quote(gateway_tls / 'server/tls.key')}
client_ca_path = {quote(gateway_tls / 'ca.crt')}
[openshell.gateway.gateway_jwt]
signing_key_path = {quote(gateway_tls / 'jwt/signing.pem')}
public_key_path = {quote(gateway_tls / 'jwt/public.pem')}
kid_path = {quote(gateway_tls / 'jwt/kid')}
gateway_id = "froglet-qualification"
[openshell.gateway.auth]
allow_unauthenticated_users = false
[openshell.gateway.mtls_auth]
enabled = true
[openshell.supervisor.network]
additional_ca_bundle = {quote(root / 'tls/backend-ca.crt')}
[openshell.drivers.docker]
socket_path = {quote(docker_host.removeprefix('unix://'))}
image_pull_policy = "if_not_present"
sandbox_label = "froglet-qualification"
supervisor_network_mode = "bridge"
grpc_endpoint = "https://host.docker.internal:18670"
enable_bind_mounts = false
sandbox_pids_limit = 2048
""")
    checked(["openshell-gateway", "config", "preflight", "--path", str(root / "config/gateway.toml")])
    (root / "image/Dockerfile").write_text("""FROM node:22.14.0-bookworm-slim@sha256:1c18d9ab3af4585870b92e4dbc5cac5a0dc77dd13df1a5905cea89fc720eb05b
WORKDIR /sandbox
RUN chown node:node /sandbox
USER node
CMD ["sleep", "infinity"]
""")
    policy = {
        "version": 1,
        "filesystem_policy": {"include_workdir": True,
            "read_only": ["/usr", "/lib", "/etc", "/proc", "/dev/urandom", "/var/log"],
            "read_write": ["/tmp", "/dev/null"]},
        "landlock": {"compatibility": "hard_requirement"},
        "process": {"run_as_user": "1000", "run_as_group": "1000"},
        "network_policies": {"froglet_read_qualification": {
            "endpoints": [{"host": "host.openshell.internal", "port": 19443,
                "protocol": "rest", "enforcement": "enforce", "rules": [
                    {"allow": {"method": "GET", "path": "/v1/provider/services"}},
                    {"allow": {"method": "GET", "path": "/v1/runtime/spend"}}]}],
            "binaries": [{"path": "/usr/local/bin/node"}]}}
    }
    for mode in ["baseline", "restricted"]:
        (root / f"config/policy-{mode}.yaml").write_text(json.dumps(policy, indent=2) + "\n")
        policy["network_policies"]["froglet_read_qualification"]["endpoints"][0]["rules"] = [
            {"allow": {"method": "GET", "path": "/v1/provider/services"}}]
    sys.path.insert(0, str(repo / "examples"))
    from a2a_compute_demo import Node
    node = Node(binary, root, "test-node", "dual")
    atexit.register(node.stop)
    node.identity()
    node.start()
    provider = wait_for_provider(node)
    runtime = node.url
    for source, target in [(node.data / "runtime/froglet-control.token", "provider.token"),
                           (node.data / "runtime/auth.token", "runtime.token")]:
        shutil.copyfile(source, root / "stage" / target)
        (root / "stage" / target).chmod(0o600)
    audit = root / "evidence/backend-access.jsonl"
    opener = urllib.request.build_opener(urllib.request.ProxyHandler({}))

    class Handler(http.server.BaseHTTPRequestHandler):
        def do_GET(self):
            upstream = runtime if self.path.startswith("/v1/runtime/") else provider
            request = urllib.request.Request(upstream + self.path,
                headers={"Authorization": self.headers.get("Authorization", "")})
            try:
                response = opener.open(request, timeout=15)
            except urllib.error.HTTPError as error:
                response = error
            payload = response.read()
            with audit.open("a") as output:
                output.write(json.dumps({"method": "GET", "path": self.path,
                    "upstream_status": response.status}) + "\n")
            self.send_response(response.status)
            self.send_header("Content-Type", "application/json")
            self.send_header("Content-Length", str(len(payload)))
            self.end_headers()
            self.wfile.write(payload)

        def log_message(self, *args):
            pass

    server = http.server.ThreadingHTTPServer(("127.0.0.1", 19443), Handler)
    context = ssl.SSLContext(ssl.PROTOCOL_TLS_SERVER)
    context.load_cert_chain(root / "tls/backend.crt", root / "tls/backend.key")
    server.socket = context.wrap_socket(server.socket, server_side=True)
    (root / "ready.json").write_text(json.dumps({
        "provider": provider, "runtime": runtime, "binary": str(binary),
        "api_kind": "real_froglet_node", "credentials": "disposable_test_node",
    }, indent=2) + "\n")
    signal.signal(signal.SIGTERM, lambda *_: sys.exit(0))
    signal.signal(signal.SIGINT, lambda *_: sys.exit(0))
    try:
        print("Disposable Froglet endpoint ready; private state must not be published", flush=True)
        server.serve_forever()
    finally:
        server.server_close()
        node.stop()


if __name__ == "__main__":
    main()
