use serde::{Deserialize, Serialize};

use super::kernel::{
    ARTIFACT_TYPE_DEAL, ARTIFACT_TYPE_DESCRIPTOR, ARTIFACT_TYPE_OFFER, ARTIFACT_TYPE_QUOTE,
    ARTIFACT_TYPE_RECEIPT, DealPayload, DescriptorPayload, ExecutionLimits, InvoiceBundleLeg,
    InvoiceBundlePayload, OfferPayload, QuotePayload, ReceiptLegState, ReceiptPayload,
    SETTLEMENT_METHOD_LIGHTNING_ESCROW, SETTLEMENT_METHOD_LIGHTNING_PREPAID,
    SETTLEMENT_METHOD_NONE, SETTLEMENT_METHOD_OPERATOR_LEDGER, SETTLEMENT_METHOD_STRIPE_MPP,
    SETTLEMENT_METHOD_X402_EIP3009, SignedArtifact, TRANSPORT_TYPE_INVOICE_BUNDLE,
    validate_deal_artifact, validate_descriptor_artifact, validate_invoice_bundle_artifact,
    validate_offer_artifact, validate_quote_artifact, validate_receipt_artifact, verify_artifact,
};
use crate::crypto;

pub const ISSUE_ARTIFACT_TYPE_MISMATCH: &str = "artifact_type_mismatch";
pub const ISSUE_ARTIFACT_ENVELOPE_INVALID: &str = "artifact_envelope_invalid";
pub const ISSUE_ARTIFACT_SEMANTIC_INVALID: &str = "artifact_semantic_invalid";
pub const ISSUE_ARTIFACT_EXPIRED: &str = "artifact_expired";
pub const ISSUE_PROVIDER_MISMATCH: &str = "provider_mismatch";
pub const ISSUE_REQUESTER_MISMATCH: &str = "requester_mismatch";
pub const ISSUE_DESCRIPTOR_HASH_MISMATCH: &str = "descriptor_hash_mismatch";
pub const ISSUE_OFFER_HASH_MISMATCH: &str = "offer_hash_mismatch";
pub const ISSUE_QUOTE_HASH_MISMATCH: &str = "quote_hash_mismatch";
pub const ISSUE_DEAL_HASH_MISMATCH: &str = "deal_hash_mismatch";
pub const ISSUE_WORKLOAD_KIND_MISMATCH: &str = "workload_kind_mismatch";
pub const ISSUE_WORKLOAD_HASH_MISMATCH: &str = "workload_hash_mismatch";
pub const ISSUE_CONFIDENTIAL_SESSION_HASH_MISMATCH: &str = "confidential_session_hash_mismatch";
pub const ISSUE_QUOTE_EXPIRY_EXCEEDS_OFFER: &str = "quote_expiry_exceeds_offer";
pub const ISSUE_SETTLEMENT_METHOD_MISMATCH: &str = "settlement_method_mismatch";
pub const ISSUE_SETTLEMENT_TERMS_MISMATCH: &str = "settlement_terms_mismatch";
pub const ISSUE_EXECUTION_LIMITS_EXCEED_OFFER: &str = "execution_limits_exceed_offer";
pub const ISSUE_DEADLINE_ORDER_INVALID: &str = "deadline_order_invalid";
pub const ISSUE_DEADLINE_EXCEEDS_QUOTE: &str = "deadline_exceeds_quote";
pub const ISSUE_INVOICE_BUNDLE_FOR_NON_LIGHTNING: &str = "invoice_bundle_for_non_lightning_method";
pub const ISSUE_INVOICE_AMOUNT_MISMATCH: &str = "invoice_amount_mismatch";
pub const ISSUE_INVOICE_DESTINATION_MISMATCH: &str = "invoice_destination_mismatch";
pub const ISSUE_INVOICE_SUCCESS_PAYMENT_HASH_MISMATCH: &str =
    "invoice_success_payment_hash_mismatch";
pub const ISSUE_INVOICE_MIN_CLTV_MISMATCH: &str = "invoice_min_cltv_mismatch";
pub const ISSUE_INVOICE_HASH_MISMATCH: &str = "invoice_hash_mismatch";
pub const ISSUE_INVOICE_EXPIRY_EXCEEDS_DEAL: &str = "invoice_expiry_exceeds_deal";
pub const ISSUE_INVOICE_BUNDLE_REQUIRED_FOR_LIGHTNING: &str =
    "invoice_bundle_required_for_lightning_method";
