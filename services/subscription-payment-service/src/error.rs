//! Error types for subscription service

use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Database error: {0}")]
    DatabaseError(#[from] sqlx::Error),
    
    #[error("Subscription not found: {0}")]
    SubscriptionNotFound(String),
    
    #[error("Invalid subscription status: {0}")]
    InvalidStatus(String),
    
    #[error("Payment failed: {0}")]
    PaymentFailed(String),
    
    #[error("Invoice generation failed: {0}")]
    InvoiceGenerationFailed(String),
    
    #[error("Event emission failed: {0}")]
    EventEmissionFailed(String),
    
    #[error("Configuration error: {0}")]
    ConfigurationError(String),
    
    #[error("Scheduling error: {0}")]
    SchedulingError(String),
    
    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),
    
    #[error("Invalid subscription configuration: {0}")]
    InvalidConfiguration(String),
    
    #[error("Internal error: {0}")]
    InternalError(String),
}

pub type Result<T> = std::result::Result<T, Error>;
