# Provider File Reorganization Plan - Remaining Work

**Status**: Phases 1-3 Complete ✅ | Phases 4-5 Remaining
**Updated**: 2025-11-07
**Estimated Time Remaining**: 1-2 hours

## Executive Summary

**Completed (Phases 1-3)**:
- ✅ All watchers moved to provider directories
- ✅ All parsers/utils moved to provider directories
- ✅ Common SessionInfo and timing modules extracted
- ✅ db_helpers moved to common/
- ✅ Consistent provider structure established

**Remaining Work (Phases 4-5)**:
Break up the monolithic `session_scanner.rs` (currently ~948 lines) into provider-specific scanner modules (~150-250 lines each), leaving only a dispatcher (~100 lines).

---

## Current Structure (After Phase 3)

```
providers/
├── claude/
│   ├── mod.rs ✅
│   ├── types.rs ✅
│   ├── converter.rs ✅
│   ├── converter_utils.rs ✅
│   ├── scanner.rs ✅ (already exists)
│   └── watcher.rs ✅ (moved)
│
├── codex/
│   ├── mod.rs ✅
│   ├── converter.rs ✅
│   └── watcher.rs ✅ (moved)
│   └── scanner.rs ❌ (NEEDS EXTRACTION)
│
├── copilot/
│   ├── mod.rs ✅
│   ├── converter.rs ✅
│   ├── parser.rs ✅ (moved)
│   ├── utils.rs ✅ (moved)
│   ├── watcher.rs ✅ (moved)
│   └── scanner.rs ❌ (NEEDS EXTRACTION)
│
├── cursor/
│   ├── mod.rs ✅
│   ├── types.rs ✅
│   ├── converter.rs ✅
│   ├── db.rs ✅
│   ├── scanner.rs ✅ (already exists - may need integration)
│   ├── debug.rs ✅
│   ├── protobuf.rs ✅
│   └── watcher.rs ✅ (moved)
│
├── gemini/
│   ├── mod.rs ✅
│   ├── converter.rs ✅
│   ├── parser.rs ✅ (moved)
│   ├── utils.rs ✅ (moved)
│   ├── registry.rs ✅ (moved)
│   ├── watcher.rs ✅ (moved)
│   └── scanner.rs ❌ (NEEDS EXTRACTION)
│
├── opencode/
│   ├── mod.rs ✅
│   ├── converter.rs ✅
│   ├── parser.rs ✅ (moved)
│   ├── watcher.rs ✅ (moved)
│   └── scanner.rs ❌ (NEEDS EXTRACTION)
│
├── canonical/ ✅
├── common/
│   ├── mod.rs ✅
│   ├── session_info.rs ✅ (extracted)
│   ├── timing.rs ✅ (extracted)
│   ├── db_helpers.rs ✅ (moved)
│   └── ... (other common files) ✅
│
└── session_scanner.rs ⚠️ (NEEDS REDUCTION to dispatcher only)
```

---

## Remaining Work

### Phase 4: Extract Provider-Specific Scanners

Create scanner modules for providers that don't have them yet.

#### **codex/scanner.rs** (~200 lines)

Extract from `session_scanner.rs`:
- `scan_codex_sessions_filtered()` function
- `parse_codex_session()` helper function
- Any codex-specific types (ClaudeLogEntry, CodexPayload, etc.)

Public API:
```rust
pub fn scan_sessions_filtered(
    base_path: &Path,
    selected_projects: Option<&[String]>,
) -> Result<Vec<SessionInfo>, String>
```

#### **copilot/scanner.rs** (~150 lines)

Extract from `session_scanner.rs`:
- `scan_copilot_sessions_filtered()` function
- `parse_copilot_session()` helper function

Public API:
```rust
pub fn scan_sessions_filtered(
    base_path: &Path,
    selected_projects: Option<&[String]>,
) -> Result<Vec<SessionInfo>, String>
```

#### **gemini/scanner.rs** (~200 lines)

