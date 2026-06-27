//! Batch Processing Service for non-critical, high-volume operations
//!
//! This service provides a job queue system with support for:
//! - Job scheduling and execution
//! - Batch processing for archival
//! - Cleanup jobs for expired records
//! - Statistics aggregation
//! - Automatic retry on failure
//! - Job status monitoring

pub mod config;
pub mod error;
pub mod job;
pub mod queue;
pub mod scheduler;
pub mod status;

pub use config::BatchProcessingConfig;
pub use error::{Error, Result};
pub use job::{BatchJob, JobType};
pub use queue::JobQueue;
pub use scheduler::Scheduler;
pub use status::JobStatus;

use sqlx::PgPool;
use std::sync::Arc;

/// Main service for batch processing
pub struct BatchProcessingService {
    pool: Arc<PgPool>,
    config: BatchProcessingConfig,
    queue: Arc<JobQueue>,
    scheduler: Arc<Scheduler>,
}

impl BatchProcessingService {
    /// Create a new batch processing service
    pub async fn new(
        pool: PgPool,
        config: BatchProcessingConfig,
    ) -> Result<Self> {
        let pool = Arc::new(pool);
        let queue = Arc::new(JobQueue::new(pool.clone(), config.clone()));
        let scheduler = Arc::new(Scheduler::new(
            pool.clone(),
            queue.clone(),
            config.clone(),
        ));

        Ok(Self {
            pool,
            config,
            queue,
            scheduler,
        })
    }

    /// Start the service
    pub async fn start(&self) -> Result<()> {
        tracing::info!("Starting batch processing service");
        
        // Initialize the database
        self.initialize_db().await?;
        
        // Start the scheduler
        self.scheduler.start().await?;
        
        tracing::info!("Batch processing service started successfully");
        Ok(())
    }

    /// Stop the service
    pub async fn stop(&self) -> Result<()> {
        tracing::info!("Stopping batch processing service");
        self.scheduler.stop().await?;
        Ok(())
    }

    /// Initialize database tables
    async fn initialize_db(&self) -> Result<()> {
        sqlx::query(include_str!("../schema.sql"))
            .execute(self.pool.as_ref())
            .await?;
        Ok(())
    }

    /// Enqueue a job
    pub async fn enqueue(&self, job: BatchJob) -> Result<String> {
        self.queue.enqueue(job).await
    }

    /// Get job status
    pub async fn get_job_status(&self, job_id: &str) -> Result<Option<JobStatus>> {
        self.queue.get_status(job_id).await
    }

    /// Get all pending jobs
    pub async fn get_pending_jobs(&self) -> Result<Vec<BatchJob>> {
        self.queue.get_pending_jobs().await
    }
}
