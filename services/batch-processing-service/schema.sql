-- Batch Processing Service Schema

CREATE TABLE IF NOT EXISTS batch_jobs (
  id TEXT PRIMARY KEY,
  job_type TEXT NOT NULL CHECK (job_type IN ('archival', 'cleanup', 'stats_aggregation')),
  status TEXT NOT NULL DEFAULT 'pending' CHECK (status IN ('pending', 'running', 'completed', 'failed', 'retrying', 'cancelled')),
  batch_size INTEGER NOT NULL DEFAULT 1000 CHECK (batch_size > 0),
  items_processed INTEGER NOT NULL DEFAULT 0 CHECK (items_processed >= 0),
  total_items INTEGER,
  parameters JSONB DEFAULT '{}',
  error_message TEXT,
  attempt INTEGER NOT NULL DEFAULT 0 CHECK (attempt >= 0),
  created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  started_at TIMESTAMPTZ,
  completed_at TIMESTAMPTZ,
  next_retry_at TIMESTAMPTZ,
  CONSTRAINT ck_total_items_valid CHECK (total_items IS NULL OR total_items > 0)
);

-- Indexes for efficient querying
CREATE INDEX IF NOT EXISTS idx_batch_jobs_status ON batch_jobs(status);
CREATE INDEX IF NOT EXISTS idx_batch_jobs_job_type ON batch_jobs(job_type);
CREATE INDEX IF NOT EXISTS idx_batch_jobs_created_at ON batch_jobs(created_at DESC);
CREATE INDEX IF NOT EXISTS idx_batch_jobs_next_retry_at ON batch_jobs(next_retry_at) WHERE status = 'retrying';

-- Job execution history table for audit trail
CREATE TABLE IF NOT EXISTS batch_job_history (
  id SERIAL PRIMARY KEY,
  job_id TEXT NOT NULL REFERENCES batch_jobs(id) ON DELETE CASCADE,
  status TEXT NOT NULL,
  items_processed INTEGER NOT NULL DEFAULT 0,
  error_message TEXT,
  duration_ms INTEGER,
  created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_batch_job_history_job_id ON batch_job_history(job_id);
CREATE INDEX IF NOT EXISTS idx_batch_job_history_created_at ON batch_job_history(created_at DESC);

-- Job queue table for managing pending work
CREATE TABLE IF NOT EXISTS batch_job_queue (
  id SERIAL PRIMARY KEY,
  job_id TEXT NOT NULL UNIQUE REFERENCES batch_jobs(id) ON DELETE CASCADE,
  priority INTEGER NOT NULL DEFAULT 0,
  scheduled_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_batch_job_queue_priority ON batch_job_queue(priority DESC);
CREATE INDEX IF NOT EXISTS idx_batch_job_queue_scheduled_at ON batch_job_queue(scheduled_at ASC);
