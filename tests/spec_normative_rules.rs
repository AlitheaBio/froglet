/// Executable form of the normative statements in docs/SPEC.md.
///
/// Each test names the spec anchors it covers in a `// covers:` comment;
/// `tests/spec_coverage.rs` fails CI if any anchor lacks one.
use froglet::crypto;
use froglet::protocol::{
    self, ARTIFACT_TYPE_RECEIPT, DealPayload, DescriptorPayload, InvoiceBundlePayload,
    OfferPayload, QuotePayload, ReceiptPayload, SETTLEMENT_METHOD_LIGHTNING_PREPAID,
    SETTLEMENT_METHOD_STRIPE_MPP, SETTLEMENT_METHOD_X402_EIP3009, SignedArtifact, sign_artifact,
    validate_offer_artifact, validate_receipt_artifact, verify_artifact, verify_typed_document,
};
use serde::de::DeserializeOwned;
use serde_json::Value;
use std::{fs, path::PathBuf};

fn load(name: &str) -> Value {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("conformance")
        .join(name);
    serde_json::from_str(&fs::read_to_string(&path).expect("read fixture")).expect("parse fixture")
}

fn vector<T: DeserializeOwned>(fixture: &Value, name: &str) -> SignedArtifact<T> {
    serde_json::from_value(fixture["artifacts"][name]["artifact"].clone()).expect("decode vector")
}

fn provider_key(fixture: &Value) -> crypto::NodeSigningKey {
    let seed_hex = fixture["keys"]["provider_seed_hex"].as_str().expect("seed");
    let mut seed = [0u8; 32];
    for (index, chunk) in seed_hex.as_bytes().chunks(2).take(32).enumerate() {
        seed[index] =
            u8::from_str_radix(std::str::from_utf8(chunk).unwrap(), 16).expect("hex byte");
    }
    crypto::signing_key_from_seed_bytes(&seed).expect("valid seed")
}

// covers: SPEC-VER-1, SPEC-VER-2
#[test]
fn envelope_is_verified_over_the_payload_as_received_not_a_typed_round_trip() {
    let fixture = load("kernel_v1.json");
    let signing_key = provider_key(&fixture);
    let signer = crypto::public_key_hex(&signing_key);

    // A genuinely signed artifact carrying a field this build does not know.
    let mut payload = fixture["artifacts"]["receipt"]["artifact"]["payload"].clone();
    payload["field_from_a_later_kernel"] = Value::String("present and signed".to_string());
    let future = sign_artifact(
        &signer,
        |message| crypto::sign_message_hex(&signing_key, message),
        ARTIFACT_TYPE_RECEIPT,
        1_700_000_000,
        payload,
    )
    .expect("sign");
    let document = serde_json::to_value(&future).expect("serialize");

    // SPEC-VER-1: verifying over the payload as received succeeds.
    let as_received: SignedArtifact<Value> =
        serde_json::from_value(document.clone()).expect("decode as raw value");
    assert!(
        verify_artifact(&as_received),
        "envelope must verify over the exact received payload"
    );

    // SPEC-VER-2: the typed round trip drops the unknown field and would report
    // a signature failure. A conforming verifier must not surface that as
    // forgery — froglet-verify reports envelope_only instead, which is asserted
    // in froglet-verify/tests/conformance.rs.
    let typed: SignedArtifact<ReceiptPayload> =
        serde_json::from_value(document).expect("typed decode ignores unknown fields");
    assert!(
        !verify_artifact(&typed),
        "typed round trip drops the unknown field — this is exactly the trap SPEC-VER-2 forbids reporting as a signature failure"
    );
}

