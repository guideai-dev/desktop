# Provider File Reorganization Plan

**Status**: Ready for implementation
**Created**: 2025-11-07
**Estimated Time**: 2-3 hours

## Executive Summary

Reorganize provider files into logical, provider-specific directories and break up the monolithic `session_scanner.rs` (1,185 lines) into focused, maintainable modules.

**Benefits**:
- ✅ Logical grouping: All files for a provider in one directory
- ✅ Easy navigation: Find everything for a provider instantly
- ✅ Consistent structure: Every provider follows same pattern
- ✅ Better maintainability: Changes isolated to provider directories
- ✅ Scalable: Easy to add new providers
- ✅ Reduced complexity: 1,185-line file → 6 focused modules (~150-250 lines each)

---

## Current Structure (Problems)

```
providers/
├── claude/              # ✅ Well-organized
│   ├── mod.rs
│   ├── types.rs
│   ├── converter.rs
│   ├── converter_utils.rs
│   └── scanner.rs
├── claude_watcher.rs    # ❌ Separate file
│
├── codex/               # ⚠️  Only has converter
│   ├── mod.rs
│   └── converter.rs
├── codex_watcher.rs     # ❌ Separate file
│
├── copilot/             # ⚠️  Only has converter
│   ├── mod.rs
│   └── converter.rs
├── copilot_parser.rs    # ❌ Separate file
├── copilot_utils.rs     # ❌ Separate file
├── copilot_watcher.rs   # ❌ Separate file
│
├── cursor/              # ✅ Well-organized
│   ├── mod.rs
│   ├── types.rs
│   ├── converter.rs
│   ├── db.rs
│   ├── scanner.rs
│   ├── debug.rs
│   └── protobuf.rs
├── cursor_watcher.rs    # ❌ Separate file
│
├── gemini/              # ⚠️  Only has converter
│   ├── mod.rs
│   └── converter.rs
├── gemini_parser.rs     # ❌ Separate file
├── gemini_utils.rs      # ❌ Separate file
├── gemini_registry.rs   # ❌ Separate file
├── gemini_watcher.rs    # ❌ Separate file
│
├── opencode/            # ⚠️  Only has converter
│   ├── mod.rs
│   └── converter.rs
├── opencode_parser.rs   # ❌ Separate file
├── opencode_watcher.rs  # ❌ Separate file
│
├── session_scanner.rs   # ❌ MONOLITH: 1,185 lines, 6 providers
├── canonical/           # ✅ Good structure
├── common/              # ✅ Good structure
└── db_helpers.rs        # ⚠️  Should be in common/
```

**Issues**:
1. Watchers scattered at root level
2. Parser/utils files scattered at root level
3. Monolithic `session_scanner.rs` mixes all provider logic
4. Inconsistent organization (claude/cursor good, others incomplete)
5. Hard to find all files related to a provider
6. `db_helpers.rs` should be in `common/`

---

## Proposed Structure (Solution)

```
providers/
├── claude/
│   ├── mod.rs           # Re-exports, public API
│   ├── types.rs         # ✅ Already exists
│   ├── converter.rs     # ✅ Already exists
│   ├── converter_utils.rs # ✅ Already exists
│   ├── scanner.rs       # ← EXTRACT from session_scanner.rs
│   └── watcher.rs       # ← MOVE from claude_watcher.rs
│
├── codex/
│   ├── mod.rs           # Update re-exports
│   ├── converter.rs     # ✅ Already exists
│   ├── scanner.rs       # ← EXTRACT from session_scanner.rs
│   └── watcher.rs       # ← MOVE from codex_watcher.rs
│
├── copilot/
│   ├── mod.rs           # Update re-exports
│   ├── converter.rs     # ✅ Already exists
│   ├── parser.rs        # ← MOVE from copilot_parser.rs
│   ├── utils.rs         # ← MOVE from copilot_utils.rs
│   ├── scanner.rs       # ← EXTRACT from session_scanner.rs
│   └── watcher.rs       # ← MOVE from copilot_watcher.rs
│
├── cursor/
│   ├── mod.rs           # ✅ Update re-exports
│   ├── types.rs         # ✅ Already exists
│   ├── converter.rs     # ✅ Already exists
│   ├── db.rs            # ✅ Already exists
│   ├── scanner.rs       # ✅ Already exists (good)
│   ├── debug.rs         # ✅ Already exists
│   ├── protobuf.rs      # ✅ Already exists
│   └── watcher.rs       # ← MOVE from cursor_watcher.rs
│
├── gemini/
│   ├── mod.rs           # Update re-exports
│   ├── converter.rs     # ✅ Already exists
│   ├── parser.rs        # ← MOVE from gemini_parser.rs
│   ├── utils.rs         # ← MOVE from gemini_utils.rs
│   ├── registry.rs      # ← MOVE from gemini_registry.rs
│   ├── scanner.rs       # ← EXTRACT from session_scanner.rs
│   └── watcher.rs       # ← MOVE from gemini_watcher.rs
│
├── opencode/
│   ├── mod.rs           # Update re-exports
│   ├── converter.rs     # ✅ Already exists
│   ├── parser.rs        # ← MOVE from opencode_parser.rs
│   ├── scanner.rs       # ← EXTRACT from session_scanner.rs
│   └── watcher.rs       # ← MOVE from opencode_watcher.rs
│
├── canonical/           # ✅ Keep as-is
├── common/
│   ├── mod.rs
│   ├── session_info.rs  # ← EXTRACT SessionInfo from session_scanner.rs
│   ├── timing.rs        # ← EXTRACT timing helpers from session_scanner.rs
│   ├── db_helpers.rs    # ← MOVE from root providers/
│   └── ... (other existing common files)
│
└── mod.rs               # Update provider exports
```