pub const ISSUE_RECEIPT_BUNDLE_HASH_MISMATCH: &str = "receipt_bundle_hash_mismatch";
pub const ISSUE_INVOICE_PAYMENT_HASH_MISMATCH: &str = "invoice_payment_hash_mismatch";
pub const ISSUE_INVOICE_STATE_MISMATCH: &str = "invoice_state_mismatch";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ChainPath {
    DescriptorOffer,
    OfferQuote,
    QuoteDeal,
    QuoteInvoiceBundleDeal,
    QuoteDealReceipt,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChainValidationIssue {
    pub code: String,
    pub artifact_type: String,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChainValidationReport {
    pub path: ChainPath,
    pub valid: bool,
    pub issues: Vec<ChainValidationIssue>,
    /// Non-fatal findings (for example unresolved evidence refs). Warnings
    /// never affect `valid`; absent-when-empty keeps the serialized shape of
    /// pre-warning reports byte-identical.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub warnings: Vec<ChainValidationIssue>,
}

impl ChainValidationReport {
    fn new(path: ChainPath) -> Self {
        Self {
            path,
            valid: true,
            issues: Vec::new(),
            warnings: Vec::new(),
        }
    }

    fn push_issue(
        &mut self,
        code: &'static str,
        artifact_type: &'static str,
        message: impl Into<String>,
    ) {
        self.valid = false;
        self.issues.push(ChainValidationIssue {
            code: code.to_string(),
            artifact_type: artifact_type.to_string(),
            message: message.into(),
        });
    }
}

pub fn validate_descriptor_offer(
    descriptor: &SignedArtifact<DescriptorPayload>,
    offer: &SignedArtifact<OfferPayload>,
    now: Option<i64>,
) -> ChainValidationReport {
    let mut report = ChainValidationReport::new(ChainPath::DescriptorOffer);
    verify_descriptor_envelope(&mut report, descriptor);
    verify_offer_envelope(&mut report, offer);
    validate_descriptor_semantics(&mut report, descriptor, now);
    validate_offer_semantics(&mut report, offer, now);

    if offer.payload.provider_id != descriptor.payload.provider_id {
        report.push_issue(
            ISSUE_PROVIDER_MISMATCH,
            ARTIFACT_TYPE_OFFER,
            "offer provider_id must match descriptor provider_id",
        );
    }
    if offer.payload.descriptor_hash != descriptor.hash {
        report.push_issue(
            ISSUE_DESCRIPTOR_HASH_MISMATCH,
            ARTIFACT_TYPE_OFFER,
            "offer descriptor_hash must match descriptor hash",
        );
    }

    report
}

pub fn validate_offer_quote(
    offer: &SignedArtifact<OfferPayload>,
    quote: &SignedArtifact<QuotePayload>,
    now: Option<i64>,
) -> ChainValidationReport {
    let mut report = ChainValidationReport::new(ChainPath::OfferQuote);
    verify_offer_envelope(&mut report, offer);
    verify_quote_envelope(&mut report, quote);
    validate_offer_semantics(&mut report, offer, now);
    validate_quote_semantics(&mut report, quote, now);
    validate_offer_quote_links(&mut report, offer, quote);
    report
}

pub fn validate_quote_deal(
    quote: &SignedArtifact<QuotePayload>,
    deal: &SignedArtifact<DealPayload>,
    now: Option<i64>,
) -> ChainValidationReport {
    let mut report = ChainValidationReport::new(ChainPath::QuoteDeal);
    verify_quote_envelope(&mut report, quote);
    verify_deal_envelope(&mut report, deal);
    validate_quote_semantics(&mut report, quote, now);
    validate_deal_semantics(&mut report, deal);
    validate_quote_deal_links(&mut report, quote, deal);
    report
}

pub fn validate_quote_invoice_bundle_deal(
    quote: &SignedArtifact<QuotePayload>,
    invoice_bundle: &SignedArtifact<InvoiceBundlePayload>,
    deal: &SignedArtifact<DealPayload>,
    now: Option<i64>,
) -> ChainValidationReport {
    let mut report = ChainValidationReport::new(ChainPath::QuoteInvoiceBundleDeal);
    verify_quote_envelope(&mut report, quote);
    verify_invoice_bundle_envelope(&mut report, invoice_bundle);
    verify_deal_envelope(&mut report, deal);
    validate_quote_semantics(&mut report, quote, now);
    validate_invoice_bundle_semantics(&mut report, invoice_bundle, now);
    validate_deal_semantics(&mut report, deal);
    validate_quote_deal_links(&mut report, quote, deal);
    validate_invoice_bundle_links(&mut report, quote, invoice_bundle, deal);
    report
}

pub fn validate_quote_deal_receipt(
    quote: &SignedArtifact<QuotePayload>,
    deal: &SignedArtifact<DealPayload>,
    receipt: &SignedArtifact<ReceiptPayload>,
    now: Option<i64>,
) -> ChainValidationReport {
    let mut report = ChainValidationReport::new(ChainPath::QuoteDealReceipt);
    verify_quote_envelope(&mut report, quote);
    verify_deal_envelope(&mut report, deal);
    verify_receipt_envelope(&mut report, receipt);
    validate_quote_semantics(&mut report, quote, now);
    validate_deal_semantics(&mut report, deal);
    validate_receipt_semantics(&mut report, receipt);
    validate_quote_deal_links(&mut report, quote, deal);
    validate_receipt_links(&mut report, quote, deal, receipt);
    report
}

/// A complete artifact chain for one deal, ready for full validation.
///
/// `invoice_bundle` is present only on the Lightning escrow method;
/// `receipt` is `None` for chains captured before execution finished.
#[derive(Debug, Clone, Copy)]
pub struct FullChain<'a> {
    pub descriptor: &'a SignedArtifact<DescriptorPayload>,
    pub offer: &'a SignedArtifact<OfferPayload>,
    pub quote: &'a SignedArtifact<QuotePayload>,
    pub invoice_bundle: Option<&'a SignedArtifact<InvoiceBundlePayload>>,
    pub deal: &'a SignedArtifact<DealPayload>,
    pub receipt: Option<&'a SignedArtifact<ReceiptPayload>>,
}

/// Aggregation of the pairwise chain validations for a [`FullChain`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FullChainReport {
    pub valid: bool,
    pub reports: Vec<ChainValidationReport>,
}

/// Validate a full chain by composing the pairwise validators:
/// descriptor→offer, offer→quote, quote→(invoice_bundle→)deal, and
/// quote→deal→receipt when a receipt is present. Full-chain validation also
/// enforces settlement-method topology and binds Lightning receipt settlement
/// references to the signed invoice bundle and deal. `valid` is the
/// conjunction of the per-path reports.
pub fn validate_full_chain(chain: &FullChain<'_>, now: Option<i64>) -> FullChainReport {
    let mut reports = vec![
        validate_descriptor_offer(chain.descriptor, chain.offer, now),
        validate_offer_quote(chain.offer, chain.quote, now),
    ];
    let uses_lightning_bundle =
        chain.quote.payload.settlement_terms.method == SETTLEMENT_METHOD_LIGHTNING_ESCROW;
    match (uses_lightning_bundle, chain.invoice_bundle) {
        (_, Some(invoice_bundle)) => reports.push(validate_quote_invoice_bundle_deal(
            chain.quote,
            invoice_bundle,
            chain.deal,
            now,
        )),
        (true, None) => {
            let mut report = validate_quote_deal(chain.quote, chain.deal, now);
            report.push_issue(
                ISSUE_INVOICE_BUNDLE_REQUIRED_FOR_LIGHTNING,
                TRANSPORT_TYPE_INVOICE_BUNDLE,
                "lightning.base_fee_plus_success_fee.v1 chains require an invoice_bundle",
            );
            reports.push(report);
        }
        (false, None) => reports.push(validate_quote_deal(chain.quote, chain.deal, now)),
    }
    if let Some(receipt) = chain.receipt {
        let mut report = validate_quote_deal_receipt(chain.quote, chain.deal, receipt, now);
        if uses_lightning_bundle && let Some(invoice_bundle) = chain.invoice_bundle {
            validate_receipt_invoice_bundle_links(&mut report, invoice_bundle, chain.deal, receipt);
        }
        reports.push(report);
    }
    let valid = reports.iter().all(|report| report.valid);
    FullChainReport { valid, reports }
}

fn verify_descriptor_envelope(
    report: &mut ChainValidationReport,
    descriptor: &SignedArtifact<DescriptorPayload>,
) {
    verify_common_artifact(report, descriptor, ARTIFACT_TYPE_DESCRIPTOR);
}

fn validate_descriptor_semantics(
    report: &mut ChainValidationReport,
    descriptor: &SignedArtifact<DescriptorPayload>,
    now: Option<i64>,
) {
    if let Err(message) = validate_descriptor_artifact(descriptor) {
        report.push_issue(
            ISSUE_ARTIFACT_SEMANTIC_INVALID,
            ARTIFACT_TYPE_DESCRIPTOR,
            message,
        );
    }
    if let (Some(now), Some(expires_at)) = (now, descriptor.payload.expires_at)
        && expires_at < now
    {
        report.push_issue(
            ISSUE_ARTIFACT_EXPIRED,
            ARTIFACT_TYPE_DESCRIPTOR,
            "descriptor expires_at is earlier than now",
        );
    }
}

