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

/// Chat message struct for database operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub id: i64,
    pub briefing_id: i64,
    pub card_index: i32,
    pub role: String,
    pub content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tokens_used: Option<i32>,
    pub created_at: String,
}

/// Represents a card that has chat messages (briefing_id + card_index)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CardWithChat {
    pub briefing_id: i64,
    pub card_index: i32,
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

    // Run migrations
    if let Err(e) = migrate_chat_messages_add_card_index(&conn) {
        warn!("Chat messages migration encountered an issue: {}", e);
    }

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
// Chat message CRUD operations
// ============================================================================

/// Get all chat messages for a specific card, ordered by creation time
pub fn get_chat_messages(conn: &Connection, briefing_id: i64, card_index: i32) -> std::result::Result<Vec<ChatMessage>, String> {
    let mut stmt = conn.prepare(
        "SELECT id, briefing_id, card_index, role, content, tokens_used, created_at
         FROM chat_messages
         WHERE briefing_id = ?1 AND card_index = ?2
         ORDER BY created_at ASC"
    ).map_err(|e| format!("Failed to prepare statement: {}", e))?;

    let messages = stmt.query_map(params![briefing_id, card_index], |row| {
        Ok(ChatMessage {
            id: row.get(0)?,
            briefing_id: row.get(1)?,
            card_index: row.get(2)?,
            role: row.get(3)?,
            content: row.get(4)?,
            tokens_used: row.get(5)?,
            created_at: row.get(6)?,
        })
    }).map_err(|e| format!("Query failed: {}", e))?
    .collect::<std::result::Result<Vec<_>, _>>()
    .map_err(|e| format!("Failed to collect results: {}", e))?;

    Ok(messages)
}

/// Insert a new chat message and return its ID
pub fn insert_chat_message(
    conn: &Connection,
    briefing_id: i64,
    card_index: i32,
    role: &str,
    content: &str,
    tokens_used: Option<i32>,
) -> std::result::Result<i64, String> {
    conn.execute(
        "INSERT INTO chat_messages (briefing_id, card_index, role, content, tokens_used)
         VALUES (?1, ?2, ?3, ?4, ?5)",
        params![briefing_id, card_index, role, content, tokens_used],
    ).map_err(|e| format!("Failed to insert chat message: {}", e))?;

    let id = conn.last_insert_rowid();
    Ok(id)
}

/// Get a single chat message by ID
pub fn get_chat_message_by_id(conn: &Connection, id: i64) -> std::result::Result<Option<ChatMessage>, String> {
    let mut stmt = conn.prepare(
        "SELECT id, briefing_id, card_index, role, content, tokens_used, created_at
         FROM chat_messages
         WHERE id = ?1"
    ).map_err(|e| format!("Failed to prepare statement: {}", e))?;

    let result = stmt.query_row([id], |row| {
        Ok(ChatMessage {
            id: row.get(0)?,
            briefing_id: row.get(1)?,
            card_index: row.get(2)?,
            role: row.get(3)?,
            content: row.get(4)?,
            tokens_used: row.get(5)?,
            created_at: row.get(6)?,
        })
    });

    match result {
        Ok(message) => Ok(Some(message)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(format!("Failed to get chat message: {}", e)),
    }
}

/// Delete all chat messages for a specific card
pub fn delete_chat_messages(conn: &Connection, briefing_id: i64, card_index: i32) -> std::result::Result<usize, String> {
    let rows_affected = conn.execute(
        "DELETE FROM chat_messages WHERE briefing_id = ?1 AND card_index = ?2",
        params![briefing_id, card_index],
    ).map_err(|e| format!("Failed to delete chat messages: {}", e))?;

    Ok(rows_affected)
}

/// Get all cards that have chat messages
pub fn get_cards_with_chats(conn: &Connection) -> std::result::Result<Vec<CardWithChat>, String> {
    let mut stmt = conn.prepare(
        "SELECT DISTINCT briefing_id, card_index FROM chat_messages"
    ).map_err(|e| format!("Failed to prepare statement: {}", e))?;

    let cards = stmt.query_map([], |row| {
        Ok(CardWithChat {
            briefing_id: row.get(0)?,
            card_index: row.get(1)?,
        })
    }).map_err(|e| format!("Failed to query cards with chats: {}", e))?;

    let result: Vec<CardWithChat> = cards.flatten().collect();
    Ok(result)
}

// ============================================================================
// Bookmark CRUD operations
// ============================================================================

/// Bookmark struct for database operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bookmark {
    pub id: i64,
    pub briefing_id: i64,
    pub card_index: i32,
    pub created_at: String,
}

