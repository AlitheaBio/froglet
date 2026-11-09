# Examples

## Synthetic immunopeptidomics with NVIDIA FLARE

[immunopeptidomics_demo.py](immunopeptidomics_demo.py) sends an exact peptide/HLA
cohort query through installed FLARE components to two local Froglet providers.
A small Wasm module reads separate local SQLite fixtures and returns counts with
explicit measured denominators and missingness. Signed evidence, retries and
refusal of incomplete aggregates are checked. All data is synthetic.

See [the reproducible reference](../docs/IMMUNOPEPTIDOMICS_REFERENCE.md) for the
pinned environment, build/run commands, scientific boundaries and failure test.

## Local Three-Role Stack

```bash
docker compose up --build
```

## Core Examples

- `confidential_policy.example.toml`: protocol/test fixture for the reserved
  confidential policy shape; mock providers do not enable production execution
- `wasm-services/`: sample `froglet.wasm.run_json.v1` services in Rust (an adder, a
  Fibonacci function and a synthetic terminology audit) with the contract and build steps; the docs-site playground's
  editor starts from AssemblyScript ports of them, held to the same answers by tests

## Local MCP and A2A compute demo

Run two disposable, independent Nodes: Alice supplies the Rust adder program and
`{"a":6,"b":7}` through native MCP; Bob executes it through the configured A2A
transport and returns `{"sum":13,"product":42}` with verified receipt evidence.
The demo also checks an exact retry, refusal of changed input under the same key,
termination of an infinite loop, and recovery after Bob restarts.

Prerequisites: Python 3.10+, Cargo/Rust, and the Wasm target. From this checkout:

```bash
rustup target add wasm32-unknown-unknown
python3 examples/a2a_compute_demo.py
```

The script builds the node and existing Wasm examples with `--locked`, then starts
real provider/requester processes on literal localhost ports. Use `--no-build`
after the initial build, or `--output /path/to/report.json` to save the sanitized
JSON report. `--keep-state` retains the private temporary directory for inspection.

With an already installed native binary and a precompiled Wasm program, pass both
paths explicitly. This skips `rustup` and Cargo entirely; Python 3.10+ still runs
the disposable setup/check helper:

```bash
python3 examples/a2a_compute_demo.py --scenario ontology \
  --binary /absolute/path/to/froglet-node \
  --module /absolute/path/to/ontology_check.wasm --serve
```

Native MCP `run_compute` also accepts `wasm_module_path` for an absolute local
regular nonsymlink file, at most 262144 bytes, instead of copying its full hex into
an agent prompt. It cannot be combined with `wasm_module_hex`. Both forms bind the
same exact compiled bytes, input and retry key through the existing runtime.
Changing a file's bytes while retaining a key is new work and is refused. This
file option belongs to the native bridge; the JavaScript adapter still uses hex.

To leave both Nodes available for an agent host:

```bash
python3 examples/a2a_compute_demo.py --no-build --serve
```

After its checks pass, this prints paths to a private `mcp-host.json` containing
the native binary command and local token-file paths, and `agent-call.json`
containing Bob's reference and copyable `froglet` tool arguments. No credentials
are embedded in the host JSON, and the script does not edit host settings. Use
the host configuration while the demo runs; Ctrl-C stops both Nodes and removes
their temporary state unless `--keep-state` was supplied. The agent call uses a
new key; changing its input or program requires another new key. Replace the
recovery template's `task_id` with the `deal_id` returned by your agent's call.

Each run uses fresh keys and data directories, scrubs inherited Froglet and proxy
configuration, and writes separate A2A credentials with mode `0600`. No wallet or
paid backend is configured. Child processes stop on success, failure or interrupt;
successful state is removed by default, while failure logs remain in the reported
private directory. Exact retries produce one provider Deal; this execution path
does not enqueue a separate `jobs` row.

This qualifies a free, local, operator-configured MCP/A2A workflow. It does not
demonstrate real-money settlement, Tor, public federation or scientific result
quality. The adder example accepts flat integer inputs; it is not a general JSON
schema validator. See [the interoperability plan](../docs/AGENT_INTEROPERABILITY_PLAN.md)
for the interface and recovery boundaries.

## Replay an already generated program without an LLM

`generated_program_replay.py` runs an existing, explicitly pinned Wasm program
against caller-supplied original and held-out JSON inputs and expected answers.
It starts two disposable localhost nodes and no Codex/Claude session, compiler,
installer or public service. It does not read agent-host settings. This tests the
program and runtime; it does not repeat or establish a fresh agent's ability to
generate the program.