fn verify_offer_envelope(report: &mut ChainValidationReport, offer: &SignedArtifact<OfferPayload>) {
    verify_common_artifact(report, offer, ARTIFACT_TYPE_OFFER);
}

fn validate_offer_semantics(
    report: &mut ChainValidationReport,
    offer: &SignedArtifact<OfferPayload>,
    now: Option<i64>,
) {
    if let Err(message) = validate_offer_artifact(offer) {
        report.push_issue(
            ISSUE_ARTIFACT_SEMANTIC_INVALID,
            ARTIFACT_TYPE_OFFER,
            message,
        );
    }
    validate_offer_settlement(report, offer);
    if let (Some(now), Some(expires_at)) = (now, offer.payload.expires_at)
        && expires_at < now
    {
        report.push_issue(
            ISSUE_ARTIFACT_EXPIRED,
            ARTIFACT_TYPE_OFFER,
            "offer expires_at is earlier than now",
        );
    }
}

fn verify_quote_envelope(report: &mut ChainValidationReport, quote: &SignedArtifact<QuotePayload>) {
    verify_common_artifact(report, quote, ARTIFACT_TYPE_QUOTE);
}

fn validate_quote_semantics(
    report: &mut ChainValidationReport,
    quote: &SignedArtifact<QuotePayload>,
    now: Option<i64>,
) {
    if let Err(message) = validate_quote_artifact(quote) {
        report.push_issue(
            ISSUE_ARTIFACT_SEMANTIC_INVALID,
            ARTIFACT_TYPE_QUOTE,
            message,
        );
    }
    if let Some(now) = now
        && quote.payload.expires_at < now
    {
        report.push_issue(
            ISSUE_ARTIFACT_EXPIRED,
            ARTIFACT_TYPE_QUOTE,
            "quote expires_at is earlier than now",
        );
    }
}

fn verify_deal_envelope(report: &mut ChainValidationReport, deal: &SignedArtifact<DealPayload>) {
    verify_common_artifact(report, deal, ARTIFACT_TYPE_DEAL);
}

fn validate_deal_semantics(report: &mut ChainValidationReport, deal: &SignedArtifact<DealPayload>) {
    if let Err(message) = validate_deal_artifact(deal) {
        report.push_issue(ISSUE_ARTIFACT_SEMANTIC_INVALID, ARTIFACT_TYPE_DEAL, message);
    }
}

fn verify_invoice_bundle_envelope(
    report: &mut ChainValidationReport,
    invoice_bundle: &SignedArtifact<InvoiceBundlePayload>,
) {
    verify_common_artifact(report, invoice_bundle, TRANSPORT_TYPE_INVOICE_BUNDLE);
}

fn validate_invoice_bundle_semantics(
    report: &mut ChainValidationReport,
    invoice_bundle: &SignedArtifact<InvoiceBundlePayload>,
    now: Option<i64>,
) {
    if let Err(message) = validate_invoice_bundle_artifact(invoice_bundle) {
        report.push_issue(
            ISSUE_ARTIFACT_SEMANTIC_INVALID,
            TRANSPORT_TYPE_INVOICE_BUNDLE,
            message,
        );
    }
    if let Some(now) = now
        && invoice_bundle.payload.expires_at < now
    {
        report.push_issue(
            ISSUE_ARTIFACT_EXPIRED,
            TRANSPORT_TYPE_INVOICE_BUNDLE,
            "invoice_bundle expires_at is earlier than now",
        );
    }
}

fn verify_receipt_envelope(
    report: &mut ChainValidationReport,
    receipt: &SignedArtifact<ReceiptPayload>,
) {
    verify_common_artifact(report, receipt, ARTIFACT_TYPE_RECEIPT);
}

fn validate_receipt_semantics(
    report: &mut ChainValidationReport,
    receipt: &SignedArtifact<ReceiptPayload>,
) {
    if let Err(message) = validate_receipt_artifact(receipt) {
        report.push_issue(
            ISSUE_ARTIFACT_SEMANTIC_INVALID,
            ARTIFACT_TYPE_RECEIPT,
            message,
        );
    }
}

fn verify_common_artifact<T: Serialize>(
    report: &mut ChainValidationReport,
    artifact: &SignedArtifact<T>,
    expected_artifact_type: &'static str,
) {
    if artifact.artifact_type != expected_artifact_type {
        report.push_issue(
            ISSUE_ARTIFACT_TYPE_MISMATCH,
            expected_artifact_type,
            format!(
                "expected artifact_type {expected_artifact_type}, got {}",
                artifact.artifact_type
            ),
        );
    }
    if !verify_artifact(artifact) {
        report.push_issue(
            ISSUE_ARTIFACT_ENVELOPE_INVALID,
            expected_artifact_type,
            "artifact envelope hash, payload hash, schema version, or signature is invalid",
        );
    }
}

fn validate_offer_settlement(
    report: &mut ChainValidationReport,
    offer: &SignedArtifact<OfferPayload>,
) {
    let fees_are_zero = offer.payload.price_schedule.base_fee_msat == 0
        && offer.payload.price_schedule.success_fee_msat == 0;
    if fees_are_zero && offer.payload.settlement_method != SETTLEMENT_METHOD_NONE {
        report.push_issue(
            ISSUE_ARTIFACT_SEMANTIC_INVALID,
            ARTIFACT_TYPE_OFFER,
            "zero-fee offers must use settlement_method none",
        );
    } else if !fees_are_zero && !is_known_paid_settlement_method(&offer.payload.settlement_method) {
        report.push_issue(
            ISSUE_ARTIFACT_SEMANTIC_INVALID,
            ARTIFACT_TYPE_OFFER,
            "paid offers must use a known paid settlement method",
        );
    }
}

