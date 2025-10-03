-- Add sync_failed_reason column to track upload failures
ALTER TABLE agent_sessions ADD COLUMN sync_failed_reason TEXT;