Use Python 3.10+ and a compatible native executable. Input paths can be outside
this checkout. The expected original document can be the result directly or an
object containing `expected_result`. Held-out input and expected documents use
`{"cases":[{"label":"case-name","input":...}]}` and
`{"cases":[{"label":"case-name","expected_result":...}]}`. Labels must match;
no expected answer is calculated by this runner.

```sh
python3 examples/generated_program_replay.py \
  --binary /absolute/path/to/froglet-node \
  --binary-sha256 REVIEWED_BINARY_SHA256 \
  --module /absolute/path/to/generated-program.wasm \
  --module-sha256 REVIEWED_PROGRAM_SHA256 \
  --original-input /absolute/path/to/original-input.json \
  --original-expected /absolute/path/to/original-expected.json \
  --held-out-inputs /absolute/path/to/held-out-inputs.json \
  --held-out-expected /absolute/path/to/held-out-expected.json \
  --output-dir /absolute/path/to/new-replay-evidence
```

For the assay example, optional `--source /absolute/path/to/catalog.json` also
publishes and retrieves only `record_id`, `batch_id`, `signal_milliunits` and
`qc_status` from `assay_readouts`. The original input must equal those selected
rows exactly. The source can be a private mixed export or an already selected
public snapshot; replaying the latter does not requalify privacy against the
private original. Optional `--research-profile` supplies explicit declarations
for the existing signed output schema; matching declarations are not scientific
validation. A facilitator performs preparation/publication and a local owner
canary. Those steps are not autonomous-agent evidence.

The runner checks exact retries, refusal of changed input, a separate looping
program stopped by its execution budget, and recovery after the provider
restarts. It then exports **every admitted Deal** before stopping the provider
and restarting the requester. Exact known compute and data invocations recover
the cached signed results with their original keys. The runner also requires
`get_task` to revalidate every saved terminal operation locally while the provider
is stopped, including selected data and signed execution failure. The historical
pre-correction HTTP 502 evidence remains preserved. Unsigned or unresolved tasks
still need provider synchronization; recovery of a known completed invocation
does not imply recovery of all uncertain submissions.

Each run requires a **new** evidence directory and fresh temporary identities.
The selected snapshot is preserved as actual bytes. Full Descriptor, Offer,
Quote, Deal and Receipt envelopes, persisted workloads and result bytes are
exported for offline independent verification. Provider-local operation IDs can
differ from signed Deal hashes; both references are retained. A successful read
of a failed task has `isError: false`, distinct from a failed execution call.
Native signature checks and expected-answer comparisons are reported separately
from subsequent independent verification. Signed evidence does not attest
physical execution.

The public runner needs only `generated_program_replay.py`,
`a2a_compute_demo.py` and `research_profile.py` beside each other, Python's
standard library, the pinned executable/program and the explicit JSON files.
Reset by choosing a new output directory; it never changes an existing provider,
wallet or agent host. Nodes stop on completion or interruption. Complete evidence
is saved before private state is removed; incomplete exports retain stopped
private state for explicit recovery.

Focused checks:
`python3 -W error -m unittest examples.test_generated_program_replay -v`.

## Reproducible unpublished demo bundle

Package an already reviewed native executable and ontology Wasm with their
explicit SHA-256 pins. This helper does not build, publish, install, restart or
tag anything. Create a private output directory first, then run from this checkout:

```sh
python3 scripts/package_demo_bundle.py \
  --binary /absolute/path/to/froglet-node \
  --binary-sha256 REVIEWED_BINARY_SHA256 \
  --module /absolute/path/to/ontology_check.wasm \
  --module-sha256 REVIEWED_MODULE_SHA256 \
  --candidate local.20261001.demo --platform darwin --arch arm64 \
  --out-dir /absolute/private-output-directory
```

Use `linux`/`darwin` and `arm64`/`x86_64` only for the matching supplied executable.
The helper checks its basic ELF/Mach-O header; this does not establish runtime
compatibility, library availability, fresh-machine installation or Gatekeeper
acceptance. Candidate labels must begin `local.` or `candidate.`. Input paths
must be absolute, contain no symbolic links, and name bounded regular files:
node at most 128 MiB, Wasm v1 at most 262144 bytes. A changed pinned input is
refused, and an existing archive is never overwritten.

The archive contains only the pinned node and Wasm, the two disposable demo
helpers and research preflight, the reviewed counterparty setup helper and its
instructions, launchers, license and a member-integrity manifest. It never walks
build directories or collects credentials, databases, handoffs or installer
fixtures. `MANIFEST.json` records exact member hashes, sizes and permissions.
With the same Python/zlib runtime, identical payload bytes and candidate arguments
produce identical archive bytes; checkout helper changes intentionally change
those hashes. This manifest is
neither a signature nor a release/attestation claim. Preserve the printed archive
SHA-256 separately through the reviewed delivery channel.