fn validate_offer_quote_links(
    report: &mut ChainValidationReport,
    offer: &SignedArtifact<OfferPayload>,
    quote: &SignedArtifact<QuotePayload>,
) {
    if quote.payload.provider_id != offer.payload.provider_id {
        report.push_issue(
            ISSUE_PROVIDER_MISMATCH,
            ARTIFACT_TYPE_QUOTE,
            "quote provider_id must match offer provider_id",
        );
    }
    if quote.payload.descriptor_hash != offer.payload.descriptor_hash {
        report.push_issue(
            ISSUE_DESCRIPTOR_HASH_MISMATCH,
            ARTIFACT_TYPE_QUOTE,
            "quote descriptor_hash must match offer descriptor_hash",
        );
    }
    if quote.payload.offer_hash != offer.hash {
        report.push_issue(
            ISSUE_OFFER_HASH_MISMATCH,
            ARTIFACT_TYPE_QUOTE,
            "quote offer_hash must match offer hash",
        );
    }
    if let Some(offer_expires_at) = offer.payload.expires_at
        && quote.payload.expires_at > offer_expires_at
    {
        report.push_issue(
            ISSUE_QUOTE_EXPIRY_EXCEEDS_OFFER,
            ARTIFACT_TYPE_QUOTE,
            "quote expires_at must not exceed offer expires_at",
        );
    }
    if quote.payload.workload_kind != offer.payload.offer_kind {
        report.push_issue(
            ISSUE_WORKLOAD_KIND_MISMATCH,
            ARTIFACT_TYPE_QUOTE,
            "quote workload_kind must match offer offer_kind",
        );
    }
    if quote.payload.settlement_terms.method != offer.payload.settlement_method {
        report.push_issue(
            ISSUE_SETTLEMENT_METHOD_MISMATCH,
            ARTIFACT_TYPE_QUOTE,
            "quote settlement_terms.method must match offer settlement_method",
        );
    }
    if quote.payload.settlement_terms.base_fee_msat != offer.payload.price_schedule.base_fee_msat
        || quote.payload.settlement_terms.success_fee_msat
            != offer.payload.price_schedule.success_fee_msat
    {
        report.push_issue(
            ISSUE_SETTLEMENT_TERMS_MISMATCH,
            ARTIFACT_TYPE_QUOTE,
            "quote settlement fee amounts must match offer price_schedule",
        );
    }
    if !limits_within_offer(&quote.payload.execution_limits, offer) {
        report.push_issue(
            ISSUE_EXECUTION_LIMITS_EXCEED_OFFER,
            ARTIFACT_TYPE_QUOTE,
            "quote execution_limits must not exceed offer execution_profile maxima",
        );
    }
}

fn validate_quote_deal_links(
    report: &mut ChainValidationReport,
    quote: &SignedArtifact<QuotePayload>,
    deal: &SignedArtifact<DealPayload>,
) {
    if deal.payload.provider_id != quote.payload.provider_id {
        report.push_issue(
            ISSUE_PROVIDER_MISMATCH,
            ARTIFACT_TYPE_DEAL,
            "deal provider_id must match quote provider_id",
        );
    }
    if deal.payload.requester_id != quote.payload.requester_id {
        report.push_issue(
            ISSUE_REQUESTER_MISMATCH,
            ARTIFACT_TYPE_DEAL,
            "deal requester_id must match quote requester_id",
        );
    }
    if deal.payload.quote_hash != quote.hash {
        report.push_issue(
            ISSUE_QUOTE_HASH_MISMATCH,
            ARTIFACT_TYPE_DEAL,
            "deal quote_hash must match quote hash",
        );
    }
    if deal.payload.workload_hash != quote.payload.workload_hash {
        report.push_issue(
            ISSUE_WORKLOAD_HASH_MISMATCH,
            ARTIFACT_TYPE_DEAL,
            "deal workload_hash must match quote workload_hash",
        );
    }
    if deal.payload.confidential_session_hash != quote.payload.confidential_session_hash {
        report.push_issue(
            ISSUE_CONFIDENTIAL_SESSION_HASH_MISMATCH,
            ARTIFACT_TYPE_DEAL,
            "deal confidential_session_hash must match quote confidential_session_hash",
        );
    }
    if deal.payload.admission_deadline > quote.payload.expires_at {
        report.push_issue(
            ISSUE_DEADLINE_EXCEEDS_QUOTE,
            ARTIFACT_TYPE_DEAL,
            "deal admission_deadline must not exceed quote expires_at",
        );
    }
    if deal.payload.completion_deadline <= deal.payload.admission_deadline
        || deal.payload.acceptance_deadline < deal.payload.completion_deadline
    {
        report.push_issue(
            ISSUE_DEADLINE_ORDER_INVALID,
            ARTIFACT_TYPE_DEAL,
            "deal deadlines must satisfy admission < completion <= acceptance",
        );
    }
}

fn validate_invoice_bundle_links(
    report: &mut ChainValidationReport,
    quote: &SignedArtifact<QuotePayload>,
    invoice_bundle: &SignedArtifact<InvoiceBundlePayload>,
    deal: &SignedArtifact<DealPayload>,
) {
    if quote.payload.settlement_terms.method != SETTLEMENT_METHOD_LIGHTNING_ESCROW {
        report.push_issue(
            ISSUE_INVOICE_BUNDLE_FOR_NON_LIGHTNING,
            TRANSPORT_TYPE_INVOICE_BUNDLE,
            "invoice_bundle is only valid for lightning.base_fee_plus_success_fee.v1 quotes",
        );
    }
    if invoice_bundle.payload.provider_id != quote.payload.provider_id
        || invoice_bundle.payload.provider_id != deal.payload.provider_id
    {
        report.push_issue(
            ISSUE_PROVIDER_MISMATCH,
            TRANSPORT_TYPE_INVOICE_BUNDLE,
            "invoice_bundle provider_id must match quote and deal provider_id",
        );
    }
    if invoice_bundle.payload.requester_id != quote.payload.requester_id
        || invoice_bundle.payload.requester_id != deal.payload.requester_id
    {
        report.push_issue(
            ISSUE_REQUESTER_MISMATCH,
            TRANSPORT_TYPE_INVOICE_BUNDLE,
            "invoice_bundle requester_id must match quote and deal requester_id",
        );
    }
    if invoice_bundle.payload.quote_hash != quote.hash {
        report.push_issue(
            ISSUE_QUOTE_HASH_MISMATCH,
            TRANSPORT_TYPE_INVOICE_BUNDLE,
            "invoice_bundle quote_hash must match quote hash",
        );
    }
    if invoice_bundle.payload.deal_hash != deal.hash {
        report.push_issue(
            ISSUE_DEAL_HASH_MISMATCH,
            TRANSPORT_TYPE_INVOICE_BUNDLE,
            "invoice_bundle deal_hash must match deal hash",
        );
    }
    if invoice_bundle.payload.destination_identity
        != quote.payload.settlement_terms.destination_identity
    {
        report.push_issue(
            ISSUE_INVOICE_DESTINATION_MISMATCH,
            TRANSPORT_TYPE_INVOICE_BUNDLE,
            "invoice_bundle destination_identity must match quote settlement terms",
        );
    }
    if invoice_bundle.payload.base_fee.amount_msat != quote.payload.settlement_terms.base_fee_msat
        || invoice_bundle.payload.success_fee.amount_msat
            != quote.payload.settlement_terms.success_fee_msat
    {
        report.push_issue(
            ISSUE_INVOICE_AMOUNT_MISMATCH,
            TRANSPORT_TYPE_INVOICE_BUNDLE,
            "invoice_bundle fee amounts must match quote settlement terms",
        );
    }
    if invoice_bundle.payload.success_fee.payment_hash != deal.payload.success_payment_hash {
        report.push_issue(
            ISSUE_INVOICE_SUCCESS_PAYMENT_HASH_MISMATCH,
            TRANSPORT_TYPE_INVOICE_BUNDLE,
            "invoice_bundle success_fee.payment_hash must match deal success_payment_hash",
        );
    }
    if invoice_bundle.payload.min_final_cltv_expiry
        != quote.payload.settlement_terms.min_final_cltv_expiry
    {
        report.push_issue(
            ISSUE_INVOICE_MIN_CLTV_MISMATCH,
            TRANSPORT_TYPE_INVOICE_BUNDLE,
            "invoice_bundle min_final_cltv_expiry must match quote settlement terms",
        );
    }
    if invoice_bundle.payload.expires_at > quote.payload.expires_at {
        report.push_issue(
            ISSUE_DEADLINE_EXCEEDS_QUOTE,
            TRANSPORT_TYPE_INVOICE_BUNDLE,
            "invoice_bundle expires_at must not exceed quote expires_at",
        );
    }
    if invoice_bundle.payload.expires_at > deal.payload.admission_deadline {
        report.push_issue(
            ISSUE_INVOICE_EXPIRY_EXCEEDS_DEAL,
            TRANSPORT_TYPE_INVOICE_BUNDLE,
            "invoice_bundle expires_at must not exceed deal admission_deadline",
        );
    }
    validate_invoice_leg_hash_link(report, &invoice_bundle.payload.base_fee);
    validate_invoice_leg_hash_link(report, &invoice_bundle.payload.success_fee);
}

