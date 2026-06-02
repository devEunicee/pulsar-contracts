# ADR-0005: On-Chain vs. Off-Chain Analytics Strategy

**Status:** Accepted  
**Date:** 2024-01-20  
**Deciders:** Pulsar Contributors

## Context

The payment processing contract needs to provide analytics and reporting capabilities. Currently, only global aggregate statistics are available (`get_global_payment_stats`). There is no per-merchant breakdown, per-token analysis, or time-series data.

## Problem Statement

- Global stats provide only aggregate totals (total payments, total volume, total refunds)
- No per-merchant analytics — merchants cannot see their own performance metrics
- No per-token breakdown — cannot analyze payment volume by token
- No time-series data — cannot track trends over time
- Off-chain indexer (BE-001) would provide richer analytics but requires separate infrastructure

## Decision

Implement a **hybrid analytics strategy** with two tiers:

### Tier 1: On-Chain Per-Merchant Stats (Phase 1)
- Add `get_merchant_stats(merchant, date_start, date_end)` function
- Returns per-merchant totals: payment count, volume, refund count, refund volume
- Cached stats for unfiltered queries; computed on-demand for date-filtered queries
- Accessible by merchant (own stats) or admin (any merchant)
- Stored in persistent storage with TTL management

### Tier 2: Off-Chain Indexer (Phase 2 - BE-001)
- Separate indexer service monitors contract events
- Provides richer analytics: per-token breakdown, time-series, trends, comparisons
- Enables complex queries not feasible on-chain (e.g., top merchants, category analysis)
- Reduces on-chain computation and storage overhead

## Implementation Details

### Tier 1: On-Chain Per-Merchant Stats

**New Type** (`types.rs`):
```rust
#[contracttype]
pub struct MerchantStats {
    pub merchant_address: Address,
    pub total_payments: u64,
    pub total_volume: i128,
    pub total_refunds: u64,
    pub total_refund_volume: i128,
}
```

**New Storage Key** (`types.rs`):
```rust
pub enum DataKey {
    // ...
    MerchantStats(Address),
    // ...
}
```

**New Storage Functions** (`storage.rs`):
- `get_merchant_stats(merchant)` — retrieve cached stats
- `save_merchant_stats(stats)` — persist stats
- `increment_merchant_payment_stats(merchant, amount)` — increment on payment
- `increment_merchant_refund_stats(merchant, amount)` — increment on refund

**New Contract Function** (`lib.rs`):
```rust
pub fn get_merchant_stats(
    env: Env,
    merchant: Address,
    date_start: Option<u64>,
    date_end: Option<u64>,
) -> Result<MerchantStats, PaymentError>
```

**Access Control**:
- Merchant can query their own stats
- Admin can query any merchant's stats
- Unauthorized callers receive `PaymentError::Unauthorized`

**Query Modes**:
- **Unfiltered** (no date range): Returns cached stats (O(1))
- **Filtered** (with date range): Iterates merchant's payment IDs and computes stats (O(n) where n = merchant's payment count)

**Stat Updates**:
- Incremented on `process_payment_with_signature()` (payment count + volume)
- Incremented on `execute_refund()` (refund count + volume)
- Cached stats persist across contract upgrades

### Tier 2: Off-Chain Indexer (Future - BE-001)

**Event-Driven Architecture**:
- Indexer listens to contract events: `payment_processed`, `refund_executed`, etc.
- Aggregates data into time-series database (e.g., InfluxDB, TimescaleDB)
- Provides REST API for analytics queries

**Capabilities**:
- Per-token payment volume and count
- Per-category merchant analysis
- Time-bucketed aggregates (hourly, daily, weekly, monthly)
- Merchant rankings and comparisons
- Refund rate analysis
- Trend detection and forecasting

**Benefits**:
- Reduces on-chain storage and computation
- Enables complex queries not feasible on-chain
- Provides historical data beyond contract TTL
- Supports real-time dashboards and reporting

## Consequences

### Positive
- Merchants can monitor their own performance on-chain
- Admin can audit merchant activity on-chain
- Cached stats provide O(1) query performance for common case
- Filtered queries support date-range analysis
- Extensible design allows future off-chain indexer integration
- TTL management ensures stats persist with payment records

### Negative
- Filtered queries require O(n) iteration through merchant's payments
- No per-token breakdown on-chain (requires off-chain indexer)
- No time-series aggregates on-chain (requires off-chain indexer)
- Stats are point-in-time snapshots, not historical time-series
- Merchant stats storage grows with number of merchants

### Neutral
- Off-chain indexer is separate service (not part of contract)
- Contract events provide sufficient data for indexer to reconstruct stats
- Caching strategy balances performance and freshness

## Migration Path

### Phase 1: On-Chain Per-Merchant Stats (Current)
1. Add `MerchantStats` type and storage
2. Implement `get_merchant_stats()` function
3. Update payment/refund processing to increment merchant stats
4. Add comprehensive tests
5. Deploy contract upgrade

### Phase 2: Off-Chain Indexer (Future - BE-001)
1. Design indexer schema and API
2. Implement event listener
3. Build aggregation pipeline
4. Create REST API endpoints
5. Deploy indexer service
6. Migrate analytics queries to indexer

### Phase 3: Advanced Analytics (Future)
1. Add category-based aggregation
2. Implement token-based breakdown
3. Add trend analysis and forecasting
4. Support merchant comparisons and rankings

## Backward Compatibility

- New `get_merchant_stats()` function does not affect existing APIs
- Existing `get_global_payment_stats()` remains unchanged
- New storage key `MerchantStats(Address)` does not conflict with existing keys
- Contract upgrade is non-breaking

## Performance Considerations

**On-Chain Stats**:
- Cached stats: O(1) read, O(1) write per payment/refund
- Filtered stats: O(n) where n = merchant's payment count
- Storage: ~100 bytes per merchant (MerchantStats struct)

**Off-Chain Indexer**:
- Event processing: O(1) per event
- Aggregation: O(n) where n = events in time bucket
- Query: O(1) for pre-aggregated data

## Testing Strategy

1. **Unit Tests**:
   - `test_get_merchant_stats_unfiltered()` — cached stats
   - `test_get_merchant_stats_filtered()` — date-range filtering
   - `test_merchant_stats_increments_on_payment()` — payment tracking
   - `test_merchant_stats_increments_on_refund()` — refund tracking
   - `test_merchant_stats_access_control()` — authorization

2. **Integration Tests**:
   - Multiple merchants with overlapping payments
   - Date-range filtering across payment boundaries
   - Refund scenarios (partial, full, multiple)

3. **Performance Tests**:
   - Cached stats query performance
   - Filtered stats with varying merchant payment counts
   - Storage overhead with many merchants

## References

- ADR-0002: Per-Entity Storage Layout
- ADR-0004: Merchant Category Management
- BE-001: Off-Chain Indexer (future)
- Issue #102 MISC-007: No analytics or reporting beyond global stats
