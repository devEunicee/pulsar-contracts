-- Migration: Add full-text search indexes for events
-- Issue: #299
-- Date: 2026-06-25
-- Purpose: Implement full-text search for event metadata

-- Add search vectors to events table
ALTER TABLE IF EXISTS events
  ADD COLUMN IF NOT EXISTS search_vector TSVECTOR;

-- Create function to update search vectors for events
CREATE OR REPLACE FUNCTION update_event_search_vectors()
RETURNS TRIGGER AS $$
BEGIN
  -- Extract searchable content from event metadata
  NEW.search_vector :=
    to_tsvector('english', 
      COALESCE(NEW.event_type, '') || ' ' ||
      COALESCE(NEW.contract_id, '') || ' ' ||
      COALESCE(NEW.topics::text, '') || ' ' ||
      COALESCE(NEW.value::text, '')
    );
  
  RETURN NEW;
END
$$ LANGUAGE plpgsql;

-- Create trigger for events
DROP TRIGGER IF EXISTS events_search_vector_trigger ON events;
CREATE TRIGGER events_search_vector_trigger
BEFORE INSERT OR UPDATE ON events
FOR EACH ROW
EXECUTE FUNCTION update_event_search_vectors();

-- Create GIN index on event search vectors
CREATE INDEX IF NOT EXISTS idx_events_search_vector
  ON events USING GIN (search_vector);

-- Create index on event type and contract_id for combined searches
CREATE INDEX IF NOT EXISTS idx_events_type_contract_combined
  ON events (event_type, contract_id);

-- Improve existing indexes
CREATE INDEX IF NOT EXISTS idx_events_value_gin
  ON events USING GIN (value) WHERE value IS NOT NULL;

CREATE INDEX IF NOT EXISTS idx_events_topics_gin
  ON events USING GIN (topics);
