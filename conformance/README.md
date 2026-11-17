# Froglet conformance vectors

These vectors define conformance for the `froglet/v1` kernel. Where the prose in
[docs/SPEC.md](../docs/SPEC.md) and [docs/KERNEL.md](../docs/KERNEL.md) disagrees
with a vector, **the vector wins**. There is no certification process beyond
reproducing them.

The fixtures are checked in as data rather than regenerated at test time, so
review can focus on the irreversible wire values themselves.

## Vector registry

| File | Covers | Stability |
|---|---|---|
| `kernel_v1.json` | Paid chain (Descriptor → Offer → Quote → Deal → InvoiceBundle → Receipt), free chain, 14 artifact accept/reject cases, 5 invoice-bundle validation cases, linked-identity challenge | **Frozen forever.** Never regenerated; a change here would break the "signed `froglet/v1` artifacts verify forever" guarantee. |
| `x402_v1.json` | `x402.eip3009.v1` chain (no invoice bundle), 6 artifact accept/reject cases, evidence-bundle hash commitment | Additive. Regenerate with the ignored generator test when the method's rules change. |
| `operator_ledger_v1.json` | `operator_ledger.v1` chain (no invoice bundle), 7 artifact accept/reject cases, `SHA256(reservation_id)` binding | Additive. Regenerate with the ignored generator test when the method's rules change. |

## Fixture shape

Every fixture is a JSON object with:

- `fixture_type`, `fixture_version`, `schema_version`
- `keys` — the seeds used to derive signing keys, so a third party can re-derive
  every signature rather than trusting the recorded ones
- `artifacts` — a map of name → **artifact vector**, each carrying:
  - `artifact` — the complete signed artifact document
  - `canonical_signing_bytes_hex` — the exact canonical bytes supplied to the
    Froglet signer, which hashes them once with SHA-256 before BIP-340 signing
  - `payload_hash` — `SHA256(JCS(payload))`
  - `artifact_hash` — `SHA256(canonical signing bytes)`
- `conformance_path.artifact_order` — the order in which artifacts form a chain
- `artifact_verification_cases[]` — `{name, artifact_type, artifact, expected_valid}`
- optionally `invoice_bundle_validation_cases[]`, `linked_identity`, `evidence`

## What a conforming runner must assert

For each fixture file:

1. **Byte reproduction.** For every artifact vector, recompute the canonical
   signing bytes from the artifact's own fields and assert they equal
   `canonical_signing_bytes_hex` exactly. Then assert the recomputed
   `payload_hash` and `artifact_hash` match the recorded values. This is the
   strongest single check: it pins canonical JSON, field ordering, and the
   signing-bytes construction in one comparison.
2. **Envelope verification.** Every artifact in `artifacts` must verify:
   schema version, payload hash, artifact hash, and BIP-340 signature over the
   32-byte `SHA256(canonical signing bytes)` digest, with the signer treated as
   an x-only public key. The digest bytes, not their hex encoding, are the
   BIP-340 message.
3. **Semantics.** Every artifact must pass its kind-specific validator,
   including the signer-binding rule.
4. **Chain validation.** Walk `conformance_path.artifact_order` and validate the
   links; the chain must be valid with no reference clock supplied.
5. **Accept/reject table.** For every entry in `artifact_verification_cases`,
   the outcome must equal `expected_valid`. A runner that accepts a case marked
   `false` is not conforming, and neither is one that rejects a case marked
   `true` for the wrong reason — check the failure is attributable.
6. **Invoice-bundle cases**, where present: reproduce `expected_valid` and, when
   given, the `expected_issue_codes` set.
7. **Evidence commitments**, where present: recompute `SHA256(JCS(bundle))` and
   assert it equals the recorded `bundle_hash` *and* the value the settled
   receipt commits to.

Issue codes are part of the contract; see the registry in
[docs/SPEC.md §3.2](../docs/SPEC.md).

## Reference runners

| Runner | Language | Command |
|---|---|---|
| [tests/kernel_conformance_vectors.rs](../tests/kernel_conformance_vectors.rs) | Rust | `cargo test -p froglet --test kernel_conformance_vectors` |
| [tests/x402_conformance_vectors.rs](../tests/x402_conformance_vectors.rs) | Rust | `cargo test -p froglet --test x402_conformance_vectors` |
| [froglet-verify/tests/conformance.rs](../froglet-verify/tests/conformance.rs) | Rust (standalone verifier) | `cargo test -p froglet-verify` |
| [froglet-wasm/tests/conformance.rs](../froglet-wasm/tests/conformance.rs) | Rust (browser signing build; signs the vectors again, byte for byte) | `cargo test -p froglet-wasm` |
| [python/tests/test_conformance_vectors.py](../python/tests/test_conformance_vectors.py) | Python | `python3 -m unittest python.tests.test_conformance_vectors` |

The standalone verifier is also the easiest way to check a chain by hand:

```bash
cargo run -p froglet-verify -- --json path/to/chain.json
```

## Regenerating (additive fixtures only)

```bash
cargo test -p froglet --test generate_x402_vectors generate_x402_conformance_vectors -- --ignored
```

`kernel_v1.json` has no regeneration path by design. Each generator has a
companion read-only test asserting the checked-in file still matches, so a stale
fixture fails CI rather than drifting silently.