/// Add a bookmark for a card (idempotent - ignores if already exists)
pub fn add_bookmark(conn: &Connection, briefing_id: i64, card_index: i32) -> std::result::Result<(), String> {
    conn.execute(
        "INSERT OR IGNORE INTO bookmarks (briefing_id, card_index) VALUES (?1, ?2)",
        params![briefing_id, card_index],
    ).map_err(|e| format!("Failed to add bookmark: {}", e))?;
    Ok(())
}

/// Remove a bookmark for a card
pub fn remove_bookmark(conn: &Connection, briefing_id: i64, card_index: i32) -> std::result::Result<bool, String> {
    let rows_affected = conn.execute(
        "DELETE FROM bookmarks WHERE briefing_id = ?1 AND card_index = ?2",
        params![briefing_id, card_index],
    ).map_err(|e| format!("Failed to remove bookmark: {}", e))?;
    Ok(rows_affected > 0)
}

/// Check if a card is bookmarked
pub fn is_bookmarked(conn: &Connection, briefing_id: i64, card_index: i32) -> std::result::Result<bool, String> {
    let count: i32 = conn.query_row(
        "SELECT COUNT(*) FROM bookmarks WHERE briefing_id = ?1 AND card_index = ?2",
        params![briefing_id, card_index],
        |row| row.get(0),
    ).map_err(|e| format!("Failed to check bookmark: {}", e))?;
    Ok(count > 0)
}

/// Get all bookmarks ordered by creation time (newest first)
pub fn get_all_bookmarks(conn: &Connection) -> std::result::Result<Vec<Bookmark>, String> {
    let mut stmt = conn.prepare(
        "SELECT id, briefing_id, card_index, created_at
         FROM bookmarks
         ORDER BY created_at DESC"
    ).map_err(|e| format!("Failed to prepare statement: {}", e))?;

    let bookmarks = stmt.query_map([], |row| {
        Ok(Bookmark {
            id: row.get(0)?,
            briefing_id: row.get(1)?,
            card_index: row.get(2)?,
            created_at: row.get(3)?,
        })
    }).map_err(|e| format!("Query failed: {}", e))?
    .collect::<std::result::Result<Vec<_>, _>>()
    .map_err(|e| format!("Failed to collect results: {}", e))?;

    Ok(bookmarks)
}

/// Toggle bookmark status for a card (add if not exists, remove if exists)
/// Returns true if bookmark was added, false if removed
pub fn toggle_bookmark(conn: &Connection, briefing_id: i64, card_index: i32) -> std::result::Result<bool, String> {
    if is_bookmarked(conn, briefing_id, card_index)? {
        remove_bookmark(conn, briefing_id, card_index)?;
        Ok(false)
    } else {
        add_bookmark(conn, briefing_id, card_index)?;
        Ok(true)
    }
}

// ============================================================================
// Housekeeping / Cleanup functions
// ============================================================================

/// Delete briefings older than `days`, excluding any briefings that have bookmarked cards.
/// Returns the count of deleted briefings.
pub fn cleanup_old_briefings(conn: &Connection, days: i32) -> std::result::Result<usize, String> {
    let deleted = conn.execute(
        "DELETE FROM briefings
         WHERE date < date('now', '-' || ?1 || ' days')
           AND id NOT IN (SELECT DISTINCT briefing_id FROM bookmarks)",
        [days],
    ).map_err(|e| format!("Failed to cleanup old briefings: {}", e))?;

    Ok(deleted)
}

/// Delete a specific briefing by ID.
/// Returns true if a briefing was deleted, false if not found.
pub fn delete_briefing(conn: &Connection, id: i64) -> std::result::Result<bool, String> {
    let deleted = conn.execute(
        "DELETE FROM briefings WHERE id = ?1",
        [id],
    ).map_err(|e| format!("Failed to delete briefing: {}", e))?;

    Ok(deleted > 0)
}

