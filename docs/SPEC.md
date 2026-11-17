# Froglet Protocol Specification

Version: 1.1 (2026-07-31)
Status: normative umbrella. Stability guarantees and the change process are in [VERSIONING.md](./VERSIONING.md).

Froglet is an **evidence layer for agent transactions**. Whatever rail moves the
money, a Froglet interaction produces a hash-linked chain of signed artifacts
that a third party can verify offline: who authorised what, under which scope
and limits, and what the executing party attested to afterwards.

This document is the umbrella. It references the frozen kernel contract rather
than restating it, and adds the chapters that previously existed only in code.

| Chapter | Where |
|---|---|
| 1. Kernel — envelope, artifacts, hashing, states | **[KERNEL.md](./KERNEL.md)** (normative, frozen) |
| 2. Verification algorithm | §2 below |
| 3. Chain validation and the issue-code registry | §3 below |
| 4. Settlement-method registry: what each method proves | §4 below |
| 5. Conformance | §5 below |
| 6. Non-goals | §6 below |
| 7. Deferred: selective disclosure | §7 below |

Every normative statement in §2–§5 carries a bracketed anchor — `SPEC-`, an area
code, and a number. `tests/spec_coverage.rs` fails CI if any anchor has no test
claiming it, so a statement here cannot drift from the code without turning the
build red.

---

## 2. Verification algorithm

A conforming verifier, given one artifact document, MUST perform these steps in
this order.

**[SPEC-VER-1]** Decode the document into a signed envelope whose payload is
retained as **uninterpreted JSON**. Verification of the envelope — payload hash,
artifact hash, and signature — MUST be computed over the payload exactly as
received, not over a re-serialization of a typed structure.

**[SPEC-VER-2]** A verifier that cannot interpret every field of a payload MUST
NOT report a signature failure on that basis. If the envelope verifies but the
payload carries fields the verifier does not know, the correct report is
"envelope verified, semantics not evaluated".

*Rationale for both.* Payload hashing is defined over canonical JSON of the
payload. An implementation that decodes into a fixed struct and re-serializes
silently drops unknown fields, changing the hash, and would report a valid
future artifact as forged. The reference implementation
(`froglet-verify::verify_document`) decodes into `SignedArtifact<serde_json::Value>`
first for exactly this reason.

**[SPEC-VER-3]** After the envelope verifies, a verifier SHOULD apply the
kind-specific semantic rules from KERNEL.md, including the signer-binding rule
(`signer == payload.provider_id`, or `payload.requester_id` for a Deal). A
verifier that projects artifacts into a derived store MUST apply them: a valid
signature alone does not bind an artifact to the provider it names.

**[SPEC-VER-4]** Expiry MUST NOT be evaluated against an implicit clock. A
verifier evaluates expiry only when the caller supplies a reference time. The
kernel has no clock; offline verification of an archived chain is a first-class
use, and an expired-but-valid chain is a different finding from an invalid one.

---

## 3. Chain validation

### 3.1 Paths

A chain is validated as a set of pairwise paths, each producing a report. The
composed form is `validate_full_chain`.

| Path | Artifacts | Present when |
|---|---|---|
| `descriptor_offer` | Descriptor → Offer | always |
| `offer_quote` | Offer → Quote | always |
| `quote_deal` | Quote → Deal | no invoice bundle |
| `quote_invoice_bundle_deal` | Quote → InvoiceBundle → Deal | Lightning escrow only |
| `quote_deal_receipt` | Quote → Deal → Receipt | execution finished |

**[SPEC-CHN-1]** Each link is verified against the parent's **envelope hash**,
not its payload hash. The chain is child-to-parent: a parent carries no pointer
to its children.

**[SPEC-CHN-2]** Envelope and artifact-type findings MUST be reported before
cross-link findings within a path, so report ordering is deterministic across
implementations.

**[SPEC-CHN-3]** A full-chain result is valid if and only if every constituent
path report is valid. Warnings never affect validity.

**[SPEC-CHN-4]** An InvoiceBundle is valid only under
`lightning.base_fee_plus_success_fee.v1`; presenting one under any other
settlement method is an error (`invoice_bundle_for_non_lightning_method`).

