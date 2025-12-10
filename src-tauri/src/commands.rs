use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use chrono::{Local, Utc};
use uuid::Uuid;
use tauri::Emitter;
use crate::db::{self, Topic};
use crate::research_state;
use crate::research::CancelledEvent;

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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MCPServer {
    pub id: String,
    pub name: String,
    pub enabled: bool,
    pub config: serde_json::Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_used: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MCPServersConfig {
    pub servers: Vec<MCPServer>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResearchSettings {
    pub schedule_cron: String,
    pub model: String,
    pub research_depth: String,
    pub max_sources_per_topic: i32,
    pub enable_notifications: bool,
    #[serde(default = "default_notification_sound")]
    pub notification_sound: bool,
    #[serde(default)]
    pub enable_web_search: bool,
}

fn default_notification_sound() -> bool {
    true
}

fn get_config_dir() -> PathBuf {
    let home = dirs::home_dir().expect("Could not find home directory");
    home.join(".claudius")
}

fn ensure_config_dir() -> Result<PathBuf, String> {
    let config_dir = get_config_dir();
    std::fs::create_dir_all(&config_dir)
        .map_err(|e| format!("Failed to create config directory: {}", e))?;
    Ok(config_dir)
}

fn get_mcp_servers_path() -> PathBuf {
    get_config_dir().join("mcp-servers.json")
}

fn get_preferences_path() -> PathBuf {
    get_config_dir().join("preferences.json")
}

fn get_logs_dir() -> PathBuf {
    get_config_dir().join("logs")
}

/// Write an error to the agent log file
fn log_agent_error(context: &str, error: &str) {
    let logs_dir = get_logs_dir();
    if std::fs::create_dir_all(&logs_dir).is_err() {
        tracing::warn!("Failed to create logs directory");
        return;
    }

    let log_file = logs_dir.join("agent.log");
    let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
    let log_entry = format!("[{}] {}: {}\n", timestamp, context, error);

    // Append to log file
    if let Ok(mut file) = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_file)
    {
        use std::io::Write;
        let _ = file.write_all(log_entry.as_bytes());
    }
}

fn read_mcp_servers() -> Result<MCPServersConfig, String> {
    let path = get_mcp_servers_path();
    if !path.exists() {
        return Ok(MCPServersConfig { servers: vec![] });
    }
    let content = std::fs::read_to_string(&path)
        .map_err(|e| format!("Failed to read MCP servers: {}", e))?;
    serde_json::from_str(&content)
        .map_err(|e| format!("Failed to parse MCP servers: {}", e))
}

fn write_mcp_servers(config: &MCPServersConfig) -> Result<(), String> {
    ensure_config_dir()?;
    let path = get_mcp_servers_path();
    let content = serde_json::to_string_pretty(&config)
        .map_err(|e| format!("Failed to serialize MCP servers: {}", e))?;
    std::fs::write(&path, content)
        .map_err(|e| format!("Failed to write MCP servers: {}", e))
}

fn read_settings() -> Result<ResearchSettings, String> {
    let path = get_preferences_path();
    if !path.exists() {
        return Ok(ResearchSettings {
            schedule_cron: "0 6 * * *".to_string(),
            model: "claude-haiku-4-5-20251001".to_string(),
            research_depth: "medium".to_string(),
            max_sources_per_topic: 10,
            enable_notifications: true,
            notification_sound: true,
            enable_web_search: false,
        });
    }
    let content = std::fs::read_to_string(&path)
        .map_err(|e| format!("Failed to read settings: {}", e))?;
    serde_json::from_str(&content)
        .map_err(|e| format!("Failed to parse settings: {}", e))
}

fn write_settings(settings: &ResearchSettings) -> Result<(), String> {
    ensure_config_dir()?;
    let path = get_preferences_path();
    let content = serde_json::to_string_pretty(&settings)
        .map_err(|e| format!("Failed to serialize settings: {}", e))?;
    std::fs::write(&path, content)
        .map_err(|e| format!("Failed to write settings: {}", e))
}

// Legacy config helpers for backwards compatibility
fn get_config_path() -> PathBuf {
    get_config_dir().join("config.json")
}

fn read_config() -> Result<serde_json::Value, String> {
    let config_path = get_config_path();

    if !config_path.exists() {
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
    ensure_config_dir()?;
    let config_path = get_config_path();
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
pub async fn trigger_research(app: tauri::AppHandle) -> Result<String, String> {
    use crate::research::ResearchAgent;
    use crate::notifications::{notify_research_complete, notify_research_error};

    tracing::info!("Starting research via Rust agent");

    // Try to acquire the research lock and get the cancellation token
    let cancellation_token = match research_state::set_running("starting") {
        Ok(token) => token,
        Err(e) => {
            tracing::warn!("Cannot start research: {}", e);
            return Err(e);
        }
    };

    // Helper to ensure we always clean up the state
    struct StateGuard;
    impl Drop for StateGuard {
        fn drop(&mut self) {
            if let Err(e) = research_state::set_stopped() {
                tracing::error!("Failed to clear research state in guard: {}", e);
            }
        }
    }
    let _guard = StateGuard;

    // Get settings
    let settings = read_settings().unwrap_or_else(|_| ResearchSettings {
        schedule_cron: "0 6 * * *".to_string(),
        model: "claude-haiku-4-5-20251001".to_string(),
        research_depth: "medium".to_string(),
        max_sources_per_topic: 10,
        enable_notifications: true,
        notification_sound: true,
        enable_web_search: false,
    });

    // Get API key from file-based storage
    let api_key = match get_api_key_for_research() {
        Some(key) => key,
        None => {
            let err = "No API key configured. Please set your Anthropic API key in Settings.";
            log_agent_error("RESEARCH", err);
            if settings.enable_notifications {
                let _ = notify_research_error(&app, err);
            }
            return Err(err.to_string());
        }
    };

    // Get enabled topics from SQLite
    let conn = match db::get_connection() {
        Ok(c) => c,
        Err(e) => {
            let err = format!("Database connection failed: {}", e);
            if settings.enable_notifications {
                let _ = notify_research_error(&app, &err);
            }
            return Err(err);
        }
    };

    let all_topics = match db::get_all_topics(&conn) {
        Ok(t) => t,
        Err(e) => {
            if settings.enable_notifications {
                let _ = notify_research_error(&app, &e);
            }
            return Err(e);
        }
    };

    let topics: Vec<String> = all_topics
        .iter()
        .filter(|t| t.enabled)
        .map(|t| t.name.clone())
        .collect();

    if topics.is_empty() {
        let err = "No topics configured. Please add topics in Settings.";
        if settings.enable_notifications {
            let _ = notify_research_error(&app, err);
        }
        return Err(err.to_string());
    }

    tracing::info!("Researching {} topics: {:?}", topics.len(), topics);

    // Update phase
    research_state::set_phase("researching");

    // Create research agent and set cancellation token
    let mut agent = ResearchAgent::new(api_key, Some(settings.model.clone()), settings.enable_web_search);
    agent.set_cancellation_token(cancellation_token);

    let result = match agent.run_research(topics, Some(app.clone())).await {
        Ok(r) => r,
        Err(e) => {
            // Check if this was a cancellation
            if e.contains("cancelled") {
                tracing::info!("Research was cancelled by user");
            } else if settings.enable_notifications {
                let _ = notify_research_error(&app, &e);
            }
            return Err(e);
        }
    };

    // Update phase to saving
    research_state::set_phase("saving");

    // Emit research:saving event
    let _ = app.emit("research:saving", serde_json::json!({
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "total_cards": result.cards.len(),
    }));

    // Save to database
    let cards_json = serde_json::to_string(&result.cards)
        .map_err(|e| format!("Failed to serialize cards: {}", e))?;

    let conn = db::get_connection()
        .map_err(|e| format!("Database connection failed: {}", e))?;

    conn.execute(
        "INSERT INTO briefings (date, title, cards, research_time_ms, model_used, total_tokens)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        rusqlite::params![
            result.date,
            result.title,
            cards_json,
            result.research_time_ms as i64,
            result.model_used,
            result.total_tokens as i64,
        ],
    ).map_err(|e| format!("Failed to insert briefing: {}", e))?;

    tracing::info!(
        "Research completed: {} cards saved, {}ms",
        result.cards.len(),
        result.research_time_ms
    );

    // Clear research state
    if let Err(e) = research_state::set_stopped() {
        tracing::error!("Failed to clear research state: {}", e);
    }

    // Emit research:completed event after successful save
    let _ = app.emit("research:completed", serde_json::json!({
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "total_cards": result.cards.len(),
        "duration_ms": result.research_time_ms,
    }));

    // Send success notification
    if settings.enable_notifications {
        let _ = notify_research_complete(&app, result.cards.len(), settings.notification_sound);
    }

    Ok(format!(
        "Research completed: {} briefing cards generated in {}ms",
        result.cards.len(),
        result.research_time_ms
    ))
}

/// Run research without notifications (for scheduler when AppHandle is unavailable)
pub async fn trigger_research_no_notify() -> Result<String, String> {
    use crate::research::ResearchAgent;

    tracing::info!("Starting research via Rust agent (no notifications)");

    // Get settings
    let settings = read_settings().unwrap_or_else(|_| ResearchSettings {
        schedule_cron: "0 6 * * *".to_string(),
        model: "claude-haiku-4-5-20251001".to_string(),
        research_depth: "medium".to_string(),
        max_sources_per_topic: 10,
        enable_notifications: true,
        notification_sound: true,
        enable_web_search: false,
    });

    // Get API key from file-based storage
    let api_key = match get_api_key_for_research() {
        Some(key) => key,
        None => {
            let err = "No API key configured. Please set your Anthropic API key in Settings.";
            log_agent_error("RESEARCH", err);
            return Err(err.to_string());
        }
    };

    // Get enabled topics from SQLite
    let conn = db::get_connection()
        .map_err(|e| format!("Database connection failed: {}", e))?;
    let all_topics = db::get_all_topics(&conn)?;

    let topics: Vec<String> = all_topics
        .iter()
        .filter(|t| t.enabled)
        .map(|t| t.name.clone())
        .collect();

    if topics.is_empty() {
        return Err("No topics configured. Please add topics in Settings.".to_string());
    }

    tracing::info!("Researching {} topics: {:?}", topics.len(), topics);

    // Create research agent and run research
    let mut agent = ResearchAgent::new(api_key, Some(settings.model.clone()), settings.enable_web_search);
    let result = agent.run_research(topics, None).await?;

    // Save to database
    let cards_json = serde_json::to_string(&result.cards)
        .map_err(|e| format!("Failed to serialize cards: {}", e))?;

    let conn = db::get_connection()
        .map_err(|e| format!("Database connection failed: {}", e))?;

    conn.execute(
        "INSERT INTO briefings (date, title, cards, research_time_ms, model_used, total_tokens)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        rusqlite::params![
            result.date,
            result.title,
            cards_json,
            result.research_time_ms as i64,
            result.model_used,
            result.total_tokens as i64,
        ],
    ).map_err(|e| format!("Failed to insert briefing: {}", e))?;

    tracing::info!(
        "Research completed: {} cards saved, {}ms",
        result.cards.len(),
        result.research_time_ms
    );

    Ok(format!(
        "Research completed: {} briefing cards generated in {}ms",
        result.cards.len(),
        result.research_time_ms
    ))
}

// ============================================================================
// Topics commands (SQLite-backed)
// ============================================================================

#[tauri::command]
pub fn get_topics() -> Result<Vec<Topic>, String> {
    let conn = db::get_connection()
        .map_err(|e| format!("Database connection failed: {}", e))?;
    db::get_all_topics(&conn)
}

#[tauri::command]
pub fn add_topic(name: String, description: Option<String>) -> Result<Topic, String> {
    let conn = db::get_connection()
        .map_err(|e| format!("Database connection failed: {}", e))?;

    // Check if topic already exists
    if db::topic_name_exists(&conn, &name)? {
        return Err(format!("Topic '{}' already exists", name));
    }

    let now = Utc::now().to_rfc3339();
    let topic = Topic {
        id: Uuid::new_v4().to_string(),
        name,
        description,
        enabled: true,
        created_at: now.clone(),
        updated_at: now,
    };

    let sort_order = db::get_next_sort_order(&conn)?;
    db::insert_topic(&conn, &topic, sort_order)?;

    Ok(topic)
}

#[tauri::command]
pub fn update_topic(
    id: String,
    name: Option<String>,
    description: Option<String>,
    enabled: Option<bool>,
) -> Result<Topic, String> {
    let conn = db::get_connection()
        .map_err(|e| format!("Database connection failed: {}", e))?;

    // Get existing topic
    let mut topic = db::get_topic_by_id(&conn, &id)?
        .ok_or_else(|| format!("Topic with id '{}' not found", id))?;

    // Update fields
    if let Some(new_name) = name {
        topic.name = new_name;
    }
    if let Some(new_desc) = description {
        topic.description = Some(new_desc);
    }
    if let Some(new_enabled) = enabled {
        topic.enabled = new_enabled;
    }
    topic.updated_at = Utc::now().to_rfc3339();

    db::update_topic(&conn, &topic)?;

    Ok(topic)
}

#[tauri::command]
pub fn delete_topic(id: String) -> Result<(), String> {
    let conn = db::get_connection()
        .map_err(|e| format!("Database connection failed: {}", e))?;
    db::delete_topic(&conn, &id)
}

#[tauri::command]
pub fn reorder_topics(ids: Vec<String>) -> Result<(), String> {
    let conn = db::get_connection()
        .map_err(|e| format!("Database connection failed: {}", e))?;
    db::reorder_topics(&conn, &ids)
}

// ============================================================================
// MCP Server commands
// ============================================================================

#[tauri::command]
pub fn get_mcp_servers() -> Result<Vec<MCPServer>, String> {
    let config = read_mcp_servers()?;
    Ok(config.servers)
}

#[tauri::command]
pub fn toggle_mcp_server(id: String, enabled: bool) -> Result<MCPServer, String> {
    let mut config = read_mcp_servers()?;

    let server = config.servers.iter_mut()
        .find(|s| s.id == id)
        .ok_or_else(|| format!("MCP server with id '{}' not found", id))?;

    server.enabled = enabled;
    let updated_server = server.clone();

    write_mcp_servers(&config)?;

    Ok(updated_server)
}

#[tauri::command]
pub fn add_mcp_server(name: String, config_data: serde_json::Value) -> Result<MCPServer, String> {
    let mut config = read_mcp_servers()?;

    let server = MCPServer {
        id: Uuid::new_v4().to_string(),
        name,
        enabled: true,
        config: config_data,
        last_used: None,
    };

    config.servers.push(server.clone());
    write_mcp_servers(&config)?;

    Ok(server)
}

#[tauri::command]
pub fn remove_mcp_server(id: String) -> Result<(), String> {
    let mut config = read_mcp_servers()?;

    let original_len = config.servers.len();
    config.servers.retain(|s| s.id != id);

    if config.servers.len() == original_len {
        return Err(format!("MCP server with id '{}' not found", id));
    }

    write_mcp_servers(&config)?;
    Ok(())
}

#[tauri::command]
pub fn update_mcp_server(id: String, name: Option<String>, config_data: Option<serde_json::Value>) -> Result<MCPServer, String> {
    let mut config = read_mcp_servers()?;

    let server = config.servers.iter_mut()
        .find(|s| s.id == id)
        .ok_or_else(|| format!("MCP server with id '{}' not found", id))?;

    if let Some(new_name) = name {
        server.name = new_name;
    }

    if let Some(new_config) = config_data {
        server.config = new_config;
    }

    let updated_server = server.clone();

    write_mcp_servers(&config)?;

    Ok(updated_server)
}

// ============================================================================
// Settings commands
// ============================================================================

#[tauri::command]
pub fn get_settings() -> Result<ResearchSettings, String> {
    read_settings()
}

#[tauri::command]
pub fn update_settings(settings: ResearchSettings) -> Result<ResearchSettings, String> {
    write_settings(&settings)?;
    Ok(settings)
}

// ============================================================================
// Notification commands
// ============================================================================

/// Request notification permission from the OS
#[tauri::command]
pub async fn request_notification_permission(app: tauri::AppHandle) -> Result<bool, String> {
    use crate::notifications::check_notification_permission;
    tracing::info!("Requesting notification permission...");
    let result = check_notification_permission(&app).await;
    tracing::info!("Notification permission result: {}", result);
    Ok(result)
}

// ============================================================================
// API Key commands - Using file-based storage in ~/.claudius/.env
// ============================================================================

fn get_env_file_path() -> PathBuf {
    get_config_dir().join(".env")
}

/// Read API key from ~/.claudius/.env file
fn read_api_key_from_file() -> Option<String> {
    let env_path = get_env_file_path();

    if !env_path.exists() {
        tracing::debug!("No .env file found at {:?}", env_path);
        return None;
    }

    match std::fs::read_to_string(&env_path) {
        Ok(content) => {
            for line in content.lines() {
                let line = line.trim();
                if line.starts_with("ANTHROPIC_API_KEY=") {
                    let key = line.trim_start_matches("ANTHROPIC_API_KEY=").trim();
                    // Remove quotes if present
                    let key = key.trim_matches('"').trim_matches('\'');
                    if !key.is_empty() {
                        tracing::info!("Successfully read API key from .env file");
                        return Some(key.to_string());
                    }
                }
            }
            tracing::debug!("No ANTHROPIC_API_KEY found in .env file");
            None
        }
        Err(e) => {
            tracing::warn!("Failed to read .env file: {}", e);
            None
        }
    }
}

/// Write API key to ~/.claudius/.env file
fn write_api_key_to_file(api_key: &str) -> Result<(), String> {
    ensure_config_dir()?;
    let env_path = get_env_file_path();

    tracing::info!("Saving API key to {:?}", env_path);

    // Read existing content to preserve other variables
    let mut lines: Vec<String> = Vec::new();
    let mut key_updated = false;

    if env_path.exists() {
        if let Ok(content) = std::fs::read_to_string(&env_path) {
            for line in content.lines() {
                if line.trim().starts_with("ANTHROPIC_API_KEY=") {
                    lines.push(format!("ANTHROPIC_API_KEY={}", api_key));
                    key_updated = true;
                } else {
                    lines.push(line.to_string());
                }
            }
        }
    }

    if !key_updated {
        lines.push(format!("ANTHROPIC_API_KEY={}", api_key));
    }

    let content = lines.join("\n") + "\n";

    std::fs::write(&env_path, content)
        .map_err(|e| format!("Failed to write .env file: {}", e))?;

    // Set restrictive permissions (owner read/write only)
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let permissions = std::fs::Permissions::from_mode(0o600);
        let _ = std::fs::set_permissions(&env_path, permissions);
    }

    tracing::info!("Successfully saved API key to .env file");
    Ok(())
}

