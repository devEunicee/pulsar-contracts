# Migration: Full-Text Search Indexes

**Issue:** #299  
**Date:** 2026-06-25  
**Purpose:** Add full-text search indexes for efficient searching

## Changes

### Analytics Service

1. **Merchants table enhancements:**
   - Add `name_search_vector` TSVECTOR column
   - Add `description_search_vector` TSVECTOR column
   - Add `combined_search_vector` TSVECTOR column
   - Create GIN indexes on all three search vectors

2. **Payments table enhancements:**
   - Add `metadata_search_vector` TSVECTOR column
   - Add `metadata` JSONB column for flexible metadata storage
   - Create GIN index on metadata_search_vector
   - Create GIN index on metadata column

3. **Search infrastructure:**
   - Create `fts_synonyms` table for synonym management
   - Create `fts_search_stats` table for search analytics
   - Create triggers to automatically update search vectors

4. **Functions:**
   - `update_merchant_search_vectors()`: Auto-update on merchant changes
   - `update_payment_search_vectors()`: Auto-update on payment changes
   - `search_merchants()`: Search merchants by name/description
   - `search_payments()`: Search payments by metadata
   - `rebuild_all_search_vectors()`: Bulk rebuild search vectors
   - `add_fts_synonym()`: Manage search synonyms
   - `maintain_search_indexes()`: Maintenance routine

### Indexer Service

1. **Events table enhancements:**
   - Add `search_vector` TSVECTOR column
   - Create GIN index on search_vector
   - Optimize existing type/contract indexes

2. **Functions:**
   - `update_event_search_vectors()`: Auto-update on event changes
   - `search_events()`: Search events by metadata

## Performance

- GIN indexes provide 2-10x faster full-text searches vs sequential scans
- Automatic trigger updates maintain index currency
- Configurable batch maintenance for large datasets
- Index statistics maintained regularly

## Rollback

If needed to rollback, run:

```sql
-- Drop triggers and vectors from merchants
DROP TRIGGER merchants_search_vector_trigger ON merchants;
ALTER TABLE merchants DROP COLUMN combined_search_vector;
ALTER TABLE merchants DROP COLUMN description_search_vector;
ALTER TABLE merchants DROP COLUMN name_search_vector;

-- Drop triggers and vectors from payments
DROP TRIGGER payments_search_vector_trigger ON payments;
ALTER TABLE payments DROP COLUMN metadata_search_vector;
ALTER TABLE payments DROP COLUMN metadata;

-- Drop triggers and vectors from events
DROP TRIGGER events_search_vector_trigger ON events;
ALTER TABLE events DROP COLUMN search_vector;

-- Drop support tables
DROP TABLE fts_search_stats;
DROP TABLE fts_synonyms;

-- Drop functions
DROP FUNCTION search_merchants(TEXT, INTEGER);
DROP FUNCTION search_payments(TEXT, INTEGER);
DROP FUNCTION search_events(TEXT, INTEGER);
DROP FUNCTION add_fts_synonym(TEXT, TEXT[]);
DROP FUNCTION get_search_stats(INTEGER);
DROP FUNCTION maintain_search_indexes();
DROP FUNCTION rebuild_all_search_vectors();
DROP FUNCTION update_merchant_search_vectors();
DROP FUNCTION update_payment_search_vectors();
DROP FUNCTION update_event_search_vectors();
```

## Testing

Before deploying to production, verify:
1. Search vectors are populated
2. Indexes are being used (EXPLAIN ANALYZE)
3. Synonyms work correctly
4. Trigger updates function properly
5. Ranking produces relevant results

## Maintenance

Regular maintenance tasks:
- Weekly: `SELECT maintain_search_indexes();`
- Monthly: Check index size and fragmentation
- Quarterly: Rebuild large indexes if needed

## Related Issues

- Issue: #299 - Implement Full-Text Search Indexes
- Dependency: #275 (Search API)
