-- Add unique constraint to session_metrics.session_id and clean up duplicates

-- Clean up duplicate session_metrics, keeping only the most recent one (highest id)
DELETE FROM session_metrics
WHERE id NOT IN (
    SELECT MAX(id)
    FROM session_metrics
    GROUP BY session_id
);

-- Create unique index to prevent future duplicates
CREATE UNIQUE INDEX IF NOT EXISTS session_metrics_session_id_unique ON session_metrics(session_id);
