# ADR-0003: Cursor-Based Pagination for Payment History Queries

**Status:** Accepted  
**Date:** 2024-01-01  
**Deciders:** Pulsar Contributors

## Context

Payment history queries (`get_merchant_payment_history`, `get_payer_payment_history`) may return large result sets. Returning all records in one call is impractical — Soroban has per-transaction CPU and memory limits.

Options considered:

1. **Offset pagination** — caller passes `offset` + `limit`. Simple but unstable: inserting a new record shifts all offsets.
2. **Cursor pagination** — caller passes the `order_id` of the last seen record as a cursor. The next page starts after that record in the sorted result set.
3. **No pagination / hard cap** — return at most N records, no continuation. Simple but loses data for active entities.

## Decision

Use cursor-based pagination (option 2). `get_payer_payment_history` and `get_merchant_payment_history` accept `cursor: Option<Bytes>` and `limit: u32` (capped at 100). The response includes `next_cursor: Option<Bytes>` pointing to the last record on the page; passing it as `cursor` on the next call retrieves the following page.

### Cursor format and migration

The cursor is currently the raw `order_id` bytes of the last record returned.
Callers that need to transmit the cursor over textual transports (CLI, HTTP)
should encode it, for example using base64. Because the contract treats the
cursor as an opaque `Bytes` value, callers should not attempt to parse its
contents.

If the cursor format ever needs to change, the change must be versioned and
documented in an ADR that includes a migration strategy. A recommended
pattern is to use a small version prefix (e.g. `v1:`) followed by a stable
encoding (base64 of the order_id) so that callers can detect incompatible
cursor formats.

Sorting is applied before pagination so the cursor position is stable within a sort order.

## Consequences

### Positive
- Stable pagination: new payments appended during iteration do not shift existing pages.
- Caller controls page size up to the 100-record cap, balancing throughput and ledger cost.
- No global index required — each entity's own ID list is sufficient.

### Negative
- Cursor is opaque (`order_id` bytes); callers cannot jump to an arbitrary page — only forward iteration is supported.
- Sorting requires collecting all matching records into a Rust `Vec`, sorting in-place, then truncating — O(n log n) in the number of matching records. For very large histories this may approach ledger limits.
- No total count of filtered results is returned (only total before truncation), so callers cannot compute total pages without iterating all pages.

### Neutral
- Max page size of 100 is a constant; it can be made configurable via admin config in a future iteration.
