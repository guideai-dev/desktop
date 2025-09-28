use super::sort_projects_by_modified;
use crate::config::ProjectInfo;
use chrono::{DateTime, Utc};
use shellexpand::tilde;
use std::fs;
use std::path::Path;

pub fn scan_projects(home_directory: &str) -> Result<Vec<ProjectInfo>, String> {
    let expanded = tilde(home_directory);
    let base_path = Path::new(expanded.as_ref());

    if !base_path.exists() {
        return Ok(Vec::new());
    }

    let projects_path = base_path.join("projects");
    if !projects_path.exists() {
        return Ok(Vec::new());
    }

    let entries = fs::read_dir(&projects_path)
        .map_err(|e| format!("Failed to read projects directory: {}", e))?;

    let mut projects = Vec::new();

    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }

        let Some(name) = path.file_name().and_then(|n| n.to_str()) else {
            continue;
        };

        let Ok(metadata) = entry.metadata() else {
            continue;
        };

        let Ok(modified) = metadata.modified() else {
            continue;
        };

        let modified: DateTime<Utc> = modified.into();
        projects.push((
            modified,
            ProjectInfo {
                name: name.to_string(),
                path: path.to_string_lossy().to_string(),
                last_modified: modified.to_rfc3339(),
            },
        ));
    }

    Ok(sort_projects_by_modified(projects))
}
