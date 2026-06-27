//! Rate Limiting & Throttling Service (#276)
//!
//! Token-bucket rate limiter with:
//! - Per-endpoint, per-identity limits
//! - Separate limits for authenticated vs unauthenticated callers
//! - Trusted-identity whitelist (bypass)
//! - Standard rate-limit response headers
//! - Violation monitoring counters

use std::collections::HashMap;
use std::fmt;
use std::time::{Duration, Instant};

// ── Configuration ─────────────────────────────────────────────────────────────

/// Per-endpoint rate-limit policy.
#[derive(Debug, Clone)]
pub struct RatePolicy {
    /// Maximum tokens (requests) in the bucket.
    pub capacity: u32,
    /// Tokens added per second (refill rate).
    pub refill_rate: f64,
}

impl RatePolicy {
    pub fn new(capacity: u32, refill_per_sec: f64) -> Self {
        Self {
            capacity,
            refill_rate: refill_per_sec,
        }
    }
}

/// Global rate-limiter configuration.
#[derive(Debug, Clone)]
pub struct RateLimiterConfig {
    /// Policy applied to authenticated callers (keyed by endpoint).
    pub authenticated: HashMap<String, RatePolicy>,
    /// Policy applied to unauthenticated callers (keyed by endpoint).
    pub unauthenticated: HashMap<String, RatePolicy>,
    /// Default policy when no endpoint-specific policy exists.
    pub default_authenticated: RatePolicy,
    pub default_unauthenticated: RatePolicy,
}

impl RateLimiterConfig {
    pub fn new(
        default_auth: RatePolicy,
        default_unauth: RatePolicy,
    ) -> Self {
        Self {
            authenticated: HashMap::new(),
            unauthenticated: HashMap::new(),
            default_authenticated: default_auth,
            default_unauthenticated: default_unauth,
        }
    }

    /// Override the policy for a specific endpoint.
    pub fn set_endpoint_policy(
        &mut self,
        endpoint: impl Into<String>,
        auth_policy: RatePolicy,
        unauth_policy: RatePolicy,
    ) {
        let ep = endpoint.into();
        self.authenticated.insert(ep.clone(), auth_policy);
        self.unauthenticated.insert(ep, unauth_policy);
    }
}

// ── Token bucket ──────────────────────────────────────────────────────────────

struct Bucket {
    tokens: f64,
    capacity: f64,
    refill_rate: f64,
    last_refill: Instant,
}

impl Bucket {
    fn new(policy: &RatePolicy) -> Self {
        Self {
            tokens: policy.capacity as f64,
            capacity: policy.capacity as f64,
            refill_rate: policy.refill_rate,
            last_refill: Instant::now(),
        }
    }

    /// Refill tokens based on elapsed time, then attempt to consume one.
    fn try_consume(&mut self) -> bool {
        let now = Instant::now();
        let elapsed = now.duration_since(self.last_refill).as_secs_f64();
        self.tokens = (self.tokens + elapsed * self.refill_rate).min(self.capacity);
        self.last_refill = now;

        if self.tokens >= 1.0 {
            self.tokens -= 1.0;
            true
        } else {
            false
        }
    }

    /// Seconds until the next token is available.
    fn retry_after_secs(&self) -> f64 {
        if self.tokens >= 1.0 {
            0.0
        } else {
            (1.0 - self.tokens) / self.refill_rate
        }
    }
}

// ── Rate-limit decision ───────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Decision {
    /// Request is allowed.
    Allow,
    /// Request is denied — too many requests.
    Deny,
}

/// Headers to return in the HTTP response.
#[derive(Debug, Clone)]
pub struct RateLimitHeaders {
    /// Maximum requests allowed in the current window.
    pub x_ratelimit_limit: u32,
    /// Remaining tokens in this window.
    pub x_ratelimit_remaining: u32,
    /// Seconds until the next request will be accepted (only set on Deny).
    pub retry_after_secs: Option<u64>,
}

#[derive(Debug, Clone)]
pub struct RateLimitResult {
    pub decision: Decision,
    pub headers: RateLimitHeaders,
}

// ── Errors ────────────────────────────────────────────────────────────────────

#[derive(Debug, PartialEq, Eq)]
pub enum RateLimitError {
    AlreadyWhitelisted,
    NotWhitelisted,
}

impl fmt::Display for RateLimitError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::AlreadyWhitelisted => write!(f, "Identity is already whitelisted"),
            Self::NotWhitelisted => write!(f, "Identity is not whitelisted"),
        }
    }
}