/// Delete API key from ~/.claudius/.env file
fn delete_api_key_from_file() -> Result<(), String> {
    let env_path = get_env_file_path();

    if !env_path.exists() {
        return Ok(());
    }

    // Read and filter out the API key line
    if let Ok(content) = std::fs::read_to_string(&env_path) {
        let lines: Vec<&str> = content
            .lines()
            .filter(|line| !line.trim().starts_with("ANTHROPIC_API_KEY="))
            .collect();

        if lines.is_empty() {
            // If no other content, delete the file
            let _ = std::fs::remove_file(&env_path);
        } else {
            let content = lines.join("\n") + "\n";
            std::fs::write(&env_path, content)
                .map_err(|e| format!("Failed to update .env file: {}", e))?;
        }
    }

    Ok(())
}

/// Get the API key for use in research (returns full key, not masked)
pub fn get_api_key_for_research() -> Option<String> {
    read_api_key_from_file()
}

#[tauri::command]
pub fn get_api_key() -> Result<Option<String>, String> {
    // Return masked version with dots for security - typical password field style
    if let Some(key) = read_api_key_from_file() {
        // Show dots representing the key length (capped at 20 for display)
        let dot_count = std::cmp::min(key.len(), 20);
        let masked = "•".repeat(dot_count);
        Ok(Some(masked))
    } else {
        Ok(None)
    }
}

