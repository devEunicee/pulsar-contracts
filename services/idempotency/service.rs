/// Idempotency Key Support — Issue #292
///
/// Ensures duplicate payment/refund requests return the same cached response
/// for a configurable TTL window. Thread-safe via a store trait.

use std::hash::{Hash, Hasher};
use std::collections::hash_map::DefaultHasher;

/// Default TTL: 24 hours (seconds).
pub const DEFAULT_TTL_SECONDS: u64 = 86_400;

/// A cached entry stored against an idempotency key.
#[derive(Debug, Clone)]
pub struct IdempotentEntry {
    pub key: String,
    pub operation: String,     // "payment" | "refund"
    pub request_hash: u64,     // hash of the canonical request body
    pub response_body: String, // JSON-serialised response
    pub created_at: u64,
    pub expires_at: u64,
}

/// Result of an idempotency check.
pub enum IdempotencyResult {
    /// No prior entry — caller should execute the operation and then call `store`.
    New,
    /// A cached response exists and the request body matches — return it as-is.
    Duplicate(String),
    /// Same key, different request body — this is a conflict.
    Conflict,
}

/// Storage backend trait — implement with your DB/Redis layer.
pub trait IdempotencyStore: Send + Sync {
    fn get(&self, key: &str) -> Option<IdempotentEntry>;
    fn set(&mut self, entry: IdempotentEntry);
    /// Remove all entries whose `expires_at` ≤ `now`. Called periodically.
    fn evict_expired(&mut self, now: u64);
}

pub struct IdempotencyService<S: IdempotencyStore> {
    store: S,
    ttl: u64,
}

impl<S: IdempotencyStore> IdempotencyService<S> {
    pub fn new(store: S) -> Self {
        Self { store, ttl: DEFAULT_TTL_SECONDS }
    }

    pub fn with_ttl(store: S, ttl_seconds: u64) -> Self {
        Self { store, ttl: ttl_seconds }
    }

    /// Check whether `idempotency_key` has been seen before for `operation`.
    ///
    /// `request_body` — canonical string representation of the incoming request
    ///   (e.g. JSON-serialised after sorting keys).
    pub fn check(
        &self,
        idempotency_key: &str,
        operation: &str,
        request_body: &str,
        now: u64,
    ) -> IdempotencyResult {
        let Some(entry) = self.store.get(idempotency_key) else {
            return IdempotencyResult::New;
        };
        // Expired entries are treated as new.
        if entry.expires_at <= now {
            return IdempotencyResult::New;
        }
        if entry.operation != operation {
            return IdempotencyResult::Conflict;
        }
        let incoming_hash = hash_str(request_body);
        if entry.request_hash != incoming_hash {
            return IdempotencyResult::Conflict;
        }
        IdempotencyResult::Duplicate(entry.response_body)
    }

    /// Store the response for a successfully executed operation.
    ///
    /// Must be called **after** the operation succeeds so partial failures
    /// don't pollute the cache.
    pub fn store(
        &mut self,
        idempotency_key: String,
        operation: String,
        request_body: &str,
        response_body: String,
        now: u64,
    ) {
        self.store.set(IdempotentEntry {
            key: idempotency_key,
            operation,
            request_hash: hash_str(request_body),
            response_body,
            created_at: now,
            expires_at: now + self.ttl,
        });
    }

    /// Evict entries that have passed their TTL. Call from a background job.
    pub fn cleanup(&mut self, now: u64) {
        self.store.evict_expired(now);
    }
}

fn hash_str(s: &str) -> u64 {
    let mut h = DefaultHasher::new();
    s.hash(&mut h);
    h.finish()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    struct MemStore(HashMap<String, IdempotentEntry>);

    impl MemStore {
        fn new() -> Self { Self(HashMap::new()) }
    }

    impl IdempotencyStore for MemStore {
        fn get(&self, key: &str) -> Option<IdempotentEntry> { self.0.get(key).cloned() }
        fn set(&mut self, entry: IdempotentEntry) { self.0.insert(entry.key.clone(), entry); }
        fn evict_expired(&mut self, now: u64) { self.0.retain(|_, e| e.expires_at > now); }
    }

    fn svc() -> IdempotencyService<MemStore> {
        IdempotencyService::new(MemStore::new())
    }

    #[test]
    fn new_key_returns_new() {
        let s = svc();
        assert!(matches!(s.check("k1", "payment", r#"{"a":1}"#, 1000), IdempotencyResult::New));
    }

    #[test]
    fn duplicate_request_returns_cached_response() {
        let mut s = svc();
        s.store("k1".into(), "payment".into(), r#"{"a":1}"#, r#"{"ok":true}"#.into(), 1000);
        match s.check("k1", "payment", r#"{"a":1}"#, 1001) {
            IdempotencyResult::Duplicate(resp) => assert_eq!(resp, r#"{"ok":true}"#),
            _ => panic!("expected Duplicate"),
        }
    }

    #[test]
    fn different_body_is_conflict() {
        let mut s = svc();
        s.store("k1".into(), "payment".into(), r#"{"a":1}"#, r#"{"ok":true}"#.into(), 1000);
        assert!(matches!(s.check("k1", "payment", r#"{"a":2}"#, 1001), IdempotencyResult::Conflict));
    }

    #[test]
    fn different_operation_is_conflict() {
        let mut s = svc();
        s.store("k1".into(), "payment".into(), r#"{"a":1}"#, r#"{"ok":true}"#.into(), 1000);
        assert!(matches!(s.check("k1", "refund", r#"{"a":1}"#, 1001), IdempotencyResult::Conflict));
    }

    #[test]
    fn expired_entry_treated_as_new() {
        let mut s = IdempotencyService::with_ttl(MemStore::new(), 100);
        s.store("k1".into(), "payment".into(), r#"{"a":1}"#, r#"{"ok":true}"#.into(), 1000);
        // now = 1101, ttl = 100 → expires_at = 1100, so expired
        assert!(matches!(s.check("k1", "payment", r#"{"a":1}"#, 1101), IdempotencyResult::New));
    }

    #[test]
    fn cleanup_removes_expired_entries() {
        let mut s = IdempotencyService::with_ttl(MemStore::new(), 50);
        s.store("k1".into(), "payment".into(), "b", "r".into(), 1000);
        s.store("k2".into(), "payment".into(), "b", "r".into(), 2000);
        s.cleanup(1060); // k1 expired (1050 < 1060), k2 alive
        assert!(matches!(s.check("k1", "payment", "b", 1060), IdempotencyResult::New));
        assert!(matches!(s.check("k2", "payment", "b", 2001), IdempotencyResult::Duplicate(_)));
    }

    #[test]
    fn concurrent_same_key_same_body_returns_duplicate() {
        let mut s = svc();
        s.store("k1".into(), "payment".into(), r#"{"order":"X"}"#, r#"{"id":"X"}"#.into(), 5000);
        // Simulated second concurrent request arriving slightly later
        let r1 = s.check("k1", "payment", r#"{"order":"X"}"#, 5001);
        let r2 = s.check("k1", "payment", r#"{"order":"X"}"#, 5002);
        assert!(matches!(r1, IdempotencyResult::Duplicate(_)));
        assert!(matches!(r2, IdempotencyResult::Duplicate(_)));
    }
}