// ── Violation stats ───────────────────────────────────────────────────────────

#[derive(Debug, Default, Clone)]
pub struct ViolationStats {
    /// Total number of denied requests per identity.
    pub by_identity: HashMap<String, u64>,
    /// Total number of denied requests per endpoint.
    pub by_endpoint: HashMap<String, u64>,
}

// ── Limiter ───────────────────────────────────────────────────────────────────

/// Key used to identify a bucket: (identity, endpoint, is_authenticated).
type BucketKey = (String, String, bool);

pub struct RateLimiter {
    config: RateLimiterConfig,
    buckets: HashMap<BucketKey, Bucket>,
    whitelist: HashMap<String, ()>,
    pub violations: ViolationStats,
}

impl RateLimiter {
    pub fn new(config: RateLimiterConfig) -> Self {
        Self {
            config,
            buckets: HashMap::new(),
            whitelist: HashMap::new(),
            violations: ViolationStats::default(),
        }
    }

    // ── Whitelist management ──────────────────────────────────────────────────

    pub fn add_to_whitelist(&mut self, identity: &str) -> Result<(), RateLimitError> {
        if self.whitelist.contains_key(identity) {
            return Err(RateLimitError::AlreadyWhitelisted);
        }
        self.whitelist.insert(identity.to_owned(), ());
        Ok(())
    }

    pub fn remove_from_whitelist(&mut self, identity: &str) -> Result<(), RateLimitError> {
        if self.whitelist.remove(identity).is_none() {
            return Err(RateLimitError::NotWhitelisted);
        }
        Ok(())
    }

    pub fn is_whitelisted(&self, identity: &str) -> bool {
        self.whitelist.contains_key(identity)
    }

    // ── Check & record ────────────────────────────────────────────────────────

    /// Check if the request should be allowed.
    ///
    /// - `identity`: user ID, IP address, or API key.
    /// - `endpoint`: path or operation name (e.g. `"process_payment"`).
    /// - `authenticated`: whether the caller is authenticated.
    pub fn check(
        &mut self,
        identity: &str,
        endpoint: &str,
        authenticated: bool,
    ) -> RateLimitResult {
        // Whitelisted identities are always allowed.
        if self.is_whitelisted(identity) {
            let policy = self.policy(endpoint, authenticated);
            return RateLimitResult {
                decision: Decision::Allow,
                headers: RateLimitHeaders {
                    x_ratelimit_limit: policy.capacity,
                    x_ratelimit_remaining: policy.capacity,
                    retry_after_secs: None,
                },
            };
        }

        let policy = self.policy(endpoint, authenticated).clone();
        let key: BucketKey = (identity.to_owned(), endpoint.to_owned(), authenticated);

        let bucket = self
            .buckets
            .entry(key)
            .or_insert_with(|| Bucket::new(&policy));

        let allowed = bucket.try_consume();
        let remaining = bucket.tokens.floor() as u32;
        let retry_after = if allowed {
            None
        } else {
            Some(bucket.retry_after_secs().ceil() as u64)
        };

        if !allowed {
            *self
                .violations
                .by_identity
                .entry(identity.to_owned())
                .or_default() += 1;
            *self
                .violations
                .by_endpoint
                .entry(endpoint.to_owned())
                .or_default() += 1;
        }

        RateLimitResult {
            decision: if allowed { Decision::Allow } else { Decision::Deny },
            headers: RateLimitHeaders {
                x_ratelimit_limit: policy.capacity,
                x_ratelimit_remaining: remaining,
                retry_after_secs: retry_after,
            },
        }
    }

    /// Flush all buckets for a specific identity (e.g. after ban lifted).
    pub fn reset_identity(&mut self, identity: &str) {
        self.buckets.retain(|(id, _, _), _| id != identity);
    }

    // ── Helpers ───────────────────────────────────────────────────────────────

