-- Add git diff metrics columns for desktop-only git analysis
ALTER TABLE session_metrics ADD COLUMN git_total_files_changed INTEGER;
ALTER TABLE session_metrics ADD COLUMN git_lines_added INTEGER;
ALTER TABLE session_metrics ADD COLUMN git_lines_removed INTEGER;
ALTER TABLE session_metrics ADD COLUMN git_lines_modified INTEGER;
ALTER TABLE session_metrics ADD COLUMN git_net_lines_changed INTEGER;
ALTER TABLE session_metrics ADD COLUMN git_lines_read_per_line_changed REAL;
ALTER TABLE session_metrics ADD COLUMN git_reads_per_file_changed REAL;
ALTER TABLE session_metrics ADD COLUMN git_lines_changed_per_minute REAL;
ALTER TABLE session_metrics ADD COLUMN git_lines_changed_per_tool_use REAL;
ALTER TABLE session_metrics ADD COLUMN total_lines_read INTEGER;
