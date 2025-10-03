-- Create projects table for organizing sessions by project
CREATE TABLE IF NOT EXISTS projects (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    github_repo TEXT,
    cwd TEXT NOT NULL UNIQUE,
    type TEXT NOT NULL,
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL
);

-- Create indexes for common queries
CREATE INDEX IF NOT EXISTS projects_name_idx ON projects(name);
CREATE INDEX IF NOT EXISTS projects_type_idx ON projects(type);
CREATE INDEX IF NOT EXISTS projects_updated_at_idx ON projects(updated_at);