fn validate_receipt_links(
    report: &mut ChainValidationReport,
    quote: &SignedArtifact<QuotePayload>,
    deal: &SignedArtifact<DealPayload>,
    receipt: &SignedArtifact<ReceiptPayload>,
) {
    if receipt.payload.provider_id != quote.payload.provider_id
        || receipt.payload.provider_id != deal.payload.provider_id
    {
        report.push_issue(
            ISSUE_PROVIDER_MISMATCH,
            ARTIFACT_TYPE_RECEIPT,
            "receipt provider_id must match quote and deal provider_id",
        );
    }
    if receipt.payload.requester_id != quote.payload.requester_id
        || receipt.payload.requester_id != deal.payload.requester_id
    {
        report.push_issue(
            ISSUE_REQUESTER_MISMATCH,
            ARTIFACT_TYPE_RECEIPT,
            "receipt requester_id must match quote and deal requester_id",
        );
    }
    if receipt.payload.quote_hash != quote.hash
        || receipt.payload.quote_hash != deal.payload.quote_hash
    {
        report.push_issue(
            ISSUE_QUOTE_HASH_MISMATCH,
            ARTIFACT_TYPE_RECEIPT,
            "receipt quote_hash must match quote hash and deal quote_hash",
        );
    }
    if receipt.payload.deal_hash != deal.hash {
        report.push_issue(
            ISSUE_DEAL_HASH_MISMATCH,
            ARTIFACT_TYPE_RECEIPT,
            "receipt deal_hash must match deal hash",
        );
    }
    if receipt.payload.confidential_session_hash != quote.payload.confidential_session_hash
        || receipt.payload.confidential_session_hash != deal.payload.confidential_session_hash
    {
        report.push_issue(
            ISSUE_CONFIDENTIAL_SESSION_HASH_MISMATCH,
            ARTIFACT_TYPE_RECEIPT,
            "receipt confidential_session_hash must match quote and deal confidential_session_hash",
        );
    }
    if receipt.payload.settlement_refs.method != quote.payload.settlement_terms.method {
        report.push_issue(
            ISSUE_SETTLEMENT_METHOD_MISMATCH,
            ARTIFACT_TYPE_RECEIPT,
            "receipt settlement_refs.method must match quote settlement terms",
        );
    }
    if receipt.payload.settlement_refs.base_fee.amount_msat
        != quote.payload.settlement_terms.base_fee_msat
        || receipt.payload.settlement_refs.success_fee.amount_msat
            != quote.payload.settlement_terms.success_fee_msat
    {
        report.push_issue(
            ISSUE_SETTLEMENT_TERMS_MISMATCH,
            ARTIFACT_TYPE_RECEIPT,
            "receipt settlement fee amounts must match quote settlement terms",
        );
    }
    if receipt.payload.settlement_refs.destination_identity
        != quote.payload.settlement_terms.destination_identity
    {
        report.push_issue(
            ISSUE_INVOICE_DESTINATION_MISMATCH,
            ARTIFACT_TYPE_RECEIPT,
            "receipt destination_identity must match quote settlement terms",
        );
    }
    if quote.payload.settlement_terms.method == SETTLEMENT_METHOD_OPERATOR_LEDGER
        && quote.payload.settlement_terms.success_fee_msat > 0
        && receipt.payload.settlement_refs.success_fee.payment_hash
            != deal.payload.success_payment_hash
    {
        report.push_issue(
            ISSUE_INVOICE_SUCCESS_PAYMENT_HASH_MISMATCH,
            ARTIFACT_TYPE_RECEIPT,
            "receipt success_fee.payment_hash must match deal success_payment_hash",
        );
    }
    if !limits_within_quote(&receipt.payload.limits_applied, quote) {
        report.push_issue(
            ISSUE_EXECUTION_LIMITS_EXCEED_OFFER,
            ARTIFACT_TYPE_RECEIPT,
            "receipt limits_applied must not exceed quote execution_limits",
        );
    }
}

