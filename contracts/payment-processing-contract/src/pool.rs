/// Connection Pooling Module
/// 
/// Optimizes contract state access patterns and manages efficient resource allocation.
/// Implements connection-like pooling for contract operations to improve performance
/// and graceful degradation under load.
///
/// Acceptance Criteria:
/// - Connection pool size optimized
/// - Idle connection cleanup
/// - Connection timeout handling
/// - Pool monitoring metrics
/// - Graceful degradation under load
/// - Environment-based pool size
/// - Performance benchmarked

use soroban_sdk::{Env, Vec};

use crate::error::PaymentError;

/// Connection pool configuration
#[derive(Clone, Debug)]
pub struct ConnectionPoolConfig {
    /// Maximum number of concurrent operations
    pub max_pool_size: u32,
    /// Minimum idle connections to maintain
    pub min_idle_connections: u32,
    /// Idle connection timeout in seconds
    pub idle_timeout_secs: u32,
    /// Connection operation timeout in seconds
    pub operation_timeout_secs: u32,
    /// Whether pool management is enabled
    pub enabled: bool,
}

/// Connection pool entry
#[derive(Clone, Debug)]
pub struct PoolConnection {
    /// Unique connection ID
    pub connection_id: u32,
    /// Timestamp of last activity
    pub last_activity_at: u64,
    /// Whether connection is currently in use
    pub in_use: bool,
    /// Number of operations performed on this connection
    pub operation_count: u32,
}

/// Connection pool statistics
#[derive(Clone, Debug)]
pub struct PoolStats {
    /// Total connections in pool
    pub total_connections: u32,
    /// Currently active (in-use) connections
    pub active_connections: u32,
    /// Idle connections available
    pub idle_connections: u32,
    /// Total operations performed
    pub total_operations: u64,
    /// Operations in last 60 seconds
    pub operations_per_minute: u32,
    /// Peak concurrent connections reached
    pub peak_concurrent_connections: u32,
    /// Average operation latency in milliseconds
    pub avg_operation_latency_ms: u32,
    /// Pool utilization percentage (0-100)
    pub utilization_percent: u32,
}

/// Connection pool state tracker
pub struct ConnectionPool {
    /// Current number of connections
    pub current_size: u32,
    /// Configuration for the pool
    pub config: ConnectionPoolConfig,
    /// Total operations performed
    pub total_operations: u64,
    /// Peak connections reached
    pub peak_connections: u32,
    /// Timestamp of creation
    pub created_at: u64,
}

impl ConnectionPoolConfig {
    /// Create default connection pool configuration
    /// - Max pool size: 20 concurrent operations
    /// - Min idle: 2 connections
    /// - Idle timeout: 300 seconds (5 minutes)
    /// - Operation timeout: 60 seconds
    pub fn default() -> Self {
        ConnectionPoolConfig {
            max_pool_size: 20,
            min_idle_connections: 2,
            idle_timeout_secs: 300,
            operation_timeout_secs: 60,
            enabled: true,
        }
    }

    /// Create configuration for light load (development/testing)
    pub fn light_load() -> Self {
        ConnectionPoolConfig {
            max_pool_size: 5,
            min_idle_connections: 1,
            idle_timeout_secs: 180,
            operation_timeout_secs: 30,
            enabled: true,
        }
    }

    /// Create configuration for heavy load (production)
    pub fn heavy_load() -> Self {
        ConnectionPoolConfig {
            max_pool_size: 50,
            min_idle_connections: 5,
            idle_timeout_secs: 600,
            operation_timeout_secs: 120,
            enabled: true,
        }
    }

    /// Create configuration from environment
    pub fn from_environment(env: &Env) -> Self {
        // In production, this could read from environment variables or ledger state
        // For now, return default configuration
        ConnectionPoolConfig::default()
    }
}

impl ConnectionPool {
    /// Create a new connection pool
    pub fn new(config: ConnectionPoolConfig, env: &Env) -> Self {
        ConnectionPool {
            current_size: config.min_idle_connections,
            config,
            total_operations: 0,
            peak_connections: config.min_idle_connections,
            created_at: env.ledger().timestamp(),
        }
    }

    /// Acquire a connection from the pool
    pub fn acquire_connection(&mut self) -> Result<u32, PaymentError> {
        if !self.config.enabled {
            return Err(PaymentError::Unauthorized);
        }

        if self.current_size < self.config.max_pool_size {
            self.current_size += 1;
            if self.current_size > self.peak_connections {
                self.peak_connections = self.current_size;
            }
            Ok(self.current_size - 1)
        } else {
            Err(PaymentError::Unauthorized) // Pool exhausted
        }
    }

    /// Release a connection back to the pool
    pub fn release_connection(&mut self, _connection_id: u32) {
        if self.current_size > self.config.min_idle_connections {
            self.current_size -= 1;
        }
    }

    /// Increment operation count
    pub fn record_operation(&mut self) {
        self.total_operations = self.total_operations.saturating_add(1);
    }
}

/// Check if a connection has exceeded idle timeout
pub fn is_idle_timeout_exceeded(
    connection: &PoolConnection,
    current_time: u64,
    config: &ConnectionPoolConfig,
) -> bool {
    if connection.in_use {
        return false;
    }

    let idle_duration = current_time.saturating_sub(connection.last_activity_at);
    idle_duration >= (config.idle_timeout_secs as u64)
}

/// Check if an operation has exceeded timeout
pub fn is_operation_timeout_exceeded(
    operation_start_time: u64,
    current_time: u64,
    config: &ConnectionPoolConfig,
) -> bool {
    let operation_duration = current_time.saturating_sub(operation_start_time);
    operation_duration >= (config.operation_timeout_secs as u64)
}

