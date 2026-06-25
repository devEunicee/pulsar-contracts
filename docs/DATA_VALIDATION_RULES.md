# Data Validation Rules Implementation

## Issue: #302

This implementation adds comprehensive database-level validation rules to ensure data integrity across the Pulsar system.

## Overview

Database constraints are enforced at the database level to ensure:
- Data consistency across applications
- Prevention of invalid data entry
- Referential integrity
- Business logic compliance

## Constraints by Table

### Analytics Service - Payments Table

| Constraint | Type | Purpose |
|-----------|------|---------|
| `id` | PRIMARY KEY | Unique identifier for each payment |
| `merchant_id` | NOT NULL | Ensures every payment is associated with a merchant |
| `customer_id` | NOT NULL | Ensures every payment has a customer (made non-null in this update) |
| `amount` | CHECK (amount > 0) | Prevents zero or negative payment amounts |
| `created_at` | CHECK (created_at <= NOW()) | Prevents future-dated payments |
| `uq_payments_merchant_customer_date` | UNIQUE | Prevents duplicate payments for same merchant-customer combination on the same day |

### Analytics Service - Refunds Table

| Constraint | Type | Purpose |
|-----------|------|---------|
| `id` | PRIMARY KEY | Unique identifier for each refund |
| `payment_id` | NOT NULL, FOREIGN KEY | Links refund to original payment, ensures referential integrity |
| `payment_id` | UNIQUE | Prevents duplicate refunds for the same payment |
| `merchant_id` | NOT NULL | Ensures refund is associated with a merchant |
| `amount` | CHECK (amount > 0) | Prevents zero or negative refund amounts |
| `ck_refunds_amount_valid` | CHECK | Ensures refund amount doesn't exceed original payment amount |
| Foreign Key | ON DELETE CASCADE | Automatically removes refunds when payment is deleted |

### Indexer - Events Table

| Constraint | Type | Purpose |
|-----------|------|---------|
| `id` | PRIMARY KEY | Unique identifier for each event |
| `ledger` | NOT NULL, CHECK (ledger >= 0) | Ledger number must be non-negative |
| `tx_hash` | NOT NULL, CHECK (tx_hash != '') | Transaction hash cannot be empty |
| `contract_id` | NOT NULL, CHECK (contract_id != '') | Contract ID cannot be empty |
| `event_type` | NOT NULL, CHECK (event_type != '') | Event type cannot be empty |
| `topics` | NOT NULL, CHECK (jsonb_typeof(topics) = 'array') | Topics must be a valid JSON array |
| `uq_events_ledger_tx_contract_type` | UNIQUE | Prevents duplicate events for same ledger-tx-contract-type combination |

## Migration Strategy

Two migration scripts are provided:

1. **`001-data-validation-rules.sql`** (Analytics Service): Alters existing tables to add constraints
2. **`001-data-validation-rules.sql`** (Indexer): Alters indexer tables to add constraints

New installations use the updated `schema.sql` files which include all constraints from the start.

## Breaking Changes

- **`payments.customer_id`** is now NOT NULL (previously optional)
- Existing records with NULL `customer_id` must be updated or removed before applying this migration
- Existing refunds with `amount > payment.amount` will prevent the constraint from being applied

## Rollback

To rollback constraints:

```sql
-- Analytics Service
ALTER TABLE payments DROP CONSTRAINT ck_payments_amount_positive;
ALTER TABLE payments DROP CONSTRAINT ck_payments_no_future_dates;
ALTER TABLE payments DROP CONSTRAINT uq_payments_merchant_customer_date;
ALTER TABLE refunds DROP CONSTRAINT ck_refunds_amount_positive;
ALTER TABLE refunds DROP CONSTRAINT ck_refunds_amount_valid;
ALTER TABLE refunds DROP CONSTRAINT uq_refunds_payment_id;

-- Indexer
ALTER TABLE events DROP CONSTRAINT ck_events_ledger_non_negative;
ALTER TABLE events DROP CONSTRAINT ck_events_contract_id_not_empty;
ALTER TABLE events DROP CONSTRAINT ck_events_type_not_empty;
ALTER TABLE events DROP CONSTRAINT ck_events_tx_hash_not_empty;
ALTER TABLE events DROP CONSTRAINT ck_events_topics_valid_array;
ALTER TABLE events DROP CONSTRAINT uq_events_ledger_tx_contract_type;
```

## Testing

Before deploying to production, test that:
1. Valid data can be inserted
2. Invalid data is rejected
3. Existing data meets constraints (fix if needed)
4. Cascade deletes work as expected

## References

- Issue: #302 - Create Data Validation Rules
- Dependency: #293 (Schema)
- Related: Data Integrity Initiative
