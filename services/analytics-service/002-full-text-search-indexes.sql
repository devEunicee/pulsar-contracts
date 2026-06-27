-- Migration: Add full-text search indexes
-- Issue: #299
-- Date: 2026-06-25
-- Purpose: Implement full-text search for merchant names, descriptions, and transaction metadata

-- Create dictionary for stemming configuration
CREATE TEXT SEARCH DICTIONARY english_dict (
  TEMPLATE = snowball,
  LANGUAGE = english
);

CREATE TEXT SEARCH CONFIGURATION english_config (
  COPY = english
);

ALTER TEXT SEARCH CONFIGURATION english_config
  ALTER MAPPING FOR
    asciiword,
    asciihword,
    hword_asciipart,
    word,
    hword,
    hword_part
  WITH english_dict;

-- Merchants table - add full-text search capability
-- First, add columns for storing generated vectors
ALTER TABLE IF EXISTS merchants
  ADD COLUMN IF NOT EXISTS name_search_vector TSVECTOR,
  ADD COLUMN IF NOT EXISTS description_search_vector TSVECTOR,
  ADD COLUMN IF NOT EXISTS combined_search_vector TSVECTOR;

-- Create function to update search vectors for merchants
CREATE OR REPLACE FUNCTION update_merchant_search_vectors()
RETURNS TRIGGER AS $$
BEGIN
  NEW.name_search_vector :=
    to_tsvector('english_config', COALESCE(NEW.name, ''));
  
  NEW.description_search_vector :=
    to_tsvector('english_config', COALESCE(NEW.description, ''));
  
  NEW.combined_search_vector :=
    to_tsvector('english_config', COALESCE(NEW.name, '') || ' ' || COALESCE(NEW.description, ''));
  
  RETURN NEW;
END
$$ LANGUAGE plpgsql;

-- Create trigger for merchants
DROP TRIGGER IF EXISTS merchants_search_vector_trigger ON merchants;
CREATE TRIGGER merchants_search_vector_trigger
BEFORE INSERT OR UPDATE ON merchants
FOR EACH ROW
EXECUTE FUNCTION update_merchant_search_vectors();

-- Create indexes on search vectors
CREATE INDEX IF NOT EXISTS idx_merchants_name_search
  ON merchants USING GIN (name_search_vector);

CREATE INDEX IF NOT EXISTS idx_merchants_description_search
  ON merchants USING GIN (description_search_vector);

CREATE INDEX IF NOT EXISTS idx_merchants_combined_search
  ON merchants USING GIN (combined_search_vector);

-- Payments table - add full-text search on transaction metadata
ALTER TABLE IF EXISTS payments
  ADD COLUMN IF NOT EXISTS metadata_search_vector TSVECTOR,
  ADD COLUMN IF NOT EXISTS metadata JSONB DEFAULT '{}';

-- Create function to update search vectors for payments
CREATE OR REPLACE FUNCTION update_payment_search_vectors()
RETURNS TRIGGER AS $$
BEGIN
  -- Extract searchable metadata (order_id, reference, etc.)
  NEW.metadata_search_vector :=
    to_tsvector('english_config', 
      COALESCE(NEW.metadata->>'order_id', '') || ' ' ||
      COALESCE(NEW.metadata->>'reference', '') || ' ' ||
      COALESCE(NEW.metadata->>'description', '')
    );
  
  RETURN NEW;
END
$$ LANGUAGE plpgsql;

-- Create trigger for payments
DROP TRIGGER IF EXISTS payments_search_vector_trigger ON payments;
CREATE TRIGGER payments_search_vector_trigger
BEFORE INSERT OR UPDATE ON payments
FOR EACH ROW
EXECUTE FUNCTION update_payment_search_vectors();

-- Create index on payment metadata search vectors
CREATE INDEX IF NOT EXISTS idx_payments_metadata_search
  ON payments USING GIN (metadata_search_vector);

-- Create index on payment metadata for filtering
CREATE INDEX IF NOT EXISTS idx_payments_metadata
  ON payments USING GIN (metadata);

-- Full-text search statistics table for query ranking
CREATE TABLE IF NOT EXISTS fts_search_stats (
  id SERIAL PRIMARY KEY,
  search_term TEXT NOT NULL,
  search_type TEXT NOT NULL,
  result_count INTEGER NOT NULL DEFAULT 0,
  avg_rank NUMERIC(8,4),
  executed_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  UNIQUE(search_term, search_type)
);

CREATE INDEX IF NOT EXISTS idx_fts_search_stats_search_term
  ON fts_search_stats(search_term);

CREATE INDEX IF NOT EXISTS idx_fts_search_stats_executed_at
  ON fts_search_stats(executed_at DESC);

-- Synonyms table for search expansion
CREATE TABLE IF NOT EXISTS fts_synonyms (
  id SERIAL PRIMARY KEY,
  term TEXT NOT NULL UNIQUE,
  synonyms TEXT[] NOT NULL,
  created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_fts_synonyms_term
  ON fts_synonyms(term);

-- Function to rebuild all search vectors
CREATE OR REPLACE FUNCTION rebuild_all_search_vectors()
RETURNS void AS $$
BEGIN
  UPDATE merchants SET name_search_vector = to_tsvector('english_config', COALESCE(name, ''));
  UPDATE merchants SET description_search_vector = to_tsvector('english_config', COALESCE(description, ''));
  UPDATE merchants SET combined_search_vector = to_tsvector('english_config', COALESCE(name, '') || ' ' || COALESCE(description, ''));
  UPDATE payments SET metadata_search_vector = to_tsvector('english_config', 
    COALESCE(metadata->>'order_id', '') || ' ' ||
    COALESCE(metadata->>'reference', '') || ' ' ||
    COALESCE(metadata->>'description', '')
  );
  RAISE NOTICE 'All search vectors rebuilt';
END;
$$ LANGUAGE plpgsql;
