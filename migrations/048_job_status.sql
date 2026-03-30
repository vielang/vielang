-- P10: Job Scheduler — add job-level status + created_time to execution history.

-- Job-level status: PENDING | RUNNING | COMPLETED | FAILED | CANCELLED
ALTER TABLE scheduled_job
    ADD COLUMN IF NOT EXISTS status TEXT NOT NULL DEFAULT 'PENDING';

-- created_time on execution row for time-based pruning
ALTER TABLE job_execution
    ADD COLUMN IF NOT EXISTS created_time BIGINT;

UPDATE job_execution SET created_time = started_at WHERE created_time IS NULL;

CREATE INDEX IF NOT EXISTS idx_job_exec_time ON job_execution(started_at);