Extract from `session_scanner.rs`:
- `scan_gemini_sessions_filtered()` function
- `parse_gemini_session()` helper function
- `extract_cwd_from_gemini_session()` helper function

Public API:
```rust
pub fn scan_sessions_filtered(
    base_path: &Path,
    selected_projects: Option<&[String]>,
) -> Result<Vec<SessionInfo>, String>
```

#### **opencode/scanner.rs** (~150 lines)

Extract from `session_scanner.rs`:
- `scan_opencode_sessions_filtered()` function
- `parse_opencode_session()` helper function

Public API:
```rust
pub fn scan_sessions_filtered(
    base_path: &Path,
    selected_projects: Option<&[String]>,
) -> Result<Vec<SessionInfo>, String>
```

#### **cursor/scanner.rs** (verify/integrate)

- ✅ Already exists with `scan_existing_sessions()`
- ⚠️ May need integration with code from `session_scanner.rs` lines 960-1126
- Verify it has the consistent public API

---

### Phase 5: Update session_scanner.rs to Dispatcher Only

Reduce `session_scanner.rs` from ~948 lines to ~100 lines by:

1. **Remove all provider-specific code**:
   - Delete all `scan_*_sessions_filtered()` implementations
   - Delete all `parse_*_session()` helper functions
   - Delete all provider-specific types

2. **Keep only the dispatcher function**:

```rust
//! Session scanner - delegates to provider-specific scanners

use crate::providers::common::SessionInfo;
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
        "claude-code" => super::claude::scanner::scan_sessions_filtered(base_path, selected_projects),
        "github-copilot" => super::copilot::scanner::scan_sessions_filtered(base_path, selected_projects),
        "opencode" => super::opencode::scanner::scan_sessions_filtered(base_path, selected_projects),
        "codex" => super::codex::scanner::scan_sessions_filtered(base_path, selected_projects),
        "gemini-code" => super::gemini::scanner::scan_sessions_filtered(base_path, selected_projects),
        "cursor" => super::cursor::scanner::scan_sessions_filtered(selected_projects),
        _ => Err(format!("Unsupported provider: {}", provider_id)),
    }
}
```

---

## Execution Plan (Remaining Steps)

### Step 1: Extract Codex Scanner (15 min)
- [ ] Create `codex/scanner.rs`
- [ ] Extract `scan_codex_sessions_filtered()` and helpers from `session_scanner.rs`
- [ ] Update `codex/mod.rs` to include scanner module
- [ ] Update imports
- [ ] Test build: `cargo check`

### Step 2: Extract Copilot Scanner (15 min)
- [ ] Create `copilot/scanner.rs`
- [ ] Extract `scan_copilot_sessions_filtered()` and helpers from `session_scanner.rs`
- [ ] Update `copilot/mod.rs` to include scanner module
- [ ] Update imports
- [ ] Test build: `cargo check`

### Step 3: Extract Gemini Scanner (20 min)
- [ ] Create `gemini/scanner.rs`
- [ ] Extract `scan_gemini_sessions_filtered()` and helpers from `session_scanner.rs`
- [ ] Update `gemini/mod.rs` to include scanner module
- [ ] Update imports
- [ ] Test build: `cargo check`

### Step 4: Extract OpenCode Scanner (15 min)
- [ ] Create `opencode/scanner.rs`
- [ ] Extract `scan_opencode_sessions_filtered()` and helpers from `session_scanner.rs`
- [ ] Update `opencode/mod.rs` to include scanner module
- [ ] Update imports
- [ ] Test build: `cargo check`

### Step 5: Verify Cursor Scanner (5 min)
- [ ] Review `cursor/scanner.rs` for consistency
- [ ] Integrate any missing code from `session_scanner.rs` if needed
- [ ] Ensure public API matches other providers
- [ ] Test build: `cargo check`

### Step 6: Reduce session_scanner.rs to Dispatcher (10 min)
- [ ] Remove all provider implementations
- [ ] Keep only dispatcher function
- [ ] Update imports to use provider scanner modules
- [ ] Test build: `cargo check`

