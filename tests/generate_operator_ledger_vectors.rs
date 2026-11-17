/// Generates the `operator_ledger.v1` conformance vectors and compares them
/// against `conformance/operator_ledger_v1.json`.
///
/// Regenerate with:
/// `cargo test --test generate_operator_ledger_vectors generate_operator_ledger_conformance_vectors -- --ignored --nocapture`
///
/// The default test path is read-only and fails if the checked-in fixture no
/// longer matches the generated vectors. `conformance/kernel_v1.json` is never
/// touched: `operator_ledger.v1` is an additive settlement method and the
/// frozen kernel vectors must keep verifying byte-for-byte.
use froglet::execution::ExecutionRuntime;
use froglet::{
    crypto,
    protocol::{
        self, ARTIFACT_TYPE_DEAL, ARTIFACT_TYPE_DESCRIPTOR, ARTIFACT_TYPE_OFFER,
        ARTIFACT_TYPE_QUOTE, ARTIFACT_TYPE_RECEIPT, DealPayload, DescriptorCapabilities,
        DescriptorPayload, ExecutionLimits, OfferExecutionProfile, OfferPayload,
        OfferPriceSchedule, PAYMENT_METHOD_OPERATOR_LEDGER, QuotePayload, QuoteSettlementTerms,
        ReceiptExecutor, ReceiptLegState, ReceiptPayload, ReceiptSettlementLeg,
        ReceiptSettlementRefs, SETTLEMENT_METHOD_OPERATOR_LEDGER, SignedArtifact,
        TransportEndpoint,
    },
};
use serde::Serialize;
use serde_json::{Value, json};
use std::{fs, path::PathBuf};

const PROVIDER_SEED: [u8; 32] = [0x11; 32];
const REQUESTER_SEED: [u8; 32] = [0x22; 32];

const LEDGER_IDENTITY: &str = "alithea.act.v1";
const RESERVATION_ID: &str = "rsv-019a0000-0000-7000-8000-000000000001";
const SUCCESS_FEE_MSAT: u64 = 5_000;
const CREATED_AT: i64 = 1_700_000_000;

fn provider_key() -> crypto::NodeSigningKey {
    crypto::signing_key_from_seed_bytes(&PROVIDER_SEED).unwrap()
}

fn requester_key() -> crypto::NodeSigningKey {
    crypto::signing_key_from_seed_bytes(&REQUESTER_SEED).unwrap()
}

fn fixture_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("conformance/operator_ledger_v1.json")
}

fn reservation_payment_hash() -> String {
    crypto::sha256_hex(RESERVATION_ID.as_bytes())
}

fn artifact_vector_json<T: Serialize + Clone>(artifact: &SignedArtifact<T>) -> Value {
    let signing_bytes = protocol::canonical_signing_bytes(
        &artifact.schema_version,
        &artifact.artifact_type,
        &artifact.signer,
        artifact.created_at,
        &artifact.payload_hash,
        &artifact.payload,
    )
    .unwrap();

    json!({
        "canonical_signing_bytes_hex": hex::encode(&signing_bytes),
        "payload_hash": protocol::payload_hash(&artifact.payload).unwrap(),
        "artifact_hash": protocol::artifact_hash(artifact).unwrap(),
        "artifact": serde_json::to_value(artifact).unwrap(),
    })
}

fn verification_case_json<T: Serialize + Clone>(
    name: &str,
    artifact_type: &str,
    artifact: &SignedArtifact<T>,
    expected_valid: bool,
) -> Value {
    json!({
        "name": name,
        "artifact_type": artifact_type,
        "artifact": serde_json::to_value(artifact).unwrap(),
        "expected_valid": expected_valid,
    })
}

fn descriptor_payload(provider_id: &str) -> DescriptorPayload {
    DescriptorPayload {
        provider_id: provider_id.to_string(),
        protocol_version: "froglet/v1".to_string(),
        descriptor_seq: 1,
        capabilities: DescriptorCapabilities {
            service_kinds: vec!["biocompass.oci.v1".to_string()],
            execution_runtimes: vec!["container".to_string()],
            max_concurrent_deals: Some(4),
        },
        transport_endpoints: vec![TransportEndpoint {
            transport: "wss".to_string(),
            uri: "wss://hub.biocompass.example/api/v1/nodes/ws".to_string(),
            created_at: Some(CREATED_AT),
            expires_at: None,
            priority: 10,
            features: vec![
                "quote_ws".to_string(),
                "confirm_ws".to_string(),
                "receipt_push".to_string(),
            ],
        }],
        accepted_payment_methods: vec![PAYMENT_METHOD_OPERATOR_LEDGER.to_string()],
        expires_at: None,
        linked_identities: Vec::new(),
    }
}