/// Get count of briefings that would be deleted by cleanup (for UI preview).
/// Excludes briefings with bookmarked cards.
pub fn count_cleanup_candidates(conn: &Connection, days: i32) -> std::result::Result<usize, String> {
    let count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM briefings
         WHERE date < date('now', '-' || ?1 || ' days')
           AND id NOT IN (SELECT DISTINCT briefing_id FROM bookmarks)",
        [days],
        |row| row.get(0),
    ).map_err(|e| format!("Failed to count cleanup candidates: {}", e))?;

    Ok(count as usize)
}

/// Get total count of briefings in database.
pub fn count_briefings(conn: &Connection) -> std::result::Result<usize, String> {
    let count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM briefings",
        [],
        |row| row.get(0),
    ).map_err(|e| format!("Failed to count briefings: {}", e))?;

    Ok(count as usize)
}

/// Count total cards across all briefings.
pub fn count_cards(conn: &Connection) -> std::result::Result<usize, String> {
    let mut stmt = conn.prepare("SELECT cards FROM briefings")
        .map_err(|e| format!("Failed to prepare statement: {}", e))?;

    let mut total_cards = 0usize;
    let rows = stmt.query_map([], |row| {
        let cards_json: String = row.get(0)?;
        Ok(cards_json)
    }).map_err(|e| format!("Failed to query briefings: {}", e))?;

    for cards_json in rows.flatten() {
        // Parse JSON array and count elements
        if let Ok(cards) = serde_json::from_str::<Vec<serde_json::Value>>(&cards_json) {
            total_cards += cards.len();
        }
    }

    Ok(total_cards)
}

/// Check if a briefing has any bookmarked cards.
pub fn briefing_has_bookmarks(conn: &Connection, briefing_id: i64) -> std::result::Result<bool, String> {
    let count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM bookmarks WHERE briefing_id = ?1",
        [briefing_id],
        |row| row.get(0),
    ).map_err(|e| format!("Failed to check briefing bookmarks: {}", e))?;

    Ok(count > 0)
}

/// Get recent card fingerprints for deduplication.
/// Returns (title, topic, summary) for all cards from the last N days.
pub fn get_recent_card_fingerprints(
    conn: &Connection,
    days: i32,
) -> std::result::Result<Vec<crate::dedup::CardFingerprint>, String> {
    let query = format!(
        "SELECT cards FROM briefings WHERE date > datetime('now', '-{} days') ORDER BY date DESC",
        days
    );

    let mut stmt = conn.prepare(&query)
        .map_err(|e| format!("Failed to prepare query: {}", e))?;

    let rows = stmt.query_map([], |row| {
        let cards_json: String = row.get(0)?;
        Ok(cards_json)
    }).map_err(|e| format!("Failed to query briefings: {}", e))?;

    let mut fingerprints = Vec::new();

    for row in rows {
        let cards_json = row.map_err(|e| format!("Failed to read row: {}", e))?;

        // Parse JSON array of cards
        if let Ok(cards) = serde_json::from_str::<Vec<serde_json::Value>>(&cards_json) {
            for card in cards {
                let title = card.get("title")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                let topic = card.get("topic")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                let summary = card.get("summary")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();

                if !title.is_empty() {
                    fingerprints.push(crate::dedup::CardFingerprint {
                        title,
                        topic,
                        summary,
                    });
                }
            }
        }
    }

    Ok(fingerprints)
}

// ============================================================================
// Chat messages migration (add card_index column)
// ============================================================================