#[tauri::command]
pub fn set_api_key(api_key: String) -> Result<(), String> {
    if api_key.is_empty() {
        return Err("API key cannot be empty".to_string());
    }

    if !api_key.starts_with("sk-ant-") {
        return Err("Invalid API key format. Anthropic API keys start with 'sk-ant-'".to_string());
    }

    write_api_key_to_file(&api_key)
}

#[tauri::command]
pub fn has_api_key() -> Result<bool, String> {
    Ok(read_api_key_from_file().is_some())
}

#[tauri::command]
pub fn clear_api_key() -> Result<(), String> {
    delete_api_key_from_file()
}

// ============================================================================
// Additional briefing commands
// ============================================================================

#[tauri::command]
pub fn get_todays_briefings() -> Result<Vec<Briefing>, String> {
    let conn = db::get_connection()
        .map_err(|e| format!("Database connection failed: {}", e))?;

    // Use date prefix to match both "2025-12-08" and "2025-12-08T10:30:00" formats
    let today_prefix = format!("{}%", Local::now().format("%Y-%m-%d"));

    // Return ALL briefings for today (not just the most recent)
    let mut stmt = conn.prepare(
        "SELECT id, date, title, cards, research_time_ms, model_used, total_tokens
         FROM briefings
         WHERE date LIKE ?1
         ORDER BY id DESC"
    ).map_err(|e| format!("Failed to prepare statement: {}", e))?;

    let briefings = stmt.query_map([&today_prefix], |row| {
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
pub fn get_briefing_by_id(id: String) -> Result<Briefing, String> {
    let id_num: i64 = id.parse()
        .map_err(|_| format!("Invalid briefing ID: {}", id))?;
    get_briefing(id_num)
}

#[tauri::command]
pub fn submit_feedback(feedback: serde_json::Value) -> Result<(), String> {
    let briefing_id = feedback.get("briefing_id")
        .and_then(|v| v.as_str())
        .ok_or("Missing briefing_id")?;

    let briefing_id_num: i64 = briefing_id.parse()
        .map_err(|_| format!("Invalid briefing_id: {}", briefing_id))?;

    let feedback_type = feedback.get("feedback_type")
        .and_then(|v| v.as_str())
        .ok_or("Missing feedback_type")?;

    // Map thumbs_up/thumbs_down to rating
    let rating = match feedback_type {
        "thumbs_up" => 5,
        "thumbs_down" => 1,
        _ => 3,
    };

    let notes = feedback.get("notes")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    add_feedback(briefing_id_num, 0, rating, notes)
}

// Alias for trigger_research to match frontend expectations
#[tauri::command]
pub async fn run_research_now(app: tauri::AppHandle) -> Result<String, String> {
    trigger_research(app).await
}

// ============================================================================
// Window control commands (for popover)
// ============================================================================

#[tauri::command]
pub fn open_main_window(app: tauri::AppHandle) {
    crate::tray::show_main_window(&app);
}

#[tauri::command]
pub fn open_settings_window(app: tauri::AppHandle) {
    crate::tray::show_settings_window(&app);
}

#[tauri::command]
pub fn hide_popover(app: tauri::AppHandle) {
    crate::tray::hide_popover(&app);
}

// ============================================================================
// Research log commands
// ============================================================================

use crate::research_log::{ResearchLogger, ResearchLogRecord};

/// Get recent research logs, optionally filtered by briefing ID.
#[tauri::command]
pub fn get_research_logs(briefing_id: Option<i64>, limit: Option<i64>) -> Result<Vec<ResearchLogRecord>, String> {
    let limit = limit.unwrap_or(100);
    ResearchLogger::get_logs(briefing_id, limit)
}

/// Get errors that require user action (e.g., invalid API key, budget exceeded).
#[tauri::command]
pub fn get_actionable_errors(limit: Option<i64>) -> Result<Vec<ResearchLogRecord>, String> {
    let limit = limit.unwrap_or(10);
    ResearchLogger::get_actionable_errors(limit)
}

// ============================================================================
// Research state control commands (cancellation, reset, status)
// ============================================================================

/// Cancel the currently running research operation.
/// This will set the cancellation token and emit a cancelled event.
#[tauri::command]
pub fn cancel_research(app: tauri::AppHandle) -> Result<(), String> {
    tracing::info!("Cancel research requested");

    // Get current state to include in the event
    let state = research_state::get_state();

    if !state.is_running {
        return Err("No research is currently running".to_string());
    }

    // Set the cancellation token
    research_state::cancel()?;

    // Emit the cancelled event
    let _ = app.emit("research:cancelled", CancelledEvent {
        timestamp: chrono::Utc::now().to_rfc3339(),
        reason: "User cancelled research".to_string(),
        phase: state.current_phase.clone(),
        topics_completed: 0, // We don't track this in the global state
        total_topics: 0,
    });

    tracing::info!("Research cancellation requested successfully");
    Ok(())
}

/// Reset the research state. This is used for recovery when research gets stuck.
/// It will reset the global state and emit a reset event.
#[tauri::command]
pub fn reset_research_state(app: tauri::AppHandle) -> Result<(), String> {
    tracing::info!("Research state reset requested");

    // Reset the global state
    research_state::reset();

    // Emit reset event so frontend can update
    let _ = app.emit("research:reset", serde_json::json!({
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "reason": "Manual reset requested",
    }));

    tracing::info!("Research state reset successfully");
    Ok(())
}

/// Get the current research status.
/// Returns whether research is running, the current phase, and when it started.
#[tauri::command]
pub fn get_research_status() -> Result<serde_json::Value, String> {
    let state = research_state::get_state();

    let started_at = state.started_at.map(|t| {
        chrono::DateTime::<chrono::Utc>::from(t).to_rfc3339()
    });

    Ok(serde_json::json!({
        "is_running": state.is_running,
        "current_phase": state.current_phase,
        "started_at": started_at,
        "is_cancelled": research_state::is_cancelled(),
    }))
}

// ============================================================================
// CLI Installation commands
// ============================================================================

#[derive(Debug, Serialize)]
pub struct CliInstallResult {
    pub success: bool,
    pub message: String,
    pub path: Option<String>,
}

/// Check if CLI is installed and return its path
#[tauri::command]
pub fn get_cli_status() -> Result<serde_json::Value, String> {
    let install_path = PathBuf::from("/usr/local/bin/claudius");
    let is_installed = install_path.exists();

    // Check if it points to our binary
    let is_valid = if is_installed {
        // Read the symlink target to verify it's our binary
        match std::fs::read_link(&install_path) {
            Ok(target) => target.to_string_lossy().contains("Claudius") || target.to_string_lossy().contains("claudius"),
            Err(_) => false, // Could be a regular file, not a symlink
        }
    } else {
        false
    };

    Ok(serde_json::json!({
        "installed": is_installed && is_valid,
        "path": if is_installed { Some(install_path.to_string_lossy().to_string()) } else { None },
    }))
}

/// Install CLI by creating a symlink to /usr/local/bin/claudius
/// This requires admin privileges on macOS, so we use osascript
#[tauri::command]
pub async fn install_cli() -> Result<CliInstallResult, String> {
    #[cfg(target_os = "macos")]
    {
        // Get the path to the CLI binary in the app bundle
        let exe_path = std::env::current_exe()
            .map_err(|e| format!("Failed to get current executable path: {}", e))?;

        // The CLI binary should be next to the main binary in the MacOS folder
        let cli_path = exe_path.parent()
            .ok_or("Failed to get parent directory")?
            .join("claudius");

        // Check if CLI binary exists
        if !cli_path.exists() {
            return Err(format!(
                "CLI binary not found at {:?}. Make sure the app was built with the CLI included.",
                cli_path
            ));
        }

        let target = "/usr/local/bin/claudius";

        // Create /usr/local/bin if it doesn't exist (requires sudo)
        let script = format!(
            r#"do shell script "mkdir -p /usr/local/bin && ln -sf '{}' '{}'" with administrator privileges"#,
            cli_path.display(),
            target
        );

        tracing::info!("Installing CLI from {:?} to {}", cli_path, target);

        let output = std::process::Command::new("osascript")
            .args(["-e", &script])
            .output()
            .map_err(|e| format!("Failed to execute osascript: {}", e))?;

        if output.status.success() {
            tracing::info!("CLI installed successfully to {}", target);
            Ok(CliInstallResult {
                success: true,
                message: format!("CLI installed successfully to {}", target),
                path: Some(target.to_string()),
            })
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            // User cancelled the admin prompt
            if stderr.contains("User canceled") || stderr.contains("-128") {
                return Err("Installation cancelled by user".to_string());
            }
            Err(format!("Failed to install CLI: {}", stderr))
        }
    }

    #[cfg(target_os = "linux")]
    {
        let exe_path = std::env::current_exe()
            .map_err(|e| format!("Failed to get current executable path: {}", e))?;

        let cli_path = exe_path.parent()
            .ok_or("Failed to get parent directory")?
            .join("claudius");

        if !cli_path.exists() {
            return Err(format!(
                "CLI binary not found at {:?}. Make sure the app was built with the CLI included.",
                cli_path
            ));
        }

        let target = "/usr/local/bin/claudius";

        // Try pkexec for graphical sudo prompt on Linux
        let output = std::process::Command::new("pkexec")
            .args(["ln", "-sf", &cli_path.to_string_lossy(), target])
            .output()
            .map_err(|e| format!("Failed to execute pkexec: {}", e))?;

        if output.status.success() {
            Ok(CliInstallResult {
                success: true,
                message: format!("CLI installed successfully to {}", target),
                path: Some(target.to_string()),
            })
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            Err(format!("Failed to install CLI: {}", stderr))
        }
    }

    #[cfg(target_os = "windows")]
    {
        // On Windows, we add the app directory to PATH or create a batch file
        let exe_path = std::env::current_exe()
            .map_err(|e| format!("Failed to get current executable path: {}", e))?;

        let cli_path = exe_path.parent()
            .ok_or("Failed to get parent directory")?
            .join("claudius.exe");

        if !cli_path.exists() {
            return Err(format!(
                "CLI binary not found at {:?}. Make sure the app was built with the CLI included.",
                cli_path
            ));
        }

        // Create a .cmd wrapper in a location that's typically in PATH
        let local_app_data = std::env::var("LOCALAPPDATA")
            .map_err(|_| "Could not find LOCALAPPDATA")?;
        let bin_dir = PathBuf::from(&local_app_data).join("Claudius").join("bin");

        std::fs::create_dir_all(&bin_dir)
            .map_err(|e| format!("Failed to create bin directory: {}", e))?;

        let cmd_path = bin_dir.join("claudius.cmd");
        let cmd_content = format!("@echo off\n\"{}\" %*\n", cli_path.display());

        std::fs::write(&cmd_path, cmd_content)
            .map_err(|e| format!("Failed to write cmd wrapper: {}", e))?;

        Ok(CliInstallResult {
            success: true,
            message: format!(
                "CLI wrapper created at {}. Add {} to your PATH to use 'claudius' command.",
                cmd_path.display(),
                bin_dir.display()
            ),
            path: Some(cmd_path.to_string_lossy().to_string()),
        })
    }

    #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
    {
        Err("CLI installation is not supported on this platform".to_string())
    }
}

/// Uninstall CLI by removing the symlink
#[tauri::command]
pub async fn uninstall_cli() -> Result<CliInstallResult, String> {
    #[cfg(target_os = "macos")]
    {
        let target = "/usr/local/bin/claudius";

        if !PathBuf::from(target).exists() {
            return Ok(CliInstallResult {
                success: true,
                message: "CLI is not installed".to_string(),
                path: None,
            });
        }

        let script = format!(
            r#"do shell script "rm -f '{}'" with administrator privileges"#,
            target
        );

        let output = std::process::Command::new("osascript")
            .args(["-e", &script])
            .output()
            .map_err(|e| format!("Failed to execute osascript: {}", e))?;

        if output.status.success() {
            Ok(CliInstallResult {
                success: true,
                message: "CLI uninstalled successfully".to_string(),
                path: None,
            })
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            if stderr.contains("User canceled") || stderr.contains("-128") {
                return Err("Uninstallation cancelled by user".to_string());
            }
            Err(format!("Failed to uninstall CLI: {}", stderr))
        }
    }

    #[cfg(target_os = "linux")]
    {
        let target = "/usr/local/bin/claudius";

        if !PathBuf::from(target).exists() {
            return Ok(CliInstallResult {
                success: true,
                message: "CLI is not installed".to_string(),
                path: None,
            });
        }

        let output = std::process::Command::new("pkexec")
            .args(["rm", "-f", target])
            .output()
            .map_err(|e| format!("Failed to execute pkexec: {}", e))?;

        if output.status.success() {
            Ok(CliInstallResult {
                success: true,
                message: "CLI uninstalled successfully".to_string(),
                path: None,
            })
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            Err(format!("Failed to uninstall CLI: {}", stderr))
        }
    }

    #[cfg(target_os = "windows")]
    {
        let local_app_data = std::env::var("LOCALAPPDATA")
            .map_err(|_| "Could not find LOCALAPPDATA")?;
        let cmd_path = PathBuf::from(&local_app_data)
            .join("Claudius")
            .join("bin")
            .join("claudius.cmd");

        if cmd_path.exists() {
            std::fs::remove_file(&cmd_path)
                .map_err(|e| format!("Failed to remove cmd wrapper: {}", e))?;
        }

        Ok(CliInstallResult {
            success: true,
            message: "CLI uninstalled successfully".to_string(),
            path: None,
        })
    }

    #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
    {
        Err("CLI uninstallation is not supported on this platform".to_string())
    }
}
