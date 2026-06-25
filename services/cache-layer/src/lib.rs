use std::collections::{HashMap, VecDeque};
use std::sync::Mutex;
use std::time::{Duration, Instant};

#[cfg(feature = "redis-backend")]
use redis::{Commands, Connection};

pub type CacheKey = String;
pub type CacheValue = Vec<u8>;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CacheError {
    RedisError(String),
    MissingValue,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CacheMetrics {
    pub hits: u64,
    pub misses: u64,
    pub evictions: u64,
}

impl CacheMetrics {
    pub fn new() -> Self {
        Self {
            hits: 0,
            misses: 0,
            evictions: 0,
        }
    }
}

pub trait Cache: Send + Sync {
    fn get(&self, key: &CacheKey) -> Result<Option<CacheValue>, CacheError>;
    fn set(&self, key: CacheKey, value: CacheValue, ttl_seconds: u64) -> Result<(), CacheError>;
    fn remove(&self, key: &CacheKey) -> Result<(), CacheError>;
    fn metrics(&self) -> CacheMetrics;
}

struct MemoryEntry {
    value: CacheValue,
    expires_at: Instant,
}

struct MemoryState {
    store: HashMap<CacheKey, MemoryEntry>,
    order: VecDeque<CacheKey>,
}

pub struct MemoryCache {
    state: Mutex<MemoryState>,
    max_items: usize,
    metrics: Mutex<CacheMetrics>,
}

impl MemoryCache {
    pub fn new(max_items: usize) -> Self {
        Self {
            state: Mutex::new(MemoryState {
                store: HashMap::new(),
                order: VecDeque::new(),
            }),
            max_items,
            metrics: Mutex::new(CacheMetrics::new()),
        }
    }

    fn prune_if_needed(&self, state: &mut MemoryState) {
        while state.store.len() > self.max_items {
            if let Some(oldest) = state.order.pop_front() {
                state.store.remove(&oldest);
                self.metrics.lock().unwrap().evictions += 1;
            }
        }
    }
}

impl Cache for MemoryCache {
    fn get(&self, key: &CacheKey) -> Result<Option<CacheValue>, CacheError> {
        let mut state = self.state.lock().unwrap();
        let now = Instant::now();
        if let Some(entry) = state.store.get(key) {
            if now >= entry.expires_at {
                state.store.remove(key);
                state.order.retain(|k| k != key);
                self.metrics.lock().unwrap().misses += 1;
                return Ok(None);
            }
            self.metrics.lock().unwrap().hits += 1;
            return Ok(Some(entry.value.clone()));
        }
        self.metrics.lock().unwrap().misses += 1;
        Ok(None)
    }

    fn set(&self, key: CacheKey, value: CacheValue, ttl_seconds: u64) -> Result<(), CacheError> {
        let mut state = self.state.lock().unwrap();
        let expires_at = Instant::now() + Duration::from_secs(ttl_seconds);
        if !state.store.contains_key(&key) {
            state.order.push_back(key.clone());
        }
        state.store.insert(key, MemoryEntry { value, expires_at });
        self.prune_if_needed(&mut state);
        Ok(())
    }

    fn remove(&self, key: &CacheKey) -> Result<(), CacheError> {
        let mut state = self.state.lock().unwrap();
        state.store.remove(key);
        state.order.retain(|k| k != key);
        Ok(())
    }

    fn metrics(&self) -> CacheMetrics {
        self.metrics.lock().unwrap().clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use std::time::Duration;

    #[test]
    fn memory_cache_stores_and_retrieves_entries() {
        let cache = MemoryCache::new(10);
        let key = "merchant:123".to_string();
        let value = b"merchant-data".to_vec();

        cache.set(key.clone(), value.clone(), 60).expect("set should succeed");
        let loaded = cache.get(&key).expect("get should succeed");

        assert_eq!(loaded, Some(value));
        let metrics = cache.metrics();
        assert_eq!(metrics.hits, 1);
        assert_eq!(metrics.misses, 0);
    }

    #[test]
    fn memory_cache_expires_values_after_ttl() {
        let cache = MemoryCache::new(10);
        let key = "payment_stats".to_string();
        cache.set(key.clone(), b"stats".to_vec(), 1).expect("set should succeed");
        thread::sleep(Duration::from_secs(2));

        let loaded = cache.get(&key).expect("get should succeed");
        assert!(loaded.is_none());
        assert_eq!(cache.metrics().misses, 1);
    }

    #[test]
    fn memory_cache_eviction_respects_max_items() {
        let cache = MemoryCache::new(2);
        cache.set("k1".to_string(), b"v1".to_vec(), 60).unwrap();
        cache.set("k2".to_string(), b"v2".to_vec(), 60).unwrap();
        cache.set("k3".to_string(), b"v3".to_vec(), 60).unwrap();

        assert!(cache.get(&"k1".to_string()).expect("get should succeed").is_none(), "k1 should be evicted");
        assert_eq!(cache.get(&"k2".to_string()).expect("get should succeed"), Some(b"v2".to_vec()));
        assert_eq!(cache.get(&"k3".to_string()).expect("get should succeed"), Some(b"v3".to_vec()));
        assert_eq!(cache.metrics().evictions, 1);
    }

    #[test]
    fn memory_cache_remove_invalidation_works() {
        let cache = MemoryCache::new(5);
        let key = "merchant:456".to_string();
        cache.set(key.clone(), b"profile".to_vec(), 60).unwrap();
        cache.remove(&key).unwrap();

        assert!(cache.get(&key).expect("get should succeed").is_none());
    }
}

#[cfg(feature = "redis-backend")]
pub struct RedisCache {
    connection: Mutex<Connection>,
    max_items: usize,
    metrics: Mutex<CacheMetrics>,
}

#[cfg(feature = "redis-backend")]
impl RedisCache {
    pub fn new(connection_string: &str, max_items: usize) -> Result<Self, CacheError> {
        let client = redis::Client::open(connection_string).map_err(|err| CacheError::RedisError(err.to_string()))?;
        let connection = client.get_connection().map_err(|err| CacheError::RedisError(err.to_string()))?;
        Ok(Self {
            connection: Mutex::new(connection),
            max_items,
            metrics: Mutex::new(CacheMetrics::new()),
        })
    }
}

#[cfg(feature = "redis-backend")]
impl Cache for RedisCache {
    fn get(&self, key: &CacheKey) -> Result<Option<CacheValue>, CacheError> {
        let mut conn = self.connection.lock().unwrap();
        let value: Option<Vec<u8>> = conn.get(key).map_err(|err| CacheError::RedisError(err.to_string()))?;
        let mut metrics = self.metrics.lock().unwrap();
        if value.is_some() {
            metrics.hits += 1;
        } else {
            metrics.misses += 1;
        }
        Ok(value)
    }

    fn set(&self, key: CacheKey, value: CacheValue, ttl_seconds: u64) -> Result<(), CacheError> {
        let mut conn = self.connection.lock().unwrap();
        let _: () = conn.set_ex(key.clone(), value, ttl_seconds as usize).map_err(|err| CacheError::RedisError(err.to_string()))?;
        Ok(())
    }

    fn remove(&self, key: &CacheKey) -> Result<(), CacheError> {
        let mut conn = self.connection.lock().unwrap();
        let _: () = conn.del(key).map_err(|err| CacheError::RedisError(err.to_string()))?;
        Ok(())
    }

    fn metrics(&self) -> CacheMetrics {
        self.metrics.lock().unwrap().clone()
    }
}
