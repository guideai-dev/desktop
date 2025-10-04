# Creating a New Provider

This guide explains how to add a new AI coding assistant provider to the GuideAI desktop app.

## Overview

A provider consists of several components:
- **Frontend configuration** - TypeScript definitions and UI components
- **Backend scanning** - Rust code to discover projects and sessions
- **Session parser** - Rust code to parse provider-specific session formats
- **File watcher** - Rust code to monitor and automatically sync sessions

## Step-by-Step Guide

### 1. Add Provider Definition (Frontend)

#### Update `src/types/providers.ts`

1. Add platform-specific default paths to `PLATFORM_DEFAULTS`:

```typescript
'your-provider': {
  win32: '~/.your-provider',
  darwin: '~/.your-provider',
  linux: '~/.your-provider'
}
```

2. Add provider to `CODING_AGENTS` array:

```typescript
{
  id: 'your-provider',
  name: 'Your Provider',
  description: 'Description of your provider',
  defaultHomeDirectory: getPlatformDefault('your-provider'),
  icon: 'M12 2L2 7l10 5 10-5-10-5z', // SVG path or identifier
  color: 'from-blue-500 to-purple-500' // Tailwind gradient classes
}
```

### 2. Add Provider Icon

#### Create icon file: `src/assets/icons/your-provider.svg`

Create an SVG file for your provider's icon. Example:

```xml
<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 16 16">
  <path fill="#000000" d="..."/>
</svg>
```

#### Update `src/components/icons/ProviderIcon.tsx`

1. Import your icon:

```typescript
import yourProviderSvg from '../../assets/icons/your-provider.svg'
```

2. Add to `iconMap`:

```typescript
const iconMap: Record<string, string> = {
  'your-provider': yourProviderSvg,
  // ... other providers
}
```

3. (Optional) Add background styling if needed:

```typescript
const needsBackground = providerId === 'your-provider'
```

### 3. Create Backend Modules (Rust)

#### Create `src-tauri/src/providers/your_provider.rs`

This module handles project discovery:

```rust
use super::sort_projects_by_modified;
use crate::config::ProjectInfo;
use chrono::{DateTime, Utc};
use shellexpand::tilde;
use std::fs;
use std::path::PathBuf;

pub fn scan_projects(home_directory: &str) -> Result<Vec<ProjectInfo>, String> {
    let expanded = tilde(home_directory);
    let base_path = PathBuf::from(expanded.into_owned());

    // Add platform-specific fallbacks
    let mut base_candidates = vec![base_path.clone()];
    if let Some(home_dir) = dirs::home_dir() {
        base_candidates.push(home_dir.join(".your-provider"));
    }

    let base_path = base_candidates
        .into_iter()
        .find(|path| path.exists())
        .ok_or_else(|| format!("Provider home directory not found"))?;

    // Scan for projects based on your provider's directory structure
    let projects_path = base_path.join("projects"); // Adjust to your structure
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
```

#### Create `src-tauri/src/providers/your_provider_parser.rs`

This module handles session parsing. The complexity depends on your provider's format:

```rust
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct YourProviderSession {
    pub session_id: String,
    pub start_time: String,
    // Add fields specific to your provider's format
}

#[derive(Debug, Clone)]
pub struct ParsedSession {
    pub session_id: String,
    pub project_name: String,
    pub session_start_time: Option<DateTime<Utc>>,
    pub session_end_time: Option<DateTime<Utc>>,
    pub duration_ms: Option<i64>,
    pub jsonl_content: String,
    pub cwd: Option<String>,
}

pub struct YourProviderParser {
    storage_path: PathBuf,
}

impl YourProviderParser {
    pub fn new(storage_path: PathBuf) -> Self {
        Self { storage_path }
    }

    pub fn parse_session(&self, session_id: &str) -> Result<ParsedSession, String> {
        // 1. Load session file
        // 2. Parse provider-specific format
        // 3. Convert to standardized JSONL format
        // 4. Return ParsedSession with metadata
        
        todo!("Implement session parsing logic")
    }

    pub fn get_sessions_for_project(&self, project_name: &str) -> Result<Vec<String>, String> {
        // Scan for session files in the project directory
        // Return list of session IDs
        
        todo!("Implement session discovery logic")
    }
}
```

**Key Points for Parser:**
- The `jsonl_content` field must be in JSONL format (one JSON object per line)
- Each line should have at minimum: `{"timestamp": "...", "type": "...", "message": {...}}`
- Convert your provider's format to match this structure
- Extract session timing information (start, end, duration)

#### Create `src-tauri/src/providers/your_provider_watcher.rs`

This module handles real-time file watching:

