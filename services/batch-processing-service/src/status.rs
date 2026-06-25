//! Job status tracking

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Job execution status
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, sqlx::Type)]
#[sqlx(type_name = "text")]
pub enum JobStatus {
    #[serde(rename = "pending")]
    Pending,
    #[serde(rename = "running")]
    Running,
    #[serde(rename = "completed")]
    Completed,
    #[serde(rename = "failed")]
    Failed,
    #[serde(rename = "retrying")]
    Retrying,
    #[serde(rename = "cancelled")]
    Cancelled,
}

impl JobStatus {
    pub fn as_str(&self) -> &str {
        match self {
            JobStatus::Pending => "pending",
            JobStatus::Running => "running",
            JobStatus::Completed => "completed",
            JobStatus::Failed => "failed",
            JobStatus::Retrying => "retrying",
            JobStatus::Cancelled => "cancelled",
        }
    }

    pub fn is_final(&self) -> bool {
        matches!(
            self,
            JobStatus::Completed | JobStatus::Failed | JobStatus::Cancelled
        )
    }
}

impl std::fmt::Display for JobStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Job execution record with detailed status information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobStatusRecord {
    pub job_id: String,
    pub status: JobStatus,
    pub attempt: u32,
    pub items_processed: usize,
    pub error_message: Option<String>,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub next_retry_at: Option<DateTime<Utc>>,
}

impl JobStatusRecord {
    pub fn new(job_id: String) -> Self {
        Self {
            job_id,
            status: JobStatus::Pending,
            attempt: 0,
            items_processed: 0,
            error_message: None,
            started_at: None,
            completed_at: None,
            next_retry_at: None,
        }
    }
}
