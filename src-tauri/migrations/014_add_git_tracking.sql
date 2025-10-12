-- Add git tracking fields to agent_sessions
ALTER TABLE agent_sessions ADD COLUMN git_branch TEXT;
ALTER TABLE agent_sessions ADD COLUMN first_commit_hash TEXT;
ALTER TABLE agent_sessions ADD COLUMN latest_commit_hash TEXT;

-- Create index for git branch queries (for future PR matching)
CREATE INDEX IF NOT EXISTS agent_sessions_git_branch_idx ON agent_sessions(git_branch);