---

## Detailed File Operations

### Phase 1: Move Watchers (6 files)

**Simple git moves to preserve history:**

```bash
git mv providers/claude_watcher.rs providers/claude/watcher.rs
git mv providers/codex_watcher.rs providers/codex/watcher.rs
git mv providers/copilot_watcher.rs providers/copilot/watcher.rs
git mv providers/cursor_watcher.rs providers/cursor/watcher.rs
git mv providers/gemini_watcher.rs providers/gemini/watcher.rs
git mv providers/opencode_watcher.rs providers/opencode/watcher.rs
```

**After moving, update each watcher.rs:**
- Change imports from `crate::providers::...` to `super::...` where appropriate
- Update mod.rs in each provider to re-export watcher

---

### Phase 2: Move Parser/Utils Files (5 files)

```bash
git mv providers/copilot_parser.rs providers/copilot/parser.rs
git mv providers/copilot_utils.rs providers/copilot/utils.rs
git mv providers/gemini_parser.rs providers/gemini/parser.rs
git mv providers/gemini_utils.rs providers/gemini/utils.rs
git mv providers/gemini_registry.rs providers/gemini/registry.rs
git mv providers/opencode_parser.rs providers/opencode/parser.rs
git mv providers/db_helpers.rs providers/common/db_helpers.rs
```

**After moving:**
- Update imports in each file
- Update mod.rs re-exports

---

### Phase 3: Extract session_scanner.rs (Complex)

#### Step 1: Create common/session_info.rs

**Extract from session_scanner.rs lines 17-31, 33-50:**

```rust
//! Common session info types used across all providers

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Session information returned by scanner
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionInfo {
    pub provider: String,
    pub project_name: String,
    pub session_id: String,
    pub file_path: PathBuf,
    pub file_name: String,
    pub session_start_time: Option<DateTime<Utc>>,
    pub session_end_time: Option<DateTime<Utc>>,
    pub duration_ms: Option<i64>,
    pub file_size: u64,
    pub content: Option<String>,
    pub cwd: Option<String>,
    pub project_hash: Option<String>,
}
```

#### Step 2: Create common/timing.rs

**Extract timing helpers:**

```rust
//! Timing extraction utilities for session JSONL files

use chrono::{DateTime, Utc};
use std::path::Path;
use std::fs;

/// Type alias for timing data tuple
pub type TimingData = (
    Option<DateTime<Utc>>, // start_time
    Option<DateTime<Utc>>, // end_time
    Option<i64>,           // duration_ms
);

/// Extract timing from JSONL file (lines 716-769)
pub fn extract_timing_from_jsonl(file_path: &Path) -> Result<TimingData, String> {
    // Implementation from session_scanner.rs
}
```

#### Step 3: Create provider-specific scanner.rs files

**For each provider, extract its scanning logic:**

**claude/scanner.rs** (~150 lines):
- Extract lines 82-156 (scan_claude_sessions_filtered)
- Extract lines 158-168 (extract_cwd_from_claude_session)
- Extract lines 170-282 (parse_claude_session)
- Add module imports