Extract into a new private directory. Python 3.10+ and a compatible POSIX host
are still required; Cargo, Rustup, Node.js and this checkout are unnecessary:

```sh
python3 /absolute/bundle/scripts/package_demo_bundle.py --verify-dir /absolute/bundle
/absolute/bundle/RUN-DEMO.sh --output /absolute/ontology-report.json
# Optional agent-host session; Ctrl-C stops its disposable Nodes:
/absolute/bundle/RUN-DEMO.sh --serve
```

The launchers verify every member before starting work, then enforce the
packaged program and binary paths. Write reports outside the bundle and keep its
contents unchanged. The default demo uses free loopback execution and synthetic
terms; it does not configure an existing provider, wallet or agent host.

Counterparty qualification remains an explicit, separate action and additionally
requires OpenSSL:

```sh
/absolute/bundle/RUN-COUNTERPARTY-CHECK.sh --output /absolute/counterparty-report.json
```

That check uses fresh disposable Nodes and process-local TLS trust. To prepare
settings for a real counterparty, first read the bundled
`docs/A2A_COUNTERPARTY_SETUP.md`, then explicitly use the separate setup helper's
read-only `--plan` and reviewed apply flow. The main launcher never applies that
configuration or restarts real services. An HTTP invitation remains provider-wide
beside an Offer-scoped A2A credential.

Focused packaging checks:
`python3 -W error -m unittest python.tests.test_package_demo_bundle -v`.

## Selected terminology and disposable audit program

```bash
python3 examples/a2a_compute_demo.py --scenario ontology
# After building, save the actual sanitized evidence:
python3 examples/a2a_compute_demo.py --scenario ontology --no-build --output /path/to/report.json
```

Bob prepares a synthetic JSON table through native MCP `prepare_service`, selecting
only `source` and `target`; the source's `curator_note` is excluded. He publishes the
immutable snapshot through `marketplace_publish` with `host: "local"`. The script
checks the selected snapshot and Alice's retrieved result for the excluded column
and private sentinel. The local publication does not register with a marketplace.

Alice uses native MCP `invoke_service` to retrieve that actual data service through
her own requester runtime and configured A2A. The mappings supplied to her Wasm
audit come only from the returned rows. She submits the locally built
`ontology_check.wasm` as requester-supplied code through `run_compute`; Bob did not
publish a dedicated ontology-audit service.

All labels use the `DEMO:` namespace. Alice's policy requires one distinct target
per source, using exact, case-sensitive strings without normalization. Multiple
targets are ambiguity under that policy; they do not prove scientific errors.
Identical duplicate rows are allowed. The checker reports conflicts across the
whole supplied mapping table, unique observed terms absent from the table, and
the number of unique observed terms having exactly one target. An unmapped term
does not make `mappings_consistent` false.

The four actual cases have these expected outcomes, recomputed independently in
Python from each explicit input:

| Case | Mapping table consistent | Sources with multiple targets | Unmapped observed terms | Unambiguously mapped observed terms |
|---|---|---|---|---|
| Consistent control | Yes | 0 | 0 | 3 |
| Conflicting targets | No | 1 | 0 | 2 |
| Unmapped term | Yes | 0 | 1 | 3 |
| Conflict and unmapped | No | 1 | 1 | 2 |

For the consistent control only, Alice explicitly retains the lexically last
target from the retrieved candidates. The actual combined audit keeps both
`DEMO:sample.blood` candidates and identifies absent `DEMO:sample.saliva`.

There are six execution Deals: one table retrieval, four audits, and one
nonterminating Wasm program stopped by its execution budget. Data and compute
retries preserve the original Deal and result hash; changed input under the same
key is refused. Restarting Bob retains both data and compute signed evidence.
Limits are read from actual signed Quotes and checked against applied receipt
limits; the report does not infer which individual limit caused the generic
`execution_limit_exceeded` failure. All work is free and local, with independent
Alice/Bob identities and no payment backend.

The report contains the returned table, explicit inputs and results, Python oracle
results, source/snapshot/module hashes, and Descriptor/Offer/Quote/Deal/Receipt
documents for each execution. Native MCP verifies signed-chain and output-hash
bindings; the Python oracle checks these small deterministic audit results.
Neither check proves ontology truth or general scientific usefulness. To inspect
an exported chain independently, build `froglet-verify`, extract one execution's
`signed_artifacts` values as a JSON array, and run
`froglet-verify --chain --json <chain.json>`.

