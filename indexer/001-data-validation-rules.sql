-- Migration: Add data validation rules to indexer schema
-- Issue: #302
-- Date: 2026-06-25
-- Purpose: Implement database-level constraints to ensure data integrity

-- Indexer: Events Table Constraints
ALTER TABLE IF EXISTS events
  ADD CONSTRAINT ck_events_ledger_non_negative CHECK (ledger >= 0);

-- Add unique constraint on event combinations to prevent duplicates
ALTER TABLE IF EXISTS events
  ADD CONSTRAINT uq_events_ledger_tx_contract_type UNIQUE (ledger, tx_hash, contract_id, event_type);

-- Add check constraint to ensure topics is a valid array
ALTER TABLE IF EXISTS events
  ADD CONSTRAINT ck_events_topics_valid_array CHECK (jsonb_typeof(topics) = 'array');

-- Add check constraint to ensure contract_id is not empty
ALTER TABLE IF EXISTS events
  ADD CONSTRAINT ck_events_contract_id_not_empty CHECK (contract_id != '');

-- Add check constraint to ensure event_type is not empty
ALTER TABLE IF EXISTS events
  ADD CONSTRAINT ck_events_type_not_empty CHECK (event_type != '');

-- Add check constraint to ensure tx_hash is not empty
ALTER TABLE IF EXISTS events
  ADD CONSTRAINT ck_events_tx_hash_not_empty CHECK (tx_hash != '');
