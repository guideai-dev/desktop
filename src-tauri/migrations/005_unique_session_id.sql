-- Add unique constraint to session_id and clean up duplicates

-- First, delete duplicate sessions, keeping only the most recent one (highest id)
DELETE FROM agent_sessions
WHERE id NOT IN (
    SELECT MAX(id)
    FROM agent_sessions
    GROUP BY session_id
);

-- Create a unique index on session_id to prevent future duplicates
CREATE UNIQUE INDEX IF NOT EXISTS agent_sessions_session_id_unique ON agent_sessions(session_id);
