-- Create session_metrics table for storing processed session metrics
CREATE TABLE IF NOT EXISTS session_metrics (
    id TEXT PRIMARY KEY,
    session_id TEXT NOT NULL,
    provider TEXT NOT NULL,
    timestamp INTEGER NOT NULL,
    response_latency_ms REAL,
    task_completion_time_ms REAL,
    performance_total_responses INTEGER,
    read_write_ratio REAL,
    input_clarity_score REAL,
    read_operations INTEGER,
    write_operations INTEGER,
    total_user_messages INTEGER,
    error_count INTEGER,
    error_types TEXT,
    last_error_message TEXT,
    recovery_attempts INTEGER,
    fatal_errors INTEGER,
    interruption_rate REAL,
    session_length_minutes REAL,
    total_interruptions INTEGER,
    engagement_total_responses INTEGER,
    task_success_rate REAL,
    iteration_count INTEGER,
    process_quality_score REAL,
    used_plan_mode INTEGER,
    used_todo_tracking INTEGER,
    over_top_affirmations INTEGER,
    successful_operations INTEGER,
    total_operations INTEGER,
    exit_plan_mode_count INTEGER,
    todo_write_count INTEGER,
    over_top_affirmations_phrases TEXT,
    improvement_tips TEXT,
    custom_metrics TEXT,
    created_at INTEGER NOT NULL
);

-- Create indexes for metrics queries
CREATE INDEX IF NOT EXISTS session_metrics_session_idx ON session_metrics(session_id);
CREATE INDEX IF NOT EXISTS session_metrics_provider_idx ON session_metrics(provider);
