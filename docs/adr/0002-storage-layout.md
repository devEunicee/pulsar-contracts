# ADR-0002: Per-Entity Storage Layout with Explicit Index Lists

**Status:** Accepted  
**Date:** 2024-01-01  
**Deciders:** Pulsar Contributors

## Context

Soroban persistent storage is a key-value store. There is no native query capability — to retrieve a set of records (e.g. all payments for a merchant) the contract must maintain its own index.

Options considered:

1. **Single global list** — one `Vec<Bytes>` of all payment IDs, scanned on every query.
2. **Per-entity index lists** — separate `Vec<Bytes>` per merchant (`MerchantPayments(Address)`) and per payer (`PayerPayments(Address)`), plus a global list for admin queries.
3. **On-chain map / trie** — not natively supported by Soroban SDK at this time.
4. **Chunked index lists** — same as (2) but each entity's list is split into fixed-size chunks (`MerchantPaymentChunk(Address, u32)`) with a count key (`MerchantPaymentCount(Address)`). Considered to reduce the cost of extending large lists. **Superseded** — the flat list approach (option 2) was chosen instead because chunk management added complexity without a meaningful benefit at current scale.

## Decision

Use per-entity flat index lists (option 2). Each entity (merchant, payer) has its own `Vec<Bytes>` of payment IDs stored under a typed `DataKey`. A separate global index supports admin stats and cleanup. Records themselves are stored under `DataKey::Payment(order_id)`.

The chunked variants (`MerchantPaymentChunk`, `MerchantPaymentCount`, `PayerPaymentChunk`, `PayerPaymentCount`) were removed in SC-059 as dead code — they were defined in `DataKey` but never written or read by any contract function.

## Consequences

### Positive
- Merchant and payer history queries only scan IDs relevant to that entity — O(n) where n is that entity's payment count, not the global count.
- Clean separation: record data and index lists are independent; archiving a record does not corrupt the index.
- Typed `DataKey` enum prevents key collisions between different entity types.
- Removing the dead chunk variants reduces `DataKey` surface area and eliminates potential confusion for future contributors.

### Negative
- Every payment write touches three index lists (merchant, payer, global) — three extra storage writes per payment.
- Index lists grow unboundedly; very active merchants/payers accumulate large lists. No compaction mechanism exists today.
- No cross-entity queries (e.g. "all payments for token X") without a full global scan.

### Neutral
- `cleanup_expired_payments` rebuilds the global index list in-place, which is O(n) in global payment count.