fn settled_settlement_refs(payment_hash: &str) -> ReceiptSettlementRefs {
    ReceiptSettlementRefs {
        method: SETTLEMENT_METHOD_OPERATOR_LEDGER.to_string(),
        bundle_hash: None,
        destination_identity: LEDGER_IDENTITY.to_string(),
        base_fee: ReceiptSettlementLeg {
            amount_msat: 0,
            invoice_hash: String::new(),
            payment_hash: String::new(),
            state: ReceiptLegState::Canceled,
        },
        success_fee: ReceiptSettlementLeg {
            amount_msat: SUCCESS_FEE_MSAT,
            invoice_hash: String::new(),
            payment_hash: payment_hash.to_string(),
            state: ReceiptLegState::Settled,
        },
    }
}

struct GeneratedOperatorLedgerVectors {
    descriptor: SignedArtifact<DescriptorPayload>,
    offer: SignedArtifact<OfferPayload>,
    quote: SignedArtifact<QuotePayload>,
    deal: SignedArtifact<DealPayload>,
    receipt: SignedArtifact<ReceiptPayload>,
    receipt_canceled_valid: SignedArtifact<ReceiptPayload>,
    receipt_expired_valid: SignedArtifact<ReceiptPayload>,
    receipt_with_bundle_hash: SignedArtifact<ReceiptPayload>,
    receipt_invalid_destination: SignedArtifact<ReceiptPayload>,
    receipt_succeeded_but_canceled: SignedArtifact<ReceiptPayload>,
    receipt_open_leg: SignedArtifact<ReceiptPayload>,
    payment_hash: String,
}

