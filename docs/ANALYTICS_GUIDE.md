# Analytics and Reporting Guide

This guide explains the analytics capabilities available in the payment processing contract and the roadmap for future enhancements.

## Overview

The contract provides two levels of analytics:

1. **On-Chain Analytics** — Per-merchant and global statistics stored in the contract
2. **Off-Chain Analytics** — Richer analytics via external indexer service (future)

## On-Chain Analytics

### Global Payment Stats

Query aggregate statistics across all merchants and payments.

**Function**: `get_global_payment_stats(admins, date_start, date_end)`

**Access**: Admin only

**Returns**:
```rust
GlobalStats {
    total_payments: u64,      // Total number of payments
    total_volume: i128,       // Sum of all payment amounts
    total_refunds: u64,       // Total number of refunds
    total_refund_volume: i128 // Sum of all refund amounts
}
```

**Example**:
```bash
# Get all-time global stats
stellar contract invoke --id $CONTRACT_ID --source-account <ADMIN_KEY> --network testnet \
  -- get_global_payment_stats \
  --admins '["<ADMIN_ADDRESS>"]' \
  --date_start null \
  --date_end null

# Get stats for a specific date range (Unix timestamps)
stellar contract invoke --id $CONTRACT_ID --source-account <ADMIN_KEY> --network testnet \
  -- get_global_payment_stats \
  --admins '["<ADMIN_ADDRESS>"]' \
  --date_start 1704067200 \
  --date_end 1704153600
```

**Use Cases**:
- Monitor total payment volume
- Track refund activity
- Audit contract usage
- Generate compliance reports

### Per-Merchant Stats

Query statistics for a specific merchant.

**Function**: `get_merchant_stats(merchant, date_start, date_end)`

**Access**: 
- Merchant can query their own stats
- Admin can query any merchant's stats

**Returns**:
```rust
MerchantStats {
    merchant_address: Address,
    total_payments: u64,      // Total payments received
    total_volume: i128,       // Total payment volume
    total_refunds: u64,       // Total refunds issued
    total_refund_volume: i128 // Total refund volume
}
```

**Example**:
```bash
# Merchant queries their own stats
stellar contract invoke --id $CONTRACT_ID --source-account <MERCHANT_KEY> --network testnet \
  -- get_merchant_stats \
  --merchant <MERCHANT_ADDRESS> \
  --date_start null \
  --date_end null

# Admin queries a merchant's stats
stellar contract invoke --id $CONTRACT_ID --source-account <ADMIN_KEY> --network testnet \
  -- get_merchant_stats \
  --merchant <MERCHANT_ADDRESS> \
  --date_start 1704067200 \
  --date_end 1704153600
```

**Use Cases**:
- Merchants monitor their own performance
- Admin audits merchant activity
- Calculate merchant commissions
- Identify high-volume merchants
- Track refund rates

### Query Modes

#### Unfiltered Query (Cached)
When no date range is specified, the contract returns cached stats.

**Performance**: O(1) — instant lookup  
**Freshness**: Updated on every payment/refund execution

```bash
stellar contract invoke --id $CONTRACT_ID --source-account <MERCHANT_KEY> --network testnet \
  -- get_merchant_stats \
  --merchant <MERCHANT_ADDRESS> \
  --date_start null \
  --date_end null
```

#### Filtered Query (Computed)
When a date range is specified, the contract computes stats on-demand.

**Performance**: O(n) where n = merchant's payment count  
**Freshness**: Real-time, includes all payments in date range

```bash
stellar contract invoke --id $CONTRACT_ID --source-account <MERCHANT_KEY> --network testnet \
  -- get_merchant_stats \
  --merchant <MERCHANT_ADDRESS> \
  --date_start 1704067200 \
  --date_end 1704153600
```

**Note**: For merchants with many payments, filtered queries may be slow. Consider using the off-chain indexer for historical analysis.

## Off-Chain Analytics (Future)

The contract is designed to support an off-chain indexer service (BE-001) that provides richer analytics.

### Planned Capabilities

**Per-Token Breakdown**:
- Payment volume by token
- Refund volume by token
- Token-specific merchant rankings

