# Synthetic immunopeptidomics with NVIDIA FLARE and Froglet

This reference asks a narrow question: among samples in a selected cohort whose
recorded HLA genotype contains the requested allele, how many accepted, available
assay results detected an exact candidate peptide?

It uses two synthetic sites, separate provider-local SQLite databases, and a
small Wasm computation. Installed NVIDIA FLARE Executor/DXO components dispatch
tasks from one Python process to real local Froglet nodes. It does not provision
a FLARE server/client network or connect to HLA-Compass production.

## Reproduce the reference

Use Python 3.11 and Rust 1.91.0 with `wasm32-unknown-unknown`. Install the optional
Python environment separately from your other applications:

```bash
python3.11 -m venv .venv
source .venv/bin/activate
python -m pip install -r python/requirements-immunopeptidomics.txt
python -m pip check
rustup target add wasm32-unknown-unknown
cargo build --locked --release -p froglet --bin froglet-node
cargo build --locked --release --target wasm32-unknown-unknown \
  --manifest-path examples/wasm-services/Cargo.toml -p immunopeptidomics
python examples/immunopeptidomics_demo.py \
  --binary target/release/froglet-node \
  --module examples/wasm-services/target/wasm32-unknown-unknown/release/immunopeptidomics.wasm \
  --output /tmp/immunopeptidomics-success.json
```

Use an exact published commit for repeatable source selection and record
`git rev-parse HEAD`. The report includes source, binary, module, fixture and
provider database hashes, plus the installed FLARE version. The direct framework
dependencies are pinned; retain `python -m pip freeze` alongside the report to
record the complete installed environment. Execution requires FLARE 2.9.0 and
does not silently substitute another framework or an arithmetic-only path.

The default provider execution allowance is 30 seconds, including cold Wasm
compilation; `--execution-timeout-secs` accepts 1–300. Providers retain bounded
admission and execution limits. Successful runs remove temporary node state;
use `--keep-state` to retain it. Execution failures retain state and write a
failure report when `--output` is supplied. Invalid preflight arguments and
missing dependencies fail before node startup.

## Scientific contract and expected result

The [fixture contract](../examples/fixtures/immunopeptidomics/README.md),
[schema](../examples/fixtures/immunopeptidomics/schema.sql), and
[hand-labelled expectations](../examples/fixtures/immunopeptidomics/expected.json)
are part of the example. All samples, labels and detection calls are invented.

The query is an exact unmodified peptide sequence, an exact two-field class-I
HLA allele, a sample-class token and an organ token. HLA selection describes the
sample genotype; it does not identify the presenting allele or prove binding.
Known target alleles establish eligibility. If the target is absent, only
complete typing at the requested locus establishes nonmatching; incomplete or
missing typing remains unknown.

Only eligible samples with accepted, completed and available assay results form
the detection denominator. A sample with an explicitly empty accepted detection
set is measured but not detected. Failed or unavailable assays remain missing,
even if partial observation rows exist. Duplicate observations count once.

| Synthetic count | Site A | Site B | Pooled |
|---|---:|---:|---:|
| Selected cohort | 8 | 8 | 16 |
| Genotype eligible | 5 | 6 | 11 |
| Measured eligible | 3 | 5 | 8 |
| Candidate detected | 2 | 3 | 5 |
| Candidate not detected | 1 | 2 | 3 |
| Eligible assay missing | 2 | 1 | 3 |
| Genotype unknown | 2 | 1 | 3 |
| Genotype nonmatching | 1 | 1 | 2 |

The expected detection fraction is **5/8**, obtained by summing counts, not
averaging site percentages. When no eligible sample is measured, the fraction
is null rather than zero. The unit is a specimen; real integrations must address
duplicate specimens and technical replicates before pooling. This example does
not deduplicate donors across sites or establish population prevalence.

## Execution and evidence

1. The coordinator declares the required provider identities, module and exact
   request/workload hashes.
2. FLARE components pass each query to a site executor, which requests a signed
   Froglet computation under the provider's granted host-compute offer.
3. The Wasm module uses fixed parameterized read-only SQL through the local
   `cohort` handle. Only counts, query metadata and interpretation labels are
   returned; no sample rows, genotype lists or observation lists are returned.
4. The requester verifies each five-artifact chain and the result's canonical
   JSON hash, validates the query/count contract, and compares the synthetic
   output with an independent Python oracle.
5. A repeated submission must recover the same signed deal and result.
6. The aggregate is promoted only after every declared site has contributed and
   the pooled result matches the independent oracle.

Each worker's report retains its signed artifacts and result for offline
rechecking. A valid signature establishes a recorded commitment and its chain;
it does not prove physical dataset identity or scientific correctness. The
completeness state is in memory, with no durable restart protocol or distributed
rollback.

## Failure and regression checks

```bash
cargo test --locked --manifest-path examples/wasm-services/Cargo.toml \
  -p immunopeptidomics
python -W error -m unittest python.tests.test_froglet_fl_evidence \
  examples.test_immunopeptidomics_demo -v
python examples/immunopeptidomics_demo.py \
  --binary target/release/froglet-node \
  --module examples/wasm-services/target/wasm32-unknown-unknown/release/immunopeptidomics.wasm \
  --unavailable-site site_b --output /tmp/immunopeptidomics-failure.json
```

The last command intentionally exits with failure. Its JSON must retain Site A's
verified contribution, mark Site B missing, and contain no promoted aggregate.
An unavailable site is not interpreted as a site with zero detections. The
dedicated `Immunopeptidomics Reference` CI workflow runs both live paths and
retains the reports and dependency inventory.

## Interpretation and deployment boundary

Non-detection is not biological absence. This example consumes accepted peptide
calls; it does not reprocess spectra, validate false-discovery thresholds,
harmonize intensities, establish assay comparability, predict immunogenicity, or
assess clinical safety. It has no GPU requirement or GPU performance claim.

Aggregate counts can disclose information. The example has no small-cell
suppression, differential privacy, secure aggregation, or independently enforced
query budget. Use only the synthetic fixtures for this reference. A real data
deployment needs a defined scientific/export contract and disclosure policy.

Agent runtime qualification is separate; see [OpenShell/NemoClaw](NEMOCLAW.md).