**codex/scanner.rs** (~200 lines):
- Extract lines 404-468 (scan_codex_sessions_filtered)
- Extract lines 531-658 (parse_codex_session)
- Add module imports

**copilot/scanner.rs** (~150 lines):
- Extract lines 473-529 (scan_copilot_sessions_filtered)
- Extract lines 660-713 (parse_copilot_session)
- Add module imports

**gemini/scanner.rs** (~200 lines):
- Extract lines 775-866 (scan_gemini_sessions_filtered)
- Extract lines 868-945 (parse_gemini_session)
- Extract lines 947-958 (extract_cwd_from_gemini_session)
- Add module imports

**opencode/scanner.rs** (~150 lines):
- Extract lines 287-350 (scan_opencode_sessions_filtered)
- Extract lines 352-400 (parse_opencode_session)
- Add module imports

**cursor/scanner.rs**:
- ✅ Already exists and well-structured
- Merge/consolidate with extracted code from lines 960-1126 if needed

#### Step 4: Update session_scanner.rs to dispatcher

**Keep only public API (lines 52-80):**

```rust
//! Session scanner - delegates to provider-specific scanners

use common::session_info::SessionInfo;
use shellexpand::tilde;
use std::path::Path;

pub fn scan_all_sessions_filtered(
    provider_id: &str,
    home_directory: &str,
    selected_projects: Option<&[String]>,
) -> Result<Vec<SessionInfo>, String> {
    let expanded = tilde(home_directory);
    let base_path = Path::new(expanded.as_ref());

    if !base_path.exists() {
        return Ok(Vec::new());
    }

    match provider_id {
        "claude-code" => claude::scanner::scan_sessions_filtered(base_path, selected_projects),
        "github-copilot" => copilot::scanner::scan_sessions_filtered(base_path, selected_projects),
        "opencode" => opencode::scanner::scan_sessions_filtered(base_path, selected_projects),
        "codex" => codex::scanner::scan_sessions_filtered(base_path, selected_projects),
        "gemini-code" => gemini::scanner::scan_sessions_filtered(base_path, selected_projects),
        "cursor" => cursor::scanner::scan_sessions_filtered(selected_projects),
        _ => Err(format!("Unsupported provider: {}", provider_id)),
    }
}
```

---

### Phase 4: Update Imports

**Files requiring import updates:**

1. **Provider mod.rs files** (6 files):
   - `providers/claude/mod.rs`
   - `providers/codex/mod.rs`
   - `providers/copilot/mod.rs`
   - `providers/cursor/mod.rs`
   - `providers/gemini/mod.rs`
   - `providers/opencode/mod.rs`

   Update to:
   ```rust
   pub mod watcher;
   pub mod scanner;
   // ... other modules

   pub use watcher::{...Watcher, ...WatcherStatus};
   pub use scanner::scan_sessions_filtered;
   ```

2. **providers/mod.rs**:
   - Update watcher imports to use provider submodules
   - Update scanner import to use new structure

3. **providers/common/mod.rs**:
   - Add `pub mod session_info;`
   - Add `pub mod timing;`
   - Add `pub mod db_helpers;` (if not already there)
   - Re-export `SessionInfo`

4. **Watcher files** (after moving):
   - Change `use crate::providers::...` to `super::...` where appropriate
   - Update any cross-provider imports

5. **Scanner files** (new):
   - Import from `crate::providers::common::session_info::SessionInfo`
   - Import from `crate::providers::common::timing::{...}`
   - Import provider-specific modules with `super::...`

6. **commands.rs**:
   - May need import path updates (verify after changes)

---

## Execution Plan

### Step 1: Prepare (5 min)
- [ ] Create feature branch: `git checkout -b refactor/provider-file-organization`
- [ ] Ensure working directory is clean
- [ ] Run tests to establish baseline

### Step 2: Move Watchers (15 min)
- [ ] Execute Phase 1 git mv commands
- [ ] Update imports in each moved watcher.rs
- [ ] Update provider mod.rs files to re-export watchers
- [ ] Test build: `cargo check`

### Step 3: Move Parsers/Utils (15 min)
- [ ] Execute Phase 2 git mv commands
- [ ] Update imports in moved files
- [ ] Update provider mod.rs re-exports
- [ ] Test build: `cargo check`