### Step 7: Update Provider mod.rs Files (10 min)
- [ ] Add scanner module to `codex/mod.rs`
- [ ] Add scanner module to `copilot/mod.rs`
- [ ] Add scanner module to `gemini/mod.rs`
- [ ] Add scanner module to `opencode/mod.rs`
- [ ] Add re-exports: `pub use scanner::scan_sessions_filtered;`
- [ ] Test build: `cargo check`

### Step 8: Verification (20 min)
- [ ] Run full build: `cargo build`
- [ ] Run tests: `cargo test`
- [ ] Run clippy: `cargo clippy -- -D warnings`
- [ ] Verify no regressions in functionality
- [ ] Manual smoke test: Start watchers, verify scanning works

### Step 9: Commit (5 min)
- [ ] Stage all changes
- [ ] Write comprehensive commit message
- [ ] Commit Phase 4-5 changes

---

## Key Considerations

### Import Strategy
Each new scanner module should:
```rust
use crate::providers::common::{SessionInfo, extract_timing_from_jsonl};
use super::parser::...;  // Provider-specific types
use std::fs;
use std::path::{Path, PathBuf};
```

### Function Signature Consistency
All scanners should expose:
```rust
pub fn scan_sessions_filtered(
    base_path: &Path,
    selected_projects: Option<&[String]>,
) -> Result<Vec<SessionInfo>, String>
```

Note: Cursor may have a different signature due to its SQLite-based structure.

### Helper Functions
- Keep provider-specific helpers in the scanner module (not exported)
- Use `super::utils::...` for shared utilities within a provider
- Use `crate::providers::common::...` for cross-provider utilities

---

## Benefits of Completion

### For Developers
- ✅ Find all provider code in one directory
- ✅ ~150-250 line files instead of 1,185-line monolith
- ✅ Changes isolated to single provider
- ✅ Easy to add new providers following the pattern

### For Maintainability
- ✅ Clear separation of concerns
- ✅ Provider logic self-contained
- ✅ Easier code review (changes scoped to provider)
- ✅ Better testing (can test each provider independently)

### For Code Quality
- ✅ Reduced complexity per file
- ✅ Improved readability
- ✅ Easier to understand data flow
- ✅ Less cognitive load

---

## Success Criteria

### Completed ✅
1. ✅ All watchers in provider directories
2. ✅ All parsers/utils in provider directories
3. ✅ Common SessionInfo and timing modules created
4. ✅ db_helpers in common/ directory
5. ✅ Git history preserved (git mv tracking)

### Remaining ❌
6. ❌ Provider-specific scanner files created (4 files: codex, copilot, gemini, opencode)
7. ❌ session_scanner.rs < 150 lines (dispatcher only)
8. ❌ All builds pass after extraction
9. ❌ All tests pass after extraction
10. ❌ No regressions in functionality

---

## Risk Assessment

### Low Risk ✅
- Extracting scanner functions (code is well-isolated)
- Creating new scanner modules (additive)
- Provider mod.rs updates (straightforward)

### Medium Risk ⚠️
- Reducing session_scanner.rs (ensure all code is preserved)
- Import path updates (multiple files)
- Cursor scanner integration (may need special handling)

### Mitigation
- Test after each scanner extraction
- Keep session_scanner.rs code until all scanners are verified
- Use compiler to find missing imports
- Extensive verification before final commit

---

## Timeline

- **Estimated**: 1-2 hours
- **Best case**: 1 hour (smooth extraction)
- **Worst case**: 2.5 hours (with troubleshooting)
- **Recommended**: Block 2 hours, work without interruptions

---

## Next Steps

1. Start with Step 1 (Extract Codex Scanner)
2. Work through providers sequentially
3. Test after each extraction
4. Reduce session_scanner.rs last
5. Comprehensive verification
6. Commit and push

**Ready to proceed when you are!**
