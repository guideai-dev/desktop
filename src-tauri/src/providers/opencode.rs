use super::sort_projects_by_modified;
use crate::config::ProjectInfo;
use chrono::{DateTime, Utc};
use serde::Deserialize;
use shellexpand::tilde;
use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

#[derive(Debug, Deserialize, Default)]
struct OpenCodeTime {
    created: Option<i64>,
    initialized: Option<i64>,
    updated: Option<i64>,
}

#[derive(Debug, Deserialize, Default)]
struct OpenCodeProjectRecord {
    worktree: Option<String>,
    #[serde(default)]
    time: OpenCodeTime,
}

pub fn scan_projects(home_directory: &str) -> Result<Vec<ProjectInfo>, String> {
    let expanded = tilde(home_directory);
    let primary_base = PathBuf::from(expanded.into_owned());

    let mut base_candidates = Vec::new();

    let mut push_candidate = |candidate: PathBuf| {
        if !base_candidates
            .iter()
            .any(|existing| existing == &candidate)
        {
            base_candidates.push(candidate);
        }
    };

    push_candidate(primary_base.clone());

    if let Some(parent) = primary_base.parent() {
        if primary_base
            .file_name()
            .map(|name| name == OsStr::new(".opencode"))
            .unwrap_or(false)
        {
            push_candidate(parent.join(".local/share/opencode"));
        }
    }

    if let Some(home_dir) = dirs::home_dir() {
        push_candidate(home_dir.join(".local/share/opencode"));
    }

    if let Some(data_dir) = dirs::data_dir() {
        push_candidate(data_dir.join("opencode"));
    }

    let storage_dir = base_candidates.into_iter().find_map(|base| {
        let storage = base.join("storage").join("project");
        storage.is_dir().then_some(storage)
    });

    let Some(project_storage_dir) = storage_dir else {
        return Ok(Vec::new());
    };

    let entries = fs::read_dir(&project_storage_dir)
        .map_err(|e| format!("Failed to read OpenCode project storage: {}", e))?;

    let mut projects = Vec::new();

    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|ext| ext.to_str()) != Some("json") {
            continue;
        }

        let Ok(contents) = fs::read_to_string(&path) else {
            continue;
        };

        let Ok(record) = serde_json::from_str::<OpenCodeProjectRecord>(&contents) else {
            continue;
        };

        let Some(worktree) = record.worktree else {
            continue;
        };

        let worktree_path = Path::new(&worktree);
        if !worktree_path.exists() {
            continue;
        }

        let Some(name) = worktree_path.file_name().and_then(|n| n.to_str()) else {
            continue;
        };

        let metadata_time = fs::metadata(worktree_path)
            .and_then(|metadata| metadata.modified())
            .map(|time| DateTime::<Utc>::from(time))
            .ok();

        let fallback_time = record
            .time
            .updated
            .or(record.time.initialized)
            .or(record.time.created)
            .and_then(DateTime::<Utc>::from_timestamp_millis);

        let modified = metadata_time
            .or(fallback_time)
            .unwrap_or_else(|| DateTime::<Utc>::from(SystemTime::UNIX_EPOCH));

        projects.push((
            modified,
            ProjectInfo {
                name: name.to_string(),
                path: worktree_path.to_string_lossy().to_string(),
                last_modified: modified.to_rfc3339(),
            },
        ));
    }

    Ok(sort_projects_by_modified(projects))
}