// covers: SPEC-VER-3
#[test]
fn semantic_validation_enforces_signer_binding_beyond_the_signature() {
    let fixture = load("kernel_v1.json");
    let mut receipt: SignedArtifact<ReceiptPayload> = vector(&fixture, "receipt");

    // Re-sign a receipt that claims a different provider than its signer, with
    // a valid signature over the tampered payload: signature alone accepts it.
    let signing_key = provider_key(&fixture);
    receipt.payload.provider_id = "ab".repeat(32);
    let resigned = sign_artifact(
        &crypto::public_key_hex(&signing_key),
        |message| crypto::sign_message_hex(&signing_key, message),
        ARTIFACT_TYPE_RECEIPT,
        receipt.created_at,
        receipt.payload.clone(),
    )
    .expect("sign");

    assert!(
        verify_artifact(&resigned),
        "the signature itself is valid — that is the point"
    );
    assert!(
        validate_receipt_artifact(&resigned).is_err(),
        "semantic validation must reject a receipt whose signer is not its provider_id"
    );
}

// covers: SPEC-VER-4
#[test]
fn expiry_is_only_evaluated_against_a_caller_supplied_clock() {
    let fixture = load("kernel_v1.json");
    let offer: SignedArtifact<OfferPayload> = vector(&fixture, "offer");
    let quote: SignedArtifact<QuotePayload> = vector(&fixture, "quote");

    let without_clock = protocol::validate_offer_quote(&offer, &quote, None);
    assert!(
        without_clock.valid,
        "an archived chain must verify offline with no clock"
    );

    let with_future_clock =
        protocol::validate_offer_quote(&offer, &quote, Some(quote.payload.expires_at + 1));
    assert!(!with_future_clock.valid);
    assert!(
        with_future_clock
            .issues
            .iter()
            .any(|issue| issue.code == protocol::chain::ISSUE_ARTIFACT_EXPIRED),
        "expiry must be reported as its own distinct finding"
    );
}

// covers: SPEC-CHN-1, SPEC-CHN-3
#[test]
fn links_bind_to_parent_envelope_hashes_and_full_chain_is_the_conjunction() {
    let fixture = load("kernel_v1.json");
    let descriptor: SignedArtifact<DescriptorPayload> = vector(&fixture, "descriptor");
    let offer: SignedArtifact<OfferPayload> = vector(&fixture, "offer");
    let quote: SignedArtifact<QuotePayload> = vector(&fixture, "quote");
    let deal: SignedArtifact<DealPayload> = vector(&fixture, "deal");
    let bundle: SignedArtifact<InvoiceBundlePayload> = vector(&fixture, "invoice_bundle");
    let receipt: SignedArtifact<ReceiptPayload> = vector(&fixture, "receipt");

    // SPEC-CHN-1: each child names its parent's envelope hash.
    assert_eq!(offer.payload.descriptor_hash, descriptor.hash);
    assert_eq!(quote.payload.offer_hash, offer.hash);
    assert_eq!(deal.payload.quote_hash, quote.hash);
    assert_eq!(receipt.payload.deal_hash, deal.hash);

    let chain = protocol::FullChain {
        descriptor: &descriptor,
        offer: &offer,
        quote: &quote,
        invoice_bundle: Some(&bundle),
        deal: &deal,
        receipt: Some(&receipt),
    };
    let report = protocol::validate_full_chain(&chain, None);
    assert!(report.valid);
    // SPEC-CHN-3: validity is the conjunction of the path reports.
    assert_eq!(
        report.valid,
        report.reports.iter().all(|path| path.valid),
        "full-chain validity must be exactly the conjunction of its paths"
    );
    assert!(
        report.reports.iter().all(|path| path.warnings.is_empty()),
        "a clean chain carries no warnings"
    );
}

// covers: SPEC-CHN-2
#[test]
fn envelope_findings_precede_link_findings_within_a_path() {
    let fixture = load("kernel_v1.json");
    let offer: SignedArtifact<OfferPayload> = vector(&fixture, "offer");
    let mut quote: SignedArtifact<QuotePayload> = vector(&fixture, "quote");
    quote.artifact_type = "deal".to_string();
    quote.payload.offer_hash = "aa".repeat(32);

    let report = protocol::validate_offer_quote(&offer, &quote, None);
    let codes: Vec<&str> = report.issues.iter().map(|i| i.code.as_str()).collect();
    assert_eq!(
        &codes[..3],
        &[
            protocol::chain::ISSUE_ARTIFACT_TYPE_MISMATCH,
            protocol::chain::ISSUE_ARTIFACT_ENVELOPE_INVALID,
            protocol::chain::ISSUE_OFFER_HASH_MISMATCH,
        ],
        "report ordering must be deterministic: type, envelope, then links"
    );
}