### Step 4: Extract Common Modules (20 min)
- [ ] Create `common/session_info.rs`
- [ ] Create `common/timing.rs`
- [ ] Update `common/mod.rs`
- [ ] Test build: `cargo check`

### Step 5: Create Provider Scanners (60 min)
- [ ] Create `claude/scanner.rs` (extract + adapt)
- [ ] Create `codex/scanner.rs` (extract + adapt)
- [ ] Create `copilot/scanner.rs` (extract + adapt)
- [ ] Create `gemini/scanner.rs` (extract + adapt)
- [ ] Create `opencode/scanner.rs` (extract + adapt)
- [ ] Verify `cursor/scanner.rs` (already exists)
- [ ] Update each provider's mod.rs
- [ ] Test build after each scanner: `cargo check`

### Step 6: Update session_scanner.rs (10 min)
- [ ] Reduce to dispatcher-only (remove all provider implementations)
- [ ] Update imports
- [ ] Test build: `cargo check`

### Step 7: Update All Imports (20 min)
- [ ] Update providers/mod.rs
- [ ] Update commands.rs if needed
- [ ] Update any other files with import errors
- [ ] Test build: `cargo check`

### Step 8: Verification (20 min)
- [ ] Run full build: `cargo build`
- [ ] Run tests: `cargo test`
- [ ] Run clippy: `cargo clippy -- -D warnings`
- [ ] Run workspace typecheck: `pnpm typecheck`
- [ ] Manual smoke test: Start watchers, verify scanning works

### Step 9: Cleanup (10 min)
- [ ] Remove any temporary files
- [ ] Verify git history is clean (all renames tracked)
- [ ] Final build + test pass

### Step 10: Commit (5 min)
- [ ] Stage all changes
- [ ] Write comprehensive commit message
- [ ] Push branch

---

## Verification Checklist

### Build
- [ ] `cargo check` passes
- [ ] `cargo build` passes
- [ ] No new warnings introduced

### Tests
- [ ] All Rust tests pass: `cargo test`
- [ ] Scanner tests still work
- [ ] Watcher tests still work

### Quality
- [ ] `cargo clippy -- -D warnings` passes
- [ ] No dead code warnings for production code
- [ ] Imports are clean and organized

### Functionality
- [ ] Each provider's watcher can still be instantiated
- [ ] Scanning works for all 6 providers
- [ ] Canonical conversion still works
- [ ] Desktop app starts without errors

---

## Rollback Plan

If issues arise during reorganization:

1. **Rollback to last good commit:**
   ```bash
   git reset --hard <last-good-commit>
   ```

2. **Restart from specific phase:**
   - Each phase is a logical checkpoint
   - Can rollback to end of any phase and continue

3. **Emergency abort:**
   ```bash
   git checkout main
   git branch -D refactor/provider-file-organization
   ```

---

## Risk Assessment

### Low Risk ✅
- Moving watchers (simple file moves with git mv)
- Moving parser/utils files (simple file moves)
- Creating common modules (additive changes)

### Medium Risk ⚠️
- Extracting provider scanners (code duplication/splits)
- Import path updates (many files affected)

### Mitigation
- Test after each phase
- Use git mv to preserve history
- Keep phase 1-3 in separate commits if needed
- Extensive verification before final commit

---

## Post-Reorganization Benefits

### For Developers
- Find all provider files in one place
- Understand provider structure at a glance
- Make changes without affecting other providers
- Easy to add new providers following the pattern

### For Maintainability
- Smaller, focused files (~150-250 lines vs 1,185 lines)
- Clear separation of concerns
- Easier code review (changes isolated to provider)
- Better git history (rename tracking)

### For Testing
- Test providers independently
- Mock provider interfaces cleanly
- Provider-specific fixtures in same directory

---

## Success Criteria

1. ✅ All watchers in provider directories
2. ✅ All parsers/utils in provider directories
3. ✅ session_scanner.rs < 100 lines (dispatcher only)
4. ✅ Common code in common/ directory
5. ✅ All builds pass
6. ✅ All tests pass
7. ✅ No regressions in functionality
8. ✅ Git history preserved (git mv tracking)

---

## Timeline

- **Estimated**: 2-3 hours
- **Best case**: 2 hours (if smooth)
- **Worst case**: 4 hours (with troubleshooting)
- **Recommended**: Block 3 hours, work without interruptions

---

## Next Steps

1. Review this plan
2. Get approval/feedback
3. Set aside dedicated time block
4. Execute plan step by step
5. Create PR for review
