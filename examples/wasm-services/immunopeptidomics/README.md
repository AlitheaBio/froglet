# Synthetic immunopeptidomics counts

This small `froglet.wasm.host_json.v1` module reads the provider's local SQLite
handle `cohort`. Its sole requested capability is
`db.sqlite.query.read.cohort`. It is independent of `fl-compute` and requires no
GPU, model training, or protocol changes.

The input has exactly four fields, for example:

```json
{
  "candidate_peptide": "ACDEFGHIK",
  "hla_allele": "HLA-A*02:01",
  "sample_class": "healthy_tissue",
  "organ": "synthetic_organ_a"
}
```

The supported query is deliberately narrow: an uppercase, unmodified peptide
using the twenty standard amino-acid letters and a two-field HLA-A, HLA-B, or
HLA-C allele (two or three digits in each field). Matching is exact. The module
does not normalize identifiers, infer HLA restriction, translate allele
resolution, remove allele suffixes, or interpret modified peptides.

The four provider-local tables are defined by the
[fixture schema](../../fixtures/immunopeptidomics/schema.sql). They
store samples, recorded HLA alleles, typing completeness by locus, and peptide
detections. The fixture and schema contain only synthetic examples. A measured
assay means that the assay completed, passed the fixture's inclusion rules,
and its accepted observation set is available. A missing observation export
must not be labeled measured. This does not imply exhaustive detection of
presented peptides.

For samples matching the requested sample class and organ:

- A recorded requested allele makes the sample eligible, even when typing is
  incomplete or its completeness assessment is missing.
- Without that allele, complete typing at its locus means nonmatching;
  incomplete or missing typing means unknown. Missing completeness assessment
  does not mean no alleles were recorded.
- Only measured eligible samples form the detection denominator. Each sample
  contributes at most once, regardless of duplicate peptide observations.
- Failed and not-measured eligible samples remain missing. Their partial or
  stale detection rows cannot turn them into measured positives or negatives.

The result has `schema_version: "froglet.immunopeptidomics.counts.v1"`, the
original `query`, an `interpretation` object, and these integer `counts`:

```text
cohort_total, hla_unknown, hla_nonmatching, eligible, measured,
detected, not_detected, not_measured, failed, missing
```

The identities are `cohort_total = hla_unknown + hla_nonmatching + eligible`,
`eligible = measured + missing`, `measured = detected + not_detected`, and
`missing = not_measured + failed`. A presenter may compute `detected / measured`;
when measured is zero, the rate is undefined, not zero. These are sample counts,
not patient prevalence. Pooling requires disjoint sample sets; this module
does not deduplicate across sites.

The interpretation strings are:

```json
{
  "hla_selection": "recorded_genotype_not_peptide_restriction",
  "not_detected": "not_detected_in_measured_assay_not_biological_absence",
  "denominator": "measured_eligible_samples"
}
```

Only counts and query metadata leave the guest. No sample identifiers or local
observation records are returned. Aggregate counts are not, by themselves, a
privacy guarantee; the example does not implement disclosure thresholds or
differential privacy.

The fixed SQL uses parameters and existence predicates. Duplicate sample IDs,
duplicate completeness rows, malformed host responses, and invalid statuses
produce `invalid_local_data`. Invalid requests produce `invalid_request` before
any host call. A host policy/query error traps in the runtime; it must be treated
as a failed execution, never a successful zero-count result. The SQLite host
rejects excessive row counts or output bytes rather than returning partial rows.

From the repository root:

```sh
cargo test --locked -p immunopeptidomics --manifest-path examples/wasm-services/Cargo.toml
cargo build --locked --release --target wasm32-unknown-unknown -p immunopeptidomics --manifest-path examples/wasm-services/Cargo.toml
```

The Wasm artifact is
`examples/wasm-services/target/wasm32-unknown-unknown/release/immunopeptidomics.wasm`.
Native tests exercise aggregation, missingness, strict validation, host request
shape, and record exclusion from outputs. They do not replace a live signed
execution against the provider runtime.
