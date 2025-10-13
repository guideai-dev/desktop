use git2::{Diff, DiffFormat, DiffOptions, Repository};
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileDiff {
    pub old_path: String,
    pub new_path: String,
    pub change_type: String, // "added", "deleted", "modified", "renamed"
    pub language: Option<String>,
    pub hunks: Vec<String>,
    pub stats: DiffStats,
    pub is_binary: bool,
    pub old_content: Option<String>,
    pub new_content: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffStats {
    pub additions: u32,
    pub deletions: u32,
}

/// Get diff between two commits in a repository, with optional timestamp filtering
///
/// This function implements smart diff logic based on session state and timestamps:
/// - If session is active (live): Shows both committed changes in time range + uncommitted changes
/// - If session is inactive with same hash: Uses timestamp filtering to show changes from that period
/// - If session is inactive with different hashes: Shows all changes between commits (filtered by time if provided)
///
/// # Arguments
/// * `cwd` - Working directory path
/// * `first_commit_hash` - Starting commit hash
/// * `latest_commit_hash` - Ending commit hash (can be same as first)
/// * `is_active` - Whether the session is currently active
/// * `session_start_time` - Optional session start timestamp (Unix milliseconds)
/// * `session_end_time` - Optional session end timestamp (Unix milliseconds)
pub fn get_commit_diff(
    cwd: &str,
    first_commit_hash: &str,
    latest_commit_hash: &str,
    is_active: bool,
    session_start_time: Option<i64>,
    session_end_time: Option<i64>,
) -> Result<Vec<FileDiff>, String> {
    // Open repository
    let repo = Repository::open(cwd)
        .map_err(|e| format!("Failed to open git repository at {}: {}", cwd, e))?;

    // Create diff options
    let mut diff_opts = DiffOptions::new();
    diff_opts.context_lines(3); // Standard 3 lines of context
    diff_opts.include_untracked(true); // Include untracked files
    diff_opts.recurse_untracked_dirs(true); // Recurse into untracked directories

    // Convert timestamps to seconds for git comparison
    let start_time_sec = session_start_time.map(|ms| ms / 1000);
    let end_time_sec = session_end_time.map(|ms| ms / 1000);

    // Get the first commit object
    let first_oid = repo
        .revparse_single(first_commit_hash)
        .map_err(|e| format!("Failed to find first commit {}: {}", first_commit_hash, e))?;
    let first_commit = first_oid
        .peel_to_commit()
        .map_err(|e| format!("Failed to peel first commit: {}", e))?;

    // Determine if we should include uncommitted changes
    let include_uncommitted = is_active || (first_commit_hash == latest_commit_hash && end_time_sec.is_some());

    // If commits are the same, we're looking at changes in the repo during the session period
    // Even if no commits were made, we want to show what changed in the branch during that time
    if first_commit_hash == latest_commit_hash {
        println!("DEBUG: Same hash scenario - first_hash: {}, is_active: {}", first_commit_hash, is_active);

        let first_tree = first_commit
            .tree()
            .map_err(|e| format!("Failed to get first commit tree: {}", e))?;

        // For sessions with same hash, we look at what exists in the branch now
        // This could be the working directory (if active) or HEAD (if inactive)
        // Either way, we show the diff from the session's starting point to the current state
        if is_active {
            println!("DEBUG: Active session - diffing tree to workdir");
            // Active: show working directory changes
            let diff = repo.diff_tree_to_workdir_with_index(Some(&first_tree), Some(&mut diff_opts))
                .map_err(|e| format!("Failed to create diff to working directory: {}", e))?;

            let delta_count = diff.deltas().len();
            println!("DEBUG: Created diff with {} deltas", delta_count);

            let result = parse_diff(&repo, diff, Some(&first_tree), None, cwd);
            println!("DEBUG: parse_diff returned {} files", result.as_ref().map(|f| f.len()).unwrap_or(0));
            return result;
        } else {
            println!("DEBUG: Inactive session - diffing first_tree to workdir (showing uncommitted changes from session period)");
            // Inactive: show diff from first_commit to current working directory
            // Even though the session ended, we want to show what changes exist that were made during that time
            let diff = repo.diff_tree_to_workdir_with_index(Some(&first_tree), Some(&mut diff_opts))
                .map_err(|e| format!("Failed to create tree to workdir diff: {}", e))?;

            let delta_count = diff.deltas().len();
            println!("DEBUG: Created diff with {} deltas", delta_count);

            let result = parse_diff(&repo, diff, Some(&first_tree), None, cwd);
            println!("DEBUG: parse_diff returned {} files", result.as_ref().map(|f| f.len()).unwrap_or(0));
            return result;
        }
    }

    // Different commits - show all changes between them (no timestamp filtering)
    // The session tracking already captured the correct start and end commits
    println!("DEBUG: Different commits scenario - first: {}, latest: {}", first_commit_hash, latest_commit_hash);

    let latest_oid = repo
        .revparse_single(latest_commit_hash)
        .map_err(|e| format!("Failed to find latest commit {}: {}", latest_commit_hash, e))?;
    let latest_commit = latest_oid
        .peel_to_commit()
        .map_err(|e| format!("Failed to peel latest commit: {}", e))?;

    // Use the provided commits directly - no timestamp filtering needed
    let actual_first_commit = first_commit;
    let actual_latest_commit = latest_commit;

    println!("DEBUG: Comparing commits - first: {}, latest: {}", actual_first_commit.id(), actual_latest_commit.id());

    // Get trees for the actual commits we're diffing
    let first_tree = actual_first_commit
        .tree()
        .map_err(|e| format!("Failed to get first commit tree: {}", e))?;
    let latest_tree = actual_latest_commit
        .tree()
        .map_err(|e| format!("Failed to get latest commit tree: {}", e))?;

    // Create base diff between the two commit trees
    let mut diff = repo
        .diff_tree_to_tree(Some(&first_tree), Some(&latest_tree), Some(&mut diff_opts))
        .map_err(|e| format!("Failed to create diff: {}", e))?;

    println!("DEBUG: Created tree-to-tree diff with {} deltas", diff.deltas().len());

    // If session is active or we're showing uncommitted changes, merge with working directory diff
    if include_uncommitted {
        println!("DEBUG: Including uncommitted changes (is_active: {})", is_active);
        let workdir_diff = repo
            .diff_tree_to_workdir_with_index(Some(&latest_tree), Some(&mut diff_opts))
            .map_err(|e| format!("Failed to create working directory diff: {}", e))?;

        println!("DEBUG: Workdir diff has {} deltas", workdir_diff.deltas().len());

        // Merge the diffs
        diff.merge(&workdir_diff)
            .map_err(|e| format!("Failed to merge diffs: {}", e))?;

        println!("DEBUG: After merge, diff has {} deltas", diff.deltas().len());
    }

    // Parse diff into FileDiff structures
    let result = parse_diff(&repo, diff, Some(&first_tree), if include_uncommitted { None } else { Some(&latest_tree) }, cwd);
    println!("DEBUG: parse_diff returned {} files", result.as_ref().map(|f| f.len()).unwrap_or(0));
    result
}

/// Filter commits to those within a specific time range
/// Returns the earliest and latest commits within the range
fn filter_commits_by_time<'repo>(
    repo: &'repo Repository,
    start_commit: &git2::Commit<'repo>,
    end_commit: &git2::Commit<'repo>,
    start_time: i64,
    end_time: i64,
) -> Result<(git2::Commit<'repo>, git2::Commit<'repo>), String> {
    // Walk from start to end commit, collecting those in time range
    let mut revwalk = repo.revwalk()
        .map_err(|e| format!("Failed to create revwalk: {}", e))?;

    // Set up walk from end back to start
    revwalk.push(end_commit.id())
        .map_err(|e| format!("Failed to push end commit: {}", e))?;
    revwalk.hide(start_commit.id())
        .map_err(|e| format!("Failed to hide start commit: {}", e))?;

    let mut commits_in_range = Vec::new();

    // Check start commit first
    let start_commit_time = start_commit.time().seconds();
    if start_commit_time >= start_time && start_commit_time <= end_time {
        commits_in_range.push(start_commit.clone());
    }

    // Walk commits between start and end
    for oid in revwalk {
        let oid = oid.map_err(|e| format!("Failed to walk commit: {}", e))?;
        let commit = repo.find_commit(oid)
            .map_err(|e| format!("Failed to find commit: {}", e))?;

        let commit_time = commit.time().seconds();
        if commit_time >= start_time && commit_time <= end_time {
            commits_in_range.push(commit);
        }
    }

    // Check end commit
    let end_commit_time = end_commit.time().seconds();
    if end_commit_time >= start_time && end_commit_time <= end_time {
        commits_in_range.push(end_commit.clone());
    }

    // If no commits in range, fall back to original commits
    if commits_in_range.is_empty() {
        return Ok((start_commit.clone(), end_commit.clone()));
    }

    // Sort by time (oldest first)
    commits_in_range.sort_by_key(|c| c.time().seconds());

    // Return first and last commits in the time range
    let first = commits_in_range.first().unwrap().clone();
    let last = commits_in_range.last().unwrap().clone();

    Ok((first, last))
}

