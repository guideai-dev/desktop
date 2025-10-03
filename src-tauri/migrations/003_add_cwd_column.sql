-- Add cwd (current working directory) column to agent_sessions table
ALTER TABLE agent_sessions ADD COLUMN cwd TEXT;

-- Create index for cwd queries
CREATE INDEX IF NOT EXISTS agent_sessions_cwd_idx ON agent_sessions(cwd);
