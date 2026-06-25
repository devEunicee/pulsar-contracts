-- Migration: Add data validation rules to analytics service schema
-- Issue: #302
-- Date: 2026-06-25
-- Purpose: Implement database-level constraints to ensure data integrity

-- Analytics Service: Payments Table Constraints
ALTER TABLE IF EXISTS payments
  ADD CONSTRAINT ck_payments_amount_positive CHECK (amount > 0),
  ADD CONSTRAINT ck_payments_no_future_dates CHECK (created_at <= NOW()),
  ALTER COLUMN customer_id SET NOT NULL;

-- Add unique constraint on payment combinations (merchant, customer, timestamp)
ALTER TABLE IF EXISTS payments
  ADD CONSTRAINT uq_payments_merchant_customer_date UNIQUE (merchant_id, customer_id, created_at);

-- Analytics Service: Refunds Table Constraints
ALTER TABLE IF EXISTS refunds
  ALTER COLUMN payment_id SET NOT NULL;

-- Add unique constraint to prevent duplicate refunds per payment
ALTER TABLE IF EXISTS refunds
  ADD CONSTRAINT uq_refunds_payment_id UNIQUE (payment_id);

-- Add check constraint for positive refund amounts
ALTER TABLE IF EXISTS refunds
  ADD CONSTRAINT ck_refunds_amount_positive CHECK (amount > 0);

-- Add check constraint to ensure refund amount doesn't exceed original payment
ALTER TABLE IF EXISTS refunds
  ADD CONSTRAINT ck_refunds_amount_valid CHECK (
    amount <= (SELECT amount FROM payments WHERE id = refunds.payment_id)
  );

-- Ensure foreign key constraint is properly enforced
ALTER TABLE IF EXISTS refunds
  DROP CONSTRAINT IF EXISTS refunds_payment_id_fkey;

ALTER TABLE IF EXISTS refunds
  ADD CONSTRAINT fk_refunds_payment_id 
    FOREIGN KEY (payment_id) 
    REFERENCES payments(id) 
    ON DELETE CASCADE;

-- Indexer: Events Table Constraints
ALTER TABLE IF EXISTS events
  ADD CONSTRAINT ck_events_ledger_non_negative CHECK (ledger >= 0);

-- Add unique constraint on event combinations to prevent duplicates
ALTER TABLE IF EXISTS events
  ADD CONSTRAINT uq_events_ledger_tx_contract_type UNIQUE (ledger, tx_hash, contract_id, event_type);

-- Add check constraint to ensure topics is a valid array
ALTER TABLE IF EXISTS events
  ADD CONSTRAINT ck_events_topics_valid_array CHECK (jsonb_typeof(topics) = 'array');
