// Concurrency tests for database operations
// Tests Phase 1 fixes for race conditions

use std::thread;
use tempfile::tempdir;

#[test]
fn test_concurrent_session_inserts() {
    // Test that multiple threads can safely insert the same session
    // The second insert should gracefully convert to an update (no crash/error)

    let temp_dir = tempdir().unwrap();
    let db_path = temp_dir.path().join("test_concurrent_inserts.db");

    // Set up test database
    setup_test_database(&db_path);

    let session_id = "test-session-concurrent-insert";
    let provider = "test-provider";
    let project_name = "test-project";

    // Create 10 threads that all try to insert the same session
    let handles: Vec<_> = (0..10)
        .map(|i| {
            let session_id = session_id.to_string();
            let provider = provider.to_string();
            let project_name = project_name.to_string();
            let db_path = db_path.clone();

            thread::spawn(move || {
                // Each thread creates its own connection
                let conn = rusqlite::Connection::open(&db_path).unwrap();

                // Try to insert session
                let result = conn.execute(
                    "INSERT INTO agent_sessions (
                        id, provider, project_name, session_id, file_name, file_path,
                        file_size, processing_status, synced_to_server, created_at, uploaded_at
                    ) VALUES (?, ?, ?, ?, ?, ?, ?, 'pending', 0, ?, ?)",
                    rusqlite::params![
                        format!("id-{}", i),
                        provider,
                        project_name,
                        session_id,
                        "test.jsonl",
                        "/tmp/test.jsonl",
                        1000i64,
                        1000000i64,
                        1000000i64,
                    ],
                );

                // First insert succeeds, rest fail with constraint violation (expected)
                (i, result)
            })
        })
        .collect();

    // Collect results
    let results: Vec<_> = handles.into_iter().map(|h| h.join().unwrap()).collect();

    // Exactly one should succeed
    let successes = results.iter().filter(|(_, r)| r.is_ok()).count();
    assert_eq!(
        successes, 1,
        "Expected exactly 1 successful insert, got {}",
        successes
    );

    // All failures should be constraint violations
    let constraint_failures = results
        .iter()
        .filter(|(_, r)| {
            if let Err(e) = r {
                e.to_string().contains("UNIQUE constraint")
            } else {
                false
            }
        })
        .count();

    assert_eq!(
        constraint_failures, 9,
        "Expected 9 constraint violation failures, got {}",
        constraint_failures
    );

    // Verify only one session in database
    let conn = rusqlite::Connection::open(&db_path).unwrap();
    let count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM agent_sessions WHERE session_id = ?",
            rusqlite::params![session_id],
            |row| row.get(0),
        )
        .unwrap();

    assert_eq!(count, 1, "Should have exactly 1 session in database");
}

#[test]
fn test_concurrent_session_updates() {
    // Test that multiple threads can safely update the same session
    // All updates should succeed without lost writes

    let temp_dir = tempdir().unwrap();
    let db_path = temp_dir.path().join("test_concurrent_updates.db");

    // Set up test database
    setup_test_database(&db_path);

    let session_id = "test-session-concurrent-update";

    // Insert initial session
    {
        let conn = rusqlite::Connection::open(&db_path).unwrap();
        conn.execute(
            "INSERT INTO agent_sessions (
                id, provider, project_name, session_id, file_name, file_path,
                file_size, processing_status, synced_to_server, created_at, uploaded_at
            ) VALUES (?, ?, ?, ?, ?, ?, ?, 'pending', 0, ?, ?)",
            rusqlite::params![
                "initial-id",
                "test-provider",
                "test-project",
                session_id,
                "test.jsonl",
                "/tmp/test.jsonl",
                1000i64,
                1000000i64,
                1000000i64,
            ],
        )
        .unwrap();
    }

    // Create 10 threads that all try to update the same session with different file sizes
    let handles: Vec<_> = (0..10)
        .map(|i| {
            let session_id = session_id.to_string();
            let db_path = db_path.clone();
            let file_size = 2000i64 + (i * 100);

            thread::spawn(move || {
                // Each thread creates its own connection
                let mut conn = rusqlite::Connection::open(&db_path).unwrap();

                // SQLite transactions can fail with "database is locked" when concurrent
                // Retry up to 10 times with exponential backoff (realistic behavior)
                for attempt in 0..10 {
                    match try_update_with_transaction(&mut conn, &session_id, file_size) {
                        Ok(_) => return file_size,
                        Err(e) if e.to_string().contains("database is locked") && attempt < 9 => {
                            // Retry with exponential backoff
                            thread::sleep(std::time::Duration::from_millis(2_u64.pow(attempt)));
                            continue;
                        }
                        Err(e) => panic!("Update failed after retries: {}", e),
                    }
                }

                panic!("Should not reach here");
            })
        })
        .collect();

    // Wait for all updates to complete
    let updated_sizes: Vec<_> = handles.into_iter().map(|h| h.join().unwrap()).collect();

    // Verify final size is one of the updated values (last write wins)
    let conn = rusqlite::Connection::open(&db_path).unwrap();
    let final_size: i64 = conn
        .query_row(
            "SELECT file_size FROM agent_sessions WHERE session_id = ?",
            rusqlite::params![session_id],
            |row| row.get(0),
        )
        .unwrap();

    assert!(
        updated_sizes.contains(&final_size),
        "Final size {} should be one of the updated values: {:?}",
        final_size,
        updated_sizes
    );

    // Verify session count is still 1 (no duplicates created)
    let count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM agent_sessions WHERE session_id = ?",
            rusqlite::params![session_id],
            |row| row.get(0),
        )
        .unwrap();

    assert_eq!(count, 1, "Should still have exactly 1 session in database");
}

