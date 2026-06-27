//! Error types for batch processing service

use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Database error: {0}")]
    DatabaseError(#[from] sqlx::Error),
    
    #[error("Job not found: {0}")]
    JobNotFound(String),
    
    #[error("Invalid job type: {0}")]
    InvalidJobType(String),
    
    #[error("Job failed: {0}")]
    JobFailed(String),
    
    #[error("Job execution error: {0}")]
    ExecutionError(String),
    
    #[error("Configuration error: {0}")]
    ConfigurationError(String),
    
    #[error("Scheduling error: {0}")]
    SchedulingError(String),
    
    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),
    
    #[error("Internal error: {0}")]
    InternalError(String),
}

pub type Result<T> = std::result::Result<T, Error>;