/// Calculate pool utilization percentage
pub fn calculate_pool_utilization(
    active_connections: u32,
    max_pool_size: u32,
) -> u32 {
    if max_pool_size == 0 {
        return 0;
    }
    ((active_connections as u64) * 100 / (max_pool_size as u64)) as u32
}

/// Get pool statistics
pub fn get_pool_stats(
    pool: &ConnectionPool,
    active_connections: u32,
    idle_connections: u32,
    operations_per_minute: u32,
    avg_latency_ms: u32,
) -> PoolStats {
    let total = active_connections.saturating_add(idle_connections);
    let utilization = calculate_pool_utilization(active_connections, pool.config.max_pool_size);

    PoolStats {
        total_connections: total,
        active_connections,
        idle_connections,
        total_operations: pool.total_operations,
        operations_per_minute,
        peak_concurrent_connections: pool.peak_connections,
        avg_operation_latency_ms: avg_latency_ms,
        utilization_percent: utilization,
    }
}

/// Handle graceful degradation when pool is under load
pub fn should_apply_backpressure(stats: &PoolStats) -> bool {
    // Apply backpressure when utilization exceeds 80%
    stats.utilization_percent > 80
}

/// Suggest backoff strategy based on pool load
pub fn calculate_backoff_delay(stats: &PoolStats) -> u32 {
    // Milliseconds to back off
    match stats.utilization_percent {
        0..=50 => 0,      // No backoff
        51..=70 => 10,    // 10ms backoff
        71..=85 => 50,    // 50ms backoff
        86..=95 => 100,   // 100ms backoff
        _ => 200,         // 200ms backoff for 95%+
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_connection_pool_config_default() {
        let config = ConnectionPoolConfig::default();
        assert_eq!(config.max_pool_size, 20);
        assert_eq!(config.min_idle_connections, 2);
        assert_eq!(config.idle_timeout_secs, 300);
        assert!(config.enabled);
    }

    #[test]
    fn test_connection_pool_config_light_load() {
        let config = ConnectionPoolConfig::light_load();
        assert_eq!(config.max_pool_size, 5);
        assert_eq!(config.min_idle_connections, 1);
    }

    #[test]
    fn test_connection_pool_config_heavy_load() {
        let config = ConnectionPoolConfig::heavy_load();
        assert_eq!(config.max_pool_size, 50);
        assert_eq!(config.min_idle_connections, 5);
    }

    #[test]
    fn test_pool_utilization_calculation() {
        assert_eq!(calculate_pool_utilization(10, 20), 50);
        assert_eq!(calculate_pool_utilization(18, 20), 90);
        assert_eq!(calculate_pool_utilization(0, 20), 0);
        assert_eq!(calculate_pool_utilization(20, 20), 100);
    }

    #[test]
    fn test_idle_timeout_detection() {
        let config = ConnectionPoolConfig::default();
        let connection = PoolConnection {
            connection_id: 1,
            last_activity_at: 1000,
            in_use: false,
            operation_count: 5,
        };

        let current_time_before = 1000 + 299;
        let current_time_after = 1000 + 301;

        assert!(!is_idle_timeout_exceeded(&connection, current_time_before, &config));
        assert!(is_idle_timeout_exceeded(&connection, current_time_after, &config));
    }

    #[test]
    fn test_operation_timeout_detection() {
        let config = ConnectionPoolConfig::default();
        let start_time = 1000;
        let before_timeout = 1000 + 59;
        let after_timeout = 1000 + 61;

        assert!(!is_operation_timeout_exceeded(start_time, before_timeout, &config));
        assert!(is_operation_timeout_exceeded(start_time, after_timeout, &config));
    }

    #[test]
    fn test_backpressure_logic() {
        let stats_low = PoolStats {
            total_connections: 5,
            active_connections: 2,
            idle_connections: 3,
            total_operations: 100,
            operations_per_minute: 10,
            peak_concurrent_connections: 5,
            avg_operation_latency_ms: 5,
            utilization_percent: 40,
        };

        assert!(!should_apply_backpressure(&stats_low));

        let stats_high = PoolStats {
            total_connections: 20,
            active_connections: 18,
            idle_connections: 2,
            total_operations: 1000,
            operations_per_minute: 100,
            peak_concurrent_connections: 20,
            avg_operation_latency_ms: 50,
            utilization_percent: 90,
        };

        assert!(should_apply_backpressure(&stats_high));
    }

    #[test]
    fn test_backoff_delay_calculation() {
        let stats_empty = PoolStats {
            total_connections: 0,
            active_connections: 0,
            idle_connections: 0,
            total_operations: 0,
            operations_per_minute: 0,
            peak_concurrent_connections: 0,
            avg_operation_latency_ms: 0,
            utilization_percent: 30,
        };
        assert_eq!(calculate_backoff_delay(&stats_empty), 0);

        let stats_medium = PoolStats {
            total_connections: 20,
            active_connections: 14,
            idle_connections: 6,
            total_operations: 500,
            operations_per_minute: 50,
            peak_concurrent_connections: 20,
            avg_operation_latency_ms: 20,
            utilization_percent: 70,
        };
        assert_eq!(calculate_backoff_delay(&stats_medium), 10);

        let stats_high = PoolStats {
            total_connections: 20,
            active_connections: 19,
            idle_connections: 1,
            total_operations: 1000,
            operations_per_minute: 100,
            peak_concurrent_connections: 20,
            avg_operation_latency_ms: 100,
            utilization_percent: 95,
        };
        assert_eq!(calculate_backoff_delay(&stats_high), 200);
    }
}
