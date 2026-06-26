use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use thiserror::Error;

pub type Result<T> = std::result::Result<T, CircuitBreakerError>;

#[derive(Debug, Error)]
pub enum CircuitBreakerError {
    #[error("circuit open")]
    Open,
    #[error("invalid circuit state")]
    InvalidState,
    #[error("operation failed")]
    OperationFailed,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum CircuitState {
    Closed,
    Open,
    HalfOpen,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CircuitBreakerConfig {
    pub failure_threshold: u32,
    pub recovery_timeout_secs: u64,
    pub max_half_open_attempts: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CircuitBreaker {
    pub state: CircuitState,
    pub failure_count: u32,
    pub last_failure: Option<DateTime<Utc>>,
    pub last_attempt: Option<DateTime<Utc>>,
    pub half_open_attempts: u32,
    pub config: CircuitBreakerConfig,
}

impl CircuitBreaker {
    pub fn new(config: CircuitBreakerConfig) -> Self {
        Self {
            state: CircuitState::Closed,
            failure_count: 0,
            last_failure: None,
            last_attempt: None,
            half_open_attempts: 0,
            config,
        }
    }

    pub fn can_execute(&mut self) -> bool {
        self.last_attempt = Some(Utc::now());

        match self.state {
            CircuitState::Closed => true,
            CircuitState::Open => {
                if let Some(last_failure) = self.last_failure {
                    let timeout = Duration::seconds(self.config.recovery_timeout_secs as i64);
                    if Utc::now() >= last_failure + timeout {
                        self.state = CircuitState::HalfOpen;
                        self.half_open_attempts = 0;
                        return true;
                    }
                }
                false
            }
            CircuitState::HalfOpen => self.half_open_attempts < self.config.max_half_open_attempts,
        }
    }

    pub fn record_success(&mut self) {
        self.failure_count = 0;
        self.half_open_attempts = 0;
        self.state = CircuitState::Closed;
    }

    pub fn record_failure(&mut self) {
        self.failure_count += 1;
        self.last_failure = Some(Utc::now());

        if self.failure_count >= self.config.failure_threshold {
            self.state = CircuitState::Open;
        }

        if self.state == CircuitState::HalfOpen {
            self.half_open_attempts += 1;
            if self.half_open_attempts >= self.config.max_half_open_attempts {
                self.state = CircuitState::Open;
            }
        }
    }

    pub fn execute<F, T>(&mut self, mut operation: F) -> Result<T>
    where
        F: FnMut() -> std::result::Result<T, CircuitBreakerError>,
    {
        if !self.can_execute() {
            return Err(CircuitBreakerError::Open);
        }

        match operation() {
            Ok(value) => {
                self.record_success();
                Ok(value)
            }
            Err(err) => {
                self.record_failure();
                Err(err)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn default_config() -> CircuitBreakerConfig {
        CircuitBreakerConfig {
            failure_threshold: 2,
            recovery_timeout_secs: 1,
            max_half_open_attempts: 1,
        }
    }

    #[test]
    fn circuit_closes_after_success() {
        let mut breaker = CircuitBreaker::new(default_config());
        assert!(breaker.can_execute());
        assert!(breaker.execute(|| Ok(1)).is_ok());
        assert_eq!(breaker.state, CircuitState::Closed);
    }

    #[test]
    fn circuit_opens_after_failures() {
        let mut breaker = CircuitBreaker::new(default_config());
        breaker.execute(|| Err(CircuitBreakerError::OperationFailed)).unwrap_err();
        breaker.execute(|| Err(CircuitBreakerError::OperationFailed)).unwrap_err();
        assert_eq!(breaker.state, CircuitState::Open);
        assert!(!breaker.can_execute());
    }
}
