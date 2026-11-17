/// Read-only conformance runner for `conformance/operator_ledger_v1.json`.
///
/// Mirrors `tests/x402_conformance_vectors.rs` for the additive
/// `operator_ledger.v1` settlement method: exact signing-bytes and hash
/// reproduction, envelope verification, per-kind semantics, full chain
/// validation without an `invoice_bundle`, reservation hash binding between
/// `Deal` and `Receipt`, and the accept/reject case table.
use froglet::{
    crypto,
    protocol::{
        self, ARTIFACT_TYPE_RECEIPT, DealPayload, DescriptorPayload, OfferPayload, QuotePayload,
        ReceiptPayload, SETTLEMENT_METHOD_OPERATOR_LEDGER, SignedArtifact, verify_artifact,
        verify_typed_document,
    },
};
use serde::{Serialize, de::DeserializeOwned};
use serde_json::Value;
use std::{fs, path::PathBuf};

fn load_fixture() -> Value {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("conformance/operator_ledger_v1.json");
    let bytes =
        fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
    serde_json::from_str(&bytes).unwrap_or_else(|e| panic!("parse {}: {e}", path.display()))
}

fn vector<T: DeserializeOwned>(fixture: &Value, name: &str) -> SignedArtifact<T> {
    serde_json::from_value(fixture["artifacts"][name]["artifact"].clone())
        .unwrap_or_else(|e| panic!("decode {name}: {e}"))
}

fn assert_exact_bytes<T: Serialize + Clone>(
    fixture: &Value,
    name: &str,
    artifact: &SignedArtifact<T>,
) {
    let recorded = &fixture["artifacts"][name];
    let signing_bytes = protocol::canonical_signing_bytes(
        &artifact.schema_version,
        &artifact.artifact_type,
        &artifact.signer,
        artifact.created_at,
        &artifact.payload_hash,
        &artifact.payload,
    )
    .unwrap();

    assert_eq!(
        hex::encode(&signing_bytes),
        recorded["canonical_signing_bytes_hex"].as_str().unwrap(),
        "{name}: canonical signing bytes must reproduce byte-for-byte"
    );
    assert_eq!(
        protocol::payload_hash(&artifact.payload).unwrap(),
        recorded["payload_hash"].as_str().unwrap(),
        "{name}: payload_hash must reproduce"
    );
    assert_eq!(
        protocol::artifact_hash(artifact).unwrap(),
        recorded["artifact_hash"].as_str().unwrap(),
        "{name}: artifact_hash must reproduce"
    );
    assert!(verify_artifact(artifact), "{name}: envelope must verify");
}

#[test]
fn operator_ledger_chain_artifacts_reproduce_exact_bytes_and_verify() {
    let fixture = load_fixture();
    assert_exact_bytes(
        &fixture,
        "descriptor",
        &vector::<DescriptorPayload>(&fixture, "descriptor"),
    );
    assert_exact_bytes(
        &fixture,
        "offer",
        &vector::<OfferPayload>(&fixture, "offer"),
    );
    assert_exact_bytes(
        &fixture,
        "quote",
        &vector::<QuotePayload>(&fixture, "quote"),
    );
    assert_exact_bytes(&fixture, "deal", &vector::<DealPayload>(&fixture, "deal"));
    assert_exact_bytes(
        &fixture,
        "receipt",
        &vector::<ReceiptPayload>(&fixture, "receipt"),
    );
}

#[test]
fn operator_ledger_chain_validates_end_to_end_and_rejects_mismatched_reservation_hash() {
    let fixture = load_fixture();
    let descriptor = vector::<DescriptorPayload>(&fixture, "descriptor");
    let offer = vector::<OfferPayload>(&fixture, "offer");
    let quote = vector::<QuotePayload>(&fixture, "quote");
    let deal = vector::<DealPayload>(&fixture, "deal");
    let receipt = vector::<ReceiptPayload>(&fixture, "receipt");

    let report = protocol::validate_full_chain(
        &protocol::FullChain {
            descriptor: &descriptor,
            offer: &offer,
            quote: &quote,
            invoice_bundle: None,
            deal: &deal,
            receipt: Some(&receipt),
        },
        None,
    );
    assert!(
        report.valid,
        "operator_ledger.v1 chain must validate: {:?}",
        report.reports
    );
    assert_eq!(
        receipt.payload.settlement_refs.method,
        SETTLEMENT_METHOD_OPERATOR_LEDGER
    );

    let reservation_id = fixture["reservation"]["reservation_id"].as_str().unwrap();
    let expected_hash = crypto::sha256_hex(reservation_id.as_bytes());
    assert_eq!(deal.payload.success_payment_hash, expected_hash);
    assert_eq!(
        receipt.payload.settlement_refs.success_fee.payment_hash,
        expected_hash
    );

    // Re-sign receipt with a different reservation payment_hash and assert
    // chain validation flags ISSUE_INVOICE_SUCCESS_PAYMENT_HASH_MISMATCH.
    let provider_seed_hex = fixture["keys"]["provider_seed_hex"].as_str().unwrap();
    let mut seed = [0u8; 32];
    hex::decode_to_slice(provider_seed_hex, &mut seed).unwrap();
    let provider_key = crypto::signing_key_from_seed_bytes(&seed).unwrap();
    let mut mismatched_payload = receipt.payload.clone();
    mismatched_payload.settlement_refs.success_fee.payment_hash = "ff".repeat(32);
    let mismatched_receipt = protocol::sign_artifact(
        &receipt.signer,
        |msg| crypto::sign_message_hex(&provider_key, msg),
        ARTIFACT_TYPE_RECEIPT,
        receipt.created_at,
        mismatched_payload,
    )
    .unwrap();
    let bad_report =
        protocol::validate_quote_deal_receipt(&quote, &deal, &mismatched_receipt, None);
    assert!(!bad_report.valid);
    assert!(
        bad_report
            .issues
            .iter()
            .any(|issue| issue.code == protocol::chain::ISSUE_INVOICE_SUCCESS_PAYMENT_HASH_MISMATCH),
        "expected invoice_success_payment_hash_mismatch, got {:?}",
        bad_report.issues
    );
}

#[test]
fn operator_ledger_artifact_verification_cases_match_expectations() {
    let fixture = load_fixture();
    let cases = fixture["artifact_verification_cases"].as_array().unwrap();
    assert_eq!(cases.len(), 7, "expected the full accept/reject table");

    for case in cases {
        let name = case["name"].as_str().unwrap();
        let expected_valid = case["expected_valid"].as_bool().unwrap();
        let artifact_type = case["artifact_type"].as_str().unwrap();
        let observed = verify_typed_document(&case["artifact"], artifact_type);
        assert_eq!(
            observed.is_ok(),
            expected_valid,
            "{name}: expected valid={expected_valid}, got {observed:?}"
        );
    }
}
