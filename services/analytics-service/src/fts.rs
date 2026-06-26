/// Full-text search service for Pulsar
/// Provides search capabilities for merchants, payments, and events

use serde::{Deserialize, Serialize};
use sqlx::PgPool;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult<T> {
    pub results: Vec<T>,
    pub total_count: usize,
    pub query: String,
    pub search_time_ms: u128,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MerchantSearchResult {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub rank: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentSearchResult {
    pub id: i64,
    pub merchant_id: String,
    pub customer_id: Option<String>,
    pub amount: String, // Using String for NUMERIC
    pub rank: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventSearchResult {
    pub id: i64,
    pub event_type: String,
    pub contract_id: String,
    pub ledger: i64,
    pub rank: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Synonym {
    pub term: String,
    pub synonyms: Vec<String>,
}

pub struct FullTextSearchService {
    pool: sqlx::PgPool,
}

impl FullTextSearchService {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Search merchants by name and description
    pub async fn search_merchants(
        &self,
        query: &str,
        limit: i32,
    ) -> Result<SearchResult<MerchantSearchResult>, sqlx::Error> {
        let start = std::time::Instant::now();

        let results: Vec<MerchantSearchResult> = sqlx::query_as(
            r#"
            SELECT 
                id,
                name,
                description,
                ts_rank(combined_search_vector, plainto_tsquery('english_config', $1)) as rank
            FROM merchants
            WHERE combined_search_vector @@ plainto_tsquery('english_config', $1)
            ORDER BY rank DESC, name ASC
            LIMIT $2
            "#,
        )
        .bind(query)
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        let elapsed = start.elapsed().as_millis();

        Ok(SearchResult {
            total_count: results.len(),
            results,
            query: query.to_string(),
            search_time_ms: elapsed,
        })
    }

    /// Search payments by metadata
    pub async fn search_payments(
        &self,
        query: &str,
        limit: i32,
    ) -> Result<SearchResult<PaymentSearchResult>, sqlx::Error> {
        let start = std::time::Instant::now();

        let results: Vec<PaymentSearchResult> = sqlx::query_as(
            r#"
            SELECT 
                id,
                merchant_id,
                customer_id,
                amount::text as amount,
                ts_rank(metadata_search_vector, plainto_tsquery('english_config', $1)) as rank
            FROM payments
            WHERE metadata_search_vector @@ plainto_tsquery('english_config', $1)
            ORDER BY rank DESC, created_at DESC
            LIMIT $2
            "#,
        )
        .bind(query)
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        let elapsed = start.elapsed().as_millis();

        Ok(SearchResult {
            total_count: results.len(),
            results,
            query: query.to_string(),
            search_time_ms: elapsed,
        })
    }

    /// Search events
    pub async fn search_events(
        &self,
        query: &str,
        limit: i32,
    ) -> Result<SearchResult<EventSearchResult>, sqlx::Error> {
        let start = std::time::Instant::now();

        let results: Vec<EventSearchResult> = sqlx::query_as(
            r#"
            SELECT 
                id,
                event_type,
                contract_id,
                ledger,
                ts_rank(search_vector, plainto_tsquery('english', $1)) as rank
            FROM events
            WHERE search_vector @@ plainto_tsquery('english', $1)
            ORDER BY rank DESC, created_at DESC
            LIMIT $2
            "#,
        )
        .bind(query)
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        let elapsed = start.elapsed().as_millis();

        Ok(SearchResult {
            total_count: results.len(),
            results,
            query: query.to_string(),
            search_time_ms: elapsed,
        })
    }

    /// Add or update a synonym for search term expansion
    pub async fn add_synonym(
        &self,
        term: &str,
        synonyms: &[&str],
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            INSERT INTO fts_synonyms (term, synonyms, created_at, updated_at)
            VALUES ($1, $2, NOW(), NOW())
            ON CONFLICT (term) DO UPDATE
            SET synonyms = $2, updated_at = NOW()
            "#,
        )
        .bind(term)
        .bind(synonyms)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Get all synonyms
    pub async fn get_synonyms(&self) -> Result<Vec<Synonym>, sqlx::Error> {
        let synonyms: Vec<Synonym> = sqlx::query_as(
            r#"
            SELECT term, synonyms FROM fts_synonyms ORDER BY term
            "#,
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(synonyms)
    }

    /// Get search statistics
    pub async fn get_search_stats(
        &self,
        days_back: i32,
    ) -> Result<Vec<SearchStats>, sqlx::Error> {
        let stats = sqlx::query_as(
            r#"
            SELECT 
                search_term,
                search_type,
                result_count,
                avg_rank,
                executed_at
            FROM fts_search_stats
            WHERE executed_at > NOW() - ($1 || ' days')::INTERVAL
            ORDER BY executed_at DESC
            "#,
        )
        .bind(days_back)
        .fetch_all(&self.pool)
        .await?;

        Ok(stats)
    }

    /// Rebuild all search vectors
    pub async fn rebuild_search_vectors(&self) -> Result<(), sqlx::Error> {
        sqlx::query("SELECT rebuild_all_search_vectors()")
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    /// Maintain search indexes
    pub async fn maintain_indexes(&self) -> Result<(), sqlx::Error> {
        sqlx::query("SELECT maintain_search_indexes()")
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    /// Advanced search with multiple criteria
    pub async fn advanced_search(
        &self,
        merchant_query: Option<&str>,
        payment_query: Option<&str>,
        event_query: Option<&str>,
        limit: i32,
    ) -> Result<AdvancedSearchResults, sqlx::Error> {
        let mut merchants = Vec::new();
        let mut payments = Vec::new();
        let mut events = Vec::new();

        if let Some(q) = merchant_query {
            merchants = self.search_merchants(q, limit / 3).await?.results;
        }

        if let Some(q) = payment_query {
            payments = self.search_payments(q, limit / 3).await?.results;
        }

        if let Some(q) = event_query {
            events = self.search_events(q, limit / 3).await?.results;
        }

        Ok(AdvancedSearchResults {
            merchants,
            payments,
            events,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct SearchStats {
    pub search_term: String,
    pub search_type: String,
    pub result_count: i32,
    pub avg_rank: Option<f64>,
    pub executed_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdvancedSearchResults {
    pub merchants: Vec<MerchantSearchResult>,
    pub payments: Vec<PaymentSearchResult>,
    pub events: Vec<EventSearchResult>,
}

impl sqlx::FromRow<'_, sqlx::postgres::PgRow> for MerchantSearchResult {
    fn from_row(row: &sqlx::postgres::PgRow) -> Result<Self, sqlx::Error> {
        use sqlx::Row;
        Ok(MerchantSearchResult {
            id: row.try_get("id")?,
            name: row.try_get("name")?,
            description: row.try_get("description")?,
            rank: row.try_get("rank")?,
        })
    }
}

impl sqlx::FromRow<'_, sqlx::postgres::PgRow> for PaymentSearchResult {
    fn from_row(row: &sqlx::postgres::PgRow) -> Result<Self, sqlx::Error> {
        use sqlx::Row;
        Ok(PaymentSearchResult {
            id: row.try_get("id")?,
            merchant_id: row.try_get("merchant_id")?,
            customer_id: row.try_get("customer_id")?,
            amount: row.try_get("amount")?,
            rank: row.try_get("rank")?,
        })
    }
}

impl sqlx::FromRow<'_, sqlx::postgres::PgRow> for EventSearchResult {
    fn from_row(row: &sqlx::postgres::PgRow) -> Result<Self, sqlx::Error> {
        use sqlx::Row;
        Ok(EventSearchResult {
            id: row.try_get("id")?,
            event_type: row.try_get("event_type")?,
            contract_id: row.try_get("contract_id")?,
            ledger: row.try_get("ledger")?,
            rank: row.try_get("rank")?,
        })
    }
}

impl sqlx::FromRow<'_, sqlx::postgres::PgRow> for Synonym {
    fn from_row(row: &sqlx::postgres::PgRow) -> Result<Self, sqlx::Error> {
        use sqlx::Row;
        Ok(Synonym {
            term: row.try_get("term")?,
            synonyms: row.try_get("synonyms")?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore]
    async fn test_search_merchants() {
        // This would require a test database
        // let pool = sqlx::postgres::PgPoolOptions::new()
        //     .connect("postgresql://localhost/test")
        //     .await
        //     .unwrap();
        //
        // let service = FullTextSearchService::new(pool);
        // let results = service.search_merchants("payment", 10).await.unwrap();
        // assert!(!results.results.is_empty());
    }
}
