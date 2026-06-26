# Migration: Subscription Payment Service Schema

**Issue:** #280  
**Date:** 2026-06-25  
**Purpose:** Implement database schema for subscription payment management

## Changes

### New Tables

1. **subscriptions**
   - Stores subscription records
   - Tracks status, payment schedule, metadata
   - Supports multiple billing frequencies
   - Status: active, paused, pending_payment, past_due, cancelled

2. **payment_attempts**
   - Records all payment attempts
   - Tracks status and error messages
   - Attempts are numbered for retry tracking
   - Timestamps for created and completion

3. **invoices**
   - Stores generated invoices
   - Unique invoice numbers per subscription
   - Status tracking (issued, paid, etc.)
   - Track issue and due dates

4. **subscription_events**
   - Complete audit trail of subscription events
   - Event types and associated data
   - Chronological ordering for analysis

### Key Features

1. **Data Integrity**
   - NOT NULL constraints for required fields
   - CHECK constraints for valid statuses and frequencies
   - Foreign key constraints with CASCADE delete
   - Unique constraints on invoice numbers

2. **Performance**
   - Comprehensive indexes on frequently queried columns
   - Separate indexes on merchant_id, customer_id, status
   - Date-based indexes for payment processing
   - Event indexes for audit trail queries

3. **Views and Functions**
   - `subscription_status_summary`: Quick overview by status
   - `calculate_next_payment_date()`: Compute next payment
   - `mark_past_due_subscriptions()`: Automatic past due detection
   - `get_subscription_metrics()`: Revenue and subscription metrics

## Billing Frequencies Supported

- Daily (every 1 day)
- Weekly (every 7 days)
- BiWeekly (every 14 days)
- Monthly (every 30 days)
- Quarterly (every 90 days)
- Annually (every 365 days)

## Subscription Statuses

- **active**: Normal operation, payments being processed
- **paused**: Temporarily paused, no automatic payments
- **pending_payment**: Payment in progress
- **past_due**: Payment failed and grace period exceeded
- **cancelled**: Subscription is terminated

## Invoice Statuses

- issued: Created and ready to send
- sent: Sent to customer
- viewed: Customer viewed invoice
- paid: Payment received
- partial: Partial payment received
- refunded: Invoice refunded
- voided: Invoice voided

## Rollback

If needed to rollback:

```sql
DROP VIEW subscription_status_summary;
DROP FUNCTION get_subscription_metrics(INTEGER);
DROP FUNCTION mark_past_due_subscriptions();
DROP FUNCTION calculate_next_payment_date(TEXT, TIMESTAMPTZ);
DROP TABLE subscription_events CASCADE;
DROP TABLE invoices CASCADE;
DROP TABLE payment_attempts CASCADE;
DROP TABLE subscriptions CASCADE;
```

## Testing

Before deployment, verify:
1. Subscription creation works
2. Payment attempt recording works
3. Invoice generation works
4. Event emission works
5. Status updates function correctly
6. Indexes are being used (EXPLAIN ANALYZE)
7. Cascade deletes work properly

## Monitoring

Track key metrics:
```sql
-- Subscription count by status
SELECT status, COUNT(*) FROM subscriptions GROUP BY status;

-- Payment success rate
SELECT status, COUNT(*) FROM payment_attempts GROUP BY status;

-- Revenue summary
SELECT SUM(amount) FROM subscriptions WHERE status = 'active';

-- Past due subscriptions
SELECT COUNT(*) FROM subscriptions WHERE status = 'past_due';
```

## Performance Optimization

- Batch payment processing for efficiency
- Event aggregation and deduplication
- Configurable retention for old records
- Index maintenance schedule

## Security Considerations

- All queries use parameterized statements
- Foreign key constraints prevent orphaned records
- Event audit trail for compliance
- Merchant/customer separation for data isolation

## Related Issues

- Issue: #280 - Create Subscription Payment Service
- Dependency: #272 (Payment Processing)
