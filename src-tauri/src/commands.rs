use crate::auth_server::{AuthError, AuthServer};
use crate::config::{
    clear_config, delete_provider_config, ensure_logs_dir, load_config, load_provider_config,
    save_config, save_provider_config, ActivityLogEntry, GuideAIConfig, ProjectInfo,
    ProviderConfig,
};
use crate::logging::{read_provider_logs, LogEntry};
use crate::providers::{ClaudeWatcher, ClaudeWatcherStatus, SessionInfo, scan_all_sessions};
use crate::upload_queue::{UploadQueue, UploadStatus};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tauri::State;

#[tauri::command]
pub async fn load_config_command() -> Result<GuideAIConfig, String> {
    load_config().map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn save_config_command(config: GuideAIConfig) -> Result<(), String> {
    save_config(&config).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn clear_config_command() -> Result<(), String> {
    clear_config().map_err(|e| e.to_string())
}

#[derive(Debug, Serialize, Deserialize)]
struct SessionResponse {
    user: UserInfo,
}

#[derive(Debug, Serialize, Deserialize)]
struct UserInfo {
    username: String,
    name: Option<String>,
    #[serde(rename = "avatarUrl")]
    avatar_url: Option<String>,
}

#[tauri::command]
pub async fn login_command(server_url: String) -> Result<(), String> {
    // Start the auth server - this handles automatic port selection and cleanup
    let (auth_server, result_rx) = AuthServer::start()
        .await
        .map_err(|e| format!("Failed to start authentication server: {}", e))?;

    let callback_url = &auth_server.callback_url;
    let auth_url = format!(
        "{}/auth/cli?redirect_uri={}",
        server_url,
        urlencoding::encode(callback_url)
    );

    // Log server details for debugging
    println!(
        "Authentication server started on port: {}",
        auth_server.port
    );
    println!("Callback URL: {}", callback_url);
    println!("Opening browser to: {}", auth_url);

    // Authentication flow with guaranteed cleanup
    let result = async {
        // Open the browser to the OAuth URL
        open::that(auth_url).map_err(|e| format!("Failed to open browser: {}", e))?;

        // Wait for callback with 5-minute timeout (matching CLI behavior)
        let auth_data =
            AuthServer::wait_for_callback_with_timeout(result_rx, Duration::from_secs(300))
                .await
                .map_err(|e| match e {
                    AuthError::TimeoutError => {
                        "Authentication timed out after 5 minutes. Please try again.".to_string()
                    }
                    AuthError::CallbackError(msg) => format!("Authentication failed: {}", msg),
                    _ => format!("Authentication error: {}", e),
                })?;

        // Verify the credentials by calling the session endpoint
        println!("Verifying session with server: {}", server_url);
        let user_info = verify_session(&server_url, &auth_data.api_key)
            .await
            .map_err(|e| format!("Failed to verify credentials: {}", e))?;
        println!(
            "Session verified successfully for user: {}",
            user_info.username
        );

        // Save the complete configuration
        let config = GuideAIConfig {
            api_key: Some(auth_data.api_key.clone()),
            server_url: Some(server_url.clone()),
            username: Some(user_info.username.clone()),
            name: user_info.name.clone(),
            avatar_url: user_info.avatar_url.clone(),
            tenant_id: Some(auth_data.tenant_id.clone()),
            tenant_name: Some(auth_data.tenant_name.clone()),
        };

        println!("Saving config: {:?}", config);
        save_config(&config).map_err(|e| format!("Failed to save configuration: {}", e))?;
        println!("Config saved successfully");

        Ok::<(), String>(())
    }
    .await;

    // ALWAYS shutdown the server, regardless of success or failure
    auth_server.shutdown().await;

    result
}

async fn verify_session(
    server_url: &str,
    api_key: &str,
) -> Result<UserInfo, Box<dyn std::error::Error + Send + Sync>> {
    let client = reqwest::Client::new();
    let url = format!("{}/auth/session", server_url);

    println!("Making request to: {}", url);
    println!("Using API key: {}...", &api_key[..20]); // Only show first 20 chars for security

    let response = client
        .get(&url)
        .header("Authorization", format!("Bearer {}", api_key))
        .send()
        .await?;

    let status = response.status();
    println!("Response status: {}", status);

    if !status.is_success() {
        let error_text = response
            .text()
            .await
            .unwrap_or_else(|_| "Unable to read error response".to_string());
        println!("Error response body: {}", error_text);
        return Err(format!(
            "Session verification failed with status: {} - {}",
            status, error_text
        )
        .into());
    }

    let response_text = response.text().await?;
    println!("Response body: {}", response_text);

    let session: SessionResponse = serde_json::from_str(&response_text)?;
    Ok(session.user)
}

#[tauri::command]
pub async fn logout_command() -> Result<(), String> {
    clear_config_command().await
}

// Provider config commands
#[tauri::command]
pub async fn load_provider_config_command(provider_id: String) -> Result<ProviderConfig, String> {
    load_provider_config(&provider_id).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn save_provider_config_command(
    provider_id: String,
    config: ProviderConfig,
) -> Result<(), String> {
    save_provider_config(&provider_id, &config).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn delete_provider_config_command(provider_id: String) -> Result<(), String> {
    delete_provider_config(&provider_id).map_err(|e| e.to_string())
}

// Project scanning commands
#[tauri::command]
pub async fn scan_projects_command(
    provider_id: String,
    directory: String,
) -> Result<Vec<ProjectInfo>, String> {
    crate::providers::scan_projects(&provider_id, &directory)
}

// Activity logging commands
#[tauri::command]
pub async fn add_activity_log_command(entry: ActivityLogEntry) -> Result<(), String> {
    ensure_logs_dir().map_err(|e| e.to_string())?;

    let logs_dir = crate::config::get_logs_dir().map_err(|e| e.to_string())?;
    let today = chrono::Utc::now().format("%Y-%m-%d").to_string();
    let log_file = logs_dir.join(format!("{}.jsonl", today));

    let log_line = serde_json::to_string(&entry).map_err(|e| e.to_string())?;
    let log_entry = format!("{}\n", log_line);

    use std::io::Write;
    if log_file.exists() {
        let mut file = fs::OpenOptions::new()
            .append(true)
            .open(&log_file)
            .map_err(|e| e.to_string())?;
        file.write_all(log_entry.as_bytes())
            .map_err(|e| e.to_string())?;
    } else {
        fs::write(&log_file, log_entry).map_err(|e| e.to_string())?;
    }

    Ok(())
}

#[tauri::command]
pub async fn get_activity_logs_command(
    limit: Option<usize>,
) -> Result<Vec<ActivityLogEntry>, String> {
    let logs_dir = crate::config::get_logs_dir().map_err(|e| e.to_string())?;

    if !logs_dir.exists() {
        return Ok(Vec::new());
    }

    let mut all_logs = Vec::new();

    // Read log files from most recent to oldest
    let mut log_files = Vec::new();
    if let Ok(entries) = fs::read_dir(&logs_dir) {
        for entry in entries {
            if let Ok(entry) = entry {
                let path = entry.path();
                if path.extension().and_then(|ext| ext.to_str()) == Some("jsonl") {
                    log_files.push(path);
                }
            }
        }
    }

    log_files.sort_by(|a, b| b.cmp(a)); // Reverse sort for most recent first

    for log_file in log_files {
        if let Ok(content) = fs::read_to_string(&log_file) {
            for line in content.lines().rev() {
                // Reverse lines to get most recent first
                if let Ok(entry) = serde_json::from_str::<ActivityLogEntry>(line) {
                    all_logs.push(entry);
                    if let Some(limit) = limit {
                        if all_logs.len() >= limit {
                            break;
                        }
                    }
                }
            }
        }
        if let Some(limit) = limit {
            if all_logs.len() >= limit {
                break;
            }
        }
    }

    Ok(all_logs)
}

// Application state for managing watchers and upload queue
pub struct AppState {
    pub watchers: Arc<Mutex<HashMap<String, ClaudeWatcher>>>,
    pub upload_queue: Arc<UploadQueue>,
}

impl AppState {
    pub fn new() -> Self {
        let upload_queue = Arc::new(UploadQueue::new());

        // Start the upload queue processor
        if let Err(e) = upload_queue.start_processing() {
            eprintln!("Failed to start upload queue processor: {}", e);
        }

        Self {
            watchers: Arc::new(Mutex::new(HashMap::new())),
            upload_queue,
        }
    }
}

// Claude watcher commands
#[tauri::command]
pub async fn start_claude_watcher(
    state: State<'_, AppState>,
    projects: Vec<String>,
) -> Result<(), String> {
    // Update upload queue with current config
    if let Ok(config) = load_config() {
        state.upload_queue.set_config(config);
    }

    // Create new watcher
    let watcher = ClaudeWatcher::new(projects, Arc::clone(&state.upload_queue))
        .map_err(|e| format!("Failed to create Claude watcher: {}", e))?;

    // Store watcher in state
    if let Ok(mut watchers) = state.watchers.lock() {
        watchers.insert("claude-code".to_string(), watcher);
    }

    Ok(())
}

#[tauri::command]
pub async fn stop_claude_watcher(state: State<'_, AppState>) -> Result<(), String> {
    if let Ok(mut watchers) = state.watchers.lock() {
        if let Some(watcher) = watchers.remove("claude-code") {
            watcher.stop();
        }
    }
    Ok(())
}

#[tauri::command]
pub async fn get_claude_watcher_status(state: State<'_, AppState>) -> Result<ClaudeWatcherStatus, String> {
    if let Ok(watchers) = state.watchers.lock() {
        if let Some(watcher) = watchers.get("claude-code") {
            Ok(watcher.get_status())
        } else {
            Ok(ClaudeWatcherStatus {
                is_running: false,
                pending_uploads: 0,
                processing_uploads: 0,
                failed_uploads: 0,
            })
        }
    } else {
        Err("Failed to access watcher state".to_string())
    }
}

#[tauri::command]
pub async fn get_upload_queue_status(state: State<'_, AppState>) -> Result<UploadStatus, String> {
    Ok(state.upload_queue.get_status())
}

#[tauri::command]
pub async fn retry_failed_uploads(state: State<'_, AppState>) -> Result<(), String> {
    state.upload_queue.retry_failed();
    Ok(())
}

#[tauri::command]
pub async fn clear_failed_uploads(state: State<'_, AppState>) -> Result<(), String> {
    state.upload_queue.clear_failed();
    Ok(())
}

#[tauri::command]
pub async fn get_provider_logs(
    provider: String,
    max_lines: Option<usize>,
) -> Result<Vec<LogEntry>, String> {
    read_provider_logs(&provider, max_lines).map_err(|e| e.to_string())
}

// Session sync state for tracking progress
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionSyncProgress {
    pub is_scanning: bool,
    pub is_syncing: bool,
    pub total_sessions: usize,
    pub synced_sessions: usize,
    pub current_provider: String,
    pub current_project: String,
    pub sessions_found: Vec<SessionInfo>,
    pub errors: Vec<String>,
    pub is_complete: bool,
}

impl Default for SessionSyncProgress {
    fn default() -> Self {
        Self {
            is_scanning: false,
            is_syncing: false,
            total_sessions: 0,
            synced_sessions: 0,
            current_provider: String::new(),
            current_project: String::new(),
            sessions_found: Vec::new(),
            errors: Vec::new(),
            is_complete: false,
        }
    }
}

// Provider-specific sync state - using std::sync::OnceLock for thread-safe initialization
use std::sync::OnceLock;
static SYNC_PROGRESS: OnceLock<Arc<Mutex<HashMap<String, SessionSyncProgress>>>> = OnceLock::new();

fn get_sync_progress_map() -> &'static Arc<Mutex<HashMap<String, SessionSyncProgress>>> {
    SYNC_PROGRESS.get_or_init(|| Arc::new(Mutex::new(HashMap::new())))
}

fn get_sync_progress_for_provider(provider_id: &str) -> Result<SessionSyncProgress, String> {
    if let Ok(progress_map) = get_sync_progress_map().lock() {
        Ok(progress_map.get(provider_id).cloned().unwrap_or_default())
    } else {
        Err("Failed to access sync progress".to_string())
    }
}

fn update_sync_progress_for_provider<F>(provider_id: &str, updater: F) -> Result<(), String>
where
    F: FnOnce(&mut SessionSyncProgress),
{
    if let Ok(mut progress_map) = get_sync_progress_map().lock() {
        let progress = progress_map.entry(provider_id.to_string()).or_default();
        updater(progress);
        Ok(())
    } else {
        Err("Failed to access sync progress".to_string())
    }
}

#[tauri::command]
pub async fn scan_historical_sessions(
    provider_id: String,
) -> Result<Vec<SessionInfo>, String> {
    // Update progress
    update_sync_progress_for_provider(&provider_id, |progress| {
        progress.is_scanning = true;
        progress.current_provider = provider_id.clone();
        progress.errors.clear();
        progress.sessions_found.clear();
    }).ok();

    // Load provider config
    let config = load_provider_config(&provider_id)
        .map_err(|e| format!("Failed to load provider config: {}", e))?;

    if !config.enabled {
        return Err(format!("Provider '{}' is not enabled", provider_id));
    }

    // Scan for sessions
    let sessions = scan_all_sessions(&provider_id, &config.home_directory)
        .map_err(|e| {
            // Update progress with error
            update_sync_progress_for_provider(&provider_id, |progress| {
                progress.errors.push(e.clone());
                progress.is_scanning = false;
            }).ok();
            e
        })?;

    // Update progress
    update_sync_progress_for_provider(&provider_id, |progress| {
        progress.is_scanning = false;
        progress.total_sessions = sessions.len();
        progress.sessions_found = sessions.clone();
    }).ok();

    Ok(sessions)
}

#[tauri::command]
pub async fn sync_historical_sessions(
    state: State<'_, AppState>,
    provider_id: String,
) -> Result<(), String> {
    // Update upload queue with current config
    if let Ok(config) = load_config() {
        state.upload_queue.set_config(config);
    }

    // Update progress
    update_sync_progress_for_provider(&provider_id, |progress| {
        progress.is_syncing = true;
        progress.synced_sessions = 0;
        progress.is_complete = false;
        progress.errors.clear();
    }).ok();

    // Get sessions from progress state (they should have been scanned first)
    let sessions = get_sync_progress_for_provider(&provider_id)?
        .sessions_found;

    if sessions.is_empty() {
        return Err("No sessions found to sync. Run scan first.".to_string());
    }

    // Add all sessions to upload queue
    for session in &sessions {
        // Update current progress
        update_sync_progress_for_provider(&provider_id, |progress| {
            progress.current_project = session.project_name.clone();
        }).ok();

        // Add to upload queue with enhanced metadata
        if let Err(e) = state.upload_queue.add_historical_session(session) {
            update_sync_progress_for_provider(&provider_id, |progress| {
                progress.errors.push(format!("Failed to queue {}: {}", session.file_name, e));
            }).ok();
        } else {
            // Increment synced count
            update_sync_progress_for_provider(&provider_id, |progress| {
                progress.synced_sessions += 1;
            }).ok();
        }
    }

    // Mark sync as complete
    update_sync_progress_for_provider(&provider_id, |progress| {
        progress.is_syncing = false;
        progress.is_complete = true;
    }).ok();

    Ok(())
}

#[tauri::command]
pub async fn get_session_sync_progress(provider_id: String) -> Result<SessionSyncProgress, String> {
    get_sync_progress_for_provider(&provider_id)
}

#[tauri::command]
pub async fn reset_session_sync_progress(provider_id: String) -> Result<(), String> {
    if let Ok(mut progress_map) = get_sync_progress_map().lock() {
        progress_map.remove(&provider_id);
        Ok(())
    } else {
        Err("Failed to reset sync progress".to_string())
    }
}