### 3.2 Issue-code registry

Issue codes are a stable public contract: implementations, indexers, and dispute
tooling match on these strings. **[SPEC-CHN-5]** A conforming implementation MUST
emit these exact codes for these conditions, and MUST NOT reuse a code for a
different condition.

| Code | Condition |
|---|---|
| `artifact_type_mismatch` | `artifact_type` is not the kind expected at this chain position |
| `artifact_envelope_invalid` | envelope hash, payload hash, schema version, or signature invalid |
| `artifact_semantic_invalid` | kind-specific semantic validator rejected the artifact |
| `artifact_expired` | `expires_at` precedes the caller-supplied reference time |
| `provider_mismatch` | `provider_id` disagrees with the linked parent artifact |
| `requester_mismatch` | `requester_id` disagrees with the linked parent artifact |
| `descriptor_hash_mismatch` | `descriptor_hash` does not match the descriptor's envelope hash |
| `offer_hash_mismatch` | `offer_hash` does not match the offer's envelope hash |
| `quote_hash_mismatch` | `quote_hash` does not match the quote's envelope hash |
| `deal_hash_mismatch` | `deal_hash` does not match the deal's envelope hash |
| `workload_kind_mismatch` | quote `workload_kind` differs from offer `offer_kind` |
| `workload_hash_mismatch` | deal `workload_hash` differs from quote `workload_hash` |
| `confidential_session_hash_mismatch` | confidential session hash disagrees across artifacts |
| `quote_expiry_exceeds_offer` | quote `expires_at` is later than the offer's |
| `settlement_method_mismatch` | settlement method disagrees across artifacts |
| `settlement_terms_mismatch` | fee amounts disagree across artifacts |
| `execution_limits_exceed_offer` | limits exceed the parent's declared maxima |
| `deadline_order_invalid` | not `admission < completion <= acceptance` |
| `deadline_exceeds_quote` | a deadline is later than the quote's `expires_at` |
| `invoice_bundle_for_non_lightning_method` | invoice bundle under a non-escrow method |
| `invoice_bundle_required_for_lightning_method` | Lightning escrow chain omits its required invoice bundle |
| `invoice_amount_mismatch` | invoice leg amounts differ from quote settlement terms |
| `invoice_destination_mismatch` | destination identity differs from quote settlement terms |
| `invoice_success_payment_hash_mismatch` | bundle success-fee `payment_hash` ≠ deal `success_payment_hash` |
| `invoice_min_cltv_mismatch` | `min_final_cltv_expiry` differs from quote settlement terms |
| `invoice_hash_mismatch` | leg `invoice_hash` ≠ `SHA256(invoice_bolt11)` |
| `invoice_payment_hash_mismatch` | receipt payment hashes differ from the signed invoice-bundle legs |
| `invoice_state_mismatch` | receipt settlement states contradict the terminal Lightning leg states |
| `invoice_expiry_exceeds_deal` | bundle `expires_at` is later than the deal `admission_deadline` |
| `receipt_bundle_hash_mismatch` | receipt `bundle_hash` does not identify the supplied invoice bundle |

---

## 4. Settlement-method registry

The evidence value of a receipt depends on the rail. Overstating any cell of
this table is the single most damaging thing this project could publish, so each
row states what is proven by mathematics and what is merely asserted by a
signer.

**[SPEC-SET-1]** A conforming implementation MUST NOT describe an attested fact
as cryptographically proven, in an API response, a report, or user-facing copy.