/// Parse git2 Diff into structured FileDiff objects
fn parse_diff(
    repo: &Repository,
    diff: Diff,
    old_tree: Option<&git2::Tree>,
    new_tree: Option<&git2::Tree>,
    cwd: &str,
) -> Result<Vec<FileDiff>, String> {
    println!("DEBUG parse_diff: old_tree: {}, new_tree: {}", old_tree.is_some(), new_tree.is_some());

    let mut file_diffs: Vec<FileDiff> = Vec::new();
    let mut current_file: Option<FileDiff> = None;
    let mut current_file_content = String::new();
    let mut current_hunk_header: Option<String> = None;
    let mut file_headers_added = false;

    // First, handle untracked files separately (they won't appear in print output)
    for delta in diff.deltas() {
        if delta.status() == git2::Delta::Untracked {
            let new_path = delta.new_file().path().unwrap_or(Path::new("")).to_string_lossy().to_string();

            // Read the entire file content from working directory
            let file_content = get_file_content_from_workdir(cwd, &new_path).ok();

            // Create hunks that show the entire file as added
            let mut hunk_content = String::new();
            hunk_content.push_str(&format!("Index: {}\n", new_path));
            hunk_content.push_str("===================================================================\n");
            hunk_content.push_str("--- /dev/null\t\n");
            hunk_content.push_str(&format!("+++ {}\t\n", new_path));

            if let Some(content) = &file_content {
                let lines: Vec<&str> = content.lines().collect();
                let line_count = lines.len();

                hunk_content.push_str(&format!("@@ -0,0 +1,{} @@", line_count));

                for line in lines {
                    hunk_content.push('\n');
                    hunk_content.push('+');
                    hunk_content.push_str(line);
                }
                hunk_content.push('\n');
            }

            file_diffs.push(FileDiff {
                old_path: String::new(),
                new_path: new_path.clone(),
                change_type: "added".to_string(),
                language: detect_language(&new_path),
                hunks: vec![hunk_content],
                stats: DiffStats {
                    additions: file_content.as_ref().map(|c| c.lines().count() as u32).unwrap_or(0),
                    deletions: 0,
                },
                is_binary: delta.new_file().is_binary(),
                old_content: None,
                new_content: file_content,
            });
        }
    }

    // Print diff and collect output
    diff.print(DiffFormat::Patch, |delta, hunk, line| {
        let old_path = delta.old_file().path().unwrap_or(Path::new("")).to_string_lossy().to_string();
        let new_path = delta.new_file().path().unwrap_or(Path::new("")).to_string_lossy().to_string();

        // Detect file change
        if current_file.is_none() ||
           current_file.as_ref().unwrap().new_path != new_path {
            // Save previous file if exists
            if let Some(mut file) = current_file.take() {
                if !current_file_content.is_empty() {
                    // Store the entire file diff as a single string
                    file.hunks.push(current_file_content.clone());
                    current_file_content.clear();
                }
                file_diffs.push(file);
            }

            // Start new file
            let change_type = match delta.status() {
                git2::Delta::Added => "added",
                git2::Delta::Untracked => "added", // Treat untracked files as added
                git2::Delta::Deleted => "deleted",
                git2::Delta::Modified => "modified",
                git2::Delta::Renamed => "renamed",
                _ => "modified",
            };

            current_file = Some(FileDiff {
                old_path: old_path.clone(),
                new_path: new_path.clone(),
                change_type: change_type.to_string(),
                language: detect_language(&new_path),
                hunks: Vec::new(),
                stats: DiffStats { additions: 0, deletions: 0 },
                is_binary: delta.new_file().is_binary(),
                old_content: None,
                new_content: None,
            });
            current_hunk_header = None;
            file_headers_added = false;
        }

        // Add file headers once at the beginning (matching @git-diff-view format)
        if !file_headers_added {
            // Add Index header
            current_file_content.push_str(&format!("Index: {}\n", new_path));
            current_file_content.push_str("===================================================================\n");
            // Add file headers with tabs (like the library expects)
            // Use /dev/null for added files (when old_path is empty)
            let old_path_display = if old_path.is_empty() { "/dev/null" } else { &old_path };
            current_file_content.push_str(&format!("--- {}\t\n", old_path_display));
            current_file_content.push_str(&format!("+++ {}\t\n", new_path));
            file_headers_added = true;
        }

        // Handle hunk header - detect new hunk by comparing header
        if let Some(hunk_data) = hunk {
            let header = format!(
                "@@ -{},{} +{},{} @@",
                hunk_data.old_start(),
                hunk_data.old_lines(),
                hunk_data.new_start(),
                hunk_data.new_lines()
            );

            // Check if this is a NEW hunk (different header than current)
            if current_hunk_header.as_ref() != Some(&header) {
                // Start new hunk
                if !current_file_content.is_empty() && !current_file_content.ends_with('\n') {
                    current_file_content.push('\n');
                }
                current_file_content.push_str(&header);
                current_hunk_header = Some(header);
            }
        }

        // Add line to current file content
        let origin = line.origin();
        let content = String::from_utf8_lossy(line.content());

        match origin {
            '+' | '-' | ' ' => {
                current_file_content.push('\n');
                current_file_content.push(origin);
                current_file_content.push_str(&content.trim_end_matches('\n'));

                // Update stats
                if let Some(ref mut file) = current_file {
                    if origin == '+' {
                        file.stats.additions += 1;
                    } else if origin == '-' {
                        file.stats.deletions += 1;
                    }
                }
            }
            _ => {}
        }

        true // Continue iteration
    })
    .map_err(|e| format!("Failed to print diff: {}", e))?;

    // Save last file
    if let Some(mut file) = current_file.take() {
        if !current_file_content.is_empty() {
            file.hunks.push(current_file_content);
        }
        file_diffs.push(file);
    }

    println!("DEBUG parse_diff: After diff.print, collected {} files", file_diffs.len());

    // Extract file contents for syntax highlighting
    for file_diff in &mut file_diffs {
        if file_diff.is_binary {
            continue; // Skip binary files
        }

        // Get old file content
        if !file_diff.old_path.is_empty() && file_diff.change_type != "added" {
            if let Some(tree) = old_tree {
                file_diff.old_content = get_file_content_from_tree(repo, tree, &file_diff.old_path).ok();
            }
        }

        // Get new file content
        if !file_diff.new_path.is_empty() && file_diff.change_type != "deleted" {
            if let Some(tree) = new_tree {
                // From tree (committed)
                file_diff.new_content = get_file_content_from_tree(repo, tree, &file_diff.new_path).ok();
            } else {
                // From working directory (uncommitted changes)
                file_diff.new_content = get_file_content_from_workdir(cwd, &file_diff.new_path).ok();
            }
        }

        println!("DEBUG parse_diff: File {} has {} hunks", file_diff.new_path, file_diff.hunks.len());
        if file_diff.hunks.is_empty() {
            println!("DEBUG parse_diff: WARNING - File {} has no hunks!", file_diff.new_path);
        }
    }

    println!("DEBUG parse_diff: Returning {} total files", file_diffs.len());

    Ok(file_diffs)
}

