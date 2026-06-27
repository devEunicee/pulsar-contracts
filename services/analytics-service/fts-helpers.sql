-- Documentation and setup for full-text search
-- This file contains helper functions and examples

-- Example: Search for merchants by name
-- SELECT id, name, RANK(combined_search_vector <=> 'payment'::tsquery) as rank
-- FROM merchants
-- WHERE combined_search_vector @@ plainto_tsquery('english_config', 'payment')
-- ORDER BY rank LIMIT 10;

-- Function to search merchants by name and description
CREATE OR REPLACE FUNCTION search_merchants(
  search_term TEXT,
  max_results INTEGER DEFAULT 20
)
RETURNS TABLE (
  merchant_id TEXT,
  name TEXT,
  description TEXT,
  rank REAL
) AS $$
DECLARE
  query_vector TSQUERY;
BEGIN
  -- Convert search term to tsquery
  query_vector := plainto_tsquery('english_config', search_term);
  
  -- Return matching merchants ranked by relevance
  RETURN QUERY
  SELECT 
    m.id,
    m.name,
    m.description,
    ts_rank(m.combined_search_vector, query_vector) as rank
  FROM merchants m
  WHERE m.combined_search_vector @@ query_vector
  ORDER BY rank DESC, m.name ASC
  LIMIT max_results;
END;
$$ LANGUAGE plpgsql;

-- Function to search payments by metadata
CREATE OR REPLACE FUNCTION search_payments(
  search_term TEXT,
  max_results INTEGER DEFAULT 20
)
RETURNS TABLE (
  payment_id INTEGER,
  merchant_id TEXT,
  customer_id TEXT,
  amount NUMERIC,
  rank REAL
) AS $$
DECLARE
  query_vector TSQUERY;
BEGIN
  query_vector := plainto_tsquery('english_config', search_term);
  
  RETURN QUERY
  SELECT 
    p.id,
    p.merchant_id,
    p.customer_id,
    p.amount,
    ts_rank(p.metadata_search_vector, query_vector) as rank
  FROM payments p
  WHERE p.metadata_search_vector @@ query_vector
  ORDER BY rank DESC, p.created_at DESC
  LIMIT max_results;
END;
$$ LANGUAGE plpgsql;

-- Function to search events
CREATE OR REPLACE FUNCTION search_events(
  search_term TEXT,
  max_results INTEGER DEFAULT 50
)
RETURNS TABLE (
  event_id INTEGER,
  event_type TEXT,
  contract_id TEXT,
  ledger BIGINT,
  rank REAL
) AS $$
DECLARE
  query_vector TSQUERY;
BEGIN
  query_vector := plainto_tsquery('english', search_term);
  
  RETURN QUERY
  SELECT 
    e.id,
    e.event_type,
    e.contract_id,
    e.ledger,
    ts_rank(e.search_vector, query_vector) as rank
  FROM events e
  WHERE e.search_vector @@ query_vector
  ORDER BY rank DESC, e.created_at DESC
  LIMIT max_results;
END;
$$ LANGUAGE plpgsql;

-- Function to add synonyms
CREATE OR REPLACE FUNCTION add_fts_synonym(
  term TEXT,
  synonyms_list TEXT[]
)
RETURNS void AS $$
BEGIN
  INSERT INTO fts_synonyms (term, synonyms, created_at, updated_at)
  VALUES (term, synonyms_list, NOW(), NOW())
  ON CONFLICT (term) DO UPDATE
  SET synonyms = synonyms_list, updated_at = NOW();
END;
$$ LANGUAGE plpgsql;

-- Function to get search performance statistics
CREATE OR REPLACE FUNCTION get_search_stats(
  days_back INTEGER DEFAULT 7
)
RETURNS TABLE (
  search_term TEXT,
  search_type TEXT,
  result_count INTEGER,
  avg_rank NUMERIC,
  executed_at TIMESTAMPTZ
) AS $$
BEGIN
  RETURN QUERY
  SELECT 
    fts_search_stats.search_term,
    fts_search_stats.search_type,
    fts_search_stats.result_count,
    fts_search_stats.avg_rank,
    fts_search_stats.executed_at
  FROM fts_search_stats
  WHERE fts_search_stats.executed_at > NOW() - (days_back || ' days')::INTERVAL
  ORDER BY fts_search_stats.executed_at DESC;
END;
$$ LANGUAGE plpgsql;

-- Function to maintain indexes (VACUUM and ANALYZE)
CREATE OR REPLACE FUNCTION maintain_search_indexes()
RETURNS void AS $$
BEGIN
  VACUUM ANALYZE merchants;
  VACUUM ANALYZE payments;
  VACUUM ANALYZE events;
  REINDEX INDEX CONCURRENTLY IF EXISTS idx_merchants_combined_search;
  REINDEX INDEX CONCURRENTLY IF EXISTS idx_payments_metadata_search;
  REINDEX INDEX CONCURRENTLY IF EXISTS idx_events_search_vector;
  RAISE NOTICE 'Search indexes maintenance completed';
END;
$$ LANGUAGE plpgsql;