| Method | Proof of payment | Independently checkable by | Notes |
|---|---|---|---|
| `none` | n/a — free | n/a | Legs are zero-valued canceled placeholders |
| `lightning.base_fee_plus_success_fee.v1` | **Attested** by the provider's signed receipt; the requester separately controls the success-fee preimage | Counterparties who observed the HTLC | Escrow-style, two legs, uses an `invoice_bundle` |
| `lightning.prepaid.v1` | **Cryptographic**: `SHA256(preimage) == payment_hash`, both carried in the receipt | Anyone, offline | Strongest non-escrow evidence in the protocol |
| `stripe_mpp.v1` | **Attested** only: receipt carries a PaymentIntent id | Stripe, or an account holder via Stripe's API | No offline-checkable proof of payment |
| `x402.eip3009.v1` | **Split**: the payer's EIP-712 authorization is cryptographically verifiable offline; transaction inclusion is attested | Authorization: anyone offline. Inclusion: any chain view | See KERNEL.md §5.6 |
| `operator_ledger.v1` | **Attested** by the provider's signed receipt over `SHA256(reservation_id)` bound in `Deal.success_payment_hash` | Closed-loop operator ledger and parties holding `reservation_id` | Two-leg reservation method without `invoice_bundle`; finality is an operator-ledger transition keyed to `receipt.hash` |

**[SPEC-SET-2]** For `x402.eip3009.v1`, value equivalence between the quoted
`base_fee_msat` and the transferred stablecoin units is a provider attestation.
An implementation MUST document the conversion it applies and MUST NOT present
the equivalence as established by the artifact chain.

**[SPEC-SET-3]** A verifier encountering an unrecognized settlement method MUST
reject the artifact rather than fall back to a weaker check. New methods are
additive: they never invalidate previously valid artifacts, but a verifier that
does not know a method cannot vouch for a deal using it.

---

## 5. Conformance

**[SPEC-CNF-1]** Conformance is defined by the checked-in vectors, not by this
prose. Where they disagree, the vectors win.

**[SPEC-CNF-2]** An implementation is conforming when, for every vector file in
`conformance/`, it reproduces every recorded `canonical_signing_bytes_hex`,
`payload_hash`, and `artifact_hash` byte-for-byte, verifies every artifact whose
case expects acceptance, and rejects every artifact whose case expects rejection.

| Vector file | Covers | Stability |
|---|---|---|
| `kernel_v1.json` | Full Lightning escrow paid chain, free chain, 14 accept/reject cases, 5 invoice-bundle cases, linked-identity challenge | **Frozen forever.** Never regenerated. |
| `x402_v1.json` | `x402.eip3009.v1` chain, 6 accept/reject cases, evidence-bundle commitment | Additive |
| `operator_ledger_v1.json` | `operator_ledger.v1` chain, 7 accept/reject cases, reservation hash commitment | Additive |

Reference runners: `tests/kernel_conformance_vectors.rs` and
`tests/x402_conformance_vectors.rs` (Rust), `python/tests/test_conformance_vectors.py`
(Python), `froglet-verify/tests/conformance.rs` (the standalone verifier).

There is no certification process beyond the vectors.

---

## 6. Non-goals

Froglet is deliberately **not** the following, and no future version will make
it so without a new major version and a new name for the guarantee:

- **Not a payment rail.** Froglet does not move money. It records what was
  authorised and what was attested, alongside whichever rail moved it.
- **No custody.** No Froglet component holds funds, keys to funds, or escrow on
  behalf of a counterparty.
- **No settlement finality.** A Froglet receipt is evidence about settlement,
  never the settlement itself. Finality belongs to the rail.
- **No consensus and no chain.** There is no global ordering, no validator set,
  no shared state machine. Artifacts are verified independently by whoever holds
  them.
- **No token.**
- **Not proof of correctness.** A receipt proves the provider signed a result
  hash, not that the result is right. Verifying execution correctness without
  re-running the work is an open research problem, and the protocol says so
  rather than pretending otherwise.

---

## 7. Deferred: selective disclosure

Regulated counterparties cannot always publish full artifact contents, so
redaction-with-preserved-verifiability is a known requirement. It is **not in
v1**, and the reason is structural: `payload_hash` commits to the whole payload
as one canonical JSON document, so there is no way to reveal one field and prove
it belongs to the signed artifact without revealing the rest.

Supporting it requires per-field commitments (a Merkle root over canonicalised
fields) as an *additional* signed field, alongside a verification mode that
accepts a redacted payload plus inclusion proofs. That is an additive change to
the envelope, so a future minor version can introduce it without invalidating
`froglet/v1` artifacts — but it cannot be retrofitted onto artifacts already
signed. Implementations that will need it should say so before v2 field
selection is frozen.
