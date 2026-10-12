---
title: Kernel Specification
description: The normative protocol specification.
---

:::caution
This is a summary. The full normative specification is in [`docs/KERNEL.md`](https://github.com/AlitheaBio/froglet/blob/main/docs/KERNEL.md) in the froglet repository.
:::

## Scope

Froglet v1 is a small economic primitive for short-lived, bounded, fixed-price resource deals. A deal may represent a predefined service, a data service, or open-ended compute. These are product-layer distinctions over the same signed economic primitive.

## Global constants

- `schema_version`: always `froglet/v1`
- Hashes: lowercase hex SHA-256
- Timestamps: Unix seconds
- Canonical JSON: RFC 8785 JCS
- Identities: 32-byte secp256k1 x-only public keys (lowercase hex)
- Signatures: 64-byte BIP340 Schnorr (lowercase hex)

## Six artifact types

1. **Descriptor** — provider identity and capabilities
2. **Offer** — specific service with pricing and execution profile
3. **Quote** — priced workload for a specific requester (ephemeral)
4. **Deal** — requester commitment (signed by requester)
5. **InvoiceBundle** — Lightning escrow payment instructions (two legs), when applicable
6. **Receipt** — signed terminal report of execution and settlement

A free exchange uses five artifacts and omits the InvoiceBundle. These records
authenticate commitments and the provider's reported outcome; they do not
independently prove that an arbitrary computation was performed correctly.

## How one artifact is signed

Every artifact is a signed envelope around a payload:

1. `payload_hash` is the SHA-256 of the payload in RFC 8785 canonical JSON.
2. The signing bytes are the canonical JSON of `[schema_version, artifact_type, signer, created_at, payload_hash, payload]`.
3. The artifact's `hash` is the SHA-256 of those bytes. It is the artifact's identity, used by the relationships that bind the signed records together.
4. The signature is BIP-340 Schnorr over that 32-byte digest. The signer is an x-only secp256k1 public key.

The [conformance vectors](/spec/conformance/) record the exact signing bytes in hex, so an implementation can find its first differing byte. Where this summary and a vector disagree, the vector wins.

## Settlement methods

- `none` — free execution, no payment
- `lightning.base_fee_plus_success_fee.v1` — two-leg Lightning settlement
- `stripe_mpp.v1` — Stripe manual-capture PaymentIntent settlement (agentic Shared Payment Token model)

Note: `x402_usdc` is a daemon-level payment kind for lower-level compute endpoints, not a marketplace receipt settlement method.

## What stays out

The kernel does not hardwire: marketplace, discovery, transport, storage engine, execution runtime, ranking/broker logic, deployment topology, or long-running sessions.