fn validate_receipt_invoice_bundle_links(
    report: &mut ChainValidationReport,
    invoice_bundle: &SignedArtifact<InvoiceBundlePayload>,
    deal: &SignedArtifact<DealPayload>,
    receipt: &SignedArtifact<ReceiptPayload>,
) {
    let settlement_refs = &receipt.payload.settlement_refs;

    if settlement_refs.bundle_hash.as_deref() != Some(invoice_bundle.hash.as_str()) {
        report.push_issue(
            ISSUE_RECEIPT_BUNDLE_HASH_MISMATCH,
            ARTIFACT_TYPE_RECEIPT,
            "receipt bundle_hash must match the invoice_bundle hash",
        );
    }
    if settlement_refs.base_fee.amount_msat != invoice_bundle.payload.base_fee.amount_msat
        || settlement_refs.success_fee.amount_msat != invoice_bundle.payload.success_fee.amount_msat
    {
        report.push_issue(
            ISSUE_INVOICE_AMOUNT_MISMATCH,
            ARTIFACT_TYPE_RECEIPT,
            "receipt settlement amounts must match the invoice_bundle legs",
        );
    }
    if settlement_refs.base_fee.invoice_hash != invoice_bundle.payload.base_fee.invoice_hash
        || settlement_refs.success_fee.invoice_hash
            != invoice_bundle.payload.success_fee.invoice_hash
    {
        report.push_issue(
            ISSUE_INVOICE_HASH_MISMATCH,
            ARTIFACT_TYPE_RECEIPT,
            "receipt invoice hashes must match the invoice_bundle legs",
        );
    }
    if settlement_refs.base_fee.payment_hash != invoice_bundle.payload.base_fee.payment_hash
        || settlement_refs.success_fee.payment_hash
            != invoice_bundle.payload.success_fee.payment_hash
    {
        report.push_issue(
            ISSUE_INVOICE_PAYMENT_HASH_MISMATCH,
            ARTIFACT_TYPE_RECEIPT,
            "receipt payment hashes must match the invoice_bundle legs",
        );
    }
    if settlement_refs.success_fee.payment_hash != deal.payload.success_payment_hash {
        report.push_issue(
            ISSUE_INVOICE_SUCCESS_PAYMENT_HASH_MISMATCH,
            ARTIFACT_TYPE_RECEIPT,
            "receipt success_fee.payment_hash must match deal success_payment_hash",
        );
    }

    let base_fee_must_be_settled = receipt.payload.settlement_state == "settled"
        || receipt.payload.execution_state != "not_started";
    let success_fee_must_be_settled = receipt.payload.settlement_state == "settled";
    if (base_fee_must_be_settled && settlement_refs.base_fee.state != ReceiptLegState::Settled)
        || (success_fee_must_be_settled
            && settlement_refs.success_fee.state != ReceiptLegState::Settled)
    {
        report.push_issue(
            ISSUE_INVOICE_STATE_MISMATCH,
            ARTIFACT_TYPE_RECEIPT,
            "settled or executed lightning receipts require the corresponding invoice legs to be settled",
        );
    }
}

fn limits_within_offer(limits: &ExecutionLimits, offer: &SignedArtifact<OfferPayload>) -> bool {
    limits.max_input_bytes <= offer.payload.execution_profile.max_input_bytes
        && limits.max_runtime_ms <= offer.payload.execution_profile.max_runtime_ms
        && limits.max_memory_bytes <= offer.payload.execution_profile.max_memory_bytes
        && limits.max_output_bytes <= offer.payload.execution_profile.max_output_bytes
        && limits.fuel_limit <= offer.payload.execution_profile.fuel_limit
}

fn limits_within_quote(limits: &ExecutionLimits, quote: &SignedArtifact<QuotePayload>) -> bool {
    limits.max_input_bytes <= quote.payload.execution_limits.max_input_bytes
        && limits.max_runtime_ms <= quote.payload.execution_limits.max_runtime_ms
        && limits.max_memory_bytes <= quote.payload.execution_limits.max_memory_bytes
        && limits.max_output_bytes <= quote.payload.execution_limits.max_output_bytes
        && limits.fuel_limit <= quote.payload.execution_limits.fuel_limit
}

fn is_known_paid_settlement_method(method: &str) -> bool {
    matches!(
        method,
        SETTLEMENT_METHOD_LIGHTNING_ESCROW
            | SETTLEMENT_METHOD_STRIPE_MPP
            | SETTLEMENT_METHOD_LIGHTNING_PREPAID
            | SETTLEMENT_METHOD_X402_EIP3009
            | SETTLEMENT_METHOD_OPERATOR_LEDGER
    )
}

