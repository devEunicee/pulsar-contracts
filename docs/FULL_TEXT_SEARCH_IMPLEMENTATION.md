# Full-Text Search Implementation

**Issue:** #299  
**Date:** 2026-06-25  
**Purpose:** Implement full-text search indexes for enhanced search capabilities

## Overview

Full-text search provides efficient, language-aware searching across merchant data, payment metadata, and event information in the Pulsar system.

## Features

### Full-Text Indexes

#### Merchant Search
- **Index on merchant names**: Fast search by merchant name
- **Index on descriptions**: Search merchant descriptions
- **Combined index**: Search both name and description together

#### Payment Search
- **Index on transaction metadata**: Search by order ID, reference, description
- **Metadata storage**: JSON field for flexible metadata storage
- **Order-based search**: Find payments by order reference

#### Event Search
- **Index on event data**: Search events by type, contract, and metadata
- **JSON search**: Full-text search through event topics and values
- **Combined index**: Search across multiple event fields

### Language-Specific Stemming

- Uses PostgreSQL's native English stemming dictionary (Snowball)
- Automatically handles word variations (e.g., "payment", "payments", "paying")
- Configurable for different languages
- Improves recall and relevance

### Synonym Support

- Synonym management table (`fts_synonyms`)
- Expandable search terms (e.g., "pay" → "payment", "remit")
- Easy synonym addition and updates
- Used to improve search recall

### Search Ranking

- BM25-like ranking via PostgreSQL's `ts_rank` function
- Normalized ranking scores (0-1)
- Results ordered by relevance
- Tie-breaking by recency or name

### Index Maintenance

- Automatic vector updates via triggers
- Bulk rebuild function for reindexing
- Maintenance function for VACUUM and ANALYZE
- Scheduled maintenance recommendations

## Database Schema

### New Columns

**merchants table:**
- `name_search_vector`: TSVECTOR for name search
- `description_search_vector`: TSVECTOR for description search
- `combined_search_vector`: TSVECTOR for combined search

**payments table:**
- `metadata_search_vector`: TSVECTOR for metadata search
- `metadata`: JSONB for flexible metadata storage

**events table:**
- `search_vector`: TSVECTOR for event search

### New Tables

**fts_synonyms:** Store search term synonyms
- `term`: Search term
- `synonyms`: Array of synonym terms

**fts_search_stats:** Track search performance
- `search_term`: The search query
- `search_type`: Type of search (merchant, payment, event)
- `result_count`: Number of results returned
- `avg_rank`: Average ranking score
- `executed_at`: When search was performed

## Search Functions

### Search Merchants
```sql
SELECT * FROM search_merchants('payment', 20);
```

Parameters:
- `search_term`: Text to search for
- `max_results`: Maximum results to return (default: 20)

Returns: merchant_id, name, description, rank

### Search Payments
```sql
SELECT * FROM search_payments('invoice-2024-001', 20);
```

Parameters:
- `search_term`: Text to search for
- `max_results`: Maximum results to return (default: 20)

Returns: payment_id, merchant_id, customer_id, amount, rank

### Search Events
```sql
SELECT * FROM search_events('PaymentProcessed', 50);
```

Parameters:
- `search_term`: Text to search for
- `max_results`: Maximum results to return (default: 50)

Returns: event_id, event_type, contract_id, ledger, rank

## Rust API

### Initialize Service
```rust
use analytics_service::fts::FullTextSearchService;

let service = FullTextSearchService::new(pool);
```

### Search Merchants
```rust
let results = service.search_merchants("payment", 20).await?;
println!("Found {} merchants in {:.2}ms", 
    results.total_count, 
    results.search_time_ms);
```

### Search Payments
```rust
let results = service.search_payments("invoice-001", 20).await?;
for payment in results.results {
    println!("{}: {} (rank: {})", 
        payment.id, 
        payment.amount, 
        payment.rank);
}
```

### Search Events
```rust
let results = service.search_events("PaymentProcessed", 50).await?;
for event in results.results {
    println!("{}: {} on ledger {}", 
        event.event_type, 
        event.contract_id, 
        event.ledger);
}
```

### Manage Synonyms
```rust
// Add synonyms
service.add_synonym("pay", &["payment", "remit", "transfer"]).await?;

// Get all synonyms
let synonyms = service.get_synonyms().await?;
```

### Maintenance
```rust
// Rebuild all search vectors
service.rebuild_search_vectors().await?;

// Maintain indexes (VACUUM, ANALYZE, REINDEX)
service.maintain_indexes().await?;
```

### Search Statistics
```rust
let stats = service.get_search_stats(7).await?; // Last 7 days
for stat in stats {
    println!("{}: {} results (avg rank: {})", 
        stat.search_term, 
        stat.result_count, 
        stat.avg_rank.unwrap_or(0.0));
}
```

## Performance Considerations

### Index Types
- GIN indexes for best full-text search performance
- B-tree indexes for combined queries
- JSONB GIN index for metadata queries

### Query Optimization
- Use `plainto_tsquery` for simple queries
- Use `websearch_to_tsquery` for web-like syntax
- Leverage `ts_rank` for relevance sorting

### Maintenance Schedule
- Daily: Automatic trigger updates
- Weekly: VACUUM and ANALYZE (off-peak hours)
- Monthly: Full index rebuild if needed

## Examples

### Basic Search
```sql
-- Search for merchants named "Acme"
SELECT * FROM search_merchants('Acme', 10);
```

### Advanced Search with Ranking
```sql
-- Find payments with high relevance to "invoice"
SELECT * FROM search_payments('invoice', 50)
WHERE rank > 0.1;
```

### Combined Search
```sql
-- Search across all types
SELECT 
  (SELECT COUNT(*) FROM search_merchants($1, 100)) as merchant_count,
  (SELECT COUNT(*) FROM search_payments($1, 100)) as payment_count,
  (SELECT COUNT(*) FROM search_events($1, 100)) as event_count;
```

### Search with Synonyms
```sql
-- When searching "pay", also considers "payment", "remit", "transfer"
SELECT * FROM search_merchants('payment', 20);
```

## Troubleshooting

### Slow Searches
1. Check index freshness: `VACUUM ANALYZE merchants; REINDEX INDEX CONCURRENTLY idx_merchants_combined_search;`
2. Verify query: Use `EXPLAIN ANALYZE` to check plan
3. Increase work_mem if available

### Missing Results
1. Check search_vectors are populated: `SELECT COUNT(*) FROM merchants WHERE combined_search_vector IS NULL;`
2. Rebuild vectors if needed: `SELECT rebuild_all_search_vectors();`
3. Verify synonyms are configured

### Index Bloat
1. Monitor index size: `SELECT * FROM pg_stat_user_indexes WHERE relname LIKE 'idx_%search%';`
2. Run maintenance: `SELECT maintain_search_indexes();`
3. Consider partitioning for very large tables

## Acceptance Criteria Met

✅ Full-text index on merchant names/descriptions  
✅ Full-text index on transaction metadata  
✅ Search performance optimized  
✅ Language-specific stemming  
✅ Synonym support  
✅ Search ranking configuration  
✅ Regular index maintenance  

## Related Issues

- Depends on: #275 (Search API)
- Part of: Search and Discovery Initiative
