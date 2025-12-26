//! Housekeeping module for automatic cleanup of old briefings.
//!
//! This module provides functions for cleaning up old briefings based on
//! user-configured retention settings. Bookmarked briefings are always preserved.

use crate::config::read_settings;
use crate::db;
use rusqlite::Connection;
use tracing::{info, warn};

/// Result of a housekeeping run
#[derive(Debug, PartialEq)]
pub struct CleanupResult {
    pub deleted_count: usize,
    pub remaining_count: usize,
    pub skipped_reason: Option<String>,
}

/// Run cleanup with given retention days on a specific connection.
/// This is the testable core of the cleanup logic.
pub fn run_cleanup_with_conn(
    conn: &Connection,
    retention_days: Option<i32>,
) -> Result<CleanupResult, String> {
    // Check if cleanup is enabled
    let days = match retention_days {
        Some(d) => d,
        None => {
            info!("Housekeeping: retention_days is None, skipping cleanup");
            return Ok(CleanupResult {
                deleted_count: 0,
                remaining_count: 0,
                skipped_reason: Some("Retention is set to 'Never delete'".to_string()),
            });
        }
    };

    // Get count before cleanup
    let before_count = db::count_briefings(conn)?;

    // Run cleanup
    let deleted_count = db::cleanup_old_briefings(conn, days)?;
    let remaining_count = db::count_briefings(conn)?;

    if deleted_count > 0 {
        info!(
            "Housekeeping: deleted {} briefing(s) older than {} days ({} remaining)",
            deleted_count, days, remaining_count
        );
    } else {
        info!(
            "Housekeeping: no briefings to clean up (retention: {} days, {} total)",
            days, before_count
        );
    }

    Ok(CleanupResult {
        deleted_count,
        remaining_count,
        skipped_reason: None,
    })
}

/// Run cleanup based on current settings.
/// This is safe to call at any time - it will do nothing if retention_days is None.
pub fn run_cleanup() -> Result<CleanupResult, String> {
    let settings = read_settings()?;
    let conn =
        db::get_connection().map_err(|e| format!("Failed to get database connection: {}", e))?;

    run_cleanup_with_conn(&conn, settings.retention_days)
}

/// Run cleanup on app startup (non-blocking, logs errors but doesn't fail)
pub fn run_startup_cleanup() {
    match run_cleanup() {
        Ok(result) => {
            if result.deleted_count > 0 {
                info!(
                    "Startup cleanup complete: {} briefing(s) deleted",
                    result.deleted_count
                );
            }
        }
        Err(e) => {
            warn!("Startup cleanup failed: {}", e);
            // Don't fail app startup if cleanup fails
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;

    fn setup_test_db() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch(include_str!("schema.sql")).unwrap();
        conn
    }

    #[test]
    fn test_cleanup_skipped_when_retention_none() {
        let conn = setup_test_db();

        // Add a briefing
        conn.execute(
            "INSERT INTO briefings (date, title, cards) VALUES ('2020-01-01', 'Old', '[]')",
            [],
        )
        .unwrap();

        // Run cleanup with retention_days = None
        let result = run_cleanup_with_conn(&conn, None).unwrap();

        assert_eq!(result.deleted_count, 0);
        assert_eq!(
            result.skipped_reason,
            Some("Retention is set to 'Never delete'".to_string())
        );

        // Briefing should still exist
        let count = db::count_briefings(&conn).unwrap();
        assert_eq!(count, 1);
    }

    #[test]
    fn test_cleanup_deletes_old_briefings() {
        let conn = setup_test_db();

        // Add an old briefing (100 days ago)
        conn.execute(
            "INSERT INTO briefings (date, title, cards) VALUES (date('now', '-100 days'), 'Old', '[]')",
            [],
        ).unwrap();

        // Add a recent briefing (today)
        conn.execute(
            "INSERT INTO briefings (date, title, cards) VALUES (date('now'), 'Recent', '[]')",
            [],
        )
        .unwrap();

        assert_eq!(db::count_briefings(&conn).unwrap(), 2);

        // Run cleanup with 30 day retention
        let result = run_cleanup_with_conn(&conn, Some(30)).unwrap();

        assert_eq!(result.deleted_count, 1);
        assert_eq!(result.remaining_count, 1);
        assert_eq!(result.skipped_reason, None);
    }

    #[test]
    fn test_cleanup_preserves_bookmarked_briefings() {
        let conn = setup_test_db();

        // Add an old briefing (100 days ago)
        conn.execute(
            "INSERT INTO briefings (date, title, cards) VALUES (date('now', '-100 days'), 'Old Bookmarked', '[]')",
            [],
        ).unwrap();
        let old_id = conn.last_insert_rowid();

        // Bookmark it
        db::add_bookmark(&conn, old_id, 0).unwrap();

        // Run cleanup with 30 day retention
        let result = run_cleanup_with_conn(&conn, Some(30)).unwrap();

        assert_eq!(result.deleted_count, 0);
        assert_eq!(result.remaining_count, 1);

        // Briefing should still exist
        assert_eq!(db::count_briefings(&conn).unwrap(), 1);
    }

    #[test]
    fn test_cleanup_empty_database() {
        let conn = setup_test_db();

        // Run cleanup on empty database
        let result = run_cleanup_with_conn(&conn, Some(30)).unwrap();

        assert_eq!(result.deleted_count, 0);
        assert_eq!(result.remaining_count, 0);
        assert_eq!(result.skipped_reason, None);
    }

    #[test]
    fn test_cleanup_all_recent_briefings() {
        let conn = setup_test_db();

        // Add only recent briefings
        conn.execute(
            "INSERT INTO briefings (date, title, cards) VALUES (date('now'), 'Today', '[]')",
            [],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO briefings (date, title, cards) VALUES (date('now', '-5 days'), '5 days ago', '[]')",
            [],
        ).unwrap();

        // Run cleanup with 30 day retention
        let result = run_cleanup_with_conn(&conn, Some(30)).unwrap();

        assert_eq!(result.deleted_count, 0);
        assert_eq!(result.remaining_count, 2);
    }

    #[test]
    fn test_cleanup_result_equality() {
        let r1 = CleanupResult {
            deleted_count: 5,
            remaining_count: 10,
            skipped_reason: None,
        };
        let r2 = CleanupResult {
            deleted_count: 5,
            remaining_count: 10,
            skipped_reason: None,
        };
        let r3 = CleanupResult {
            deleted_count: 3,
            remaining_count: 10,
            skipped_reason: None,
        };

        assert_eq!(r1, r2);
        assert_ne!(r1, r3);
    }
}