fn generate() -> GeneratedOperatorLedgerVectors {
    let provider_key = provider_key();
    let requester_key = requester_key();
    let provider_id = crypto::public_key_hex(&provider_key);
    let requester_id = crypto::public_key_hex(&requester_key);
    let provider_sign = |message: &[u8]| crypto::sign_message_hex(&provider_key, message);
    let requester_sign = |message: &[u8]| crypto::sign_message_hex(&requester_key, message);
    let payment_hash = reservation_payment_hash();

    let descriptor = protocol::sign_artifact(
        &provider_id,
        provider_sign,
        ARTIFACT_TYPE_DESCRIPTOR,
        CREATED_AT,
        descriptor_payload(&provider_id),
    )
    .unwrap();

    let offer = protocol::sign_artifact(
        &provider_id,
        provider_sign,
        ARTIFACT_TYPE_OFFER,
        CREATED_AT,
        OfferPayload {
            provider_id: provider_id.clone(),
            descriptor_hash: descriptor.hash.clone(),
            offer_id: "standard".to_string(),
            offer_kind: "biocompass.oci.v1".to_string(),
            settlement_method: SETTLEMENT_METHOD_OPERATOR_LEDGER.to_string(),
            quote_ttl_secs: 300,
            execution_profile: OfferExecutionProfile {
                runtime: ExecutionRuntime::Container,
                package_kind: "oci_image".to_string(),
                contract_version: "biocompass.oci.v1".to_string(),
                access_handles: Vec::new(),
                abi_version: "biocompass.oci.v1".to_string(),
                capabilities: Vec::new(),
                max_input_bytes: 1_048_576,
                max_runtime_ms: 120_000,
                max_memory_bytes: 1_073_741_824,
                max_output_bytes: 10_485_760,
                fuel_limit: 0,
            },
            price_schedule: OfferPriceSchedule {
                base_fee_msat: 0,
                success_fee_msat: SUCCESS_FEE_MSAT,
            },
            expires_at: None,
            terms_hash: None,
            confidential_profile_hash: None,
        },
    )
    .unwrap();

    let execution_limits = ExecutionLimits {
        max_input_bytes: 1_048_576,
        max_runtime_ms: 120_000,
        max_memory_bytes: 1_073_741_824,
        max_output_bytes: 10_485_760,
        fuel_limit: 0,
    };

    let quote = protocol::sign_artifact(
        &provider_id,
        provider_sign,
        ARTIFACT_TYPE_QUOTE,
        CREATED_AT,
        QuotePayload {
            provider_id: provider_id.clone(),
            requester_id: requester_id.clone(),
            descriptor_hash: descriptor.hash.clone(),
            offer_hash: offer.hash.clone(),
            expires_at: CREATED_AT + 300,
            workload_kind: "biocompass.oci.v1".to_string(),
            workload_hash: "11".repeat(32),
            confidential_session_hash: None,
            capabilities_granted: Vec::new(),
            extension_refs: Vec::new(),
            quote_use: None,
            settlement_terms: QuoteSettlementTerms {
                method: SETTLEMENT_METHOD_OPERATOR_LEDGER.to_string(),
                destination_identity: LEDGER_IDENTITY.to_string(),
                base_fee_msat: 0,
                success_fee_msat: SUCCESS_FEE_MSAT,
                max_base_invoice_expiry_secs: 0,
                max_success_hold_expiry_secs: 0,
                min_final_cltv_expiry: 0,
            },
            execution_limits: execution_limits.clone(),
        },
    )
    .unwrap();

    let deal = protocol::sign_artifact(
        &requester_id,
        requester_sign,
        ARTIFACT_TYPE_DEAL,
        CREATED_AT,
        DealPayload {
            requester_id: requester_id.clone(),
            provider_id: provider_id.clone(),
            quote_hash: quote.hash.clone(),
            workload_hash: "11".repeat(32),
            confidential_session_hash: None,
            extension_refs: Vec::new(),
            authority_ref: None,
            supersedes_deal_hash: None,
            client_nonce: None,
            success_payment_hash: payment_hash.clone(),
            admission_deadline: CREATED_AT + 60,
            completion_deadline: CREATED_AT + 180,
            acceptance_deadline: CREATED_AT + 240,
        },
    )
    .unwrap();

    let base_receipt = ReceiptPayload {
        provider_id: provider_id.clone(),
        requester_id: requester_id.clone(),
        deal_hash: deal.hash.clone(),
        quote_hash: quote.hash.clone(),
        extension_refs: Vec::new(),
        acceptance_ref: None,
        started_at: Some(CREATED_AT + 1),
        finished_at: CREATED_AT + 5,
        deal_state: "succeeded".to_string(),
        execution_state: "succeeded".to_string(),
        settlement_state: "settled".to_string(),
        result_hash: Some("33".repeat(32)),
        confidential_session_hash: None,
        result_envelope_hash: None,
        result_format: Some("application/json".to_string()),
        executor: ReceiptExecutor {
            runtime: "container".to_string(),
            runtime_version: "podman-5".to_string(),
            execution_mode: None,
            attestation_platform: None,
            measurement: None,
            abi_version: Some("biocompass.oci.v1".to_string()),
            module_hash: Some("44".repeat(32)),
            capabilities_granted: Vec::new(),
        },
        limits_applied: execution_limits,
        settlement_refs: settled_settlement_refs(&payment_hash),
        failure_code: None,
        failure_message: None,
        result_ref: None,
    };

    let sign_receipt = |payload: ReceiptPayload| {
        protocol::sign_artifact(
            &provider_id,
            provider_sign,
            ARTIFACT_TYPE_RECEIPT,
            CREATED_AT + 5,
            payload,
        )
        .unwrap()
    };

    let receipt = sign_receipt(base_receipt.clone());

    let receipt_canceled_valid = sign_receipt(ReceiptPayload {
        deal_state: "failed".to_string(),
        execution_state: "failed".to_string(),
        settlement_state: "canceled".to_string(),
        result_hash: None,
        result_format: None,
        settlement_refs: ReceiptSettlementRefs {
            success_fee: ReceiptSettlementLeg {
                state: ReceiptLegState::Canceled,
                ..settled_settlement_refs(&payment_hash).success_fee
            },
            ..settled_settlement_refs(&payment_hash)
        },
        failure_code: Some("output_limit_exceeded".to_string()),
        failure_message: Some("container exceeded max_output_bytes".to_string()),
        ..base_receipt.clone()
    });

    let receipt_expired_valid = sign_receipt(ReceiptPayload {
        deal_state: "failed".to_string(),
        execution_state: "failed".to_string(),
        settlement_state: "expired".to_string(),
        result_hash: None,
        result_format: None,
        settlement_refs: ReceiptSettlementRefs {
            success_fee: ReceiptSettlementLeg {
                state: ReceiptLegState::Expired,
                ..settled_settlement_refs(&payment_hash).success_fee
            },
            ..settled_settlement_refs(&payment_hash)
        },
        failure_code: Some("runtime_limit_exceeded".to_string()),
        failure_message: Some("container exceeded max_runtime_ms".to_string()),
        ..base_receipt.clone()
    });

    let receipt_with_bundle_hash = sign_receipt(ReceiptPayload {
        settlement_refs: ReceiptSettlementRefs {
            bundle_hash: Some("aa".repeat(32)),
            ..settled_settlement_refs(&payment_hash)
        },
        ..base_receipt.clone()
    });

    let receipt_invalid_destination = sign_receipt(ReceiptPayload {
        settlement_refs: ReceiptSettlementRefs {
            destination_identity: "invalid ledger space".to_string(),
            ..settled_settlement_refs(&payment_hash)
        },
        ..base_receipt.clone()
    });

    let receipt_succeeded_but_canceled = sign_receipt(ReceiptPayload {
        settlement_state: "canceled".to_string(),
        settlement_refs: ReceiptSettlementRefs {
            success_fee: ReceiptSettlementLeg {
                state: ReceiptLegState::Canceled,
                ..settled_settlement_refs(&payment_hash).success_fee
            },
            ..settled_settlement_refs(&payment_hash)
        },
        ..base_receipt.clone()
    });

    let receipt_open_leg = sign_receipt(ReceiptPayload {
        settlement_refs: ReceiptSettlementRefs {
            success_fee: ReceiptSettlementLeg {
                state: ReceiptLegState::Open,
                ..settled_settlement_refs(&payment_hash).success_fee
            },
            ..settled_settlement_refs(&payment_hash)
        },
        ..base_receipt
    });

    GeneratedOperatorLedgerVectors {
        descriptor,
        offer,
        quote,
        deal,
        receipt,
        receipt_canceled_valid,
        receipt_expired_valid,
        receipt_with_bundle_hash,
        receipt_invalid_destination,
        receipt_succeeded_but_canceled,
        receipt_open_leg,
        payment_hash,
    }
}

