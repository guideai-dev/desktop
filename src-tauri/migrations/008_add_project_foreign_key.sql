-- Add index for project_id lookups
-- Note: project_id column already exists in agent_sessions table (from migration 001)
-- This migration just adds an index for efficient project-based queries

CREATE INDEX IF NOT EXISTS agent_sessions_project_id_idx ON agent_sessions(project_id);

-- Note: SQLite doesn't support adding foreign key constraints to existing tables
-- The foreign key relationship will be enforced at the application level
