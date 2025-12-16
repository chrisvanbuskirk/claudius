//! Housekeeping module for automatic cleanup of old briefings.
//!
//! This module provides functions for cleaning up old briefings based on
//! user-configured retention settings. Bookmarked briefings are always preserved.

use tracing::{info, warn};
use crate::db;
use crate::config::read_settings;

/// Result of a housekeeping run
#[derive(Debug)]
pub struct CleanupResult {
    pub deleted_count: usize,
    pub remaining_count: usize,
    pub skipped_reason: Option<String>,
}

/// Run cleanup based on current settings.
/// This is safe to call at any time - it will do nothing if retention_days is None.
pub fn run_cleanup() -> Result<CleanupResult, String> {
    let settings = read_settings()?;

    // Check if cleanup is enabled
    let days = match settings.retention_days {
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

    let conn = db::get_connection()
        .map_err(|e| format!("Failed to get database connection: {}", e))?;

    // Get count before cleanup
    let before_count = db::count_briefings(&conn)?;

    // Run cleanup
    let deleted_count = db::cleanup_old_briefings(&conn, days)?;
    let remaining_count = db::count_briefings(&conn)?;

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

/// Run cleanup on app startup (non-blocking, logs errors but doesn't fail)
pub fn run_startup_cleanup() {
    match run_cleanup() {
        Ok(result) => {
            if result.deleted_count > 0 {
                info!("Startup cleanup complete: {} briefing(s) deleted", result.deleted_count);
            }
        }
        Err(e) => {
            warn!("Startup cleanup failed: {}", e);
            // Don't fail app startup if cleanup fails
        }
    }
}