// covers: SPEC-CHN-4
#[test]
fn invoice_bundle_is_rejected_under_a_non_escrow_method() {
    let fixture = load("kernel_v1.json");
    let quote: SignedArtifact<QuotePayload> = vector(&fixture, "free_quote");
    let deal: SignedArtifact<DealPayload> = vector(&fixture, "free_deal");
    let bundle: SignedArtifact<InvoiceBundlePayload> = vector(&fixture, "invoice_bundle");

    let report = protocol::validate_quote_invoice_bundle_deal(&quote, &bundle, &deal, None);
    assert!(!report.valid);
    assert!(
        report
            .issues
            .iter()
            .any(|i| i.code == protocol::chain::ISSUE_INVOICE_BUNDLE_FOR_NON_LIGHTNING),
        "expected invoice_bundle_for_non_lightning_method, got {:?}",
        report.issues
    );
}

// covers: SPEC-CHN-5
#[test]
fn issue_codes_are_the_exact_strings_the_registry_documents() {
    // The SPEC.md registry table and these constants are one contract; changing
    // a constant without changing the table (or vice versa) breaks consumers
    // that match on the string.
    let spec = fs::read_to_string(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("docs/SPEC.md"))
        .expect("read SPEC.md");

    for code in [
        protocol::chain::ISSUE_ARTIFACT_TYPE_MISMATCH,
        protocol::chain::ISSUE_ARTIFACT_ENVELOPE_INVALID,
        protocol::chain::ISSUE_ARTIFACT_SEMANTIC_INVALID,
        protocol::chain::ISSUE_ARTIFACT_EXPIRED,
        protocol::chain::ISSUE_PROVIDER_MISMATCH,
        protocol::chain::ISSUE_REQUESTER_MISMATCH,
        protocol::chain::ISSUE_DESCRIPTOR_HASH_MISMATCH,
        protocol::chain::ISSUE_OFFER_HASH_MISMATCH,
        protocol::chain::ISSUE_QUOTE_HASH_MISMATCH,
        protocol::chain::ISSUE_DEAL_HASH_MISMATCH,
        protocol::chain::ISSUE_WORKLOAD_KIND_MISMATCH,
        protocol::chain::ISSUE_WORKLOAD_HASH_MISMATCH,
        protocol::chain::ISSUE_CONFIDENTIAL_SESSION_HASH_MISMATCH,
        protocol::chain::ISSUE_QUOTE_EXPIRY_EXCEEDS_OFFER,
        protocol::chain::ISSUE_SETTLEMENT_METHOD_MISMATCH,
        protocol::chain::ISSUE_SETTLEMENT_TERMS_MISMATCH,
        protocol::chain::ISSUE_EXECUTION_LIMITS_EXCEED_OFFER,
        protocol::chain::ISSUE_DEADLINE_ORDER_INVALID,
        protocol::chain::ISSUE_DEADLINE_EXCEEDS_QUOTE,
        protocol::chain::ISSUE_INVOICE_BUNDLE_FOR_NON_LIGHTNING,
        protocol::chain::ISSUE_INVOICE_AMOUNT_MISMATCH,
        protocol::chain::ISSUE_INVOICE_DESTINATION_MISMATCH,
        protocol::chain::ISSUE_INVOICE_SUCCESS_PAYMENT_HASH_MISMATCH,
        protocol::chain::ISSUE_INVOICE_MIN_CLTV_MISMATCH,
        protocol::chain::ISSUE_INVOICE_HASH_MISMATCH,
        protocol::chain::ISSUE_INVOICE_EXPIRY_EXCEEDS_DEAL,
    ] {
        assert!(
            spec.contains(&format!("`{code}`")),
            "issue code {code} is not in the docs/SPEC.md registry table"
        );
    }
}