/// Get file content from a git tree
fn get_file_content_from_tree(repo: &Repository, tree: &git2::Tree, path: &str) -> Result<String, String> {
    let entry = tree
        .get_path(Path::new(path))
        .map_err(|e| format!("File not found in tree: {}", e))?;

    let object = entry
        .to_object(repo)
        .map_err(|e| format!("Failed to get object: {}", e))?;

    let blob = object
        .as_blob()
        .ok_or_else(|| "Object is not a blob".to_string())?;

    let content = String::from_utf8(blob.content().to_vec())
        .map_err(|_| "File content is not valid UTF-8".to_string())?;

    Ok(content)
}

/// Get file content from working directory
fn get_file_content_from_workdir(cwd: &str, path: &str) -> Result<String, String> {
    let full_path = Path::new(cwd).join(path);
    std::fs::read_to_string(&full_path)
        .map_err(|e| format!("Failed to read file from working directory: {}", e))
}

/// Detect programming language from file extension
fn detect_language(file_path: &str) -> Option<String> {
    let path = Path::new(file_path);
    let ext = path.extension()?.to_str()?;

    let lang = match ext {
        "rs" => "rust",
        "ts" | "tsx" => "typescript",
        "js" | "jsx" => "javascript",
        "py" => "python",
        "go" => "go",
        "java" => "java",
        "cpp" | "cc" | "cxx" => "cpp",
        "c" => "c",
        "h" | "hpp" => "cpp",
        "rb" => "ruby",
        "php" => "php",
        "swift" => "swift",
        "kt" => "kotlin",
        "cs" => "csharp",
        "sh" | "bash" => "bash",
        "sql" => "sql",
        "html" => "html",
        "css" => "css",
        "scss" | "sass" => "scss",
        "json" => "json",
        "yaml" | "yml" => "yaml",
        "toml" => "toml",
        "xml" => "xml",
        "md" => "markdown",
        _ => return None,
    };

    Some(lang.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_language() {
        assert_eq!(detect_language("main.rs"), Some("rust".to_string()));
        assert_eq!(detect_language("App.tsx"), Some("typescript".to_string()));
        assert_eq!(detect_language("script.py"), Some("python".to_string()));
        assert_eq!(detect_language("unknown.xyz"), None);
    }
}
