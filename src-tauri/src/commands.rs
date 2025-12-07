use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use crate::db;

#[derive(Debug, Serialize, Deserialize)]
pub struct Briefing {
    pub id: i64,
    pub date: String,
    pub title: String,
    pub cards: String, // JSON string
    pub research_time_ms: Option<i64>,
    pub model_used: Option<String>,
    pub total_tokens: Option<i64>,
}

fn get_config_path() -> PathBuf {
    let home = dirs::home_dir().expect("Could not find home directory");
    home.join(".claudius").join("config.json")
}

fn read_config() -> Result<serde_json::Value, String> {
    let config_path = get_config_path();

    if !config_path.exists() {
        // Return default config
        return Ok(serde_json::json!({
            "interests": [],
            "preferences": {
                "schedule": "0 6 * * *",
                "briefingLength": "medium",
                "notificationsEnabled": true
            }
        }));
    }

    let content = std::fs::read_to_string(&config_path)
        .map_err(|e| format!("Failed to read config: {}", e))?;

    serde_json::from_str(&content)
        .map_err(|e| format!("Failed to parse config: {}", e))
}

fn write_config(config: &serde_json::Value) -> Result<(), String> {
    let config_path = get_config_path();

    // Ensure directory exists
    if let Some(parent) = config_path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create config directory: {}", e))?;
    }

    let content = serde_json::to_string_pretty(&config)
        .map_err(|e| format!("Failed to serialize config: {}", e))?;

    std::fs::write(&config_path, content)
        .map_err(|e| format!("Failed to write config: {}", e))
}

#[tauri::command]
pub fn get_briefings(limit: Option<i32>) -> Result<Vec<Briefing>, String> {
    let conn = db::get_connection()
        .map_err(|e| format!("Database connection failed: {}", e))?;

    let limit_value = limit.unwrap_or(30);

    let mut stmt = conn.prepare(
        "SELECT id, date, title, cards, research_time_ms, model_used, total_tokens
         FROM briefings
         ORDER BY date DESC
         LIMIT ?1"
    ).map_err(|e| format!("Failed to prepare statement: {}", e))?;

    let briefings = stmt.query_map([limit_value], |row| {
        Ok(Briefing {
            id: row.get(0)?,
            date: row.get(1)?,
            title: row.get(2)?,
            cards: row.get(3)?,
            research_time_ms: row.get(4)?,
            model_used: row.get(5)?,
            total_tokens: row.get(6)?,
        })
    }).map_err(|e| format!("Query failed: {}", e))?
    .collect::<Result<Vec<_>, _>>()
    .map_err(|e| format!("Failed to collect results: {}", e))?;

    Ok(briefings)
}

#[tauri::command]
pub fn get_briefing(id: i64) -> Result<Briefing, String> {
    let conn = db::get_connection()
        .map_err(|e| format!("Database connection failed: {}", e))?;

    let mut stmt = conn.prepare(
        "SELECT id, date, title, cards, research_time_ms, model_used, total_tokens
         FROM briefings
         WHERE id = ?1"
    ).map_err(|e| format!("Failed to prepare statement: {}", e))?;

    let briefing = stmt.query_row([id], |row| {
        Ok(Briefing {
            id: row.get(0)?,
            date: row.get(1)?,
            title: row.get(2)?,
            cards: row.get(3)?,
            research_time_ms: row.get(4)?,
            model_used: row.get(5)?,
            total_tokens: row.get(6)?,
        })
    }).map_err(|e| format!("Failed to get briefing: {}", e))?;

    Ok(briefing)
}

#[tauri::command]
pub fn search_briefings(query: String) -> Result<Vec<Briefing>, String> {
    let conn = db::get_connection()
        .map_err(|e| format!("Database connection failed: {}", e))?;

    let search_pattern = format!("%{}%", query);

    let mut stmt = conn.prepare(
        "SELECT id, date, title, cards, research_time_ms, model_used, total_tokens
         FROM briefings
         WHERE title LIKE ?1 OR cards LIKE ?1
         ORDER BY date DESC
         LIMIT 50"
    ).map_err(|e| format!("Failed to prepare statement: {}", e))?;

    let briefings = stmt.query_map([&search_pattern], |row| {
        Ok(Briefing {
            id: row.get(0)?,
            date: row.get(1)?,
            title: row.get(2)?,
            cards: row.get(3)?,
            research_time_ms: row.get(4)?,
            model_used: row.get(5)?,
            total_tokens: row.get(6)?,
        })
    }).map_err(|e| format!("Query failed: {}", e))?
    .collect::<Result<Vec<_>, _>>()
    .map_err(|e| format!("Failed to collect results: {}", e))?;

    Ok(briefings)
}

#[tauri::command]
pub fn add_feedback(
    briefing_id: i64,
    card_index: i32,
    rating: i32,
    reason: Option<String>,
) -> Result<(), String> {
    if !(1..=5).contains(&rating) {
        return Err("Rating must be between 1 and 5".to_string());
    }

    let conn = db::get_connection()
        .map_err(|e| format!("Database connection failed: {}", e))?;

    conn.execute(
        "INSERT INTO feedback (briefing_id, card_index, rating, reason)
         VALUES (?1, ?2, ?3, ?4)",
        rusqlite::params![briefing_id, card_index, rating, reason],
    ).map_err(|e| format!("Failed to insert feedback: {}", e))?;

    Ok(())
}

#[tauri::command]
pub fn get_interests() -> Result<serde_json::Value, String> {
    let config = read_config()?;
    Ok(config.get("interests").cloned().unwrap_or(serde_json::json!([])))
}

#[tauri::command]
pub fn add_interest(topic: String) -> Result<(), String> {
    let mut config = read_config()?;

    let interests = config
        .get_mut("interests")
        .and_then(|v| v.as_array_mut())
        .ok_or_else(|| "Invalid config format".to_string())?;

    // Check if interest already exists
    if !interests.iter().any(|i| i.as_str() == Some(&topic)) {
        interests.push(serde_json::json!(topic));
        write_config(&config)?;
    }

    Ok(())
}

#[tauri::command]
pub fn remove_interest(topic: String) -> Result<(), String> {
    let mut config = read_config()?;

    let interests = config
        .get_mut("interests")
        .and_then(|v| v.as_array_mut())
        .ok_or_else(|| "Invalid config format".to_string())?;

    interests.retain(|i| i.as_str() != Some(&topic));
    write_config(&config)?;

    Ok(())
}

#[tauri::command]
pub fn get_preferences() -> Result<serde_json::Value, String> {
    let config = read_config()?;
    Ok(config.get("preferences").cloned().unwrap_or(serde_json::json!({})))
}

#[tauri::command]
pub fn update_preferences(preferences: serde_json::Value) -> Result<(), String> {
    let mut config = read_config()?;
    config["preferences"] = preferences;
    write_config(&config)?;
    Ok(())
}

#[tauri::command]
pub async fn trigger_research() -> Result<String, String> {
    use tokio::process::Command;

    // Try to find claudius CLI in PATH
    let output = Command::new("claudius")
        .arg("research")
        .output()
        .await
        .map_err(|e| format!("Failed to execute claudius CLI: {}", e))?;

    if output.status.success() {
        Ok("Research triggered successfully".to_string())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(format!("Research failed: {}", stderr))
    }
}
