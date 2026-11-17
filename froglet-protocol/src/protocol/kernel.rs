use crate::{ExecutionRuntime, canonical_json, crypto};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

pub const FROGLET_SCHEMA_V1: &str = "froglet/v1";

pub const ARTIFACT_TYPE_DESCRIPTOR: &str = "descriptor";
pub const ARTIFACT_TYPE_OFFER: &str = "offer";
pub const ARTIFACT_TYPE_QUOTE: &str = "quote";
pub const ARTIFACT_TYPE_DEAL: &str = "deal";
pub const ARTIFACT_TYPE_RECEIPT: &str = "receipt";
pub const ARTIFACT_TYPE_CURATED_LIST: &str = "curated_list";
pub const ARTIFACT_TYPE_CONFIDENTIAL_PROFILE: &str = "confidential_profile";
pub const ARTIFACT_TYPE_CONFIDENTIAL_SESSION: &str = "confidential_session";
pub const TRANSPORT_TYPE_INVOICE_BUNDLE: &str = "invoice_bundle";

pub const ARTIFACT_KIND_DESCRIPTOR: &str = ARTIFACT_TYPE_DESCRIPTOR;
pub const ARTIFACT_KIND_OFFER: &str = ARTIFACT_TYPE_OFFER;
pub const ARTIFACT_KIND_QUOTE: &str = ARTIFACT_TYPE_QUOTE;
pub const ARTIFACT_KIND_DEAL: &str = ARTIFACT_TYPE_DEAL;
pub const ARTIFACT_KIND_RECEIPT: &str = ARTIFACT_TYPE_RECEIPT;
pub const ARTIFACT_KIND_CURATED_LIST: &str = ARTIFACT_TYPE_CURATED_LIST;
pub const ARTIFACT_KIND_CONFIDENTIAL_PROFILE: &str = ARTIFACT_TYPE_CONFIDENTIAL_PROFILE;
pub const ARTIFACT_KIND_CONFIDENTIAL_SESSION: &str = ARTIFACT_TYPE_CONFIDENTIAL_SESSION;
pub const TRANSPORT_KIND_INVOICE_BUNDLE: &str = TRANSPORT_TYPE_INVOICE_BUNDLE;

// Payment method identifiers
pub const PAYMENT_METHOD_FREE: &str = "free";
pub const PAYMENT_METHOD_LIGHTNING: &str = "lightning";
pub const PAYMENT_METHOD_X402_USDC: &str = "x402_usdc";
pub const PAYMENT_METHOD_STRIPE_MPP: &str = "stripe_mpp";
pub const PAYMENT_METHOD_OPERATOR_LEDGER: &str = "operator_ledger";

// Settlement method identifiers: the values carried in
// offer.settlement_method, quote.settlement_terms.method, and
// receipt.settlement_refs.method. See docs/KERNEL.md §5 and the
// per-method receipt rules in `validate_receipt_artifact`.
pub const SETTLEMENT_METHOD_NONE: &str = "none";
pub const SETTLEMENT_METHOD_LIGHTNING_ESCROW: &str = "lightning.base_fee_plus_success_fee.v1";
pub const SETTLEMENT_METHOD_STRIPE_MPP: &str = "stripe_mpp.v1";
pub const SETTLEMENT_METHOD_LIGHTNING_PREPAID: &str = "lightning.prepaid.v1";
pub const SETTLEMENT_METHOD_X402_EIP3009: &str = "x402.eip3009.v1";
pub const SETTLEMENT_METHOD_OPERATOR_LEDGER: &str = "operator_ledger.v1";

