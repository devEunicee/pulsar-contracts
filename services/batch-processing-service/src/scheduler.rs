//! Job scheduler for automated job execution

use crate::{BatchProcessingConfig, Error, JobQueue, JobStatus, JobType, Result};
use chrono::Utc;
use sqlx::PgPool;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::time::{interval, Duration};

/// Job scheduler for managing scheduled job execution
pub struct Scheduler {
    pool: Arc<PgPool>,
    queue: Arc<JobQueue>,
    config: BatchProcessingConfig,
    running: Arc<RwLock<bool>>,
}

impl Scheduler {
    /// Create a new scheduler
    pub fn new(
        pool: Arc<PgPool>,
        queue: Arc<JobQueue>,
        config: BatchProcessingConfig,
    ) -> Self {
        Self {
            pool,
            queue,
            config,
            running: Arc::new(RwLock::new(false)),
        }
    }

    /// Start the scheduler
    pub async fn start(&self) -> Result<()> {
        let mut is_running = self.running.write().await;
        if *is_running {
            return Err(Error::SchedulingError(
                "Scheduler is already running".to_string(),
            ));
        }
        *is_running = true;
        drop(is_running);

        tracing::info!("Scheduler started");

        // Spawn background tasks for each enabled job type
        if self.config.enable_archival {
            self.spawn_archival_job();
        }
        if self.config.enable_cleanup {
            self.spawn_cleanup_job();
        }
        if self.config.enable_stats_aggregation {
            self.spawn_stats_aggregation_job();
        }

        Ok(())
    }

    /// Stop the scheduler
    pub async fn stop(&self) -> Result<()> {
        let mut is_running = self.running.write().await;
        *is_running = false;
        tracing::info!("Scheduler stopped");
        Ok(())
    }

    /// Spawn archival job task
    fn spawn_archival_job(&self) {
        let queue = self.queue.clone();
        let config = self.config.clone();
        let running = self.running.clone();

        tokio::spawn(async move {
            let mut interval = interval(Duration::from_secs(3600)); // Check every hour
            loop {
                interval.tick().await;
                
                let is_running = running.read().await;
                if !*is_running {
                    break;
                }
                drop(is_running);

                if config.enable_archival {
                    if let Err(e) = execute_archival_job(&queue, &config).await {
                        tracing::error!("Archival job failed: {:?}", e);
                    }
                }
            }
        });
    }

    /// Spawn cleanup job task
    fn spawn_cleanup_job(&self) {
        let queue = self.queue.clone();
        let config = self.config.clone();
        let running = self.running.clone();

        tokio::spawn(async move {
            let mut interval = interval(Duration::from_secs(3600)); // Check every hour
            loop {
                interval.tick().await;
                
                let is_running = running.read().await;
                if !*is_running {
                    break;
                }
                drop(is_running);

                if config.enable_cleanup {
                    if let Err(e) = queue.cleanup_old_jobs().await {
                        tracing::error!("Cleanup job failed: {:?}", e);
                    }
                }
            }
        });
    }

    /// Spawn stats aggregation job task
    fn spawn_stats_aggregation_job(&self) {
        let queue = self.queue.clone();
        let config = self.config.clone();
        let running = self.running.clone();

        tokio::spawn(async move {
            let mut interval = interval(Duration::from_secs(3600)); // Check every hour
            loop {
                interval.tick().await;
                
                let is_running = running.read().await;
                if !*is_running {
                    break;
                }
                drop(is_running);

                if config.enable_stats_aggregation {
                    if let Err(e) = execute_stats_aggregation_job(&queue, &config).await {
                        tracing::error!("Stats aggregation job failed: {:?}", e);
                    }
                }
            }
        });
    }
}

/// Execute archival job
async fn execute_archival_job(queue: &JobQueue, config: &BatchProcessingConfig) -> Result<()> {
    use crate::job::BatchJob;
    
    let job = BatchJob::new(JobType::Archival, config.batch_size);
    let job_id = queue.enqueue(job).await?;
    queue.update_status(&job_id, JobStatus::Running, 0, None).await?;
    
    tracing::info!("Archival job started: {}", job_id);
    
    // Job execution would happen elsewhere
    Ok(())
}

/// Execute cleanup job
async fn execute_cleanup_job(queue: &JobQueue, config: &BatchProcessingConfig) -> Result<()> {
    queue.cleanup_old_jobs().await?;
    tracing::info!("Cleanup job completed");
    Ok(())
}

/// Execute stats aggregation job
async fn execute_stats_aggregation_job(
    queue: &JobQueue,
    config: &BatchProcessingConfig,
) -> Result<()> {
    use crate::job::BatchJob;
    
    let job = BatchJob::new(JobType::StatsAggregation, config.batch_size);
    let job_id = queue.enqueue(job).await?;
    queue.update_status(&job_id, JobStatus::Running, 0, None).await?;
    
    tracing::info!("Stats aggregation job started: {}", job_id);
    
    // Job execution would happen elsewhere
    Ok(())
}