// covers: SPEC-SET-1, SPEC-SET-2
#[test]
fn only_the_prepaid_and_x402_authorization_paths_carry_offline_proof() {
    // SPEC-SET-1 is a claim-discipline rule; what is testable is the mechanism
    // behind each cell of the table.
    let kernel = load("kernel_v1.json");
    let receipt: SignedArtifact<ReceiptPayload> = vector(&kernel, "receipt");
    assert!(
        receipt.payload.settlement_refs.bundle_hash.is_some(),
        "the escrow method commits to an invoice bundle"
    );

    // lightning.prepaid.v1: sha256(preimage) == payment_hash is enforced, so a
    // receipt claiming settlement without the matching preimage is rejected.
    let mut forged = receipt.clone();
    forged.payload.settlement_refs.method = SETTLEMENT_METHOD_LIGHTNING_PREPAID.to_string();
    forged.payload.settlement_refs.bundle_hash = None;
    forged.payload.settlement_refs.destination_identity = String::new();
    forged.payload.settlement_refs.base_fee.invoice_hash = "11".repeat(32);
    forged.payload.settlement_refs.base_fee.payment_hash = "22".repeat(32);
    assert!(
        validate_receipt_artifact(&forged).is_err(),
        "a prepaid receipt whose preimage does not hash to payment_hash must be rejected"
    );

    // stripe_mpp.v1 carries no offline-checkable payment proof: its
    // payment_hash is an opaque PaymentIntent id, deliberately not hex-checked.
    assert_eq!(SETTLEMENT_METHOD_STRIPE_MPP, "stripe_mpp.v1");

    // SPEC-SET-2: the x402 receipt commits to an evidence bundle but states no
    // value equivalence; the msat amount and the transferred units are separate
    // facts, and only the commitment is cryptographic.
    let x402 = load("x402_v1.json");
    let x402_receipt: SignedArtifact<ReceiptPayload> = vector(&x402, "receipt");
    assert_eq!(
        x402_receipt.payload.settlement_refs.method,
        SETTLEMENT_METHOD_X402_EIP3009
    );
    assert_eq!(
        x402_receipt.payload.settlement_refs.bundle_hash.as_deref(),
        Some(x402["evidence"]["bundle_hash"].as_str().unwrap())
    );
}

// covers: SPEC-SET-3
#[test]
fn an_unrecognized_settlement_method_is_rejected_outright() {
    let fixture = load("kernel_v1.json");
    let mut offer: SignedArtifact<OfferPayload> = vector(&fixture, "offer");
    offer.payload.settlement_method = "future.rail.v9".to_string();
    assert!(
        validate_offer_artifact(&offer).is_err(),
        "an unknown paid method must be rejected at the Offer boundary"
    );

    let mut receipt: SignedArtifact<ReceiptPayload> = vector(&fixture, "receipt");
    receipt.payload.settlement_refs.method = "future.rail.v9".to_string();

    assert!(
        validate_receipt_artifact(&receipt).is_err(),
        "a verifier must reject an unknown method rather than fall back to a weaker check"
    );
}

// covers: SPEC-CNF-1, SPEC-CNF-2
#[test]
fn conformance_is_defined_by_the_vector_files_present_in_the_repo() {
    for (file, expected_cases) in [
        ("kernel_v1.json", 14usize),
        ("x402_v1.json", 6usize),
        ("operator_ledger_v1.json", 7usize),
    ] {
        let fixture = load(file);
        let cases = fixture["artifact_verification_cases"]
            .as_array()
            .unwrap_or_else(|| panic!("{file} must carry accept/reject cases"));
        assert_eq!(
            cases.len(),
            expected_cases,
            "{file} case count changed; docs/SPEC.md §5 and spec/conformance.md must be updated together"
        );

        for case in cases {
            let name = case["name"].as_str().unwrap();
            let expected_valid = case["expected_valid"].as_bool().unwrap();
            let artifact_type = case["artifact_type"].as_str().unwrap();
            assert_eq!(
                verify_typed_document(&case["artifact"], artifact_type).is_ok(),
                expected_valid,
                "{file}/{name}: vector expectation not reproduced"
            );
        }
    }
}
