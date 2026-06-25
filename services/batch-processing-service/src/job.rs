//! Job definitions and types

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Types of batch jobs
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, sqlx::Type)]
#[sqlx(type_name = "text")]
pub enum JobType {
    #[serde(rename = "archival")]
    Archival,
    #[serde(rename = "cleanup")]
    Cleanup,
    #[serde(rename = "stats_aggregation")]
    StatsAggregation,
}

impl JobType {
    pub fn as_str(&self) -> &str {
        match self {
            JobType::Archival => "archival",
            JobType::Cleanup => "cleanup",
            JobType::StatsAggregation => "stats_aggregation",
        }
    }
}

impl std::fmt::Display for JobType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl std::str::FromStr for JobType {
    type Err = crate::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "archival" => Ok(JobType::Archival),
            "cleanup" => Ok(JobType::Cleanup),
            "stats_aggregation" => Ok(JobType::StatsAggregation),
            _ => Err(crate::Error::InvalidJobType(s.to_string())),
        }
    }
}

/// A batch processing job
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchJob {
    /// Unique job ID
    pub id: String,
    /// Type of job
    pub job_type: JobType,
    /// Batch size for this job
    pub batch_size: usize,
    /// Job parameters (JSON)
    pub parameters: serde_json::Value,
    /// Number of items processed
    pub items_processed: usize,
    /// Total items to process (optional)
    pub total_items: Option<usize>,
}

impl BatchJob {
    /// Create a new batch job
    pub fn new(job_type: JobType, batch_size: usize) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            job_type,
            batch_size,
            parameters: serde_json::json!({}),
            items_processed: 0,
            total_items: None,
        }
    }

    /// Create a new batch job with parameters
    pub fn with_parameters(
        job_type: JobType,
        batch_size: usize,
        parameters: serde_json::Value,
    ) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            job_type,
            batch_size,
            parameters,
            items_processed: 0,
            total_items: None,
        }
    }

    /// Get progress percentage
    pub fn progress_percentage(&self) -> Option<f64> {
        self.total_items.map(|total| {
            if total == 0 {
                100.0
            } else {
                (self.items_processed as f64 / total as f64) * 100.0
            }
        })
    }
}