This localhost data lookup uses Bob's operator-configured public metadata origin
(`FROGLET_DAEMON_URL`) and Bob's identity, omitting an explicit `provider_url` in
`invoke_service`. Alice retains only her own runtime credentials; Bob's separate
operator context performs publication. Explicit remote/share-link invocation
still requires public HTTPS and is not qualified by this demo. `--serve` also
works with `--scenario ontology`, using the same private host-config workflow.

Focused oracle tests: `python3 -W error -m unittest examples.test_a2a_compute_demo -v`.

## Measure bounded capacity and compare a simple alternative

Use existing node and ontology Wasm artifacts from a checkout. The capacity
qualification does not build, deploy a public service, connect an LLM host,
configure a wallet, or spend money:

```bash
python3 examples/qualify_ontology_capacity.py \
  --binary /path/to/froglet-node --module /path/to/ontology_check.wasm \
  --requester-identities per-worker --output /path/to/independent-report.json

# Also test many concurrent tools sharing one requester's identity:
python3 examples/qualify_ontology_capacity.py \
  --binary /path/to/froglet-node --module /path/to/ontology_check.wasm \
  --requester-identities shared --output /path/to/shared-report.json
```

The default sweep uses 1, 2, 4, 8, 16 and 32 concurrent client workflows, with
two finite sequential iterations per worker. Each Froglet workflow retrieves
Bob's actual selected table, then submits Alice's audit Wasm through two fresh
native MCP processes and configured A2A. `per-worker` starts a separate Alice
runtime and signing key per worker; `shared` deliberately exercises one Alice
identity with distinct retry keys and identical data/program inputs. Bob remains
one independent provider process and identity. The supplied executable is copied
to a private temporary directory and pinned by digest for every node, MCP call
and restart, so a concurrent rebuild cannot silently change the measured binary.

The same five selected rows and explicit input are also checked through a small
conventional HTTP/Python service and by reading the selected JSON file and
running Python locally. Both alternatives must produce the identical audit.
The conventional service accepts a fixed server-side function; it does not run
Alice's program or provide signed authorization, a sandbox resource contract,
durable tasks, retry keys, result receipts, recovery or payment. The local file
alternative requires already having the selected data and performs no remote
transaction. Timing these alternatives cannot quantify the value of Froglet's
additional boundaries or prove developer-time savings or scientific usefulness.

The sanitized JSON report includes attempted/success/error counts, nearest-rank
p50/p95 latency over **all attempts**, successful throughput, actually observed
active workflows, hardware/load/configuration, source and executable digests,
independent identities, returned task references and actual provider Deal counts.
It also checks up to eight concurrent exact retries, provider restart recovery,
unchanged persistent reservations, refusal of new work at an exhausted Deal
allowance, and explicit requester `submission_pending` counts. Terminal data
failures expose a task reference in the invocation error; the runner recovers
that task through native MCP and verifies its signed failure receipt. An error
task reference alone is not proof that Bob admitted a Deal.

These counts are not TCP connections or a production audience limit. This path
has separate A2A operation, Wasm execution and built-in/process worker limits;
the selected-table service consumes a process worker slot. The runner explicitly
keeps process slots at 4, Wasm slots at 16 and public quota at 6000 requests per
900-second window. It raises the default 60 quote requests per identity to a
finite recorded allowance for the sweep. Current quote issuance safely refuses
duplicate identical requester/workload/terms quotes within one Unix second; the
requester retries only that recognized pre-Deal collision up to four attempts.
Bursts can therefore end in a clear refusal even when CPU execution slots remain.
Those refusals stay in the error denominator, and quote attempts consume quota.

The run uses fresh private state and cleans up all started processes. Successful
private state is removed unless `--keep-state` is supplied; failed state is
retained for diagnosis. Neither a short local sweep nor successful catalog reads
qualifies public HTTPS, many physical machines, prolonged load, malicious
clients, real payment or useful scientific collaboration. Use the report's
tested settings and scope when presenting its numbers.

Focused runner tests:
`python3 -W error -m unittest examples.test_qualify_ontology_capacity -v`.

The native demo also carries an optional research profile in the existing signed
publication schema and checks Alice's explicit namespaces, versions, field types,
units and mapping policy before analysis. This is exact declared compatibility,
not scientific truth. See [research declarations and preflight](../docs/RESEARCH_PROFILE.md)
for refusal examples and a hash-pinned public GO term catalog preparation.
