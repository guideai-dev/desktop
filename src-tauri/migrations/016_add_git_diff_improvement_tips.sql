-- Add git_diff_improvement_tips column (missed in migration 015)
ALTER TABLE session_metrics ADD COLUMN git_diff_improvement_tips TEXT;
