-- Create agent_sessions table for tracking local session files
CREATE TABLE IF NOT EXISTS agent_sessions (
    id TEXT PRIMARY KEY,
    provider TEXT NOT NULL,
    project_name TEXT NOT NULL,
    session_id TEXT NOT NULL,
    file_name TEXT NOT NULL,
    file_path TEXT NOT NULL,
    file_size INTEGER NOT NULL,
    session_start_time INTEGER,
    session_end_time INTEGER,
    duration_ms INTEGER,
    processing_status TEXT DEFAULT 'pending',
    queued_at INTEGER,
    processed_at INTEGER,
    assessment_status TEXT DEFAULT 'not_started',
    assessment_completed_at INTEGER,
    project_id TEXT,
    ai_model_summary TEXT,
    ai_model_quality_score INTEGER,
    ai_model_metadata TEXT,
    synced_to_server INTEGER DEFAULT 0,
    synced_at INTEGER,
    server_session_id TEXT,
    created_at INTEGER NOT NULL,
    uploaded_at INTEGER NOT NULL
);

-- Create indexes for common queries
CREATE INDEX IF NOT EXISTS agent_sessions_provider_idx ON agent_sessions(provider);
CREATE INDEX IF NOT EXISTS agent_sessions_session_idx ON agent_sessions(session_id);
CREATE INDEX IF NOT EXISTS agent_sessions_created_at_idx ON agent_sessions(created_at);
CREATE INDEX IF NOT EXISTS agent_sessions_sync_idx ON agent_sessions(synced_to_server);