fn fixture_json(vectors: &GeneratedOperatorLedgerVectors) -> Value {
    json!({
        "fixture_type": "froglet.operator-ledger-settlement-conformance",
        "fixture_version": 1,
        "schema_version": "froglet/v1",
        "settlement_method": SETTLEMENT_METHOD_OPERATOR_LEDGER,
        "keys": {
            "provider_seed_hex": hex::encode(PROVIDER_SEED),
            "requester_seed_hex": hex::encode(REQUESTER_SEED),
        },
        "reservation": {
            "ledger_identity": LEDGER_IDENTITY,
            "reservation_id": RESERVATION_ID,
            "payment_hash": vectors.payment_hash,
            "note": "payment_hash is sha256(reservation_id) and binds Deal.success_payment_hash to Receipt.settlement_refs.success_fee.payment_hash. Settlement finality is an operator-ledger transition keyed to receipt.hash.",
        },
        "artifacts": {
            "descriptor": artifact_vector_json(&vectors.descriptor),
            "offer": artifact_vector_json(&vectors.offer),
            "quote": artifact_vector_json(&vectors.quote),
            "deal": artifact_vector_json(&vectors.deal),
            "receipt": artifact_vector_json(&vectors.receipt),
        },
        "conformance_path": {
            "artifact_order": ["descriptor", "offer", "quote", "deal", "receipt"],
            "description": "operator_ledger.v1 is a closed-loop reservation-backed method without an invoice_bundle transport artifact.",
        },
        "artifact_verification_cases": [
            verification_case_json("operator_ledger_receipt_valid", ARTIFACT_TYPE_RECEIPT, &vectors.receipt, true),
            verification_case_json("operator_ledger_receipt_canceled_valid", ARTIFACT_TYPE_RECEIPT, &vectors.receipt_canceled_valid, true),
            verification_case_json("operator_ledger_receipt_expired_valid", ARTIFACT_TYPE_RECEIPT, &vectors.receipt_expired_valid, true),
            verification_case_json("operator_ledger_receipt_with_bundle_hash", ARTIFACT_TYPE_RECEIPT, &vectors.receipt_with_bundle_hash, false),
            verification_case_json("operator_ledger_receipt_invalid_destination", ARTIFACT_TYPE_RECEIPT, &vectors.receipt_invalid_destination, false),
            verification_case_json("operator_ledger_receipt_succeeded_but_canceled", ARTIFACT_TYPE_RECEIPT, &vectors.receipt_succeeded_but_canceled, false),
            verification_case_json("operator_ledger_receipt_open_leg", ARTIFACT_TYPE_RECEIPT, &vectors.receipt_open_leg, false),
        ],
    })
}

fn render(fixture: &Value) -> String {
    let mut rendered = serde_json::to_string_pretty(fixture).unwrap();
    rendered.push('\n');
    rendered
}

#[test]
#[ignore = "manual fixture update tool"]
fn generate_operator_ledger_conformance_vectors() {
    let vectors = generate();
    let rendered = render(&fixture_json(&vectors));
    fs::write(fixture_path(), &rendered).unwrap();
    println!("wrote {}", fixture_path().display());
}

#[test]
fn checked_in_operator_ledger_fixture_matches_generated_vectors() {
    let vectors = generate();
    let expected = render(&fixture_json(&vectors));
    let actual = fs::read_to_string(fixture_path()).expect(
        "conformance/operator_ledger_v1.json is missing; regenerate with \
         `cargo test --test generate_operator_ledger_vectors generate_operator_ledger_conformance_vectors -- --ignored`",
    );
    assert_eq!(
        actual, expected,
        "conformance/operator_ledger_v1.json is stale; regenerate it with the ignored generator test"
    );
}