pub const LINKED_IDENTITY_KIND_NOSTR: &str = "nostr";
pub const LINKED_IDENTITY_SCOPE_PUBLICATION_NOSTR: &str = "publication.nostr";
pub const LINKED_IDENTITY_SIGNATURE_ALGORITHM_BIP340: &str = "secp256k1_schnorr_bip340";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignedArtifact<T> {
    pub artifact_type: String,
    pub schema_version: String,
    pub signer: String,
    pub created_at: i64,
    pub payload_hash: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub hash: String,
    pub payload: T,
    pub signature: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LinkedIdentity {
    pub identity_kind: String,
    pub identity: String,
    pub scope: Vec<String>,
    pub created_at: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expires_at: Option<i64>,
    pub signature_algorithm: String,
    pub linked_signature: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TransportEndpoint {
    pub transport: String,
    pub uri: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub created_at: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expires_at: Option<i64>,
    pub priority: u32,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub features: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TransportEndpoints {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub clearnet_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub onion_url: Option<String>,
    pub tor_status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SettlementDescriptor {
    pub methods: Vec<String>,
    pub reservations: bool,
    pub receipts: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct FeedDescriptor {
    pub pull_api: bool,
    pub cursor_type: String,
    pub cursor_semantics: String,
    pub feed_path: String,
    pub artifact_path_template: String,
    pub max_page_size: usize,
    pub artifact_kinds: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DescriptorCapabilities {
    pub service_kinds: Vec<String>,
    pub execution_runtimes: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_concurrent_deals: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DescriptorPayload {
    pub provider_id: String,
    #[serde(default)]
    pub descriptor_seq: u64,
    pub protocol_version: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expires_at: Option<i64>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub linked_identities: Vec<LinkedIdentity>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub transport_endpoints: Vec<TransportEndpoint>,
    pub capabilities: DescriptorCapabilities,
    // Payment methods accepted by this node. Backward-compatible: old descriptors
    // without this field deserialize with an empty vec.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub accepted_payment_methods: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct OfferExecutionProfile {
    pub runtime: ExecutionRuntime,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub package_kind: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub contract_version: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub access_handles: Vec<String>,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub abi_version: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub capabilities: Vec<String>,
    pub max_input_bytes: usize,
    pub max_runtime_ms: u64,
    pub max_memory_bytes: usize,
    pub max_output_bytes: usize,
    pub fuel_limit: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct OfferConstraints {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_body_bytes: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_query_limit: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timeout_secs: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct OfferPriceSchedule {
    pub base_fee_msat: u64,
    pub success_fee_msat: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct OfferPayload {
    pub provider_id: String,
    pub offer_id: String,
    pub descriptor_hash: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expires_at: Option<i64>,
    pub offer_kind: String,
    pub settlement_method: String,
    pub quote_ttl_secs: u64,
    pub execution_profile: OfferExecutionProfile,
    pub price_schedule: OfferPriceSchedule,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub terms_hash: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub confidential_profile_hash: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct QuoteSettlementTerms {
    pub method: String,
    pub destination_identity: String,
    pub base_fee_msat: u64,
    pub success_fee_msat: u64,
    pub max_base_invoice_expiry_secs: u64,
    pub max_success_hold_expiry_secs: u64,
    pub min_final_cltv_expiry: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ExecutionLimits {
    pub max_input_bytes: usize,
    pub max_runtime_ms: u64,
    pub max_memory_bytes: usize,
    pub max_output_bytes: usize,
    pub fuel_limit: u64,
}

pub type ReceiptLimitsApplied = ExecutionLimits;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuotePayload {
    pub provider_id: String,
    pub requester_id: String,
    pub descriptor_hash: String,
    pub offer_hash: String,
    pub expires_at: i64,
    pub workload_kind: String,
    pub workload_hash: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub confidential_session_hash: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub capabilities_granted: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub extension_refs: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub quote_use: Option<String>,
    pub settlement_terms: QuoteSettlementTerms,
    pub execution_limits: ExecutionLimits,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum InvoiceBundleLegState {
    Open,
    Accepted,
    Settled,
    Canceled,
    Expired,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct InvoiceBundleLeg {
    pub amount_msat: u64,
    pub invoice_bolt11: String,
    pub invoice_hash: String,
    pub payment_hash: String,
    pub state: InvoiceBundleLegState,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct InvoiceBundlePayload {
    pub provider_id: String,
    pub requester_id: String,
    pub quote_hash: String,
    pub deal_hash: String,
    pub expires_at: i64,
    pub destination_identity: String,
    pub base_fee: InvoiceBundleLeg,
    pub success_fee: InvoiceBundleLeg,
    pub min_final_cltv_expiry: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DealPayload {
    pub requester_id: String,
    pub provider_id: String,
    pub quote_hash: String,
    pub workload_hash: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub confidential_session_hash: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub extension_refs: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub authority_ref: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub supersedes_deal_hash: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub client_nonce: Option<String>,
    pub success_payment_hash: String,
    pub admission_deadline: i64,
    pub completion_deadline: i64,
    pub acceptance_deadline: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentLock {
    pub kind: String,
    pub token_hash: String,
    pub amount_sats: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SettlementStatus {
    Reserved,
    Committed,
    Released,
    Expired,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ReceiptLegState {
    Open,
    Accepted,
    Settled,
    Canceled,
    Expired,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ReceiptSettlementLeg {
    pub amount_msat: u64,
    pub invoice_hash: String,
    pub payment_hash: String,
    pub state: ReceiptLegState,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReceiptExecutor {
    pub runtime: String,
    pub runtime_version: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub execution_mode: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attestation_platform: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub measurement: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub abi_version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub module_hash: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub capabilities_granted: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReceiptSettlementRefs {
    pub method: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bundle_hash: Option<String>,
    pub destination_identity: String,
    pub base_fee: ReceiptSettlementLeg,
    pub success_fee: ReceiptSettlementLeg,
}

pub type ReceiptSettlement = ReceiptSettlementRefs;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReceiptFailure {
    pub code: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReceiptPayload {
    pub provider_id: String,
    pub requester_id: String,
    pub deal_hash: String,
    pub quote_hash: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub extension_refs: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub acceptance_ref: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub started_at: Option<i64>,
    pub finished_at: i64,
    pub deal_state: String,
    pub execution_state: String,
    pub settlement_state: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result_hash: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub confidential_session_hash: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result_envelope_hash: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result_format: Option<String>,
    pub executor: ReceiptExecutor,
    pub limits_applied: ExecutionLimits,
    pub settlement_refs: ReceiptSettlementRefs,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub failure_code: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub failure_message: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result_ref: Option<String>,
}

pub fn sign_artifact<T: Serialize + Clone>(
    signer: &str,
    sign_message_hex: impl Fn(&[u8]) -> String,
    artifact_type: &str,
    created_at: i64,
    payload: T,
) -> Result<SignedArtifact<T>, String> {
    let payload_hash = payload_hash(&payload)?;
    let signing_bytes = canonical_signing_bytes(
        FROGLET_SCHEMA_V1,
        artifact_type,
        signer,
        created_at,
        &payload_hash,
        &payload,
    )?;
    let hash = crypto::sha256_hex(&signing_bytes);
    let signature = sign_message_hex(&signing_bytes);

    Ok(SignedArtifact {
        artifact_type: artifact_type.to_string(),
        schema_version: FROGLET_SCHEMA_V1.to_string(),
        signer: signer.to_string(),
        created_at,
        payload_hash,
        hash,
        payload,
        signature,
    })
}

pub fn linked_identity_scope_hash(scope: &[String]) -> Result<String, String> {
    let bytes = canonical_json::to_vec(&scope.to_vec()).map_err(|e| e.to_string())?;
    Ok(crypto::sha256_hex(bytes))
}

pub fn linked_identity_challenge_bytes(
    provider_id: &str,
    identity_kind: &str,
    identity: &str,
    scope: &[String],
    created_at: i64,
    expires_at: Option<i64>,
) -> Result<Vec<u8>, String> {
    let scope_hash = linked_identity_scope_hash(scope)?;
    Ok(format!(
        "froglet:identity_link:v1\n{provider_id}\n{identity_kind}\n{identity}\n{scope_hash}\n{created_at}\n{}",
        expires_at
            .map(|value| value.to_string())
            .unwrap_or_else(|| "-".to_string())
    )
    .into_bytes())
}

pub fn linked_identity_has_scope(identity: &LinkedIdentity, required_scope: &str) -> bool {
    identity.scope.iter().any(|scope| scope == required_scope)
}

fn validate_linked_nostr_identity(
    provider_id: &str,
    identity: &LinkedIdentity,
) -> Result<(), String> {
    if !is_lower_hex_len(&identity.identity, 64) {
        return Err(
            "descriptor linked Nostr identity must be a 32-byte lowercase hex key".to_string(),
        );
    }
    if identity.signature_algorithm != LINKED_IDENTITY_SIGNATURE_ALGORITHM_BIP340 {
        return Err("descriptor linked Nostr identity signature_algorithm is invalid".to_string());
    }
    if identity.scope.is_empty()
        || identity
            .scope
            .iter()
            .any(|scope| !scope.starts_with("publication."))
    {
        return Err(
            "descriptor linked Nostr identity scope must contain only publication scopes"
                .to_string(),
        );
    }
    if identity
        .expires_at
        .is_some_and(|expires_at| expires_at <= identity.created_at)
    {
        return Err(
            "descriptor linked Nostr identity expires_at must be later than created_at".to_string(),
        );
    }
    if !is_lower_hex_len(&identity.linked_signature, 128) {
        return Err(
            "descriptor linked Nostr identity signature must be 64-byte lowercase hex".to_string(),
        );
    }
    let challenge = linked_identity_challenge_bytes(
        provider_id,
        &identity.identity_kind,
        &identity.identity,
        &identity.scope,
        identity.created_at,
        identity.expires_at,
    )?;
    if !crypto::verify_message(&identity.identity, &identity.linked_signature, &challenge) {
        return Err("descriptor linked Nostr identity signature is invalid".to_string());
    }
    Ok(())
}

pub fn verify_artifact<T: Serialize>(artifact: &SignedArtifact<T>) -> bool {
    if artifact.schema_version != FROGLET_SCHEMA_V1 {
        return false;
    }

    let payload_hash = match payload_hash(&artifact.payload) {
        Ok(hash) => hash,
        Err(_) => return false,
    };

    if payload_hash != artifact.payload_hash {
        return false;
    }

    let signing_bytes = match canonical_signing_bytes(
        &artifact.schema_version,
        &artifact.artifact_type,
        &artifact.signer,
        artifact.created_at,
        &artifact.payload_hash,
        &artifact.payload,
    ) {
        Ok(bytes) => bytes,
        Err(_) => return false,
    };

    let computed_hash = crypto::sha256_hex(&signing_bytes);
    if artifact.hash != computed_hash {
        return false;
    }

    crypto::verify_message(&artifact.signer, &artifact.signature, &signing_bytes)
}

fn receipt_leg_is_empty_canceled(leg: &ReceiptSettlementLeg) -> bool {
    leg.amount_msat == 0
        && leg.invoice_hash.is_empty()
        && leg.payment_hash.is_empty()
        && leg.state == ReceiptLegState::Canceled
}

/// Decode a hex string that must represent exactly 32 bytes (a SHA-256 hash or
/// a Lightning preimage).  Returns `None` for invalid hex or wrong length.
fn decode_hash32(value: &str) -> Option<Vec<u8>> {
    hex::decode(value).ok().filter(|bytes| bytes.len() == 32)
}

pub(crate) fn is_lower_hex_len(value: &str, len: usize) -> bool {
    value.len() == len
        && value
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
}

pub(crate) fn is_valid_operator_ledger_identity(value: &str) -> bool {
    (1..=128).contains(&value.len())
        && value.bytes().all(|byte| {
            byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b'-' | b':' | b'/')
        })
}

fn validate_operator_ledger_leg(name: &str, leg: &ReceiptSettlementLeg) -> Result<(), String> {
    if matches!(
        leg.state,
        ReceiptLegState::Open | ReceiptLegState::Accepted
    ) {
        return Err(format!(
            "operator_ledger.v1 receipt {name}.state must be terminal"
        ));
    }
    if leg.amount_msat == 0 {
        if !receipt_leg_is_empty_canceled(leg) {
            return Err(format!(
                "operator_ledger.v1 receipt zero-valued {name} must be a canceled empty placeholder"
            ));
        }
        return Ok(());
    }
    if !is_lower_hex_len(&leg.payment_hash, 64) {
        return Err(format!(
            "operator_ledger.v1 receipt non-zero {name}.payment_hash must be 32-byte lowercase hex"
        ));
    }
    if !leg.invoice_hash.is_empty() && !is_lower_hex_len(&leg.invoice_hash, 64) {
        return Err(format!(
            "operator_ledger.v1 receipt {name}.invoice_hash must be empty or 32-byte lowercase hex"
        ));
    }
    Ok(())
}

pub fn validate_receipt_artifact(receipt: &SignedArtifact<ReceiptPayload>) -> Result<(), String> {
    let payload = &receipt.payload;

    if receipt.signer != payload.provider_id {
        return Err("receipt signer does not match provider_id".to_string());
    }

    if let Some(started_at) = payload.started_at
        && payload.finished_at < started_at
    {
        return Err("receipt finished_at is earlier than started_at".to_string());
    }

    let has_result_hash = payload.result_hash.is_some();
    let has_result_format = payload.result_format.is_some();
    if payload.execution_state == "succeeded" {
        if !has_result_hash || !has_result_format {
            return Err(
                "receipt with execution_state succeeded must include result_hash and result_format"
                    .to_string(),
            );
        }
    } else if has_result_hash || has_result_format {
        return Err(
            "receipt result_hash and result_format must be absent unless execution_state is succeeded"
                .to_string(),
        );
    }

    match payload.deal_state.as_str() {
        "rejected" => {
            if payload.execution_state != "not_started" {
                return Err("rejected receipt must have execution_state not_started".to_string());
            }
        }
        "succeeded" => {
            if payload.execution_state != "succeeded" {
                return Err("successful receipt must have execution_state succeeded".to_string());
            }
        }
        "failed" => {
            if payload.execution_state != "failed" {
                return Err("failed receipt must have execution_state failed".to_string());
            }
        }
        "canceled" => {
            if payload.execution_state != "not_started" && payload.execution_state != "succeeded" {
                return Err(
                    "canceled receipt must have execution_state not_started or succeeded"
                        .to_string(),
                );
            }
        }
        _ => return Err("receipt deal_state is invalid".to_string()),
    }

    match payload.settlement_refs.method.as_str() {
        "lightning.base_fee_plus_success_fee.v1" => {
            if payload
                .settlement_refs
                .bundle_hash
                .as_deref()
                .unwrap_or_default()
                .is_empty()
            {
                return Err("lightning receipt must include bundle_hash".to_string());
            }
            if payload.settlement_refs.destination_identity.is_empty() {
                return Err("lightning receipt must include destination_identity".to_string());
            }
            if matches!(
                payload.settlement_refs.base_fee.state,
                ReceiptLegState::Open | ReceiptLegState::Accepted
            ) || matches!(
                payload.settlement_refs.success_fee.state,
                ReceiptLegState::Open | ReceiptLegState::Accepted
            ) {
                return Err("lightning receipt settlement legs must be terminal".to_string());
            }

            match payload.settlement_state.as_str() {
                "settled" => {
                    if payload.settlement_refs.base_fee.state != ReceiptLegState::Settled {
                        return Err(
                            "lightning receipt settlement_state settled requires base_fee.state settled"
                                .to_string(),
                        );
                    }
                    if payload.settlement_refs.success_fee.state != ReceiptLegState::Settled {
                        return Err(
                            "lightning receipt settlement_state settled requires success_fee.state settled"
                                .to_string(),
                        );
                    }
                }
                "canceled" => {
                    if payload.settlement_refs.success_fee.state != ReceiptLegState::Canceled {
                        return Err(
                            "lightning receipt settlement_state canceled requires success_fee.state canceled"
                                .to_string(),
                        );
                    }
                }
                "expired" => {
                    if payload.settlement_refs.success_fee.state != ReceiptLegState::Expired {
                        return Err(
                            "lightning receipt settlement_state expired requires success_fee.state expired"
                                .to_string(),
                        );
                    }
                }
                _ => {
                    return Err(
                        "lightning receipt settlement_state must be settled, canceled, or expired"
                            .to_string(),
                    );
                }
            }

            if payload.deal_state == "succeeded" && payload.settlement_state != "settled" {
                return Err(
                    "successful lightning receipt must have settlement_state settled".to_string(),
                );
            }
        }
        "none" => {
            if payload.settlement_state != "none" {
                return Err("free receipt settlement_state must be none".to_string());
            }
            if payload.settlement_refs.bundle_hash.is_some() {
                return Err("free receipt must not include bundle_hash".to_string());
            }
            if !payload.settlement_refs.destination_identity.is_empty() {
                return Err("free receipt destination_identity must be empty".to_string());
            }
            if !receipt_leg_is_empty_canceled(&payload.settlement_refs.base_fee)
                || !receipt_leg_is_empty_canceled(&payload.settlement_refs.success_fee)
            {
                return Err(
                    "free receipt settlement legs must be zero-valued canceled placeholders"
                        .to_string(),
                );
            }
        }
        // Interop note: stripe_mpp.v1 is an additive settlement method introduced
        // alongside lightning.base_fee_plus_success_fee.v1.  It uses the same
        // ReceiptSettlementRefs struct shape but without hold invoices: bundle_hash
        // is absent, destination_identity is empty, base_fee carries the PaymentIntent
        // ID as payment_hash, and success_fee is an empty-canceled placeholder.
        "stripe_mpp.v1" => {
            if payload.settlement_refs.bundle_hash.is_some() {
                return Err("stripe_mpp.v1 receipt must not include bundle_hash".to_string());
            }
            if !payload.settlement_refs.destination_identity.is_empty() {
                return Err("stripe_mpp.v1 receipt destination_identity must be empty".to_string());
            }
            if !receipt_leg_is_empty_canceled(&payload.settlement_refs.success_fee) {
                return Err(
                    "stripe_mpp.v1 receipt success_fee must be a zero-valued canceled placeholder"
                        .to_string(),
                );
            }
            // The base_fee leg carries the charge; its state must be terminal.
            if matches!(
                payload.settlement_refs.base_fee.state,
                ReceiptLegState::Open | ReceiptLegState::Accepted
            ) {
                return Err(
                    "stripe_mpp.v1 receipt base_fee.state must be terminal (settled or canceled)"
                        .to_string(),
                );
            }
            match payload.settlement_state.as_str() {
                "settled" => {
                    if payload.settlement_refs.base_fee.state != ReceiptLegState::Settled {
                        return Err(
                            "stripe_mpp.v1 receipt settlement_state settled requires base_fee.state settled"
                                .to_string(),
                        );
                    }
                }
                "canceled" => {
                    if payload.settlement_refs.base_fee.state != ReceiptLegState::Canceled {
                        return Err(
                            "stripe_mpp.v1 receipt settlement_state canceled requires base_fee.state canceled"
                                .to_string(),
                        );
                    }
                }
                _ => {
                    return Err(
                        "stripe_mpp.v1 receipt settlement_state must be settled or canceled"
                            .to_string(),
                    );
                }
            }
            if payload.deal_state == "succeeded" && payload.settlement_state != "settled" {
                return Err(
                    "successful stripe_mpp.v1 receipt must have settlement_state settled"
                        .to_string(),
                );
            }
        }
        // Interop note: lightning.prepaid.v1 is the prepaid, non-escrow Lightning
        // method backed by phoenixd.  Like stripe_mpp.v1 it reuses the
        // ReceiptSettlementRefs shape without hold invoices (no bundle_hash,
        // empty-canceled success_fee). destination_identity carries the signed
        // quote's compressed Lightning payee key. Unlike
        // Stripe, a settled receipt carries a CRYPTOGRAPHIC proof of payment:
        // base_fee.payment_hash is the Lightning payment hash and
        // base_fee.invoice_hash carries the preimage, with
        // sha256(preimage) == payment_hash.
        "lightning.prepaid.v1" => {
            if payload.settlement_refs.bundle_hash.is_some() {
                return Err("lightning.prepaid.v1 receipt must not include bundle_hash".to_string());
            }
            if !is_lower_hex_len(&payload.settlement_refs.destination_identity, 66) {
                return Err(
                    "lightning.prepaid.v1 receipt destination_identity must be compressed secp256k1 lowercase hex"
                        .to_string(),
                );
            }
            if !receipt_leg_is_empty_canceled(&payload.settlement_refs.success_fee) {
                return Err(
                    "lightning.prepaid.v1 receipt success_fee must be a zero-valued canceled placeholder"
                        .to_string(),
                );
            }
            if matches!(
                payload.settlement_refs.base_fee.state,
                ReceiptLegState::Open | ReceiptLegState::Accepted
            ) {
                return Err(
                    "lightning.prepaid.v1 receipt base_fee.state must be terminal (settled or canceled)"
                        .to_string(),
                );
            }
            match payload.settlement_state.as_str() {
                "settled" => {
                    if payload.settlement_refs.base_fee.state != ReceiptLegState::Settled {
                        return Err(
                            "lightning.prepaid.v1 receipt settlement_state settled requires base_fee.state settled"
                                .to_string(),
                        );
                    }
                    // Cryptographic proof of payment: the preimage (carried in
                    // base_fee.invoice_hash) must hash to base_fee.payment_hash.
                    let payment_hash = &payload.settlement_refs.base_fee.payment_hash;
                    let preimage = &payload.settlement_refs.base_fee.invoice_hash;
                    let preimage_bytes = decode_hash32(preimage).ok_or_else(|| {
                        "lightning.prepaid.v1 settled receipt preimage must be 32-byte hex"
                            .to_string()
                    })?;
                    if decode_hash32(payment_hash).is_none() {
                        return Err(
                            "lightning.prepaid.v1 settled receipt payment_hash must be 32-byte hex"
                                .to_string(),
                        );
                    }
                    if crypto::sha256_hex(&preimage_bytes) != payment_hash.to_lowercase() {
                        return Err(
                            "lightning.prepaid.v1 receipt preimage does not match payment_hash"
                                .to_string(),
                        );
                    }
                }
                "canceled" => {
                    if payload.settlement_refs.base_fee.state != ReceiptLegState::Canceled {
                        return Err(
                            "lightning.prepaid.v1 receipt settlement_state canceled requires base_fee.state canceled"
                                .to_string(),
                        );
                    }
                    if !payload.settlement_refs.base_fee.invoice_hash.is_empty() {
                        return Err(
                            "lightning.prepaid.v1 canceled receipt must not carry a preimage"
                                .to_string(),
                        );
                    }
                }
                _ => {
                    return Err(
                        "lightning.prepaid.v1 receipt settlement_state must be settled or canceled"
                            .to_string(),
                    );
                }
            }
            if payload.deal_state == "succeeded" && payload.settlement_state != "settled" {
                return Err(
                    "successful lightning.prepaid.v1 receipt must have settlement_state settled"
                        .to_string(),
                );
            }
        }
        // Interop note: x402.eip3009.v1 is the EVM stablecoin rail settled via
        // an EIP-3009 TransferWithAuthorization (x402). Like lightning.prepaid.v1
        // it reuses the ReceiptSettlementRefs shape without hold invoices, but
        // unlike the other methods destination_identity carries the payee's EVM
        // address (20-byte lowercase hex, no 0x prefix) and bundle_hash is
        // REQUIRED on settlement: it commits to a content-addressed evidence
        // bundle carrying the payer-signed EIP-712 authorization plus the
        // settlement transaction reference. base_fee.payment_hash carries the
        // EIP-3009 authorization nonce (32-byte hex); base_fee.invoice_hash
        // carries the on-chain settle transaction hash (32-byte hex). The
        // payer's EIP-712 signature in the evidence bundle is offline-verifiable
        // (cryptographic); transaction inclusion is attested and checkable on
        // any chain view, not proven by this artifact alone.
        "x402.eip3009.v1" => {
            if !is_lower_hex_len(&payload.settlement_refs.destination_identity, 40) {
                return Err(
                    "x402.eip3009.v1 receipt destination_identity must be a 20-byte lowercase hex EVM address"
                        .to_string(),
                );
            }
            if !receipt_leg_is_empty_canceled(&payload.settlement_refs.success_fee) {
                return Err(
                    "x402.eip3009.v1 receipt success_fee must be a zero-valued canceled placeholder"
                        .to_string(),
                );
            }
            if matches!(
                payload.settlement_refs.base_fee.state,
                ReceiptLegState::Open | ReceiptLegState::Accepted
            ) {
                return Err(
                    "x402.eip3009.v1 receipt base_fee.state must be terminal (settled or canceled)"
                        .to_string(),
                );
            }
            match payload.settlement_state.as_str() {
                "settled" => {
                    if payload.settlement_refs.base_fee.state != ReceiptLegState::Settled {
                        return Err(
                            "x402.eip3009.v1 receipt settlement_state settled requires base_fee.state settled"
                                .to_string(),
                        );
                    }
                    let bundle_hash = payload
                        .settlement_refs
                        .bundle_hash
                        .as_deref()
                        .unwrap_or_default();
                    if !is_lower_hex_len(bundle_hash, 64) {
                        return Err(
                            "x402.eip3009.v1 settled receipt must include a 32-byte lowercase hex bundle_hash"
                                .to_string(),
                        );
                    }
                    if !is_lower_hex_len(&payload.settlement_refs.base_fee.payment_hash, 64) {
                        return Err(
                            "x402.eip3009.v1 settled receipt payment_hash must carry the 32-byte hex EIP-3009 authorization nonce"
                                .to_string(),
                        );
                    }
                    if !is_lower_hex_len(&payload.settlement_refs.base_fee.invoice_hash, 64) {
                        return Err(
                            "x402.eip3009.v1 settled receipt invoice_hash must carry the 32-byte hex settle transaction hash"
                                .to_string(),
                        );
                    }
                }
                "canceled" => {
                    if payload.settlement_refs.base_fee.state != ReceiptLegState::Canceled {
                        return Err(
                            "x402.eip3009.v1 receipt settlement_state canceled requires base_fee.state canceled"
                                .to_string(),
                        );
                    }
                    if payload.settlement_refs.bundle_hash.is_some() {
                        return Err(
                            "x402.eip3009.v1 canceled receipt must not include bundle_hash"
                                .to_string(),
                        );
                    }
                    if !payload.settlement_refs.base_fee.invoice_hash.is_empty() {
                        return Err(
                            "x402.eip3009.v1 canceled receipt must not carry a settle transaction hash"
                                .to_string(),
                        );
                    }
                }
                _ => {
                    return Err(
                        "x402.eip3009.v1 receipt settlement_state must be settled or canceled"
                            .to_string(),
                    );
                }
            }
            if payload.deal_state == "succeeded" && payload.settlement_state != "settled" {
                return Err(
                    "successful x402.eip3009.v1 receipt must have settlement_state settled"
                        .to_string(),
                );
            }
        }
        // Interop note: operator_ledger.v1 is a closed-loop operator-ledger
        // method (for example Alithea ACT minor units) without Lightning hold
        // invoices (`bundle_hash` is absent). `destination_identity` names the
        // closed-loop ledger (e.g. `alithea.act.v1`), `Deal.success_payment_hash`
        // commits to `sha256(reservation_id)` created before execution, and
        // `Receipt.settlement_refs` records terminal two-leg reservation
        // outcomes (`settled`, `canceled`, or `expired`).
        "operator_ledger.v1" => {
            if payload.settlement_refs.bundle_hash.is_some() {
                return Err("operator_ledger.v1 receipt must not include bundle_hash".to_string());
            }
            if !is_valid_operator_ledger_identity(&payload.settlement_refs.destination_identity) {
                return Err(
                    "operator_ledger.v1 receipt destination_identity must be a 1..=128 byte ASCII ledger identifier"
                        .to_string(),
                );
            }
            let base_fee = &payload.settlement_refs.base_fee;
            let success_fee = &payload.settlement_refs.success_fee;
            if base_fee.amount_msat == 0 && success_fee.amount_msat == 0 {
                return Err(
                    "operator_ledger.v1 receipt must carry a non-zero base_fee or success_fee"
                        .to_string(),
                );
            }
            validate_operator_ledger_leg("base_fee", base_fee)?;
            validate_operator_ledger_leg("success_fee", success_fee)?;
            match payload.settlement_state.as_str() {
                "settled" => {
                    if base_fee.amount_msat > 0 && base_fee.state != ReceiptLegState::Settled {
                        return Err(
                            "operator_ledger.v1 receipt settlement_state settled requires non-zero base_fee.state settled"
                                .to_string(),
                        );
                    }
                    if success_fee.amount_msat > 0 {
                        if success_fee.state != ReceiptLegState::Settled {
                            return Err(
                                "operator_ledger.v1 receipt settlement_state settled requires non-zero success_fee.state settled"
                                    .to_string(),
                            );
                        }
                        if payload.deal_state != "succeeded" {
                            return Err(
                                "operator_ledger.v1 receipt with settled success_fee requires deal_state succeeded"
                                    .to_string(),
                            );
                        }
                    }
                }
                "canceled" => {
                    if success_fee.amount_msat > 0 {
                        if success_fee.state != ReceiptLegState::Canceled {
                            return Err(
                                "operator_ledger.v1 receipt settlement_state canceled requires success_fee.state canceled"
                                    .to_string(),
                            );
                        }
                    } else if base_fee.state != ReceiptLegState::Canceled {
                        return Err(
                            "operator_ledger.v1 receipt settlement_state canceled requires base_fee.state canceled"
                                .to_string(),
                        );
                    }
                }
                "expired" => {
                    if success_fee.amount_msat > 0 {
                        if success_fee.state != ReceiptLegState::Expired {
                            return Err(
                                "operator_ledger.v1 receipt settlement_state expired requires success_fee.state expired"
                                    .to_string(),
                            );
                        }
                    } else if base_fee.state != ReceiptLegState::Expired {
                        return Err(
                            "operator_ledger.v1 receipt settlement_state expired requires base_fee.state expired"
                                .to_string(),
                        );
                    }
                }
                _ => {
                    return Err(
                        "operator_ledger.v1 receipt settlement_state must be settled, canceled, or expired"
                            .to_string(),
                    );
                }
            }
            if payload.deal_state == "succeeded" && payload.settlement_state != "settled" {
                return Err(
                    "successful operator_ledger.v1 receipt must have settlement_state settled"
                        .to_string(),
                );
            }
        }
        _ => return Err("receipt settlement_refs.method is invalid".to_string()),
    }

    Ok(())
}

/// Validate a descriptor artifact beyond its cryptographic signature.
///
/// Enforces `signer == payload.provider_id` so a descriptor cannot be signed by
/// one key while claiming to describe a different provider. Mirrors
/// `validate_receipt_artifact`. Always call this after `verify_artifact`; a
/// valid signature alone does not bind the artifact to the claimed provider.
pub fn validate_descriptor_artifact(
    descriptor: &SignedArtifact<DescriptorPayload>,
) -> Result<(), String> {
    if descriptor.signer != descriptor.payload.provider_id {
        return Err("descriptor signer does not match provider_id".to_string());
    }

    if descriptor.payload.protocol_version != FROGLET_SCHEMA_V1 {
        return Err("descriptor protocol_version must be froglet/v1".to_string());
    }

    for identity in &descriptor.payload.linked_identities {
        if identity.identity_kind == LINKED_IDENTITY_KIND_NOSTR {
            validate_linked_nostr_identity(&descriptor.payload.provider_id, identity)?;
        }
    }

    Ok(())
}

/// Validate an offer artifact beyond its cryptographic signature.
///
/// Enforces `signer == payload.provider_id` so an offer cannot be attributed to
/// a provider other than its signer. Also asserts required identifiers are
/// present. Mirrors `validate_receipt_artifact`. Always call this after
/// `verify_artifact`.
pub fn validate_offer_artifact(offer: &SignedArtifact<OfferPayload>) -> Result<(), String> {
    if offer.signer != offer.payload.provider_id {
        return Err("offer signer does not match provider_id".to_string());
    }

    if offer.payload.offer_id.trim().is_empty() {
        return Err("offer offer_id must be non-empty".to_string());
    }

    if offer.payload.descriptor_hash.trim().is_empty() {
        return Err("offer descriptor_hash must be non-empty".to_string());
    }

    let price = &offer.payload.price_schedule;
    let is_free = price.base_fee_msat == 0 && price.success_fee_msat == 0;
    if is_free {
        if offer.payload.settlement_method != SETTLEMENT_METHOD_NONE {
            return Err("free offer settlement_method must be none".to_string());
        }
        return Ok(());
    }

    if !matches!(
        offer.payload.settlement_method.as_str(),
        SETTLEMENT_METHOD_LIGHTNING_ESCROW
            | SETTLEMENT_METHOD_STRIPE_MPP
            | SETTLEMENT_METHOD_LIGHTNING_PREPAID
            | SETTLEMENT_METHOD_X402_EIP3009
            | SETTLEMENT_METHOD_OPERATOR_LEDGER
    ) {
        return Err("paid offer settlement_method is unsupported".to_string());
    }

    if !matches!(
        offer.payload.settlement_method.as_str(),
        SETTLEMENT_METHOD_LIGHTNING_ESCROW | SETTLEMENT_METHOD_OPERATOR_LEDGER
    ) && price.success_fee_msat != 0
    {
        return Err("single-leg paid offer success_fee_msat must be zero".to_string());
    }

    Ok(())
}

/// Validate a quote artifact beyond its cryptographic signature.
///
/// Enforces `signer == payload.provider_id` plus required identifiers and
/// per-method settlement terms. Mirrors `validate_receipt_artifact`. Always
/// call this after `verify_artifact`.
pub fn validate_quote_artifact(quote: &SignedArtifact<QuotePayload>) -> Result<(), String> {
    let payload = &quote.payload;
    if quote.signer != payload.provider_id {
        return Err("quote signer does not match provider_id".to_string());
    }
    if payload.requester_id.trim().is_empty() {
        return Err("quote requester_id must be non-empty".to_string());
    }
    if payload.descriptor_hash.trim().is_empty() {
        return Err("quote descriptor_hash must be non-empty".to_string());
    }
    if payload.offer_hash.trim().is_empty() {
        return Err("quote offer_hash must be non-empty".to_string());
    }
    if payload.workload_kind.trim().is_empty() {
        return Err("quote workload_kind must be non-empty".to_string());
    }
    if payload.workload_hash.trim().is_empty() {
        return Err("quote workload_hash must be non-empty".to_string());
    }
    validate_quote_settlement_terms(&payload.settlement_terms)?;
    Ok(())
}

/// Validate a deal artifact beyond its cryptographic signature.
///
/// The deal is the only requester-signed artifact: enforces
/// `signer == payload.requester_id`, required hashes, and the
/// admission < completion <= acceptance deadline ordering. Always call this
/// after `verify_artifact`.
pub fn validate_deal_artifact(deal: &SignedArtifact<DealPayload>) -> Result<(), String> {
    let payload = &deal.payload;
    if deal.signer != payload.requester_id {
        return Err("deal signer does not match requester_id".to_string());
    }
    if payload.provider_id.trim().is_empty() {
        return Err("deal provider_id must be non-empty".to_string());
    }
    if payload.quote_hash.trim().is_empty() {
        return Err("deal quote_hash must be non-empty".to_string());
    }
    if payload.workload_hash.trim().is_empty() {
        return Err("deal workload_hash must be non-empty".to_string());
    }
    if !is_lower_hex_len(&payload.success_payment_hash, 64) {
        return Err("deal success_payment_hash must be lowercase 32-byte hex".to_string());
    }
    if payload.completion_deadline <= payload.admission_deadline {
        return Err("deal completion_deadline must be greater than admission_deadline".to_string());
    }
    if payload.acceptance_deadline < payload.completion_deadline {
        return Err(
            "deal acceptance_deadline must be greater than or equal to completion_deadline"
                .to_string(),
        );
    }
    Ok(())
}

/// Validate an invoice-bundle transport document beyond its cryptographic
/// signature: signer binding, required links, leg shape, and issuance-state
/// rules. Always call this after `verify_artifact`.
pub fn validate_invoice_bundle_artifact(
    invoice_bundle: &SignedArtifact<InvoiceBundlePayload>,
) -> Result<(), String> {
    let payload = &invoice_bundle.payload;
    if invoice_bundle.signer != payload.provider_id {
        return Err("invoice_bundle signer does not match provider_id".to_string());
    }
    if payload.requester_id.trim().is_empty() {
        return Err("invoice_bundle requester_id must be non-empty".to_string());
    }
    if payload.quote_hash.trim().is_empty() {
        return Err("invoice_bundle quote_hash must be non-empty".to_string());
    }
    if payload.deal_hash.trim().is_empty() {
        return Err("invoice_bundle deal_hash must be non-empty".to_string());
    }
    if !is_lower_hex_len(&payload.destination_identity, 66) {
        return Err(
            "invoice_bundle destination_identity must be compressed secp256k1 lowercase hex"
                .to_string(),
        );
    }
    validate_invoice_leg("base_fee", &payload.base_fee)?;
    validate_invoice_leg("success_fee", &payload.success_fee)?;
    if payload.success_fee.state != InvoiceBundleLegState::Open {
        return Err("invoice_bundle success_fee.state must be open at issuance".to_string());
    }
    if payload.base_fee.state != InvoiceBundleLegState::Open
        && !(payload.base_fee.amount_msat == 0
            && payload.base_fee.state == InvoiceBundleLegState::Settled)
    {
        return Err(
            "invoice_bundle base_fee.state must be open unless zero-valued and settled".to_string(),
        );
    }
    Ok(())
}

fn validate_invoice_leg(name: &str, leg: &InvoiceBundleLeg) -> Result<(), String> {
    if leg.invoice_bolt11.trim().is_empty() {
        return Err(format!(
            "invoice_bundle {name}.invoice_bolt11 must be non-empty"
        ));
    }
    if !is_lower_hex_len(&leg.invoice_hash, 64) {
        return Err(format!(
            "invoice_bundle {name}.invoice_hash must be lowercase 32-byte hex"
        ));
    }
    if !is_lower_hex_len(&leg.payment_hash, 64) {
        return Err(format!(
            "invoice_bundle {name}.payment_hash must be lowercase 32-byte hex"
        ));
    }
    let invoice_hash = crypto::sha256_hex(leg.invoice_bolt11.as_bytes());
    if leg.invoice_hash != invoice_hash {
        return Err(format!(
            "invoice_bundle {name}.invoice_hash must equal SHA256(invoice_bolt11)"
        ));
    }
    Ok(())
}

/// Per-method invariants for quote settlement terms. Shared by
/// `validate_quote_artifact` and manifest/publish-path checks.
pub fn validate_quote_settlement_terms(terms: &QuoteSettlementTerms) -> Result<(), String> {
    match terms.method.as_str() {
        SETTLEMENT_METHOD_NONE => {
            if !terms.destination_identity.is_empty() {
                return Err("free quote destination_identity must be empty".to_string());
            }
            if terms.base_fee_msat != 0 || terms.success_fee_msat != 0 {
                return Err("free quote fee amounts must be zero".to_string());
            }
        }
        SETTLEMENT_METHOD_LIGHTNING_ESCROW => {
            if !is_lower_hex_len(&terms.destination_identity, 66) {
                return Err(
                    "lightning quote destination_identity must be compressed secp256k1 lowercase hex"
                        .to_string(),
                );
            }
        }
        SETTLEMENT_METHOD_STRIPE_MPP => {
            if !terms.destination_identity.is_empty() {
                return Err("non-escrow quote destination_identity must be empty".to_string());
            }
            if terms.success_fee_msat != 0 {
                return Err("non-escrow quote success_fee_msat must be zero".to_string());
            }
        }
        SETTLEMENT_METHOD_LIGHTNING_PREPAID => {
            if !is_lower_hex_len(&terms.destination_identity, 66) {
                return Err(
                    "lightning prepaid quote destination_identity must be compressed secp256k1 lowercase hex"
                        .to_string(),
                );
            }
            if terms.success_fee_msat != 0 {
                return Err("non-escrow quote success_fee_msat must be zero".to_string());
            }
        }
        SETTLEMENT_METHOD_X402_EIP3009 => {
            if !is_lower_hex_len(&terms.destination_identity, 40) {
                return Err(
                    "x402 quote destination_identity must be a 20-byte lowercase hex EVM address"
                        .to_string(),
                );
            }
            if terms.success_fee_msat != 0 {
                return Err("x402 quote success_fee_msat must be zero".to_string());
            }
        }
        SETTLEMENT_METHOD_OPERATOR_LEDGER => {
            if !is_valid_operator_ledger_identity(&terms.destination_identity) {
                return Err(
                    "operator_ledger.v1 quote destination_identity must be a 1..=128 byte ASCII ledger identifier"
                        .to_string(),
                );
            }
            if terms.base_fee_msat == 0 && terms.success_fee_msat == 0 {
                return Err(
                    "operator_ledger.v1 quote must declare a non-zero base_fee_msat or success_fee_msat"
                        .to_string(),
                );
            }
        }
        _ => return Err("quote settlement_terms.method is invalid".to_string()),
    }
    Ok(())
}

/// A signed artifact that has passed both `verify_artifact` (signature +
/// payload-hash integrity) AND the kind-specific semantic validator
/// (`signer == payload.provider_id` plus the per-kind invariants in
/// `docs/KERNEL.md`). Returned by [`verify_typed_document`].
///
/// This is exhaustive over the artifact kinds that have semantic validators:
/// descriptor, offer, quote, deal, invoice_bundle, and receipt — the full
/// evidence chain. New kinds added to the kernel will need a new variant here
/// AND a corresponding `validate_*_artifact` — the typed enum forces that link
/// at compile time, so a consuming service cannot silently fall back to
/// signature-only verification when a new kind appears.
#[derive(Debug)]
pub enum VerifiedArtifact {
    Descriptor(Box<SignedArtifact<DescriptorPayload>>),
    Offer(Box<SignedArtifact<OfferPayload>>),
    Quote(Box<SignedArtifact<QuotePayload>>),
    Deal(Box<SignedArtifact<DealPayload>>),
    InvoiceBundle(Box<SignedArtifact<InvoiceBundlePayload>>),
    /// Boxed: `ReceiptPayload` has many optional fields and lifts the enum's
    /// stack size to ~1 KiB without it. Boxing makes all variants the same
    /// pointer-sized payload (clippy::large_enum_variant).
    Receipt(Box<SignedArtifact<ReceiptPayload>>),
}

/// Errors from [`verify_typed_document`]. Distinct from the per-step
/// `Result<(), String>` returned by the legacy validators: this lets a
/// caller distinguish "I don't support this kind" from "this artifact is
/// malformed" from "this artifact is hostile". Manual enum to avoid pulling
/// `thiserror` into the kernel crate's tight dep set.
#[derive(Debug)]
pub enum VerifyError {
    /// The artifact_kind string is not one this kernel knows how to verify
    /// strongly. Callers should treat this as "stop"; the legacy
    /// `verify_artifact` is still available for kernel-internal use, but
    /// projecting unknown kinds into a downstream store via the typed API
    /// is a deliberate non-goal.
    UnknownKind(String),
    /// The signed envelope failed `verify_artifact` (signature mismatch,
    /// payload_hash mismatch, schema_version mismatch).
    SignatureFailed,
    /// The JSON document could not be deserialised into the typed payload
    /// for `artifact_kind`. Usually means a wire-format mismatch.
    Decode(String),
    /// The typed semantic validator (`validate_*_artifact`) rejected the
    /// artifact. Includes `signer != payload.provider_id` rejections —
    /// the binding check that would have caught the C2 receipt-attribution
    /// bug in marketplace-node before it shipped.
    Semantic(String),
}

impl std::fmt::Display for VerifyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            VerifyError::UnknownKind(kind) => {
                write!(f, "unknown or unsupported artifact_kind: {kind}")
            }
            VerifyError::SignatureFailed => write!(f, "artifact signature verification failed"),
            VerifyError::Decode(msg) => write!(f, "artifact decode failed: {msg}"),
            VerifyError::Semantic(msg) => write!(f, "artifact semantic validation failed: {msg}"),
        }
    }
}

impl std::error::Error for VerifyError {}

fn require_typed_artifact_kind<T>(
    artifact: &SignedArtifact<T>,
    expected: &str,
) -> Result<(), VerifyError> {
    if artifact.artifact_type != expected {
        return Err(VerifyError::Semantic(format!(
            "{expected} artifact_type must be {expected}"
        )));
    }
    Ok(())
}

/// Verify a signed artifact JSON document end to end:
///
/// 1. Parse the document into the typed `SignedArtifact<T>` for `artifact_kind`.
/// 2. Run [`verify_artifact`] (signature + payload-hash integrity).
/// 3. Run the kind-specific semantic validator
///    (`validate_descriptor_artifact` / `validate_offer_artifact` /
///    `validate_receipt_artifact`), which enforces `signer == payload.provider_id`
///    plus the per-kind invariants from `docs/KERNEL.md`.
///
/// This is the **safe-by-default** entry point for consumers who project
/// artifacts into derived stores. The standalone `verify_artifact` and
/// `validate_*_artifact` functions remain available for kernel-internal use
/// (round-tripping conformance vectors etc.), but new code in adapters or
/// service layers should call this so a missed binding check (the C2
/// receipt-attribution bug in marketplace-node) becomes a compile error
/// rather than a silent verification gap.
///
/// `artifact_kind` must match one of the `ARTIFACT_KIND_*` constants (or
/// `TRANSPORT_KIND_INVOICE_BUNDLE`). Kinds not covered by the typed enum
/// (curated_list, confidential_*) return `VerifyError::UnknownKind` —
/// callers must either add a `VerifiedArtifact` variant or handle those
/// kinds explicitly via the legacy API.
pub fn verify_typed_document(
    document: &Value,
    artifact_kind: &str,
) -> Result<VerifiedArtifact, VerifyError> {
    match artifact_kind {
        ARTIFACT_KIND_DESCRIPTOR => {
            let artifact: SignedArtifact<DescriptorPayload> =
                serde_json::from_value(document.clone())
                    .map_err(|error| VerifyError::Decode(error.to_string()))?;
            if !verify_artifact(&artifact) {
                return Err(VerifyError::SignatureFailed);
            }
            require_typed_artifact_kind(&artifact, ARTIFACT_TYPE_DESCRIPTOR)?;
            validate_descriptor_artifact(&artifact).map_err(VerifyError::Semantic)?;
            Ok(VerifiedArtifact::Descriptor(Box::new(artifact)))
        }
        ARTIFACT_KIND_OFFER => {
            let artifact: SignedArtifact<OfferPayload> =
                serde_json::from_value(document.clone())
                    .map_err(|error| VerifyError::Decode(error.to_string()))?;
            if !verify_artifact(&artifact) {
                return Err(VerifyError::SignatureFailed);
            }
            require_typed_artifact_kind(&artifact, ARTIFACT_TYPE_OFFER)?;
            validate_offer_artifact(&artifact).map_err(VerifyError::Semantic)?;
            Ok(VerifiedArtifact::Offer(Box::new(artifact)))
        }
        ARTIFACT_KIND_QUOTE => {
            let artifact: SignedArtifact<QuotePayload> =
                serde_json::from_value(document.clone())
                    .map_err(|error| VerifyError::Decode(error.to_string()))?;
            if !verify_artifact(&artifact) {
                return Err(VerifyError::SignatureFailed);
            }
            require_typed_artifact_kind(&artifact, ARTIFACT_TYPE_QUOTE)?;
            validate_quote_artifact(&artifact).map_err(VerifyError::Semantic)?;
            Ok(VerifiedArtifact::Quote(Box::new(artifact)))
        }
        ARTIFACT_KIND_DEAL => {
            let artifact: SignedArtifact<DealPayload> = serde_json::from_value(document.clone())
                .map_err(|error| VerifyError::Decode(error.to_string()))?;
            if !verify_artifact(&artifact) {
                return Err(VerifyError::SignatureFailed);
            }
            require_typed_artifact_kind(&artifact, ARTIFACT_TYPE_DEAL)?;
            validate_deal_artifact(&artifact).map_err(VerifyError::Semantic)?;
            Ok(VerifiedArtifact::Deal(Box::new(artifact)))
        }
        TRANSPORT_KIND_INVOICE_BUNDLE => {
            let artifact: SignedArtifact<InvoiceBundlePayload> =
                serde_json::from_value(document.clone())
                    .map_err(|error| VerifyError::Decode(error.to_string()))?;
            if !verify_artifact(&artifact) {
                return Err(VerifyError::SignatureFailed);
            }
            require_typed_artifact_kind(&artifact, TRANSPORT_TYPE_INVOICE_BUNDLE)?;
            validate_invoice_bundle_artifact(&artifact).map_err(VerifyError::Semantic)?;
            Ok(VerifiedArtifact::InvoiceBundle(Box::new(artifact)))
        }
        ARTIFACT_KIND_RECEIPT => {
            let artifact: SignedArtifact<ReceiptPayload> = serde_json::from_value(document.clone())
                .map_err(|error| VerifyError::Decode(error.to_string()))?;
            if !verify_artifact(&artifact) {
                return Err(VerifyError::SignatureFailed);
            }
            require_typed_artifact_kind(&artifact, ARTIFACT_TYPE_RECEIPT)?;
            validate_receipt_artifact(&artifact).map_err(VerifyError::Semantic)?;
            Ok(VerifiedArtifact::Receipt(Box::new(artifact)))
        }
        other => Err(VerifyError::UnknownKind(other.to_string())),
    }
}

pub fn artifact_hash<T: Serialize>(artifact: &SignedArtifact<T>) -> Result<String, String> {
    let payload_hash = payload_hash(&artifact.payload)?;
    canonical_signing_bytes(
        &artifact.schema_version,
        &artifact.artifact_type,
        &artifact.signer,
        artifact.created_at,
        &payload_hash,
        &artifact.payload,
    )
    .map(crypto::sha256_hex)
}

pub fn artifact_value<T: Serialize>(artifact: &SignedArtifact<T>) -> Result<Value, String> {
    serde_json::to_value(artifact).map_err(|e| e.to_string())
}

pub fn payload_hash<T: Serialize>(payload: &T) -> Result<String, String> {
    let bytes = canonical_json::to_vec(payload).map_err(|e| e.to_string())?;
    Ok(crypto::sha256_hex(bytes))
}

pub fn canonical_signing_bytes<T: Serialize>(
    schema_version: &str,
    artifact_type: &str,
    signer: &str,
    created_at: i64,
    payload_hash: &str,
    payload: &T,
) -> Result<Vec<u8>, String> {
    canonical_json::to_vec(&json!([
        schema_version,
        artifact_type,
        signer,
        created_at,
        payload_hash,
        payload
    ]))
    .map_err(|e| e.to_string())
}

#[cfg(any(feature = "generate", test))]
pub fn new_artifact_id() -> String {
    use rand::RngCore;
    let mut bytes = [0u8; 16];
    rand::rngs::OsRng.fill_bytes(&mut bytes);
    hex::encode(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crypto;
    use rand::{RngCore, SeedableRng, rngs::StdRng};
    use serde::Serialize;

    fn seeded_signing_key(rng: &mut StdRng) -> crypto::NodeSigningKey {
        loop {
            let mut seed = [0_u8; 32];
            rng.fill_bytes(&mut seed);
            if let Ok(key) = crypto::signing_key_from_seed_bytes(&seed) {
                return key;
            }
        }
    }

    fn random_hex(rng: &mut StdRng, bytes_len: usize) -> String {
        let mut bytes = vec![0_u8; bytes_len];
        rng.fill_bytes(&mut bytes);
        hex::encode(bytes)
    }

    fn flip_hex_char(value: &mut String, index: usize) {
        let replacement = match value.as_bytes()[index] {
            b'0' => "1",
            _ => "0",
        };
        value.replace_range(index..index + 1, replacement);
    }

    #[test]
    fn signed_artifact_roundtrip_verifies() {
        let signing_key = crypto::generate_signing_key();
        let signer = crypto::public_key_hex(&signing_key);
        let artifact = sign_artifact(
            &signer,
            |message| crypto::sign_message_hex(&signing_key, message),
            ARTIFACT_TYPE_QUOTE,
            123,
            QuotePayload {
                provider_id: signer.clone(),
                requester_id: "11".repeat(32),
                descriptor_hash: "22".repeat(32),
                offer_hash: "33".repeat(32),
                expires_at: 456,
                workload_kind: "compute.wasm.v1".to_string(),
                workload_hash: "44".repeat(32),
                confidential_session_hash: None,
                capabilities_granted: Vec::new(),
                extension_refs: Vec::new(),
                quote_use: None,
                settlement_terms: QuoteSettlementTerms {
                    method: "lightning.base_fee_plus_success_fee.v1".to_string(),
                    destination_identity: "02".to_string() + &"55".repeat(32),
                    base_fee_msat: 1000,
                    success_fee_msat: 9000,
                    max_base_invoice_expiry_secs: 30,
                    max_success_hold_expiry_secs: 30,
                    min_final_cltv_expiry: 18,
                },
                execution_limits: ExecutionLimits {
                    max_input_bytes: 1,
                    max_runtime_ms: 2,
                    max_memory_bytes: 3,
                    max_output_bytes: 4,
                    fuel_limit: 5,
                },
            },
        )
        .unwrap();

        assert!(verify_artifact(&artifact));
        assert_eq!(artifact_hash(&artifact).unwrap(), artifact.hash);
    }

    #[test]
    fn signed_artifact_with_empty_hash_is_rejected() {
        let signing_key = crypto::generate_signing_key();
        let signer = crypto::public_key_hex(&signing_key);
        let mut artifact = sign_artifact(
            &signer,
            |message| crypto::sign_message_hex(&signing_key, message),
            ARTIFACT_TYPE_QUOTE,
            123,
            QuotePayload {
                provider_id: signer.clone(),
                requester_id: "11".repeat(32),
                descriptor_hash: "22".repeat(32),
                offer_hash: "33".repeat(32),
                expires_at: 456,
                workload_kind: "compute.wasm.v1".to_string(),
                workload_hash: "44".repeat(32),
                confidential_session_hash: None,
                capabilities_granted: Vec::new(),
                extension_refs: Vec::new(),
                quote_use: None,
                settlement_terms: QuoteSettlementTerms {
                    method: "lightning.base_fee_plus_success_fee.v1".to_string(),
                    destination_identity: "02".to_string() + &"55".repeat(32),
                    base_fee_msat: 1000,
                    success_fee_msat: 9000,
                    max_base_invoice_expiry_secs: 30,
                    max_success_hold_expiry_secs: 30,
                    min_final_cltv_expiry: 18,
                },
                execution_limits: ExecutionLimits {
                    max_input_bytes: 1,
                    max_runtime_ms: 2,
                    max_memory_bytes: 3,
                    max_output_bytes: 4,
                    fuel_limit: 5,
                },
            },
        )
        .unwrap();
        artifact.hash.clear();

        assert!(!verify_artifact(&artifact));
    }

    #[test]
    fn payload_hash_is_stable_across_object_key_order() {
        #[derive(Serialize)]
        struct CanonicalPayload {
            config: Value,
        }

        let first = CanonicalPayload {
            config: json!({
                "b": 2,
                "a": 1
            }),
        };
        let second = CanonicalPayload {
            config: json!({
                "a": 1,
                "b": 2
            }),
        };

        assert_eq!(
            payload_hash(&first).unwrap(),
            payload_hash(&second).unwrap()
        );
    }

    #[test]
    fn linked_identity_challenge_is_stable() {
        let challenge = linked_identity_challenge_bytes(
            "provider-1",
            LINKED_IDENTITY_KIND_NOSTR,
            "abcd",
            &[LINKED_IDENTITY_SCOPE_PUBLICATION_NOSTR.to_string()],
            123,
            None,
        )
        .unwrap();

        assert_eq!(
            String::from_utf8(challenge).unwrap(),
            "froglet:identity_link:v1\nprovider-1\nnostr\nabcd\n32195491daf0054358d4a777b5e185517820f15dd18d1170f4e8351d9de46d68\n123\n-"
        );
    }

    #[test]
    fn randomized_artifact_tampering_breaks_verification() {
        let mut rng = StdRng::seed_from_u64(0x0F06_A1E7);

        for iteration in 0..32_i64 {
            let signing_key = seeded_signing_key(&mut rng);
            let signer = crypto::public_key_hex(&signing_key);
            let artifact = sign_artifact(
                &signer,
                |message| crypto::sign_message_hex(&signing_key, message),
                ARTIFACT_TYPE_QUOTE,
                1_700_000_000 + iteration,
                QuotePayload {
                    provider_id: signer.clone(),
                    requester_id: random_hex(&mut rng, 32),
                    descriptor_hash: random_hex(&mut rng, 32),
                    offer_hash: random_hex(&mut rng, 32),
                    expires_at: 1_700_000_100 + iteration,
                    workload_kind: "compute.wasm.v1".to_string(),
                    workload_hash: random_hex(&mut rng, 32),
                    confidential_session_hash: None,
                    capabilities_granted: Vec::new(),
                    extension_refs: Vec::new(),
                    quote_use: None,
                    settlement_terms: QuoteSettlementTerms {
                        method: "lightning.base_fee_plus_success_fee.v1".to_string(),
                        destination_identity: format!("02{}", random_hex(&mut rng, 32)),
                        base_fee_msat: 1_000 + iteration as u64,
                        success_fee_msat: 9_000 + iteration as u64,
                        max_base_invoice_expiry_secs: 60,
                        max_success_hold_expiry_secs: 120,
                        min_final_cltv_expiry: 18,
                    },
                    execution_limits: ExecutionLimits {
                        max_input_bytes: 1_024 + iteration as usize,
                        max_runtime_ms: 2_000 + iteration as u64,
                        max_memory_bytes: 4_096 + iteration as usize,
                        max_output_bytes: 2_048 + iteration as usize,
                        fuel_limit: 50_000 + iteration as u64,
                    },
                },
            )
            .expect("artifact should sign");

            assert!(
                verify_artifact(&artifact),
                "iteration {iteration} should verify before tampering"
            );

            let mut tampered_schema = artifact.clone();
            tampered_schema.schema_version = "froglet/v2".to_string();
            assert!(
                !verify_artifact(&tampered_schema),
                "iteration {iteration} should fail on schema tampering"
            );

            let mut tampered_type = artifact.clone();
            tampered_type.artifact_type = ARTIFACT_TYPE_DEAL.to_string();
            assert!(
                !verify_artifact(&tampered_type),
                "iteration {iteration} should fail on artifact_type tampering"
            );

            let mut tampered_created_at = artifact.clone();
            tampered_created_at.created_at += 1;
            assert!(
                !verify_artifact(&tampered_created_at),
                "iteration {iteration} should fail on created_at tampering"
            );

            let mut tampered_payload = artifact.clone();
            tampered_payload.payload.workload_hash = random_hex(&mut rng, 32);
            assert!(
                !verify_artifact(&tampered_payload),
                "iteration {iteration} should fail on payload tampering"
            );

            let mut tampered_payload_hash = artifact.clone();
            flip_hex_char(&mut tampered_payload_hash.payload_hash, 0);
            assert!(
                !verify_artifact(&tampered_payload_hash),
                "iteration {iteration} should fail on payload_hash tampering"
            );

            let mut tampered_hash = artifact.clone();
            flip_hex_char(&mut tampered_hash.hash, 0);
            assert!(
                !verify_artifact(&tampered_hash),
                "iteration {iteration} should fail on artifact hash tampering"
            );

            let mut tampered_signer = artifact.clone();
            tampered_signer.signer = random_hex(&mut rng, 32);
            assert!(
                !verify_artifact(&tampered_signer),
                "iteration {iteration} should fail on signer tampering"
            );

            let mut tampered_signature = artifact.clone();
            flip_hex_char(&mut tampered_signature.signature, 0);
            assert!(
                !verify_artifact(&tampered_signature),
                "iteration {iteration} should fail on signature tampering"
            );
        }
    }

    fn valid_free_receipt_payload(provider_id: &str) -> ReceiptPayload {
        ReceiptPayload {
            provider_id: provider_id.to_string(),
            requester_id: "11".repeat(32),
            deal_hash: "22".repeat(32),
            quote_hash: "33".repeat(32),
            extension_refs: Vec::new(),
            acceptance_ref: None,
            started_at: Some(123),
            finished_at: 124,
            deal_state: "succeeded".to_string(),
            execution_state: "succeeded".to_string(),
            settlement_state: "none".to_string(),
            result_hash: Some("44".repeat(32)),
            confidential_session_hash: None,
            result_envelope_hash: None,
            result_format: Some("application/json+jcs".to_string()),
            executor: ReceiptExecutor {
                runtime: "wasm".to_string(),
                runtime_version: "test".to_string(),
                execution_mode: None,
                attestation_platform: None,
                measurement: None,
                abi_version: Some("froglet.wasm.run_json.v1".to_string()),
                module_hash: Some("55".repeat(32)),
                capabilities_granted: Vec::new(),
            },
            limits_applied: ExecutionLimits {
                max_input_bytes: 1,
                max_runtime_ms: 2,
                max_memory_bytes: 3,
                max_output_bytes: 4,
                fuel_limit: 5,
            },
            settlement_refs: ReceiptSettlementRefs {
                method: "none".to_string(),
                bundle_hash: None,
                destination_identity: String::new(),
                base_fee: ReceiptSettlementLeg {
                    amount_msat: 0,
                    invoice_hash: String::new(),
                    payment_hash: String::new(),
                    state: ReceiptLegState::Canceled,
                },
                success_fee: ReceiptSettlementLeg {
                    amount_msat: 0,
                    invoice_hash: String::new(),
                    payment_hash: String::new(),
                    state: ReceiptLegState::Canceled,
                },
            },
            failure_code: None,
            failure_message: None,
            result_ref: None,
        }
    }

    #[test]
    fn signed_receipt_with_valid_free_semantics_passes_validation() {
        let signing_key = crypto::generate_signing_key();
        let signer = crypto::public_key_hex(&signing_key);
        let receipt = sign_artifact(
            &signer,
            |message| crypto::sign_message_hex(&signing_key, message),
            ARTIFACT_TYPE_RECEIPT,
            123,
            valid_free_receipt_payload(&signer),
        )
        .unwrap();

        assert!(verify_artifact(&receipt));
        assert!(validate_receipt_artifact(&receipt).is_ok());
    }

    #[test]
    fn signed_receipt_with_invalid_free_settlement_state_fails_validation() {
        let signing_key = crypto::generate_signing_key();
        let signer = crypto::public_key_hex(&signing_key);
        let mut payload = valid_free_receipt_payload(&signer);
        payload.settlement_state = "settled".to_string();
        let receipt = sign_artifact(
            &signer,
            |message| crypto::sign_message_hex(&signing_key, message),
            ARTIFACT_TYPE_RECEIPT,
            123,
            payload,
        )
        .unwrap();

        assert!(verify_artifact(&receipt));
        assert_eq!(
            validate_receipt_artifact(&receipt).unwrap_err(),
            "free receipt settlement_state must be none"
        );
    }

    #[test]
    fn settled_lightning_receipt_requires_both_legs_settled() {
        let signing_key = crypto::generate_signing_key();
        let signer = crypto::public_key_hex(&signing_key);
        let mut payload = valid_free_receipt_payload(&signer);
        payload.settlement_state = "settled".to_string();
        payload.settlement_refs.method = SETTLEMENT_METHOD_LIGHTNING_ESCROW.to_string();
        payload.settlement_refs.bundle_hash = Some("66".repeat(32));
        payload.settlement_refs.destination_identity = format!("02{}", "77".repeat(32));
        payload.settlement_refs.base_fee = ReceiptSettlementLeg {
            amount_msat: 1_000,
            invoice_hash: "88".repeat(32),
            payment_hash: "99".repeat(32),
            state: ReceiptLegState::Canceled,
        };
        payload.settlement_refs.success_fee = ReceiptSettlementLeg {
            amount_msat: 9_000,
            invoice_hash: "aa".repeat(32),
            payment_hash: "bb".repeat(32),
            state: ReceiptLegState::Settled,
        };
        let receipt = sign_artifact(
            &signer,
            |message| crypto::sign_message_hex(&signing_key, message),
            ARTIFACT_TYPE_RECEIPT,
            123,
            payload,
        )
        .unwrap();

        assert!(verify_artifact(&receipt));
        assert_eq!(
            validate_receipt_artifact(&receipt).unwrap_err(),
            "lightning receipt settlement_state settled requires base_fee.state settled"
        );
    }

    fn valid_descriptor_payload(provider_id: &str) -> DescriptorPayload {
        DescriptorPayload {
            provider_id: provider_id.to_string(),
            descriptor_seq: 1,
            protocol_version: FROGLET_SCHEMA_V1.to_string(),
            expires_at: None,
            linked_identities: Vec::new(),
            transport_endpoints: Vec::new(),
            capabilities: DescriptorCapabilities {
                service_kinds: vec!["compute.wasm.v1".to_string()],
                execution_runtimes: vec!["wasm".to_string()],
                max_concurrent_deals: None,
            },
            accepted_payment_methods: vec![PAYMENT_METHOD_FREE.to_string()],
        }
    }

    fn linked_nostr_identity(
        provider_id: &str,
        publication_key: &crypto::NodeSigningKey,
    ) -> LinkedIdentity {
        let identity = crypto::public_key_hex(publication_key);
        let scope = vec![LINKED_IDENTITY_SCOPE_PUBLICATION_NOSTR.to_string()];
        let challenge = linked_identity_challenge_bytes(
            provider_id,
            LINKED_IDENTITY_KIND_NOSTR,
            &identity,
            &scope,
            122,
            Some(456),
        )
        .expect("linked-identity challenge");
        LinkedIdentity {
            identity_kind: LINKED_IDENTITY_KIND_NOSTR.to_string(),
            identity,
            scope,
            created_at: 122,
            expires_at: Some(456),
            signature_algorithm: LINKED_IDENTITY_SIGNATURE_ALGORITHM_BIP340.to_string(),
            linked_signature: crypto::sign_message_hex(publication_key, &challenge),
        }
    }

    fn valid_offer_payload(provider_id: &str) -> OfferPayload {
        OfferPayload {
            provider_id: provider_id.to_string(),
            offer_id: "offer-1".to_string(),
            descriptor_hash: "aa".repeat(32),
            expires_at: None,
            offer_kind: "named.v1".to_string(),
            settlement_method: "none".to_string(),
            quote_ttl_secs: 60,
            execution_profile: OfferExecutionProfile {
                runtime: ExecutionRuntime::Wasm,
                package_kind: "inline_module".to_string(),
                contract_version: "froglet.wasm.run_json.v1".to_string(),
                access_handles: Vec::new(),
                abi_version: "froglet.wasm.run_json.v1".to_string(),
                capabilities: Vec::new(),
                max_input_bytes: 1,
                max_runtime_ms: 2,
                max_memory_bytes: 3,
                max_output_bytes: 4,
                fuel_limit: 5,
            },
            price_schedule: OfferPriceSchedule {
                base_fee_msat: 0,
                success_fee_msat: 0,
            },
            terms_hash: None,
            confidential_profile_hash: None,
        }
    }

    #[test]
    fn descriptor_with_matching_signer_passes_validation() {
        let signing_key = crypto::generate_signing_key();
        let signer = crypto::public_key_hex(&signing_key);
        let descriptor = sign_artifact(
            &signer,
            |message| crypto::sign_message_hex(&signing_key, message),
            ARTIFACT_TYPE_DESCRIPTOR,
            123,
            valid_descriptor_payload(&signer),
        )
        .unwrap();

        assert!(verify_artifact(&descriptor));
        assert!(validate_descriptor_artifact(&descriptor).is_ok());
    }

    #[test]
    fn descriptor_with_mismatched_signer_fails_validation() {
        // Attacker key signs the descriptor but payload.provider_id claims a
        // different identity. Signature alone verifies; semantic validation
        // must reject.
        let attacker_key = crypto::generate_signing_key();
        let attacker_signer = crypto::public_key_hex(&attacker_key);
        let victim_key = crypto::generate_signing_key();
        let victim_signer = crypto::public_key_hex(&victim_key);
        assert_ne!(attacker_signer, victim_signer);

        let descriptor = sign_artifact(
            &attacker_signer,
            |message| crypto::sign_message_hex(&attacker_key, message),
            ARTIFACT_TYPE_DESCRIPTOR,
            123,
            valid_descriptor_payload(&victim_signer),
        )
        .unwrap();

        assert!(verify_artifact(&descriptor));
        assert_eq!(
            validate_descriptor_artifact(&descriptor).unwrap_err(),
            "descriptor signer does not match provider_id"
        );
    }

    #[test]
    fn descriptor_with_empty_protocol_version_fails_validation() {
        let signing_key = crypto::generate_signing_key();
        let signer = crypto::public_key_hex(&signing_key);
        let mut payload = valid_descriptor_payload(&signer);
        payload.protocol_version = String::new();
        let descriptor = sign_artifact(
            &signer,
            |message| crypto::sign_message_hex(&signing_key, message),
            ARTIFACT_TYPE_DESCRIPTOR,
            123,
            payload,
        )
        .unwrap();

        assert!(verify_artifact(&descriptor));
        assert_eq!(
            validate_descriptor_artifact(&descriptor).unwrap_err(),
            "descriptor protocol_version must be froglet/v1"
        );
    }

    #[test]
    fn descriptor_with_wrong_protocol_version_fails_validation() {
        let signing_key = crypto::generate_signing_key();
        let signer = crypto::public_key_hex(&signing_key);
        let mut payload = valid_descriptor_payload(&signer);
        payload.protocol_version = "froglet/v999".to_string();
        let descriptor = sign_artifact(
            &signer,
            |message| crypto::sign_message_hex(&signing_key, message),
            ARTIFACT_TYPE_DESCRIPTOR,
            123,
            payload,
        )
        .unwrap();

        assert!(verify_artifact(&descriptor));
        assert_eq!(
            validate_descriptor_artifact(&descriptor).unwrap_err(),
            "descriptor protocol_version must be froglet/v1"
        );
    }

    #[test]
    fn descriptor_verifies_linked_nostr_identity_proof() {
        let signing_key = crypto::generate_signing_key();
        let signer = crypto::public_key_hex(&signing_key);
        let publication_key = crypto::generate_signing_key();
        let mut payload = valid_descriptor_payload(&signer);
        payload.linked_identities = vec![linked_nostr_identity(&signer, &publication_key)];
        let descriptor = sign_artifact(
            &signer,
            |message| crypto::sign_message_hex(&signing_key, message),
            ARTIFACT_TYPE_DESCRIPTOR,
            123,
            payload,
        )
        .unwrap();

        assert!(verify_artifact(&descriptor));
        assert!(validate_descriptor_artifact(&descriptor).is_ok());
    }

    #[test]
    fn descriptor_rejects_forged_linked_nostr_identity_proof() {
        let signing_key = crypto::generate_signing_key();
        let signer = crypto::public_key_hex(&signing_key);
        let publication_key = crypto::generate_signing_key();
        let mut linked = linked_nostr_identity(&signer, &publication_key);
        linked.linked_signature = "00".repeat(64);
        let mut payload = valid_descriptor_payload(&signer);
        payload.linked_identities = vec![linked];
        let descriptor = sign_artifact(
            &signer,
            |message| crypto::sign_message_hex(&signing_key, message),
            ARTIFACT_TYPE_DESCRIPTOR,
            123,
            payload,
        )
        .unwrap();

        assert!(verify_artifact(&descriptor));
        assert_eq!(
            validate_descriptor_artifact(&descriptor).unwrap_err(),
            "descriptor linked Nostr identity signature is invalid"
        );
    }

    #[test]
    fn offer_with_matching_signer_passes_validation() {
        let signing_key = crypto::generate_signing_key();
        let signer = crypto::public_key_hex(&signing_key);
        let offer = sign_artifact(
            &signer,
            |message| crypto::sign_message_hex(&signing_key, message),
            ARTIFACT_TYPE_OFFER,
            123,
            valid_offer_payload(&signer),
        )
        .unwrap();

        assert!(verify_artifact(&offer));
        assert!(validate_offer_artifact(&offer).is_ok());
    }

    #[test]
    fn offer_with_mismatched_signer_fails_validation() {
        let attacker_key = crypto::generate_signing_key();
        let attacker_signer = crypto::public_key_hex(&attacker_key);
        let victim_key = crypto::generate_signing_key();
        let victim_signer = crypto::public_key_hex(&victim_key);
        assert_ne!(attacker_signer, victim_signer);

        let offer = sign_artifact(
            &attacker_signer,
            |message| crypto::sign_message_hex(&attacker_key, message),
            ARTIFACT_TYPE_OFFER,
            123,
            valid_offer_payload(&victim_signer),
        )
        .unwrap();

        assert!(verify_artifact(&offer));
        assert_eq!(
            validate_offer_artifact(&offer).unwrap_err(),
            "offer signer does not match provider_id"
        );
    }

    #[test]
    fn offer_with_empty_offer_id_fails_validation() {
        let signing_key = crypto::generate_signing_key();
        let signer = crypto::public_key_hex(&signing_key);
        let mut payload = valid_offer_payload(&signer);
        payload.offer_id = String::new();
        let offer = sign_artifact(
            &signer,
            |message| crypto::sign_message_hex(&signing_key, message),
            ARTIFACT_TYPE_OFFER,
            123,
            payload,
        )
        .unwrap();

        assert!(verify_artifact(&offer));
        assert_eq!(
            validate_offer_artifact(&offer).unwrap_err(),
            "offer offer_id must be non-empty"
        );
    }

    #[test]
    fn offer_with_empty_descriptor_hash_fails_validation() {
        let signing_key = crypto::generate_signing_key();
        let signer = crypto::public_key_hex(&signing_key);
        let mut payload = valid_offer_payload(&signer);
        payload.descriptor_hash = String::new();
        let offer = sign_artifact(
            &signer,
            |message| crypto::sign_message_hex(&signing_key, message),
            ARTIFACT_TYPE_OFFER,
            123,
            payload,
        )
        .unwrap();

        assert!(verify_artifact(&offer));
        assert_eq!(
            validate_offer_artifact(&offer).unwrap_err(),
            "offer descriptor_hash must be non-empty"
        );
    }

    #[test]
    fn offer_rejects_unknown_or_fee_inconsistent_settlement_method() {
        let signing_key = crypto::generate_signing_key();
        let signer = crypto::public_key_hex(&signing_key);

        let mut unknown_payload = valid_offer_payload(&signer);
        unknown_payload.price_schedule.base_fee_msat = 1;
        unknown_payload.settlement_method = "future.rail.v9".to_string();
        let unknown = sign_artifact(
            &signer,
            |message| crypto::sign_message_hex(&signing_key, message),
            ARTIFACT_TYPE_OFFER,
            123,
            unknown_payload,
        )
        .unwrap();
        assert_eq!(
            validate_offer_artifact(&unknown).unwrap_err(),
            "paid offer settlement_method is unsupported"
        );

        let mut free_payload = valid_offer_payload(&signer);
        free_payload.settlement_method = SETTLEMENT_METHOD_STRIPE_MPP.to_string();
        let free = sign_artifact(
            &signer,
            |message| crypto::sign_message_hex(&signing_key, message),
            ARTIFACT_TYPE_OFFER,
            123,
            free_payload,
        )
        .unwrap();
        assert_eq!(
            validate_offer_artifact(&free).unwrap_err(),
            "free offer settlement_method must be none"
        );

        let mut paid_none_payload = valid_offer_payload(&signer);
        paid_none_payload.price_schedule.base_fee_msat = 1;
        let paid_none = sign_artifact(
            &signer,
            |message| crypto::sign_message_hex(&signing_key, message),
            ARTIFACT_TYPE_OFFER,
            123,
            paid_none_payload,
        )
        .unwrap();
        assert_eq!(
            validate_offer_artifact(&paid_none).unwrap_err(),
            "paid offer settlement_method is unsupported"
        );
    }

    #[test]
    fn prepaid_quote_requires_signed_lightning_destination() {
        let mut terms = QuoteSettlementTerms {
            method: SETTLEMENT_METHOD_LIGHTNING_PREPAID.to_string(),
            destination_identity: String::new(),
            base_fee_msat: 30_000,
            success_fee_msat: 0,
            max_base_invoice_expiry_secs: 300,
            max_success_hold_expiry_secs: 0,
            min_final_cltv_expiry: 18,
        };
        assert_eq!(
            validate_quote_settlement_terms(&terms).unwrap_err(),
            "lightning prepaid quote destination_identity must be compressed secp256k1 lowercase hex"
        );

        terms.destination_identity = format!("02{}", "55".repeat(32));
        assert!(validate_quote_settlement_terms(&terms).is_ok());
    }

    #[test]
    fn typed_verifier_rejects_valid_envelope_under_wrong_expected_kind() {
        let signing_key = crypto::generate_signing_key();
        let signer = crypto::public_key_hex(&signing_key);
        let artifact = sign_artifact(
            &signer,
            |message| crypto::sign_message_hex(&signing_key, message),
            ARTIFACT_TYPE_RECEIPT,
            123,
            valid_offer_payload(&signer),
        )
        .unwrap();
        let document = serde_json::to_value(artifact).unwrap();

        match verify_typed_document(&document, ARTIFACT_KIND_OFFER) {
            Err(VerifyError::Semantic(message)) => {
                assert_eq!(message, "offer artifact_type must be offer")
            }
            other => panic!("expected semantic artifact-type rejection, got {other:?}"),
        }
    }

    #[test]
    fn verify_typed_document_accepts_descriptor_with_matching_signer() {
        let signing_key = crypto::generate_signing_key();
        let signer = crypto::public_key_hex(&signing_key);
        let descriptor = sign_artifact(
            &signer,
            |message| crypto::sign_message_hex(&signing_key, message),
            ARTIFACT_TYPE_DESCRIPTOR,
            42,
            valid_descriptor_payload(&signer),
        )
        .unwrap();
        let document = serde_json::to_value(descriptor).unwrap();

        match verify_typed_document(&document, ARTIFACT_KIND_DESCRIPTOR) {
            Ok(VerifiedArtifact::Descriptor(_)) => {}
            Ok(other) => panic!("expected Descriptor variant, got {other:?}"),
            Err(error) => panic!("expected Ok, got {error:?}"),
        }
    }

    #[test]
    fn verify_typed_document_rejects_descriptor_signed_by_attacker_for_victim() {
        // The exact attack class C2 (marketplace-node receipt-binding gap)
        // generalised — an attacker's signature is valid under their own key,
        // but `payload.provider_id` claims a victim. The strong-by-default
        // entry point must reject before the typed artifact is returned.
        let attacker_key = crypto::generate_signing_key();
        let attacker_signer = crypto::public_key_hex(&attacker_key);
        let victim_key = crypto::generate_signing_key();
        let victim_signer = crypto::public_key_hex(&victim_key);
        let artifact = sign_artifact(
            &attacker_signer,
            |message| crypto::sign_message_hex(&attacker_key, message),
            ARTIFACT_TYPE_DESCRIPTOR,
            42,
            valid_descriptor_payload(&victim_signer),
        )
        .unwrap();
        let document = serde_json::to_value(artifact).unwrap();

        match verify_typed_document(&document, ARTIFACT_KIND_DESCRIPTOR) {
            Err(VerifyError::Semantic(message)) => {
                assert!(
                    message.contains("signer does not match provider_id"),
                    "unexpected semantic message: {message}"
                );
            }
            other => panic!("expected Err(Semantic), got {other:?}"),
        }
    }

    #[test]
    fn verify_typed_document_rejects_offer_signed_by_attacker_for_victim() {
        let attacker_key = crypto::generate_signing_key();
        let attacker_signer = crypto::public_key_hex(&attacker_key);
        let victim_key = crypto::generate_signing_key();
        let victim_signer = crypto::public_key_hex(&victim_key);
        let artifact = sign_artifact(
            &attacker_signer,
            |message| crypto::sign_message_hex(&attacker_key, message),
            ARTIFACT_TYPE_OFFER,
            42,
            valid_offer_payload(&victim_signer),
        )
        .unwrap();
        let document = serde_json::to_value(artifact).unwrap();

        match verify_typed_document(&document, ARTIFACT_KIND_OFFER) {
            Err(VerifyError::Semantic(message)) => {
                assert!(
                    message.contains("signer does not match provider_id"),
                    "unexpected semantic message: {message}"
                );
            }
            other => panic!("expected Err(Semantic), got {other:?}"),
        }
    }

    #[test]
    fn verify_typed_document_rejects_tampered_payload_with_signature_failure() {
        let signing_key = crypto::generate_signing_key();
        let signer = crypto::public_key_hex(&signing_key);
        let descriptor = sign_artifact(
            &signer,
            |message| crypto::sign_message_hex(&signing_key, message),
            ARTIFACT_TYPE_DESCRIPTOR,
            42,
            valid_descriptor_payload(&signer),
        )
        .unwrap();
        let mut document = serde_json::to_value(descriptor).unwrap();
        document["payload_hash"] = serde_json::Value::String("00".repeat(32));

        match verify_typed_document(&document, ARTIFACT_KIND_DESCRIPTOR) {
            Err(VerifyError::SignatureFailed) => {}
            other => panic!("expected Err(SignatureFailed), got {other:?}"),
        }
    }

    #[test]
    fn verify_typed_document_rejects_unknown_kind() {
        // curated_list (and confidential_*) deliberately have no typed
        // projection. Calling for an unsupported kind must return UnknownKind
        // so consumers do NOT silently fall back to signature-only
        // verification.
        let document = serde_json::json!({});
        match verify_typed_document(&document, ARTIFACT_KIND_CURATED_LIST) {
            Err(VerifyError::UnknownKind(kind)) => assert_eq!(kind, ARTIFACT_KIND_CURATED_LIST),
            other => panic!("expected Err(UnknownKind), got {other:?}"),
        }
    }

    #[test]
    fn verify_typed_document_rejects_garbage_kind_string() {
        let document = serde_json::json!({});
        match verify_typed_document(&document, "not-a-real-kind") {
            Err(VerifyError::UnknownKind(kind)) => assert_eq!(kind, "not-a-real-kind"),
            other => panic!("expected Err(UnknownKind), got {other:?}"),
        }
    }

    #[test]
    fn verify_typed_document_rejects_malformed_json_with_decode_error() {
        let document = serde_json::json!({ "not": "a signed artifact" });
        match verify_typed_document(&document, ARTIFACT_KIND_DESCRIPTOR) {
            Err(VerifyError::Decode(_)) => {}
            other => panic!("expected Err(Decode), got {other:?}"),
        }
    }

    // ─── Stripe MPP kernel validation tests ───────────────────────────────────

    /// Build a minimal but valid stripe_mpp.v1 receipt payload.
    fn valid_stripe_receipt_payload(provider_id: &str, captured: bool) -> ReceiptPayload {
        let (deal_state, execution_state, settlement_state, base_state, result_hash, result_format) =
            if captured {
                (
                    "succeeded",
                    "succeeded",
                    "settled",
                    ReceiptLegState::Settled,
                    Some("44".repeat(32)),
                    Some("application/json+jcs".to_string()),
                )
            } else {
                (
                    "failed",
                    "failed",
                    "canceled",
                    ReceiptLegState::Canceled,
                    None,
                    None,
                )
            };

        ReceiptPayload {
            provider_id: provider_id.to_string(),
            requester_id: "11".repeat(32),
            deal_hash: "22".repeat(32),
            quote_hash: "33".repeat(32),
            extension_refs: Vec::new(),
            acceptance_ref: None,
            started_at: Some(100),
            finished_at: 200,
            deal_state: deal_state.to_string(),
            execution_state: execution_state.to_string(),
            settlement_state: settlement_state.to_string(),
            result_hash,
            confidential_session_hash: None,
            result_envelope_hash: None,
            result_format,
            executor: ReceiptExecutor {
                runtime: "wasm".to_string(),
                runtime_version: "test".to_string(),
                execution_mode: None,
                attestation_platform: None,
                measurement: None,
                abi_version: Some("froglet.wasm.run_json.v1".to_string()),
                module_hash: Some("55".repeat(32)),
                capabilities_granted: Vec::new(),
            },
            limits_applied: ExecutionLimits {
                max_input_bytes: 1,
                max_runtime_ms: 2,
                max_memory_bytes: 3,
                max_output_bytes: 4,
                fuel_limit: 5,
            },
            settlement_refs: ReceiptSettlementRefs {
                method: "stripe_mpp.v1".to_string(),
                bundle_hash: None,
                destination_identity: String::new(),
                base_fee: ReceiptSettlementLeg {
                    amount_msat: 30_000,
                    invoice_hash: String::new(),
                    payment_hash: "pi_test_stripe_intent_123".to_string(),
                    state: base_state,
                },
                success_fee: ReceiptSettlementLeg {
                    amount_msat: 0,
                    invoice_hash: String::new(),
                    payment_hash: String::new(),
                    state: ReceiptLegState::Canceled,
                },
            },
            failure_code: if captured {
                None
            } else {
                Some("execution_failed".to_string())
            },
            failure_message: if captured {
                None
            } else {
                Some("test failure".to_string())
            },
            result_ref: None,
        }
    }

    #[test]
    fn stripe_mpp_v1_success_receipt_passes_kernel_validation() {
        let signing_key = crypto::generate_signing_key();
        let signer = crypto::public_key_hex(&signing_key);
        let receipt = sign_artifact(
            &signer,
            |message| crypto::sign_message_hex(&signing_key, message),
            ARTIFACT_TYPE_RECEIPT,
            100,
            valid_stripe_receipt_payload(&signer, true),
        )
        .unwrap();

        assert!(verify_artifact(&receipt), "signature must be valid");
        assert!(
            validate_receipt_artifact(&receipt).is_ok(),
            "stripe_mpp.v1 success receipt must pass kernel validation: {:?}",
            validate_receipt_artifact(&receipt)
        );
    }

    #[test]
    fn stripe_mpp_v1_failure_receipt_passes_kernel_validation() {
        let signing_key = crypto::generate_signing_key();
        let signer = crypto::public_key_hex(&signing_key);
        let receipt = sign_artifact(
            &signer,
            |message| crypto::sign_message_hex(&signing_key, message),
            ARTIFACT_TYPE_RECEIPT,
            100,
            valid_stripe_receipt_payload(&signer, false),
        )
        .unwrap();

        assert!(verify_artifact(&receipt), "signature must be valid");
        assert!(
            validate_receipt_artifact(&receipt).is_ok(),
            "stripe_mpp.v1 failure receipt must pass kernel validation: {:?}",
            validate_receipt_artifact(&receipt)
        );
    }

    #[test]
    fn stripe_mpp_v1_receipt_with_bundle_hash_fails_validation() {
        let signing_key = crypto::generate_signing_key();
        let signer = crypto::public_key_hex(&signing_key);
        let mut payload = valid_stripe_receipt_payload(&signer, true);
        payload.settlement_refs.bundle_hash = Some("aa".repeat(32));
        let receipt = sign_artifact(
            &signer,
            |message| crypto::sign_message_hex(&signing_key, message),
            ARTIFACT_TYPE_RECEIPT,
            100,
            payload,
        )
        .unwrap();

        assert!(verify_artifact(&receipt));
        assert_eq!(
            validate_receipt_artifact(&receipt).unwrap_err(),
            "stripe_mpp.v1 receipt must not include bundle_hash"
        );
    }

    #[test]
    fn stripe_mpp_v1_receipt_with_non_empty_destination_identity_fails_validation() {
        let signing_key = crypto::generate_signing_key();
        let signer = crypto::public_key_hex(&signing_key);
        let mut payload = valid_stripe_receipt_payload(&signer, true);
        payload.settlement_refs.destination_identity = "02".to_string() + &"aa".repeat(32);
        let receipt = sign_artifact(
            &signer,
            |message| crypto::sign_message_hex(&signing_key, message),
            ARTIFACT_TYPE_RECEIPT,
            100,
            payload,
        )
        .unwrap();

        assert!(verify_artifact(&receipt));
        assert_eq!(
            validate_receipt_artifact(&receipt).unwrap_err(),
            "stripe_mpp.v1 receipt destination_identity must be empty"
        );
    }

    #[test]
    fn stripe_mpp_v1_receipt_with_wrong_settlement_state_fails_validation() {
        let signing_key = crypto::generate_signing_key();
        let signer = crypto::public_key_hex(&signing_key);
        // A "succeeded" deal must have settlement_state "settled".
        let mut payload = valid_stripe_receipt_payload(&signer, true);
        payload.settlement_state = "canceled".to_string();
        payload.settlement_refs.base_fee.state = ReceiptLegState::Canceled;
        let receipt = sign_artifact(
            &signer,
            |message| crypto::sign_message_hex(&signing_key, message),
            ARTIFACT_TYPE_RECEIPT,
            100,
            payload,
        )
        .unwrap();

        assert!(verify_artifact(&receipt));
        let err = validate_receipt_artifact(&receipt).unwrap_err();
        assert!(
            err.contains("successful stripe_mpp.v1 receipt must have settlement_state settled"),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn stripe_mpp_v1_receipt_with_nonempty_success_fee_fails_validation() {
        let signing_key = crypto::generate_signing_key();
        let signer = crypto::public_key_hex(&signing_key);
        let mut payload = valid_stripe_receipt_payload(&signer, true);
        // A non-empty success_fee leg should be rejected.
        payload.settlement_refs.success_fee.payment_hash = "bb".repeat(32);
        let receipt = sign_artifact(
            &signer,
            |message| crypto::sign_message_hex(&signing_key, message),
            ARTIFACT_TYPE_RECEIPT,
            100,
            payload,
        )
        .unwrap();

        assert!(verify_artifact(&receipt));
        assert_eq!(
            validate_receipt_artifact(&receipt).unwrap_err(),
            "stripe_mpp.v1 receipt success_fee must be a zero-valued canceled placeholder"
        );
    }

    // ─── lightning.prepaid.v1 (phoenixd) ────────────────────────────────────

    /// A known preimage and its SHA-256 payment hash (the cryptographic proof
    /// of payment carried by a settled prepaid receipt).
    fn prepaid_preimage_and_hash() -> (String, String) {
        let preimage = "11".repeat(32);
        let payment_hash = crypto::sha256_hex(hex::decode(&preimage).unwrap());
        (preimage, payment_hash)
    }

    /// Build a minimal valid lightning.prepaid.v1 receipt payload.
    ///
    /// `settled` true → the buyer paid: deal succeeded, settlement settled with
    /// a valid preimage proof.  `settled` false → execution failed AFTER the
    /// buyer prepaid (the MVP no-refund case): deal failed, but settlement is
    /// still "settled" because funds moved, with the preimage proof intact.
    fn valid_prepaid_receipt_payload(provider_id: &str, settled: bool) -> ReceiptPayload {
        let (preimage, payment_hash) = prepaid_preimage_and_hash();
        let (deal_state, execution_state, result_hash, result_format, failure) = if settled {
            (
                "succeeded",
                "succeeded",
                Some("44".repeat(32)),
                Some("application/json+jcs".to_string()),
                None,
            )
        } else {
            ("failed", "failed", None, None, Some("execution_failed"))
        };

        ReceiptPayload {
            provider_id: provider_id.to_string(),
            requester_id: "11".repeat(32),
            deal_hash: "22".repeat(32),
            quote_hash: "33".repeat(32),
            extension_refs: Vec::new(),
            acceptance_ref: None,
            started_at: Some(100),
            finished_at: 200,
            deal_state: deal_state.to_string(),
            execution_state: execution_state.to_string(),
            // Both paths are "settled": the buyer prepaid before execution, so
            // funds moved regardless of execution outcome.
            settlement_state: "settled".to_string(),
            result_hash,
            confidential_session_hash: None,
            result_envelope_hash: None,
            result_format,
            executor: ReceiptExecutor {
                runtime: "wasm".to_string(),
                runtime_version: "test".to_string(),
                execution_mode: None,
                attestation_platform: None,
                measurement: None,
                abi_version: Some("froglet.wasm.run_json.v1".to_string()),
                module_hash: Some("55".repeat(32)),
                capabilities_granted: Vec::new(),
            },
            limits_applied: ExecutionLimits {
                max_input_bytes: 1,
                max_runtime_ms: 2,
                max_memory_bytes: 3,
                max_output_bytes: 4,
                fuel_limit: 5,
            },
            settlement_refs: ReceiptSettlementRefs {
                method: "lightning.prepaid.v1".to_string(),
                bundle_hash: None,
                destination_identity: format!("02{}", "55".repeat(32)),
                base_fee: ReceiptSettlementLeg {
                    amount_msat: 30_000,
                    // invoice_hash carries the preimage (the proof).
                    invoice_hash: preimage,
                    payment_hash,
                    state: ReceiptLegState::Settled,
                },
                success_fee: ReceiptSettlementLeg {
                    amount_msat: 0,
                    invoice_hash: String::new(),
                    payment_hash: String::new(),
                    state: ReceiptLegState::Canceled,
                },
            },
            failure_code: failure.map(str::to_string),
            failure_message: failure.map(|_| "test failure".to_string()),
            result_ref: None,
        }
    }

    /// Build a canceled prepaid receipt (invoice never paid → no funds moved).
    fn canceled_prepaid_receipt_payload(provider_id: &str) -> ReceiptPayload {
        let mut payload = valid_prepaid_receipt_payload(provider_id, false);
        payload.settlement_state = "canceled".to_string();
        payload.settlement_refs.base_fee.state = ReceiptLegState::Canceled;
        payload.settlement_refs.base_fee.invoice_hash = String::new();
        payload.settlement_refs.base_fee.payment_hash = String::new();
        payload.settlement_refs.base_fee.amount_msat = 0;
        payload
    }

    fn sign_prepaid(payload: ReceiptPayload) -> SignedArtifact<ReceiptPayload> {
        let signing_key = crypto::generate_signing_key();
        let signer = crypto::public_key_hex(&signing_key);
        let mut payload = payload;
        payload.provider_id = signer.clone();
        sign_artifact(
            &signer,
            |message| crypto::sign_message_hex(&signing_key, message),
            ARTIFACT_TYPE_RECEIPT,
            100,
            payload,
        )
        .unwrap()
    }

    #[test]
    fn lightning_prepaid_v1_success_receipt_passes_kernel_validation() {
        let receipt = sign_prepaid(valid_prepaid_receipt_payload("placeholder", true));
        assert!(verify_artifact(&receipt), "signature must be valid");
        assert!(
            validate_receipt_artifact(&receipt).is_ok(),
            "prepaid success receipt must pass: {:?}",
            validate_receipt_artifact(&receipt)
        );
    }

    #[test]
    fn lightning_prepaid_v1_failed_but_paid_receipt_passes_kernel_validation() {
        // The MVP no-refund case: execution failed, buyer was charged, the
        // signed receipt is the buyer's cryptographic evidence.
        let receipt = sign_prepaid(valid_prepaid_receipt_payload("placeholder", false));
        assert!(verify_artifact(&receipt));
        assert!(
            validate_receipt_artifact(&receipt).is_ok(),
            "failed-but-paid prepaid receipt must pass: {:?}",
            validate_receipt_artifact(&receipt)
        );
    }

    #[test]
    fn lightning_prepaid_v1_canceled_receipt_passes_kernel_validation() {
        let receipt = sign_prepaid(canceled_prepaid_receipt_payload("placeholder"));
        assert!(verify_artifact(&receipt));
        assert!(
            validate_receipt_artifact(&receipt).is_ok(),
            "canceled prepaid receipt must pass: {:?}",
            validate_receipt_artifact(&receipt)
        );
    }

    #[test]
    fn lightning_prepaid_v1_receipt_with_mismatched_preimage_fails_validation() {
        // The headline cryptographic check: a preimage that does NOT hash to
        // the payment_hash must be rejected.
        let mut payload = valid_prepaid_receipt_payload("placeholder", true);
        payload.settlement_refs.base_fee.invoice_hash = "22".repeat(32); // wrong preimage
        let receipt = sign_prepaid(payload);
        assert!(verify_artifact(&receipt), "signature is still valid");
        assert_eq!(
            validate_receipt_artifact(&receipt).unwrap_err(),
            "lightning.prepaid.v1 receipt preimage does not match payment_hash"
        );
    }

    #[test]
    fn lightning_prepaid_v1_receipt_with_non_hex_preimage_fails_validation() {
        let mut payload = valid_prepaid_receipt_payload("placeholder", true);
        payload.settlement_refs.base_fee.invoice_hash = "not-hex".to_string();
        let receipt = sign_prepaid(payload);
        assert_eq!(
            validate_receipt_artifact(&receipt).unwrap_err(),
            "lightning.prepaid.v1 settled receipt preimage must be 32-byte hex"
        );
    }

    #[test]
    fn lightning_prepaid_v1_receipt_with_bundle_hash_fails_validation() {
        let mut payload = valid_prepaid_receipt_payload("placeholder", true);
        payload.settlement_refs.bundle_hash = Some("aa".repeat(32));
        let receipt = sign_prepaid(payload);
        assert_eq!(
            validate_receipt_artifact(&receipt).unwrap_err(),
            "lightning.prepaid.v1 receipt must not include bundle_hash"
        );
    }

    #[test]
    fn lightning_prepaid_v1_receipt_with_empty_destination_fails_validation() {
        let mut payload = valid_prepaid_receipt_payload("placeholder", true);
        payload.settlement_refs.destination_identity.clear();
        let receipt = sign_prepaid(payload);
        assert_eq!(
            validate_receipt_artifact(&receipt).unwrap_err(),
            "lightning.prepaid.v1 receipt destination_identity must be compressed secp256k1 lowercase hex"
        );
    }

    #[test]
    fn lightning_prepaid_v1_receipt_with_nonempty_success_fee_fails_validation() {
        let mut payload = valid_prepaid_receipt_payload("placeholder", true);
        payload.settlement_refs.success_fee.payment_hash = "bb".repeat(32);
        let receipt = sign_prepaid(payload);
        assert_eq!(
            validate_receipt_artifact(&receipt).unwrap_err(),
            "lightning.prepaid.v1 receipt success_fee must be a zero-valued canceled placeholder"
        );
    }

    #[test]
    fn lightning_prepaid_v1_succeeded_with_canceled_settlement_fails_validation() {
        // deal succeeded but settlement canceled is contradictory.
        let mut payload = valid_prepaid_receipt_payload("placeholder", true);
        payload.settlement_state = "canceled".to_string();
        payload.settlement_refs.base_fee.state = ReceiptLegState::Canceled;
        payload.settlement_refs.base_fee.invoice_hash = String::new();
        let receipt = sign_prepaid(payload);
        assert_eq!(
            validate_receipt_artifact(&receipt).unwrap_err(),
            "successful lightning.prepaid.v1 receipt must have settlement_state settled"
        );
    }

    fn sign_operator_ledger(mut payload: ReceiptPayload) -> SignedArtifact<ReceiptPayload> {
        let signing_key = crypto::generate_signing_key();
        let signer = crypto::public_key_hex(&signing_key);
        payload.provider_id = signer.clone();
        sign_artifact(
            &signer,
            |message| crypto::sign_message_hex(&signing_key, message),
            ARTIFACT_TYPE_RECEIPT,
            100,
            payload,
        )
        .unwrap()
    }

    fn valid_operator_ledger_receipt_payload(
        deal_state: &str,
        execution_state: &str,
        settlement_state: &str,
        success_state: ReceiptLegState,
    ) -> ReceiptPayload {
        let succeeded = execution_state == "succeeded";
        ReceiptPayload {
            provider_id: "placeholder".to_string(),
            requester_id: "466d7fcae563e5cb09a0d1870bb580344804617879a14949cf22285f1bae3f27"
                .to_string(),
            deal_hash: "aa".repeat(32),
            quote_hash: "bb".repeat(32),
            extension_refs: Vec::new(),
            acceptance_ref: None,
            started_at: Some(90),
            finished_at: 100,
            deal_state: deal_state.to_string(),
            execution_state: execution_state.to_string(),
            settlement_state: settlement_state.to_string(),
            result_hash: succeeded.then(|| "cc".repeat(32)),
            confidential_session_hash: None,
            result_envelope_hash: None,
            result_format: succeeded.then(|| "application/json".to_string()),
            executor: ReceiptExecutor {
                runtime: "container".to_string(),
                runtime_version: "podman-5".to_string(),
                execution_mode: None,
                attestation_platform: None,
                measurement: None,
                abi_version: Some("biocompass.oci.v1".to_string()),
                module_hash: Some("dd".repeat(32)),
                capabilities_granted: Vec::new(),
            },
            limits_applied: ExecutionLimits {
                max_input_bytes: 1_048_576,
                max_runtime_ms: 60_000,
                max_memory_bytes: 536_870_912,
                max_output_bytes: 10_485_760,
                fuel_limit: 0,
            },
            settlement_refs: ReceiptSettlementRefs {
                method: SETTLEMENT_METHOD_OPERATOR_LEDGER.to_string(),
                bundle_hash: None,
                destination_identity: "alithea.act.v1".to_string(),
                base_fee: ReceiptSettlementLeg {
                    amount_msat: 0,
                    invoice_hash: String::new(),
                    payment_hash: String::new(),
                    state: ReceiptLegState::Canceled,
                },
                success_fee: ReceiptSettlementLeg {
                    amount_msat: 5_000,
                    invoice_hash: String::new(),
                    payment_hash: "ee".repeat(32),
                    state: success_state,
                },
            },
            failure_code: (!succeeded).then(|| "runtime_limit_exceeded".to_string()),
            failure_message: (!succeeded).then(|| "container exceeded max_runtime_ms".to_string()),
            result_ref: None,
        }
    }

    #[test]
    fn operator_ledger_v1_settled_canceled_and_expired_receipts_validate() {
        let settled = sign_operator_ledger(valid_operator_ledger_receipt_payload(
            "succeeded",
            "succeeded",
            "settled",
            ReceiptLegState::Settled,
        ));
        assert!(verify_artifact(&settled));
        assert!(validate_receipt_artifact(&settled).is_ok());

        let canceled = sign_operator_ledger(valid_operator_ledger_receipt_payload(
            "failed",
            "failed",
            "canceled",
            ReceiptLegState::Canceled,
        ));
        assert!(verify_artifact(&canceled));
        assert!(validate_receipt_artifact(&canceled).is_ok());

        let expired = sign_operator_ledger(valid_operator_ledger_receipt_payload(
            "failed",
            "failed",
            "expired",
            ReceiptLegState::Expired,
        ));
        assert!(verify_artifact(&expired));
        assert!(validate_receipt_artifact(&expired).is_ok());
    }

    #[test]
    fn operator_ledger_v1_rejects_bundle_hash_bad_destination_and_open_legs() {
        let mut with_bundle = valid_operator_ledger_receipt_payload(
            "succeeded",
            "succeeded",
            "settled",
            ReceiptLegState::Settled,
        );
        with_bundle.settlement_refs.bundle_hash = Some("ff".repeat(32));
        assert_eq!(
            validate_receipt_artifact(&sign_operator_ledger(with_bundle)).unwrap_err(),
            "operator_ledger.v1 receipt must not include bundle_hash"
        );

        let mut bad_dest = valid_operator_ledger_receipt_payload(
            "succeeded",
            "succeeded",
            "settled",
            ReceiptLegState::Settled,
        );
        bad_dest.settlement_refs.destination_identity = "invalid space".to_string();
        assert_eq!(
            validate_receipt_artifact(&sign_operator_ledger(bad_dest)).unwrap_err(),
            "operator_ledger.v1 receipt destination_identity must be a 1..=128 byte ASCII ledger identifier"
        );

        let open_leg = valid_operator_ledger_receipt_payload(
            "succeeded",
            "succeeded",
            "settled",
            ReceiptLegState::Open,
        );
        assert_eq!(
            validate_receipt_artifact(&sign_operator_ledger(open_leg)).unwrap_err(),
            "operator_ledger.v1 receipt success_fee.state must be terminal"
        );
    }
}
