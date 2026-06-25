# Batch Processing Service

A service for managing non-critical, high-volume batch processing operations in Pulsar.

## Features

- **Job Queue**: Flexible job queueing system with priority support
- **Batch Processing**: Process large datasets in configurable batch sizes
- **Archival**: Automatically archive old payment records
- **Cleanup**: Clean up expired jobs and records
- **Stats Aggregation**: Aggregate statistics from payment data
- **Scheduled Execution**: Run jobs on configurable schedules
- **Retry Logic**: Automatic retry with exponential backoff
- **Job Monitoring**: Track job status and progress
- **Audit Trail**: Full history of job executions

## Configuration

Environment variables:

```bash
DATABASE_URL=postgresql://user:password@localhost/batch_processing
BATCH_SIZE=1000
MAX_RETRIES=3
RETRY_DELAY_SECS=60
ARCHIVAL_RETENTION_DAYS=90
CLEANUP_RETENTION_DAYS=30
```

## Usage

```rust
use batch_processing_service::{BatchProcessingService, BatchProcessingConfig, JobType, BatchJob};

#[tokio::main]
async fn main() {
    let config = BatchProcessingConfig::from_env();
    let pool = sqlx::postgres::PgPoolOptions::new()
        .connect(&config.database_url)
        .await
        .unwrap();
    
    let service = BatchProcessingService::new(pool, config).await.unwrap();
    service.start().await.unwrap();
    
    // Enqueue a job
    let job = BatchJob::new(JobType::Archival, 1000);
    let job_id = service.enqueue(job).await.unwrap();
    
    // Check job status
    if let Some(status) = service.get_job_status(&job_id).await.unwrap() {
        println!("Job status: {}", status);
    }
}
```

## Job Types

### Archival
Moves old payment records to archive storage to reduce active database size.

**Parameters:**
- `retention_days`: Number of days to keep in active database (default: 90)

### Cleanup
Removes expired or completed jobs from the system.

**Parameters:**
- `retention_days`: Number of days to keep completed jobs (default: 30)

### Stats Aggregation
Aggregates payment statistics for reporting and analytics.

**Parameters:**
- `period`: Aggregation period (hourly, daily, weekly, monthly)

## Database Schema

See `schema.sql` for the complete schema with:
- `batch_jobs`: Main job table
- `batch_job_history`: Job execution history
- `batch_job_queue`: Job queue management

## Performance

- Configurable batch sizes for memory efficiency
- Indexed queries for fast job lookup
- Automatic cleanup of completed jobs
- Exponential backoff for failed jobs

## Error Handling

- Automatic retry on failure
- Configurable retry attempts and delays
- Detailed error messages stored in database
- Job execution history for debugging

## Monitoring

Monitor job status with:
```sql
SELECT job_type, status, COUNT(*) as count
FROM batch_jobs
GROUP BY job_type, status;
```

Track long-running jobs:
```sql
SELECT id, job_type, status, items_processed, total_items
FROM batch_jobs
WHERE status = 'running'
AND (NOW() - started_at) > INTERVAL '1 hour';
```