```rust
use crate::config::load_provider_config;
use crate::logging::{log_debug, log_error, log_info, log_warn};
use crate::providers::db_helpers::insert_session_immediately;
use crate::upload_queue::UploadQueue;
use notify::{Config, Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use serde::{Deserialize, Serialize};
use shellexpand::tilde;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{mpsc, Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

const PROVIDER_ID: &str = "your-provider";

#[derive(Debug, Clone)]
pub struct FileChangeEvent {
    pub path: PathBuf,
    pub project_name: String,
    pub last_modified: Instant,
    pub file_size: u64,
    pub session_id: String,
    pub is_new_session: bool,
}

#[derive(Debug, Clone)]
pub struct SessionState {
    pub last_modified: Instant,
    pub last_size: u64,
    pub is_active: bool,
}

#[derive(Debug)]
pub struct YourProviderWatcher {
    _watcher: RecommendedWatcher,
    _thread_handle: thread::JoinHandle<()>,
    upload_queue: Arc<UploadQueue>,
    is_running: Arc<Mutex<bool>>,
}

impl YourProviderWatcher {
    pub fn new(
        projects: Vec<String>,
        upload_queue: Arc<UploadQueue>,
    ) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        // 1. Load provider config
        // 2. Set up file system watcher
        // 3. Start background thread to process events
        // 4. Watch for session file changes
        
        todo!("Implement file watcher logic")
    }

    pub fn stop(&self) {
        if let Ok(mut running) = self.is_running.lock() {
            *running = false;
        }
    }

    pub fn get_status(&self) -> YourProviderWatcherStatus {
        // Return current watcher status
        todo!("Implement status logic")
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct YourProviderWatcherStatus {
    pub is_running: bool,
    pub pending_uploads: usize,
    pub processing_uploads: usize,
    pub failed_uploads: usize,
}
```

**Key Points for Watcher:**
- Monitor session files in the provider's directory
- Detect new sessions and updates to existing sessions
- Use `insert_session_immediately()` to save to local database
- Handle file extensions specific to your provider (.json, .jsonl, etc.)
- Implement debouncing to avoid processing duplicate events

### 4. Update Provider Module Registry

#### Update `src-tauri/src/providers/mod.rs`

1. Add module declarations:

```rust
mod your_provider;
mod your_provider_parser;
mod your_provider_watcher;
```

2. Add public exports:

```rust
pub use your_provider_watcher::{YourProviderWatcher, YourProviderWatcherStatus};
pub use your_provider_parser::YourProviderParser;
```

3. Add to `scan_projects` function:

```rust
pub fn scan_projects(provider_id: &str, home_directory: &str) -> Result<Vec<ProjectInfo>, String> {
    match provider_id {
        "your-provider" => your_provider::scan_projects(home_directory),
        // ... other providers
        other => Err(format!("Unsupported provider: {}", other)),
    }
}
```

### 5. Update Commands (if needed)

If your provider needs special handling in the Tauri commands layer, update `src-tauri/src/commands.rs` to add provider-specific command handlers.

### 6. Testing

1. **Build the application:**
   ```bash
   pnpm build
   ```

2. **Test provider detection:**
   - Open the app
   - Go to Configuration
   - Verify your provider appears in the list

3. **Test project scanning:**
   - Enable your provider
   - Verify projects are discovered correctly

4. **Test file watching:**
   - Start the watcher
   - Create a new session in your provider
   - Verify it's detected and stored in the database

5. **Test session parsing:**
   - View a session in the UI
   - Verify the content displays correctly

## Directory Structure Reference

```
apps/desktop/
├── src/
│   ├── assets/icons/
│   │   └── your-provider.svg          # Provider icon
│   ├── components/icons/
│   │   └── ProviderIcon.tsx           # Update icon mapping
│   └── types/
│       └── providers.ts               # Add provider definition
└── src-tauri/src/providers/
    ├── mod.rs                         # Update registry
    ├── your_provider.rs               # Project scanning
    ├── your_provider_parser.rs        # Session parsing
    └── your_provider_watcher.rs       # File watching
```

## Common Pitfalls

1. **Session ID Extraction:** Ensure your session ID extraction logic handles all filename patterns
2. **Path Handling:** Use `shellexpand::tilde()` for home directory expansion
3. **Cross-Platform Paths:** Test on Windows, macOS, and Linux
4. **File Extensions:** Filter for the correct file types (.json, .jsonl, etc.)
5. **Timestamp Parsing:** Handle different timestamp formats (ISO 8601, Unix timestamps, etc.)
6. **JSONL Format:** Ensure each line is valid JSON and includes required fields
7. **Error Handling:** Provide descriptive error messages for debugging

## Example Providers

Study these existing providers as references:

- **Claude Code** - Simple JSONL format, straightforward directory structure
- **OpenCode** - Complex relational format with separate files for projects, sessions, messages, and parts
- **Codex** - Similar to Claude, good reference for basic implementation

## Need Help?

- Check existing provider implementations in `src-tauri/src/providers/`
- Review the logging output when testing (`log_info`, `log_debug`, `log_error`)
- Use the provider logs viewer in the Configuration page to debug issues