fn validate_invoice_leg_hash_link(report: &mut ChainValidationReport, leg: &InvoiceBundleLeg) {
    if leg.invoice_hash != crypto::sha256_hex(leg.invoice_bolt11.as_bytes()) {
        report.push_issue(
            ISSUE_INVOICE_HASH_MISMATCH,
            TRANSPORT_TYPE_INVOICE_BUNDLE,
            "invoice leg invoice_hash must equal SHA256(invoice_bolt11)",
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Deserialize, Serialize};
    use std::{fs, path::PathBuf};

    #[derive(Debug, Deserialize)]
    struct KernelConformanceFixture {
        artifacts: ArtifactVectors,
    }

    #[derive(Debug, Deserialize)]
    struct ArtifactVectors {
        descriptor: ArtifactVector<DescriptorPayload>,
        offer: ArtifactVector<OfferPayload>,
        quote: ArtifactVector<QuotePayload>,
        deal: ArtifactVector<DealPayload>,
        invoice_bundle: ArtifactVector<InvoiceBundlePayload>,
        receipt: ArtifactVector<ReceiptPayload>,
        free_offer: ArtifactVector<OfferPayload>,
        free_quote: ArtifactVector<QuotePayload>,
        free_deal: ArtifactVector<DealPayload>,
        free_receipt: ArtifactVector<ReceiptPayload>,
    }

    #[derive(Debug, Deserialize)]
    struct ArtifactVector<T> {
        artifact: SignedArtifact<T>,
    }

    fn fixture_path() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../conformance/kernel_v1.json")
    }

    fn load_fixture() -> KernelConformanceFixture {
        let bytes = fs::read_to_string(fixture_path()).expect("read conformance fixture");
        serde_json::from_str(&bytes).expect("parse conformance fixture")
    }

    fn issue_codes(report: &ChainValidationReport) -> Vec<String> {
        report
            .issues
            .iter()
            .map(|issue| issue.code.clone())
            .collect()
    }

    fn resign_provider_artifact<T: Clone + Serialize>(
        artifact: &SignedArtifact<T>,
    ) -> SignedArtifact<T> {
        let signing_key =
            crypto::signing_key_from_seed_bytes(&[0x11; 32]).expect("conformance provider seed");
        super::super::kernel::sign_artifact(
            &crypto::public_key_hex(&signing_key),
            |message| crypto::sign_message_hex(&signing_key, message),
            &artifact.artifact_type,
            artifact.created_at,
            artifact.payload.clone(),
        )
        .expect("re-sign conformance artifact")
    }

    fn validate_paid_chain_with_receipt_mutation(
        artifacts: &ArtifactVectors,
        mutate: impl FnOnce(&mut ReceiptPayload),
    ) -> FullChainReport {
        let mut receipt = artifacts.receipt.artifact.clone();
        mutate(&mut receipt.payload);
        let receipt = resign_provider_artifact(&receipt);
        validate_full_chain(
            &FullChain {
                descriptor: &artifacts.descriptor.artifact,
                offer: &artifacts.offer.artifact,
                quote: &artifacts.quote.artifact,
                invoice_bundle: Some(&artifacts.invoice_bundle.artifact),
                deal: &artifacts.deal.artifact,
                receipt: Some(&receipt),
            },
            None,
        )
    }

    #[test]
    fn validates_canonical_paid_chain_partials() {
        let fixture = load_fixture();
        let descriptor = &fixture.artifacts.descriptor.artifact;
        let offer = &fixture.artifacts.offer.artifact;
        let quote = &fixture.artifacts.quote.artifact;
        let deal = &fixture.artifacts.deal.artifact;
        let invoice_bundle = &fixture.artifacts.invoice_bundle.artifact;
        let receipt = &fixture.artifacts.receipt.artifact;

        let descriptor_offer = validate_descriptor_offer(descriptor, offer, None);
        assert!(
            descriptor_offer.valid,
            "unexpected issues: {:?}",
            descriptor_offer.issues
        );

        let offer_quote = validate_offer_quote(offer, quote, None);
        assert!(
            offer_quote.valid,
            "unexpected issues: {:?}",
            offer_quote.issues
        );

        let quote_deal = validate_quote_deal(quote, deal, None);
        assert!(
            quote_deal.valid,
            "unexpected issues: {:?}",
            quote_deal.issues
        );

        let quote_invoice_bundle_deal =
            validate_quote_invoice_bundle_deal(quote, invoice_bundle, deal, None);
        assert!(
            quote_invoice_bundle_deal.valid,
            "unexpected issues: {:?}",
            quote_invoice_bundle_deal.issues
        );

        let quote_deal_receipt = validate_quote_deal_receipt(quote, deal, receipt, None);
        assert!(
            quote_deal_receipt.valid,
            "unexpected issues: {:?}",
            quote_deal_receipt.issues
        );
    }

    #[test]
    fn validates_canonical_free_chain_partials_without_invoice_bundle() {
        let fixture = load_fixture();
        let descriptor = &fixture.artifacts.descriptor.artifact;
        let offer = &fixture.artifacts.free_offer.artifact;
        let quote = &fixture.artifacts.free_quote.artifact;
        let deal = &fixture.artifacts.free_deal.artifact;
        let receipt = &fixture.artifacts.free_receipt.artifact;

        let descriptor_offer = validate_descriptor_offer(descriptor, offer, None);
        assert!(
            descriptor_offer.valid,
            "unexpected issues: {:?}",
            descriptor_offer.issues
        );

        let offer_quote = validate_offer_quote(offer, quote, None);
        assert!(
            offer_quote.valid,
            "unexpected issues: {:?}",
            offer_quote.issues
        );

        let quote_deal = validate_quote_deal(quote, deal, None);
        assert!(
            quote_deal.valid,
            "unexpected issues: {:?}",
            quote_deal.issues
        );

        let quote_deal_receipt = validate_quote_deal_receipt(quote, deal, receipt, None);
        assert!(
            quote_deal_receipt.valid,
            "unexpected issues: {:?}",
            quote_deal_receipt.issues
        );
    }

    #[test]
    fn full_paid_chain_requires_invoice_bundle() {
        let fixture = load_fixture();
        let artifacts = &fixture.artifacts;

        let with_bundle = validate_full_chain(
            &FullChain {
                descriptor: &artifacts.descriptor.artifact,
                offer: &artifacts.offer.artifact,
                quote: &artifacts.quote.artifact,
                invoice_bundle: Some(&artifacts.invoice_bundle.artifact),
                deal: &artifacts.deal.artifact,
                receipt: Some(&artifacts.receipt.artifact),
            },
            None,
        );
        assert!(
            with_bundle.valid,
            "unexpected issues: {:?}",
            with_bundle.reports
        );
        assert_eq!(with_bundle.reports.len(), 4);

        let without_bundle = validate_full_chain(
            &FullChain {
                descriptor: &artifacts.descriptor.artifact,
                offer: &artifacts.offer.artifact,
                quote: &artifacts.quote.artifact,
                invoice_bundle: None,
                deal: &artifacts.deal.artifact,
                receipt: Some(&artifacts.receipt.artifact),
            },
            None,
        );
        assert!(!without_bundle.valid);
        assert!(without_bundle.reports.iter().any(|report| {
            issue_codes(report).contains(&ISSUE_INVOICE_BUNDLE_REQUIRED_FOR_LIGHTNING.to_string())
        }));
        assert_eq!(without_bundle.reports.len(), 4);
    }

    #[test]
    fn full_non_lightning_chain_rejects_invoice_bundle() {
        let fixture = load_fixture();
        let artifacts = &fixture.artifacts;

        let report = validate_full_chain(
            &FullChain {
                descriptor: &artifacts.descriptor.artifact,
                offer: &artifacts.free_offer.artifact,
                quote: &artifacts.free_quote.artifact,
                invoice_bundle: Some(&artifacts.invoice_bundle.artifact),
                deal: &artifacts.free_deal.artifact,
                receipt: Some(&artifacts.free_receipt.artifact),
            },
            None,
        );

        assert!(!report.valid);
        assert!(report.reports.iter().any(|path| {
            issue_codes(path).contains(&ISSUE_INVOICE_BUNDLE_FOR_NON_LIGHTNING.to_string())
        }));
    }

    #[test]
    fn full_lightning_chain_binds_receipt_to_invoice_bundle_hash() {
        let fixture = load_fixture();
        let artifacts = &fixture.artifacts;
        let mut receipt = artifacts.receipt.artifact.clone();
        receipt.payload.settlement_refs.bundle_hash = Some("aa".repeat(32));
        let receipt = resign_provider_artifact(&receipt);

        let report = validate_full_chain(
            &FullChain {
                descriptor: &artifacts.descriptor.artifact,
                offer: &artifacts.offer.artifact,
                quote: &artifacts.quote.artifact,
                invoice_bundle: Some(&artifacts.invoice_bundle.artifact),
                deal: &artifacts.deal.artifact,
                receipt: Some(&receipt),
            },
            None,
        );

        assert!(!report.valid);
        assert!(report.reports.iter().any(|path| {
            issue_codes(path).contains(&ISSUE_RECEIPT_BUNDLE_HASH_MISMATCH.to_string())
        }));
    }

    #[test]
    fn full_lightning_chain_binds_receipt_settlement_legs() {
        let fixture = load_fixture();
        let artifacts = &fixture.artifacts;
        let cases = [
            (
                "base amount",
                validate_paid_chain_with_receipt_mutation(artifacts, |payload| {
                    payload.settlement_refs.base_fee.amount_msat += 1;
                }),
                ISSUE_INVOICE_AMOUNT_MISMATCH,
            ),
            (
                "success amount",
                validate_paid_chain_with_receipt_mutation(artifacts, |payload| {
                    payload.settlement_refs.success_fee.amount_msat += 1;
                }),
                ISSUE_INVOICE_AMOUNT_MISMATCH,
            ),
            (
                "base invoice hash",
                validate_paid_chain_with_receipt_mutation(artifacts, |payload| {
                    payload.settlement_refs.base_fee.invoice_hash = "aa".repeat(32);
                }),
                ISSUE_INVOICE_HASH_MISMATCH,
            ),
            (
                "success invoice hash",
                validate_paid_chain_with_receipt_mutation(artifacts, |payload| {
                    payload.settlement_refs.success_fee.invoice_hash = "bb".repeat(32);
                }),
                ISSUE_INVOICE_HASH_MISMATCH,
            ),
            (
                "base payment hash",
                validate_paid_chain_with_receipt_mutation(artifacts, |payload| {
                    payload.settlement_refs.base_fee.payment_hash = "cc".repeat(32);
                }),
                ISSUE_INVOICE_PAYMENT_HASH_MISMATCH,
            ),
            (
                "success payment hash",
                validate_paid_chain_with_receipt_mutation(artifacts, |payload| {
                    payload.settlement_refs.success_fee.payment_hash = "dd".repeat(32);
                }),
                ISSUE_INVOICE_PAYMENT_HASH_MISMATCH,
            ),
        ];

        for (name, report, expected_code) in cases {
            let codes: Vec<String> = report.reports.iter().flat_map(issue_codes).collect();
            assert!(!report.valid, "{name} mutation unexpectedly passed");
            assert!(
                codes.contains(&expected_code.to_string()),
                "{name} mutation did not report {expected_code}: {codes:?}"
            );
            if name == "success payment hash" {
                assert!(
                    codes.contains(&ISSUE_INVOICE_SUCCESS_PAYMENT_HASH_MISMATCH.to_string()),
                    "success payment hash mutation did not report the deal mismatch: {codes:?}"
                );
            }
        }
    }

    #[test]
    fn full_lightning_chain_requires_both_legs_settled_for_settled_receipt() {
        let fixture = load_fixture();
        let artifacts = &fixture.artifacts;
        let mut receipt = artifacts.receipt.artifact.clone();
        receipt.payload.settlement_refs.base_fee.state = ReceiptLegState::Canceled;
        let receipt = resign_provider_artifact(&receipt);

        let report = validate_full_chain(
            &FullChain {
                descriptor: &artifacts.descriptor.artifact,
                offer: &artifacts.offer.artifact,
                quote: &artifacts.quote.artifact,
                invoice_bundle: Some(&artifacts.invoice_bundle.artifact),
                deal: &artifacts.deal.artifact,
                receipt: Some(&receipt),
            },
            None,
        );

        assert!(!report.valid);
        assert!(
            report.reports.iter().any(|path| {
                issue_codes(path).contains(&ISSUE_INVOICE_STATE_MISMATCH.to_string())
            })
        );
    }

    #[test]
    fn validates_full_free_chain() {
        let fixture = load_fixture();
        let artifacts = &fixture.artifacts;

        let report = validate_full_chain(
            &FullChain {
                descriptor: &artifacts.descriptor.artifact,
                offer: &artifacts.free_offer.artifact,
                quote: &artifacts.free_quote.artifact,
                invoice_bundle: None,
                deal: &artifacts.free_deal.artifact,
                receipt: Some(&artifacts.free_receipt.artifact),
            },
            None,
        );
        assert!(report.valid, "unexpected issues: {:?}", report.reports);
    }

    #[test]
    fn full_chain_reports_cross_chain_mismatch() {
        let fixture = load_fixture();
        let artifacts = &fixture.artifacts;

        let report = validate_full_chain(
            &FullChain {
                descriptor: &artifacts.descriptor.artifact,
                offer: &artifacts.offer.artifact,
                quote: &artifacts.quote.artifact,
                invoice_bundle: None,
                deal: &artifacts.free_deal.artifact,
                receipt: None,
            },
            None,
        );
        assert!(!report.valid);
        let codes: Vec<String> = report
            .reports
            .iter()
            .flat_map(|path_report| path_report.issues.iter().map(|issue| issue.code.clone()))
            .collect();
        assert!(
            codes.contains(&ISSUE_QUOTE_HASH_MISMATCH.to_string()),
            "expected quote_hash_mismatch, got {codes:?}"
        );
    }

    #[test]
    fn reports_deterministic_envelope_issues_before_link_issues() {
        let fixture = load_fixture();
        let offer = &fixture.artifacts.offer.artifact;
        let mut quote = fixture.artifacts.quote.artifact.clone();
        quote.artifact_type = ARTIFACT_TYPE_DEAL.to_string();
        quote.payload.offer_hash = "aa".repeat(32);

        let report = validate_offer_quote(offer, &quote, None);
        assert!(!report.valid);
        let codes = issue_codes(&report);
        assert_eq!(
            &codes[..3],
            &[
                ISSUE_ARTIFACT_TYPE_MISMATCH.to_string(),
                ISSUE_ARTIFACT_ENVELOPE_INVALID.to_string(),
                ISSUE_OFFER_HASH_MISMATCH.to_string()
            ]
        );
    }

    #[test]
    fn reports_partial_chain_hash_mismatch_without_requiring_full_path() {
        let fixture = load_fixture();
        let quote = &fixture.artifacts.quote.artifact;
        let free_deal = &fixture.artifacts.free_deal.artifact;

        let report = validate_quote_deal(quote, free_deal, None);
        assert!(!report.valid);
        assert!(
            issue_codes(&report).contains(&ISSUE_QUOTE_HASH_MISMATCH.to_string()),
            "expected quote_hash_mismatch, got {:?}",
            report.issues
        );
    }

    #[test]
    fn reports_invoice_bundle_link_failures_with_stable_codes() {
        let fixture = load_fixture();
        let quote = &fixture.artifacts.quote.artifact;
        let deal = &fixture.artifacts.deal.artifact;
        let bundle = &fixture.artifacts.free_receipt.artifact;

        let report = validate_quote_deal_receipt(quote, deal, bundle, None);
        assert!(!report.valid);
        assert!(
            issue_codes(&report).contains(&ISSUE_SETTLEMENT_METHOD_MISMATCH.to_string()),
            "expected settlement_method_mismatch, got {:?}",
            report.issues
        );
    }

    #[test]
    fn reports_expiry_when_now_is_provided() {
        let fixture = load_fixture();
        let offer = &fixture.artifacts.offer.artifact;
        let quote = &fixture.artifacts.quote.artifact;

        let report = validate_offer_quote(offer, quote, Some(quote.payload.expires_at + 1));
        assert!(!report.valid);
        assert!(
            issue_codes(&report).contains(&ISSUE_ARTIFACT_EXPIRED.to_string()),
            "expected artifact_expired, got {:?}",
            report.issues
        );
    }
}