**Time-Series Data**:
- Hourly, daily, weekly, monthly aggregates
- Trend analysis and forecasting
- Seasonal patterns

**Merchant Comparisons**:
- Top merchants by volume
- Refund rate rankings
- Category-based analysis

**Advanced Queries**:
- Payment velocity (payments per time period)
- Refund rate analysis
- Customer lifetime value
- Churn detection

### Event-Driven Architecture

The contract emits events that the indexer can consume:

```rust
// Payment processed
env.events().publish(
    (String::from_str(&env, "payment_processed"),),
    (order_id, payer, merchant_address, amount),
);

// Refund executed
env.events().publish(
    (String::from_str(&env, "refund_executed"),),
    (refund_id, amount),
);
```

The indexer listens to these events and aggregates data into a time-series database.

## Best Practices

### For Merchants

1. **Monitor Performance**: Query your stats regularly to track performance
   ```bash
   # Weekly performance check
   stellar contract invoke --id $CONTRACT_ID --source-account <MERCHANT_KEY> --network testnet \
     -- get_merchant_stats \
     --merchant <MERCHANT_ADDRESS> \
     --date_start <WEEK_START> \
     --date_end <WEEK_END>
   ```

2. **Analyze Refund Rates**: Use refund stats to identify issues
   ```
   Refund Rate = total_refunds / total_payments
   Refund Ratio = total_refund_volume / total_volume
   ```

3. **Track Trends**: Compare stats across time periods
   ```
   Week-over-week growth = (current_week - previous_week) / previous_week
   ```

### For Admins

1. **Audit Merchants**: Regularly check merchant stats for anomalies
   ```bash
   # Check all merchants' stats
   for merchant in $(get_all_merchants); do
     stellar contract invoke --id $CONTRACT_ID --source-account <ADMIN_KEY> --network testnet \
       -- get_merchant_stats \
       --merchant $merchant \
       --date_start <PERIOD_START> \
       --date_end <PERIOD_END>
   done
   ```

2. **Monitor Global Activity**: Track overall contract usage
   ```bash
   # Daily global stats
   stellar contract invoke --id $CONTRACT_ID --source-account <ADMIN_KEY> --network testnet \
     -- get_global_payment_stats \
     --admins '["<ADMIN_ADDRESS>"]' \
     --date_start <DAY_START> \
     --date_end <DAY_END>
   ```

3. **Identify Trends**: Use date-range queries to spot patterns
   ```
   Daily Average = total_volume / number_of_days
   Growth Rate = (current_period - previous_period) / previous_period
   ```

## Performance Considerations

### On-Chain Queries

**Cached Stats** (no date filter):
- Response time: < 100ms
- Storage: ~100 bytes per merchant
- Suitable for: Real-time dashboards, frequent queries

**Filtered Stats** (with date filter):
- Response time: O(n) where n = merchant's payment count
- Suitable for: Historical analysis, periodic reports
- Recommendation: Use for merchants with < 10,000 payments

### Off-Chain Indexer (Future)

**Pre-Aggregated Data**:
- Response time: < 10ms
- Storage: Depends on retention period
- Suitable for: Complex queries, historical analysis, dashboards

## Troubleshooting

### Query Returns Unexpected Stats

**Issue**: Stats don't match expected values

**Solution**:
1. Verify the date range (Unix timestamps)
2. Check merchant address is correct
3. Ensure you have permission to query (merchant or admin)
4. Confirm payments were processed in the specified date range

### Filtered Query is Slow

**Issue**: Date-range query takes too long

**Solution**:
1. Reduce the date range
2. Use a merchant with fewer payments
3. Wait for off-chain indexer (BE-001) for better performance
4. Consider caching results on your end

### Stats Don't Update Immediately

**Issue**: New payment not reflected in stats

**Solution**:
1. Confirm payment was successfully processed (check events)
2. Wait a few seconds for TTL extension
3. Query again to refresh cached stats
4. Check merchant address matches payment recipient

## References

- ADR-0005: On-Chain vs. Off-Chain Analytics Strategy
- ADR-0002: Per-Entity Storage Layout
- Issue #102 MISC-007: No analytics or reporting beyond global stats
- BE-001: Off-Chain Indexer (future)