    fn policy(&self, endpoint: &str, authenticated: bool) -> &RatePolicy {
        if authenticated {
            self.config
                .authenticated
                .get(endpoint)
                .unwrap_or(&self.config.default_authenticated)
        } else {
            self.config
                .unauthenticated
                .get(endpoint)
                .unwrap_or(&self.config.default_unauthenticated)
        }
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    fn make_limiter(auth_capacity: u32, unauth_capacity: u32) -> RateLimiter {
        let config = RateLimiterConfig::new(
            RatePolicy::new(auth_capacity, 1.0),
            RatePolicy::new(unauth_capacity, 1.0),
        );
        RateLimiter::new(config)
    }

    #[test]
    fn test_allow_within_capacity() {
        let mut limiter = make_limiter(5, 2);
        for _ in 0..5 {
            let r = limiter.check("user1", "/pay", true);
            assert_eq!(r.decision, Decision::Allow);
        }
    }

    #[test]
    fn test_deny_on_exhaustion() {
        let mut limiter = make_limiter(2, 2);
        limiter.check("u", "/pay", true);
        limiter.check("u", "/pay", true);
        let r = limiter.check("u", "/pay", true);
        assert_eq!(r.decision, Decision::Deny);
        assert!(r.headers.retry_after_secs.is_some());
    }

    #[test]
    fn test_different_limits_auth_vs_unauth() {
        let mut limiter = make_limiter(10, 2);
        // Unauthenticated runs out after 2
        limiter.check("ip1", "/ep", false);
        limiter.check("ip1", "/ep", false);
        let r = limiter.check("ip1", "/ep", false);
        assert_eq!(r.decision, Decision::Deny);

        // Authenticated same identity still has 10 tokens
        let r2 = limiter.check("ip1", "/ep", true);
        assert_eq!(r2.decision, Decision::Allow);
    }

    #[test]
    fn test_whitelist_bypasses_limit() {
        let mut limiter = make_limiter(1, 1);
        limiter.add_to_whitelist("trusted").unwrap();
        for _ in 0..100 {
            let r = limiter.check("trusted", "/pay", true);
            assert_eq!(r.decision, Decision::Allow);
        }
    }

    #[test]
    fn test_whitelist_double_add_error() {
        let mut limiter = make_limiter(5, 5);
        limiter.add_to_whitelist("u").unwrap();
        assert_eq!(
            limiter.add_to_whitelist("u").unwrap_err(),
            RateLimitError::AlreadyWhitelisted
        );
    }

    #[test]
    fn test_remove_from_whitelist() {
        let mut limiter = make_limiter(1, 1);
        limiter.add_to_whitelist("u").unwrap();
        limiter.remove_from_whitelist("u").unwrap();
        // Now limited again
        limiter.check("u", "/ep", true);
        let r = limiter.check("u", "/ep", true);
        assert_eq!(r.decision, Decision::Deny);
    }

    #[test]
    fn test_violation_tracking() {
        let mut limiter = make_limiter(1, 1);
        limiter.check("u", "/ep", true);
        limiter.check("u", "/ep", true); // denied
        assert_eq!(*limiter.violations.by_identity.get("u").unwrap_or(&0), 1);
        assert_eq!(*limiter.violations.by_endpoint.get("/ep").unwrap_or(&0), 1);
    }

    #[test]
    fn test_rate_limit_headers_present() {
        let mut limiter = make_limiter(5, 5);
        let r = limiter.check("u", "/ep", true);
        assert_eq!(r.headers.x_ratelimit_limit, 5);
        assert!(r.headers.x_ratelimit_remaining <= 5);
    }

    #[test]
    fn test_token_refill_over_time() {
        let config = RateLimiterConfig::new(
            RatePolicy::new(1, 100.0), // fast refill for test
            RatePolicy::new(1, 100.0),
        );
        let mut limiter = RateLimiter::new(config);
        limiter.check("u", "/ep", true); // consume 1
        thread::sleep(Duration::from_millis(20)); // wait for refill
        let r = limiter.check("u", "/ep", true);
        assert_eq!(r.decision, Decision::Allow);
    }

    #[test]
    fn test_per_endpoint_policy() {
        let mut config = RateLimiterConfig::new(
            RatePolicy::new(100, 10.0),
            RatePolicy::new(100, 10.0),
        );
        config.set_endpoint_policy("/slow", RatePolicy::new(2, 0.1), RatePolicy::new(1, 0.1));
        let mut limiter = RateLimiter::new(config);

        // /slow endpoint has capacity 2
        limiter.check("u", "/slow", true);
        limiter.check("u", "/slow", true);
        let r = limiter.check("u", "/slow", true);
        assert_eq!(r.decision, Decision::Deny);

        // default endpoint has capacity 100
        let r2 = limiter.check("u", "/fast", true);
        assert_eq!(r2.decision, Decision::Allow);
    }
}
