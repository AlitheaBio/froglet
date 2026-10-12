---
title: Service Binding
description: How product shapes map to kernel deals.
---

:::caution
This is a summary. The full specification is in [`docs/SERVICE_BINDING.md`](https://github.com/AlitheaBio/froglet/blob/main/docs/SERVICE_BINDING.md).
:::

## Scope

The service binding layer sits between the kernel and the node surface. It defines:

1. **Service discovery** — how a requester finds and understands available services
2. **Service invocation** — how a requester invokes a discovered service

All three product shapes reduce to the same kernel deal flow: Offer, Quote, Deal, (InvoiceBundle), Receipt.

## Three product shapes

| Shape | offer_kind | What the requester sends |
|-------|-----------|-------------------------|
| **Named service** | e.g., `marketplace.search` | JSON input |
| **Data service** | e.g., `events.query` | Query parameters |
| **Direct compute** | `compute.execution.v1` | WASM module + input |

## Identifier relationships

- `service_id` — human-readable service name
- `offer_id` — the specific offer instance
- `offer_kind` — the workload category
- `resource_kind` — compute, data, or confidential

The binding layer compiles these into kernel primitives — the caller doesn't need to know about WorkloadSpec, ExecutionWorkload, or canonical hashing.
