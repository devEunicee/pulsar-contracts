//! Job queue management

use crate::{BatchJob, BatchProcessingConfig, Error, JobStatus, Result};
use chrono::Utc;
use sqlx::PgPool;
use std::sync::Arc;

/// Job queue for managing batch processing jobs
pub struct JobQueue {
    pool: Arc<PgPool>,
    config: BatchProcessingConfig,
}

impl JobQueue {
    /// Create a new job queue
    pub fn new(pool: Arc<PgPool>, config: BatchProcessingConfig) -> Self {
        Self { pool, config }
    }

    /// Enqueue a new job
    pub async fn enqueue(&self, job: BatchJob) -> Result<String> {
        let job_id = &job.id;
        let job_type = job.job_type.to_string();
        let parameters = serde_json::to_string(&job.parameters)?;

        sqlx::query(
            r#"
            INSERT INTO batch_jobs (id, job_type, status, parameters, batch_size, created_at)
            VALUES ($1, $2, $3, $4, $5, NOW())
            ON CONFLICT (id) DO NOTHING
            "#,
        )
        .bind(job_id)
        .bind(&job_type)
        .bind(JobStatus::Pending.as_str())
        .bind(&parameters)
        .bind(job.batch_size as i32)
        .execute(self.pool.as_ref())
        .await?;

        tracing::info!("Job enqueued: {} (type: {})", job_id, job_type);
        Ok(job_id.clone())
    }

    /// Get pending jobs
    pub async fn get_pending_jobs(&self) -> Result<Vec<BatchJob>> {
        let rows = sqlx::query(
            r#"
            SELECT id, job_type, parameters, batch_size
            FROM batch_jobs
            WHERE status IN ('pending', 'retrying')
            ORDER BY created_at ASC
            LIMIT 100
            "#,
        )
        .fetch_all(self.pool.as_ref())
        .await?;

        let mut jobs = Vec::new();
        for row in rows {
            let job_type: String = row.get("job_type");
            let parameters: String = row.get("parameters");
            let job = BatchJob::with_parameters(
                job_type.parse()?,
                row.get::<i32, _>("batch_size") as usize,
                serde_json::from_str(&parameters)?,
            );
            jobs.push(job);
        }

        Ok(jobs)
    }

    /// Get job status
    pub async fn get_status(&self, job_id: &str) -> Result<Option<JobStatus>> {
        let row = sqlx::query("SELECT status FROM batch_jobs WHERE id = $1")
            .bind(job_id)
            .fetch_optional(self.pool.as_ref())
            .await?;

        if let Some(row) = row {
            let status: String = row.get("status");
            let status = status.parse()?;
            Ok(Some(status))
        } else {
            Ok(None)
        }
    }

    /// Update job status
    pub async fn update_status(
        &self,
        job_id: &str,
        status: JobStatus,
        items_processed: usize,
        error_message: Option<&str>,
    ) -> Result<()> {
        sqlx::query(
            r#"
            UPDATE batch_jobs
            SET status = $2, items_processed = $3, error_message = $4, updated_at = NOW()
            WHERE id = $1
            "#,
        )
        .bind(job_id)
        .bind(status.as_str())
        .bind(items_processed as i32)
        .bind(error_message)
        .execute(self.pool.as_ref())
        .await?;

        Ok(())
    }

    /// Mark job as completed
    pub async fn complete_job(&self, job_id: &str, items_processed: usize) -> Result<()> {
        self.update_status(job_id, JobStatus::Completed, items_processed, None)
            .await
    }

    /// Mark job as failed
    pub async fn fail_job(&self, job_id: &str, error_message: &str, attempt: u32) -> Result<()> {
        if attempt < self.config.max_retries {
            self.update_status(job_id, JobStatus::Retrying, 0, Some(error_message))
                .await
        } else {
            self.update_status(job_id, JobStatus::Failed, 0, Some(error_message))
                .await
        }
    }

    /// Clean up old jobs
    pub async fn cleanup_old_jobs(&self) -> Result<()> {
        let days = self.config.cleanup_retention_days;
        
        sqlx::query(
            r#"
            DELETE FROM batch_jobs
            WHERE status IN ('completed', 'failed', 'cancelled')
            AND updated_at < NOW() - INTERVAL '1 day' * $1
            "#,
        )
        .bind(days)
        .execute(self.pool.as_ref())
        .await?;

        Ok(())
    }
}

impl std::str::FromStr for JobStatus {
    type Err = crate::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "pending" => Ok(JobStatus::Pending),
            "running" => Ok(JobStatus::Running),
            "completed" => Ok(JobStatus::Completed),
            "failed" => Ok(JobStatus::Failed),
            "retrying" => Ok(JobStatus::Retrying),
            "cancelled" => Ok(JobStatus::Cancelled),
            _ => Err(Error::InvalidJobType(s.to_string())),
        }
    }
}
