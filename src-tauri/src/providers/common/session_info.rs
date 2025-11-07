//! Common session info types used across all providers
//!
//! This module provides the SessionInfo struct that represents session metadata
//! returned by provider scanners.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Session information returned by scanner
///
/// This struct contains all the metadata needed to represent a session
/// from any AI provider (Claude Code, Copilot, OpenCode, Codex, Gemini, Cursor).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionInfo {
    /// Provider identifier (e.g., "claude-code", "github-copilot")
    pub provider: String,

    /// Project name (usually directory name)
    pub project_name: String,

    /// Unique session identifier
    pub session_id: String,

    /// Path to the canonical session file
    pub file_path: PathBuf,

    /// File name only (without path)
    pub file_name: String,

    /// When the session started (if available)
    pub session_start_time: Option<DateTime<Utc>>,

    /// When the session ended (if available)
    pub session_end_time: Option<DateTime<Utc>>,

    /// Session duration in milliseconds (if both start and end times available)
    pub duration_ms: Option<i64>,

    /// File size in bytes
    pub file_size: u64,

    /// Optional in-memory content (used by OpenCode for aggregated sessions)
    pub content: Option<String>,

    /// Working directory for the session
    pub cwd: Option<String>,

    /// Project hash (used by Gemini Code - SHA256 of CWD)
    pub project_hash: Option<String>,
}