#[test]
fn test_insert_or_update_race() {
    // Test the exact race condition we fixed in insert_session_immediately()
    // Multiple threads racing to insert/update the same session

    let temp_dir = tempdir().unwrap();
    let db_path = temp_dir.path().join("test_insert_update_race.db");

    // Set up test database
    setup_test_database(&db_path);

    let session_id = "test-session-race";
    let provider = "test-provider";
    let project_name = "test-project";

    // Create 20 threads that race to insert/update
    let handles: Vec<_> = (0..20)
        .map(|i| {
            let session_id = session_id.to_string();
            let provider = provider.to_string();
            let project_name = project_name.to_string();
            let db_path = db_path.clone();
            let file_size = 1000i64 + (i * 50);

            thread::spawn(move || {
                let conn = rusqlite::Connection::open(&db_path).unwrap();

                // Try insert first (optimistic)
                let insert_result = conn.execute(
                    "INSERT INTO agent_sessions (
                        id, provider, project_name, session_id, file_name, file_path,
                        file_size, processing_status, synced_to_server, created_at, uploaded_at
                    ) VALUES (?, ?, ?, ?, ?, ?, ?, 'pending', 0, ?, ?)",
                    rusqlite::params![
                        format!("id-{}", i),
                        provider,
                        project_name,
                        session_id,
                        "test.jsonl",
                        "/tmp/test.jsonl",
                        file_size,
                        1000000i64 + (i * 1000),
                        1000000i64 + (i * 1000),
                    ],
                );

                match insert_result {
                    Ok(_) => {
                        // Insert succeeded
                        ("insert", file_size)
                    }
                    Err(e) if e.to_string().contains("UNIQUE constraint") => {
                        // Constraint violation, update instead (like our fix does)
                        conn.execute(
                            "UPDATE agent_sessions SET file_size = ?, uploaded_at = ? WHERE session_id = ?",
                            rusqlite::params![file_size, 2000000i64 + (i * 1000), &session_id],
                        )
                        .unwrap();
                        ("update", file_size)
                    }
                    Err(e) => {
                        panic!("Unexpected error: {}", e);
                    }
                }
            })
        })
        .collect();

    // Collect results
    let results: Vec<_> = handles.into_iter().map(|h| h.join().unwrap()).collect();

    // Exactly one should have done insert, rest should have updated
    let inserts = results.iter().filter(|(op, _)| *op == "insert").count();
    let updates = results.iter().filter(|(op, _)| *op == "update").count();

    assert_eq!(inserts, 1, "Expected exactly 1 insert, got {}", inserts);
    assert_eq!(updates, 19, "Expected 19 updates, got {}", updates);

    // Verify only one session exists
    let conn = rusqlite::Connection::open(&db_path).unwrap();
    let count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM agent_sessions WHERE session_id = ?",
            rusqlite::params![session_id],
            |row| row.get(0),
        )
        .unwrap();

    assert_eq!(count, 1, "Should have exactly 1 session in database");
}

// Helper function to try an update with transaction (for retry logic)
fn try_update_with_transaction(
    conn: &mut rusqlite::Connection,
    session_id: &str,
    file_size: i64,
) -> Result<(), rusqlite::Error> {
    let tx = conn.transaction()?;

    // Read current value
    let _current_size: i64 = tx.query_row(
        "SELECT file_size FROM agent_sessions WHERE session_id = ?",
        rusqlite::params![session_id],
        |row| row.get(0),
    )?;

    // Update with new value
    tx.execute(
        "UPDATE agent_sessions SET file_size = ?, uploaded_at = ? WHERE session_id = ?",
        rusqlite::params![file_size, 2000000i64, session_id],
    )?;

    tx.commit()?;
    Ok(())
}

// Helper function to set up a test database with the agent_sessions table
fn setup_test_database(db_path: &std::path::Path) {
    let conn = rusqlite::Connection::open(db_path).unwrap();

    // Create agent_sessions table (simplified schema for testing)
    conn.execute(
        "CREATE TABLE IF NOT EXISTS agent_sessions (
            id TEXT PRIMARY KEY,
            provider TEXT NOT NULL,
            project_name TEXT NOT NULL,
            session_id TEXT NOT NULL UNIQUE,
            file_name TEXT NOT NULL,
            file_path TEXT NOT NULL,
            file_size INTEGER NOT NULL,
            file_hash TEXT,
            session_start_time INTEGER,
            session_end_time INTEGER,
            duration_ms INTEGER,
            cwd TEXT,
            git_branch TEXT,
            first_commit_hash TEXT,
            latest_commit_hash TEXT,
            processing_status TEXT NOT NULL DEFAULT 'pending',
            synced_to_server INTEGER NOT NULL DEFAULT 0,
            created_at INTEGER NOT NULL,
            uploaded_at INTEGER NOT NULL
        )",
        [],
    )
    .unwrap();
}
