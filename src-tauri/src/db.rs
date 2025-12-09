use rusqlite::{Connection, Result, params};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tauri::AppHandle;
use tracing::{info, warn, debug};

/// Topic struct for database operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Topic {
    pub id: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub enabled: bool,
    pub created_at: String,
    pub updated_at: String,
}

/// Result of migrating topics from JSON to SQLite
#[derive(Debug)]
pub struct MigrationResult {
    pub topics_migrated: usize,
    pub json_backed_up: bool,
    pub errors: Vec<String>,
}

pub fn get_db_path() -> PathBuf {
    let home = dirs::home_dir().expect("Could not find home directory");
    home.join(".claudius").join("claudius.db")
}

fn get_config_dir() -> PathBuf {
    let home = dirs::home_dir().expect("Could not find home directory");
    home.join(".claudius")
}

pub fn init_database(_app: &AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    let db_path = get_db_path();

    // Ensure directory exists
    if let Some(parent) = db_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let conn = Connection::open(&db_path)?;

    // Create tables
    conn.execute_batch(include_str!("schema.sql"))?;

    // Run topic migration from JSON (idempotent)
    if let Err(e) = migrate_topics_from_json(&conn) {
        warn!("Topics migration encountered an issue: {}", e);
    }

    Ok(())
}

pub fn get_connection() -> Result<Connection> {
    Connection::open(get_db_path())
}

// ============================================================================
// Topic CRUD operations
// ============================================================================

/// Get all topics ordered by sort_order
pub fn get_all_topics(conn: &Connection) -> std::result::Result<Vec<Topic>, String> {
    let mut stmt = conn.prepare(
        "SELECT id, name, description, enabled, created_at, updated_at
         FROM topics
         ORDER BY sort_order ASC, created_at ASC"
    ).map_err(|e| format!("Failed to prepare statement: {}", e))?;

    let topics = stmt.query_map([], |row| {
        Ok(Topic {
            id: row.get(0)?,
            name: row.get(1)?,
            description: row.get(2)?,
            enabled: row.get::<_, i32>(3)? != 0,
            created_at: row.get(4)?,
            updated_at: row.get(5)?,
        })
    }).map_err(|e| format!("Query failed: {}", e))?
    .collect::<std::result::Result<Vec<_>, _>>()
    .map_err(|e| format!("Failed to collect results: {}", e))?;

    Ok(topics)
}

/// Get a topic by ID
pub fn get_topic_by_id(conn: &Connection, id: &str) -> std::result::Result<Option<Topic>, String> {
    let mut stmt = conn.prepare(
        "SELECT id, name, description, enabled, created_at, updated_at
         FROM topics
         WHERE id = ?1"
    ).map_err(|e| format!("Failed to prepare statement: {}", e))?;

    let result = stmt.query_row([id], |row| {
        Ok(Topic {
            id: row.get(0)?,
            name: row.get(1)?,
            description: row.get(2)?,
            enabled: row.get::<_, i32>(3)? != 0,
            created_at: row.get(4)?,
            updated_at: row.get(5)?,
        })
    });

    match result {
        Ok(topic) => Ok(Some(topic)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(format!("Failed to get topic: {}", e)),
    }
}

/// Insert a new topic
pub fn insert_topic(conn: &Connection, topic: &Topic, sort_order: i32) -> std::result::Result<(), String> {
    conn.execute(
        "INSERT INTO topics (id, name, description, enabled, sort_order, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        params![
            topic.id,
            topic.name,
            topic.description,
            if topic.enabled { 1 } else { 0 },
            sort_order,
            topic.created_at,
            topic.updated_at,
        ],
    ).map_err(|e| format!("Failed to insert topic: {}", e))?;

    Ok(())
}

/// Update an existing topic
pub fn update_topic(conn: &Connection, topic: &Topic) -> std::result::Result<(), String> {
    let rows_affected = conn.execute(
        "UPDATE topics
         SET name = ?1, description = ?2, enabled = ?3, updated_at = ?4
         WHERE id = ?5",
        params![
            topic.name,
            topic.description,
            if topic.enabled { 1 } else { 0 },
            topic.updated_at,
            topic.id,
        ],
    ).map_err(|e| format!("Failed to update topic: {}", e))?;

    if rows_affected == 0 {
        return Err(format!("Topic with id '{}' not found", topic.id));
    }

    Ok(())
}

