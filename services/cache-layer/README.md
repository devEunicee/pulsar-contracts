# cache-layer

A reusable cache layer for frequently accessed merchant and payment data.

## Features

- In-memory cache backend for development and local testing
- Optional Redis backend for production workflows
- TTL-based expiration and explicit invalidation
- Simple metrics for cache hits, misses, and evictions

## Usage

Add `cache-layer` as an internal workspace dependency and choose the desired feature flag:

- `default` / `memory`: in-memory cache implementation
- `redis-backend`: Redis-backed cache implementation

## Example

```rust
use cache_layer::{Cache, MemoryCache};

let cache = MemoryCache::new(1_000);
cache.set("merchant:123".to_string(), b"data".to_vec(), 3600).unwrap();
let cached = cache.get(&"merchant:123".to_string()).unwrap();
```
