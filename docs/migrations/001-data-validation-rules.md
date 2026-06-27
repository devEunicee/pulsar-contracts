# Migration: Data Validation Rules

**Issue:** #302  
**Date:** 2026-06-25  
**Purpose:** Implement database-level constraints and validation rules to ensure data integrity

## Changes

### Analytics Service Schema

- **payments table:**
  - Add NOT NULL constraint to `customer_id` (currently optional)
  - Add UNIQUE constraint on `(merchant_id, customer_id, created_at)` to prevent duplicate payments
  - Add CHECK constraint to ensure `amount > 0`
  - Add CHECK constraint to ensure `created_at` is not in the future

- **refunds table:**
  - Add NOT NULL constraint to `payment_id` (currently optional)
  - Add UNIQUE constraint on `(payment_id)` to prevent duplicate refunds for same payment
  - Add CHECK constraint to ensure `amount > 0`
  - Add CHECK constraint to ensure `amount <= (SELECT amount FROM payments WHERE id = payment_id)`
  - Ensure foreign key constraint is properly enforced with ON DELETE CASCADE

### Indexer Schema

- **events table:**
  - Add UNIQUE constraint on `(ledger, tx_hash, contract_id, event_type)` to prevent duplicate events
  - Add CHECK constraint to ensure `ledger >= 0`
  - Add CHECK constraint to ensure `topics` is a valid JSONB array
  - Ensure `value` column has proper JSONB validation

## Rollback

If needed to rollback, simply drop the constraints created by this migration.

## Related Issues

- Depends on: #293 (Schema)
- Part of: Data Integrity Initiative
