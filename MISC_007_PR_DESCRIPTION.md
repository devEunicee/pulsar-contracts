# Pull Request: MISC-007 Per-Merchant Analytics and Reporting

## Branch
`feat/misc-007-merchant-stats`

## Title
`feat: MISC-007 add per-merchant analytics and reporting`

## Description

### Overview
This PR implements per-merchant analytics capabilities to address issue #102 MISC-007. The contract now provides merchants and admins with detailed performance metrics including payment counts, volumes, refund counts, and refund volumes.

### Changes

#### 1. Core Implementation

**New Types** (`types.rs`):
- `MerchantStats` struct with merchant address, payment metrics, and refund metrics
- New `DataKey::MerchantStats(Address)` storage key

**Storage Functions** (`storage.rs`):
- `get_merchant_stats(merchant)` — retrieve cached stats
- `save_merchant_stats(stats)` — persist stats with TTL management
- `increment_merchant_payment_stats(merchant, amount)` — increment on payment
- `increment_merchant_refund_stats(merchant, amount)` — increment on refund

**Contract Function** (`lib.rs`):
- `get_merchant_stats(merchant, date_start, date_end)` — query per-merchant stats
  - Accessible by merchant (own stats) or admin (any merchant)
  - Unfiltered queries return cached stats (O(1))
  - Filtered queries compute stats on-demand (O(n))

**Payment/Refund Processing** (`lib.rs`):
- Updated `process_payment_with_signature()` to increment merchant payment stats
- Updated `execute_refund()` to increment merchant refund stats

#### 2. Documentation

**ADR-0005: On-Chain vs. Off-Chain Analytics Strategy** (`docs/adr/0005-analytics-and-reporting.md`):
- Documents current enum-based category system
- Explains why categories require contract upgrades
- Outlines Phase 1 (on-chain) and Phase 2 (off-chain indexer) approach
- Provides migration path and backward compatibility notes

**Analytics Guide** (`docs/ANALYTICS_GUIDE.md`):
- Comprehensive usage guide for global and per-merchant stats
- Query modes (cached vs. filtered)
- Best practices for merchants and admins
- Performance considerations
- Troubleshooting section
- Roadmap for off-chain indexer (BE-001)

**README Updates**:
- Added "Merchant stats" to feature overview
- New "Analytics" section with examples
- Links to ANALYTICS_GUIDE.md and ADR-0005

#### 3. Testing

**New Test Cases** (`test.rs`):
- `test_get_merchant_stats_unfiltered()` — cached stats query
- `test_get_merchant_stats_with_refunds()` — stats with refund tracking
- `test_get_merchant_stats_filtered_by_date()` — date-range filtering
- `test_get_merchant_stats_access_control()` — authorization checks
- `test_get_merchant_stats_multiple_merchants()` — independent merchant stats

### Acceptance Criteria Met

✅ **Add get_merchant_stats(env, merchant, date_start, date_end) returning per-merchant totals**
- Implemented with full access control
- Supports both cached and filtered query modes
- Returns payment count, volume, refund count, and refund volume

✅ **Off-chain indexer (see BE-001) provides richer analytics**
- Documented in ADR-0005 as Phase 2 enhancement
- Event-driven architecture supports indexer integration
- Roadmap includes per-token breakdown, time-series data, and advanced queries

✅ **Document the on-chain vs. off-chain analytics split**
- Comprehensive ADR-0005 explaining both tiers
- ANALYTICS_GUIDE.md with usage examples
- README section linking to documentation

### Technical Details

#### Query Modes

**Unfiltered (Cached)**:
- Performance: O(1) — instant lookup
- Use case: Real-time dashboards, frequent queries
- Returns: Cached stats updated on every payment/refund

**Filtered (Computed)**:
- Performance: O(n) where n = merchant's payment count
- Use case: Historical analysis, date-range reports
- Returns: Real-time computed stats for specified date range

#### Access Control

- Merchant can query their own stats
- Admin can query any merchant's stats
- Unauthorized callers receive `PaymentError::Unauthorized`

#### Storage

- Persistent storage with TTL management (~1 year)
- ~100 bytes per merchant (MerchantStats struct)
- Auto-extends TTL on read/write

### Performance Impact

**On-Chain**:
- Cached stats: O(1) read, O(1) write per payment/refund
- Filtered stats: O(n) where n = merchant's payment count
- Storage: ~100 bytes per merchant

**Off-Chain (Future)**:
- Event processing: O(1) per event
- Aggregation: O(n) where n = events in time bucket
- Query: O(1) for pre-aggregated data

### Backward Compatibility

- New `get_merchant_stats()` function does not affect existing APIs
- Existing `get_global_payment_stats()` remains unchanged
- New storage key does not conflict with existing keys
- Contract upgrade is non-breaking

### Testing

All new tests pass:
- Unit tests for cached and filtered queries
- Integration tests with multiple merchants
- Access control verification
- Date-range filtering validation

### Related Issues

Closes #102 MISC-007

### Labels

smart-contract, feature, product

---

## How to Create the PR

1. Go to: https://github.com/MooreTheAnalyst/pulsar-contracts/pull/new/feat/misc-007-merchant-stats
2. Copy the title and description above
3. Click "Create pull request"

Or use GitHub CLI:
```bash
gh pr create \
  --repo MooreTheAnalyst/pulsar-contracts \
  --base main \
  --head feat/misc-007-merchant-stats \
  --title "feat: MISC-007 add per-merchant analytics and reporting" \
  --body "$(cat MISC_007_PR_DESCRIPTION.md | tail -n +3)"
```

## Files Changed

- `contracts/payment-processing-contract/src/types.rs` — Add MerchantStats type and storage key
- `contracts/payment-processing-contract/src/storage.rs` — Add merchant stats functions
- `contracts/payment-processing-contract/src/lib.rs` — Add get_merchant_stats function and update payment/refund processing
- `contracts/payment-processing-contract/src/test.rs` — Add comprehensive test cases
- `docs/adr/0005-analytics-and-reporting.md` — New ADR documenting analytics strategy
- `docs/ANALYTICS_GUIDE.md` — New comprehensive analytics guide
- `README.md` — Update with analytics section and documentation links