/// Delete a topic by ID
pub fn delete_topic(conn: &Connection, id: &str) -> std::result::Result<(), String> {
    let rows_affected = conn.execute(
        "DELETE FROM topics WHERE id = ?1",
        [id],
    ).map_err(|e| format!("Failed to delete topic: {}", e))?;

    if rows_affected == 0 {
        return Err(format!("Topic with id '{}' not found", id));
    }

    Ok(())
}

/// Reorder topics by updating sort_order based on the provided ID list
pub fn reorder_topics(conn: &Connection, ids: &[String]) -> std::result::Result<(), String> {
    for (index, id) in ids.iter().enumerate() {
        conn.execute(
            "UPDATE topics SET sort_order = ?1 WHERE id = ?2",
            params![index as i32, id],
        ).map_err(|e| format!("Failed to update sort order: {}", e))?;
    }

    Ok(())
}

/// Get the next available sort_order value
pub fn get_next_sort_order(conn: &Connection) -> std::result::Result<i32, String> {
    let result: std::result::Result<Option<i32>, _> = conn.query_row(
        "SELECT MAX(sort_order) FROM topics",
        [],
        |row| row.get(0),
    );

    match result {
        Ok(Some(max)) => Ok(max + 1),
        Ok(None) => Ok(0),
        Err(_) => Ok(0),
    }
}

/// Check if a topic with the given name already exists (case-insensitive)
pub fn topic_name_exists(conn: &Connection, name: &str) -> std::result::Result<bool, String> {
    let count: i32 = conn.query_row(
        "SELECT COUNT(*) FROM topics WHERE LOWER(name) = LOWER(?1)",
        [name],
        |row| row.get(0),
    ).map_err(|e| format!("Failed to check topic name: {}", e))?;

    Ok(count > 0)
}

// ============================================================================
// Topic migration from JSON
// ============================================================================

/// Migrate topics from the JSON file to SQLite.
/// This is idempotent - it checks if migration has already been done.
pub fn migrate_topics_from_json(conn: &Connection) -> std::result::Result<MigrationResult, String> {
    let mut result = MigrationResult {
        topics_migrated: 0,
        json_backed_up: false,
        errors: Vec::new(),
    };

    // Check if migration already done (topics table has data)
    let count: i32 = conn.query_row(
        "SELECT COUNT(*) FROM topics",
        [],
        |row| row.get(0),
    ).map_err(|e| format!("Failed to check topics count: {}", e))?;

    if count > 0 {
        debug!("Topics table already has {} entries, skipping migration", count);
        return Ok(result);
    }

    // Check if JSON file exists
    let json_path = get_config_dir().join("interests.json");
    if !json_path.exists() {
        debug!("No interests.json file found, nothing to migrate");
        return Ok(result);
    }

    // Read JSON file
    let content = match std::fs::read_to_string(&json_path) {
        Ok(c) => c,
        Err(e) => {
            result.errors.push(format!("Failed to read interests.json: {}", e));
            return Ok(result);
        }
    };

    // Parse JSON
    #[derive(Deserialize)]
    struct TopicsConfig {
        topics: Vec<Topic>,
    }

    let config: TopicsConfig = match serde_json::from_str(&content) {
        Ok(c) => c,
        Err(e) => {
            result.errors.push(format!("Failed to parse interests.json: {}", e));
            warn!("interests.json is corrupt, starting fresh: {}", e);
            return Ok(result);
        }
    };

    // Insert topics with sort_order based on array position
    for (index, topic) in config.topics.iter().enumerate() {
        match insert_topic(conn, topic, index as i32) {
            Ok(_) => {
                result.topics_migrated += 1;
            }
            Err(e) => {
                result.errors.push(format!("Failed to migrate topic '{}': {}", topic.name, e));
            }
        }
    }

    // Backup the JSON file (rename to .migrated)
    if result.topics_migrated > 0 {
        let backup_path = get_config_dir().join("interests.json.migrated");
        match std::fs::rename(&json_path, &backup_path) {
            Ok(_) => {
                result.json_backed_up = true;
                info!("Backed up interests.json to interests.json.migrated");
            }
            Err(e) => {
                result.errors.push(format!("Failed to backup interests.json: {}", e));
            }
        }
    }

    if result.topics_migrated > 0 {
        info!("Migrated {} topics from JSON to SQLite", result.topics_migrated);
    }

    Ok(result)
}