/// Migrate chat_messages table to add card_index column if it doesn't exist.
/// Also ensures the composite index exists (for both migrated and new databases).
/// This is idempotent.
fn migrate_chat_messages_add_card_index(conn: &Connection) -> std::result::Result<(), String> {
    // Check if card_index column exists
    let mut stmt = conn.prepare("PRAGMA table_info(chat_messages)")
        .map_err(|e| format!("Failed to get table info: {}", e))?;

    let has_card_index = stmt.query_map([], |row| {
        row.get::<_, String>(1) // column name is at index 1
    }).map_err(|e| format!("Failed to query table info: {}", e))?
    .any(|name| name.map(|n| n == "card_index").unwrap_or(false));

    if !has_card_index {
        info!("Migrating chat_messages table: adding card_index column");
        conn.execute(
            "ALTER TABLE chat_messages ADD COLUMN card_index INTEGER NOT NULL DEFAULT 0",
            [],
        ).map_err(|e| format!("Failed to add card_index column: {}", e))?;
        info!("Chat messages column migration complete");
    }

    // Always ensure the composite index exists (works for both migrated and new databases)
    // Drop old single-column index if it exists
    let _ = conn.execute("DROP INDEX IF EXISTS idx_chat_messages_briefing", []);
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_chat_messages_briefing_card ON chat_messages(briefing_id, card_index)",
        [],
    ).map_err(|e| format!("Failed to create index: {}", e))?;

    Ok(())
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

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;

    fn setup_test_db() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch(include_str!("schema.sql")).unwrap();
        conn
    }

    fn create_test_briefing(conn: &Connection) -> i64 {
        conn.execute(
            "INSERT INTO briefings (date, title, cards) VALUES (?1, ?2, ?3)",
            ["2025-01-01", "Test Briefing", "[]"],
        ).unwrap();
        conn.last_insert_rowid()
    }

    #[test]
    fn test_insert_chat_message() {
        let conn = setup_test_db();
        let briefing_id = create_test_briefing(&conn);

        let id = insert_chat_message(
            &conn,
            briefing_id,
            0, // card_index
            "user",
            "Hello, test message",
            None,
        ).unwrap();
        assert!(id > 0);
    }

    #[test]
    fn test_get_chat_messages_empty() {
        let conn = setup_test_db();
        let briefing_id = create_test_briefing(&conn);

        let messages = get_chat_messages(&conn, briefing_id, 0).unwrap();
        assert!(messages.is_empty());
    }

    #[test]
    fn test_get_chat_messages_with_data() {
        let conn = setup_test_db();
        let briefing_id = create_test_briefing(&conn);

        // Insert a user message for card 0
        insert_chat_message(
            &conn,
            briefing_id,
            0,
            "user",
            "What is this about?",
            None,
        ).unwrap();

        // Insert an assistant message for card 0
        insert_chat_message(
            &conn,
            briefing_id,
            0,
            "assistant",
            "This briefing is about...",
            Some(100),
        ).unwrap();

        let messages = get_chat_messages(&conn, briefing_id, 0).unwrap();
        assert_eq!(messages.len(), 2);
        assert_eq!(messages[0].role, "user");
        assert_eq!(messages[1].role, "assistant");
        assert_eq!(messages[1].tokens_used, Some(100));
    }

    #[test]
    fn test_chat_messages_by_card_index() {
        let conn = setup_test_db();
        let briefing_id = create_test_briefing(&conn);

        // Insert messages for card 0
        insert_chat_message(&conn, briefing_id, 0, "user", "Card 0 message", None).unwrap();

        // Insert messages for card 1
        insert_chat_message(&conn, briefing_id, 1, "user", "Card 1 message", None).unwrap();
        insert_chat_message(&conn, briefing_id, 1, "assistant", "Card 1 reply", None).unwrap();

        // Verify card 0 has 1 message
        let card0_messages = get_chat_messages(&conn, briefing_id, 0).unwrap();
        assert_eq!(card0_messages.len(), 1);
        assert_eq!(card0_messages[0].content, "Card 0 message");

        // Verify card 1 has 2 messages
        let card1_messages = get_chat_messages(&conn, briefing_id, 1).unwrap();
        assert_eq!(card1_messages.len(), 2);
    }

    #[test]
    fn test_get_chat_message_by_id() {
        let conn = setup_test_db();
        let briefing_id = create_test_briefing(&conn);

        let id = insert_chat_message(
            &conn,
            briefing_id,
            0,
            "user",
            "Test content",
            None,
        ).unwrap();

        let retrieved = get_chat_message_by_id(&conn, id).unwrap();
        assert!(retrieved.is_some());
        let retrieved = retrieved.unwrap();
        assert_eq!(retrieved.id, id);
        assert_eq!(retrieved.content, "Test content");
        assert_eq!(retrieved.card_index, 0);
    }

    #[test]
    fn test_get_chat_message_by_id_not_found() {
        let conn = setup_test_db();

        let retrieved = get_chat_message_by_id(&conn, 999).unwrap();
        assert!(retrieved.is_none());
    }

    #[test]
    fn test_delete_chat_messages() {
        let conn = setup_test_db();
        let briefing_id = create_test_briefing(&conn);

        // Insert messages for card 0
        for i in 0..3 {
            insert_chat_message(
                &conn,
                briefing_id,
                0,
                "user",
                &format!("Message {}", i),
                None,
            ).unwrap();
        }

        // Insert messages for card 1 (should not be deleted)
        insert_chat_message(&conn, briefing_id, 1, "user", "Card 1 message", None).unwrap();

        // Verify messages exist
        let messages = get_chat_messages(&conn, briefing_id, 0).unwrap();
        assert_eq!(messages.len(), 3);

        // Delete messages for card 0 only
        let deleted = delete_chat_messages(&conn, briefing_id, 0).unwrap();
        assert_eq!(deleted, 3);

        // Verify card 0 messages are gone
        let messages = get_chat_messages(&conn, briefing_id, 0).unwrap();
        assert!(messages.is_empty());

        // Verify card 1 messages are still there
        let messages = get_chat_messages(&conn, briefing_id, 1).unwrap();
        assert_eq!(messages.len(), 1);
    }

    #[test]
    fn test_chat_messages_cascade_delete() {
        let conn = setup_test_db();
        let briefing_id = create_test_briefing(&conn);

        // Insert chat messages for multiple cards
        insert_chat_message(&conn, briefing_id, 0, "user", "Test 0", None).unwrap();
        insert_chat_message(&conn, briefing_id, 1, "user", "Test 1", None).unwrap();

        // Delete the briefing (should cascade to all chat_messages)
        conn.execute("DELETE FROM briefings WHERE id = ?1", [briefing_id]).unwrap();

        // All chat messages should be gone
        let messages = get_chat_messages(&conn, briefing_id, 0).unwrap();
        assert!(messages.is_empty());
        let messages = get_chat_messages(&conn, briefing_id, 1).unwrap();
        assert!(messages.is_empty());
    }

    #[test]
    fn test_get_cards_with_chats() {
        let conn = setup_test_db();
        let briefing_id = create_test_briefing(&conn);

        // Initially no cards with chats
        let cards = get_cards_with_chats(&conn).unwrap();
        assert!(cards.is_empty());

        // Add chat to card 0
        insert_chat_message(&conn, briefing_id, 0, "user", "Hello", None).unwrap();

        // Add chat to card 2 (skipping 1)
        insert_chat_message(&conn, briefing_id, 2, "user", "World", None).unwrap();

        let cards = get_cards_with_chats(&conn).unwrap();
        assert_eq!(cards.len(), 2);
        assert!(cards.iter().any(|c| c.briefing_id == briefing_id && c.card_index == 0));
        assert!(cards.iter().any(|c| c.briefing_id == briefing_id && c.card_index == 2));
    }

    // ========================================================================
    // Bookmark tests
    // ========================================================================

    #[test]
    fn test_add_bookmark() {
        let conn = setup_test_db();
        let briefing_id = create_test_briefing(&conn);

        // Add bookmark
        add_bookmark(&conn, briefing_id, 0).unwrap();

        // Verify it exists
        assert!(is_bookmarked(&conn, briefing_id, 0).unwrap());
    }

    #[test]
    fn test_add_bookmark_idempotent() {
        let conn = setup_test_db();
        let briefing_id = create_test_briefing(&conn);

        // Add bookmark twice (should not error)
        add_bookmark(&conn, briefing_id, 0).unwrap();
        add_bookmark(&conn, briefing_id, 0).unwrap();

        // Should still only have one bookmark
        let bookmarks = get_all_bookmarks(&conn).unwrap();
        assert_eq!(bookmarks.len(), 1);
    }

    #[test]
    fn test_remove_bookmark() {
        let conn = setup_test_db();
        let briefing_id = create_test_briefing(&conn);

        // Add then remove bookmark
        add_bookmark(&conn, briefing_id, 0).unwrap();
        let removed = remove_bookmark(&conn, briefing_id, 0).unwrap();

        assert!(removed);
        assert!(!is_bookmarked(&conn, briefing_id, 0).unwrap());
    }

    #[test]
    fn test_remove_nonexistent_bookmark() {
        let conn = setup_test_db();
        let briefing_id = create_test_briefing(&conn);

        // Remove bookmark that doesn't exist
        let removed = remove_bookmark(&conn, briefing_id, 0).unwrap();
        assert!(!removed);
    }

    #[test]
    fn test_is_bookmarked() {
        let conn = setup_test_db();
        let briefing_id = create_test_briefing(&conn);

        // Not bookmarked initially
        assert!(!is_bookmarked(&conn, briefing_id, 0).unwrap());

        // Add bookmark
        add_bookmark(&conn, briefing_id, 0).unwrap();
        assert!(is_bookmarked(&conn, briefing_id, 0).unwrap());

        // Different card index should not be bookmarked
        assert!(!is_bookmarked(&conn, briefing_id, 1).unwrap());
    }

    #[test]
    fn test_get_all_bookmarks() {
        let conn = setup_test_db();
        let briefing_id = create_test_briefing(&conn);

        // No bookmarks initially
        let bookmarks = get_all_bookmarks(&conn).unwrap();
        assert!(bookmarks.is_empty());

        // Add some bookmarks
        add_bookmark(&conn, briefing_id, 0).unwrap();
        add_bookmark(&conn, briefing_id, 2).unwrap();

        let bookmarks = get_all_bookmarks(&conn).unwrap();
        assert_eq!(bookmarks.len(), 2);
    }

    #[test]
    fn test_toggle_bookmark() {
        let conn = setup_test_db();
        let briefing_id = create_test_briefing(&conn);

        // Toggle on (add)
        let added = toggle_bookmark(&conn, briefing_id, 0).unwrap();
        assert!(added);
        assert!(is_bookmarked(&conn, briefing_id, 0).unwrap());

        // Toggle off (remove)
        let added = toggle_bookmark(&conn, briefing_id, 0).unwrap();
        assert!(!added);
        assert!(!is_bookmarked(&conn, briefing_id, 0).unwrap());
    }

    #[test]
    fn test_bookmarks_cascade_delete() {
        let conn = setup_test_db();
        let briefing_id = create_test_briefing(&conn);

        // Add bookmark
        add_bookmark(&conn, briefing_id, 0).unwrap();
        assert!(is_bookmarked(&conn, briefing_id, 0).unwrap());

        // Delete the briefing (should cascade to bookmarks)
        conn.execute("DELETE FROM briefings WHERE id = ?1", [briefing_id]).unwrap();

        // Bookmark should be gone
        let bookmarks = get_all_bookmarks(&conn).unwrap();
        assert!(bookmarks.is_empty());
    }

    // ========================================================================
    // Housekeeping / Cleanup tests
    // ========================================================================

    #[test]
    fn test_delete_briefing() {
        let conn = setup_test_db();
        let briefing_id = create_test_briefing(&conn);

        // Briefing exists
        assert_eq!(count_briefings(&conn).unwrap(), 1);

        // Delete it
        let deleted = delete_briefing(&conn, briefing_id).unwrap();
        assert!(deleted);

        // Gone
        assert_eq!(count_briefings(&conn).unwrap(), 0);

        // Deleting again returns false
        let deleted = delete_briefing(&conn, briefing_id).unwrap();
        assert!(!deleted);
    }

    #[test]
    fn test_count_briefings() {
        let conn = setup_test_db();

        assert_eq!(count_briefings(&conn).unwrap(), 0);

        create_test_briefing(&conn);
        assert_eq!(count_briefings(&conn).unwrap(), 1);

        create_test_briefing(&conn);
        assert_eq!(count_briefings(&conn).unwrap(), 2);
    }

    #[test]
    fn test_count_cards() {
        let conn = setup_test_db();

        // No briefings = 0 cards
        assert_eq!(count_cards(&conn).unwrap(), 0);

        // Create briefing with 2 cards
        conn.execute(
            "INSERT INTO briefings (date, title, cards) VALUES (date('now'), 'Test', ?)",
            [r#"[{"title":"Card 1"},{"title":"Card 2"}]"#],
        ).unwrap();
        assert_eq!(count_cards(&conn).unwrap(), 2);

        // Create briefing with 3 cards
        conn.execute(
            "INSERT INTO briefings (date, title, cards) VALUES (date('now'), 'Test 2', ?)",
            [r#"[{"title":"A"},{"title":"B"},{"title":"C"}]"#],
        ).unwrap();
        assert_eq!(count_cards(&conn).unwrap(), 5); // 2 + 3

        // Empty cards array still counts as 0
        conn.execute(
            "INSERT INTO briefings (date, title, cards) VALUES (date('now'), 'Empty', '[]')",
            [],
        ).unwrap();
        assert_eq!(count_cards(&conn).unwrap(), 5); // Still 5
    }

    #[test]
    fn test_briefing_has_bookmarks() {
        let conn = setup_test_db();
        let briefing_id = create_test_briefing(&conn);

        // No bookmarks
        assert!(!briefing_has_bookmarks(&conn, briefing_id).unwrap());

        // Add bookmark
        add_bookmark(&conn, briefing_id, 0).unwrap();
        assert!(briefing_has_bookmarks(&conn, briefing_id).unwrap());
    }

    #[test]
    fn test_cleanup_old_briefings_preserves_bookmarked() {
        let conn = setup_test_db();

        // Create an old briefing (100 days ago)
        conn.execute(
            "INSERT INTO briefings (date, title, cards) VALUES (date('now', '-100 days'), 'Old', '[]')",
            [],
        ).unwrap();
        let old_id: i64 = conn.last_insert_rowid();

        // Create a recent briefing (today)
        conn.execute(
            "INSERT INTO briefings (date, title, cards) VALUES (date('now'), 'Recent', '[]')",
            [],
        ).unwrap();
        let recent_id: i64 = conn.last_insert_rowid();

        assert_eq!(count_briefings(&conn).unwrap(), 2);

        // Bookmark the old briefing
        add_bookmark(&conn, old_id, 0).unwrap();

        // Cleanup briefings older than 30 days
        let deleted = cleanup_old_briefings(&conn, 30).unwrap();
        assert_eq!(deleted, 0);  // Old one is bookmarked, so not deleted

        assert_eq!(count_briefings(&conn).unwrap(), 2);

        // Remove bookmark and try again
        remove_bookmark(&conn, old_id, 0).unwrap();
        let deleted = cleanup_old_briefings(&conn, 30).unwrap();
        assert_eq!(deleted, 1);  // Now it gets deleted

        // Only recent briefing remains
        assert_eq!(count_briefings(&conn).unwrap(), 1);

        // Verify the recent one is still there
        let exists: i64 = conn.query_row(
            "SELECT COUNT(*) FROM briefings WHERE id = ?1",
            [recent_id],
            |row| row.get(0),
        ).unwrap();
        assert_eq!(exists, 1);
    }

    #[test]
    fn test_count_cleanup_candidates() {
        let conn = setup_test_db();

        // Create old briefing (100 days ago)
        conn.execute(
            "INSERT INTO briefings (date, title, cards) VALUES (date('now', '-100 days'), 'Old1', '[]')",
            [],
        ).unwrap();
        let old_id: i64 = conn.last_insert_rowid();

        // Create another old briefing (50 days ago)
        conn.execute(
            "INSERT INTO briefings (date, title, cards) VALUES (date('now', '-50 days'), 'Old2', '[]')",
            [],
        ).unwrap();

        // Create recent briefing (today)
        conn.execute(
            "INSERT INTO briefings (date, title, cards) VALUES (date('now'), 'Recent', '[]')",
            [],
        ).unwrap();

        // 2 candidates for 30-day cleanup (100-day and 50-day old)
        assert_eq!(count_cleanup_candidates(&conn, 30).unwrap(), 2);

        // 1 candidate for 60-day cleanup (only 100-day old)
        assert_eq!(count_cleanup_candidates(&conn, 60).unwrap(), 1);

        // Bookmark oldest one - should reduce count
        add_bookmark(&conn, old_id, 0).unwrap();
        assert_eq!(count_cleanup_candidates(&conn, 30).unwrap(), 1);
    }
}
