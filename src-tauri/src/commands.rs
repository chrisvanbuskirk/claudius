use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use chrono::{Local, Utc};
use uuid::Uuid;
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopicsConfig {
    pub topics: Vec<Topic>,
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

fn get_topics_path() -> PathBuf {
    get_config_dir().join("interests.json")
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

fn read_topics() -> Result<TopicsConfig, String> {
    let path = get_topics_path();
    if !path.exists() {
        return Ok(TopicsConfig { topics: vec![] });
    }
    let content = std::fs::read_to_string(&path)
        .map_err(|e| format!("Failed to read topics: {}", e))?;
    serde_json::from_str(&content)
        .map_err(|e| format!("Failed to parse topics: {}", e))
}

fn write_topics(config: &TopicsConfig) -> Result<(), String> {
    ensure_config_dir()?;
    let path = get_topics_path();
    let content = serde_json::to_string_pretty(&config)
        .map_err(|e| format!("Failed to serialize topics: {}", e))?;
    std::fs::write(&path, content)
        .map_err(|e| format!("Failed to write topics: {}", e))
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
            model: "claude-haiku-4-5-20241022".to_string(),
            research_depth: "medium".to_string(),
            max_sources_per_topic: 10,
            enable_notifications: true,
            notification_sound: true,
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

    // Get settings
    let settings = read_settings().unwrap_or_else(|_| ResearchSettings {
        schedule_cron: "0 6 * * *".to_string(),
        model: "claude-haiku-4-5-20241022".to_string(),
        research_depth: "medium".to_string(),
        max_sources_per_topic: 10,
        enable_notifications: true,
        notification_sound: true,
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

    // Get enabled topics
    let topics_config = match read_topics() {
        Ok(config) => config,
        Err(e) => {
            if settings.enable_notifications {
                let _ = notify_research_error(&app, &e);
            }
            return Err(e);
        }
    };

    let topics: Vec<String> = topics_config
        .topics
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

    // Create research agent and run research
    let mut agent = ResearchAgent::new(api_key, Some(settings.model));
    let result = match agent.run_research(topics).await {
        Ok(r) => r,
        Err(e) => {
            if settings.enable_notifications {
                let _ = notify_research_error(&app, &e);
            }
            return Err(e);
        }
    };

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
        model: "claude-haiku-4-5-20241022".to_string(),
        research_depth: "medium".to_string(),
        max_sources_per_topic: 10,
        enable_notifications: true,
        notification_sound: true,
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

    // Get enabled topics
    let topics_config = read_topics()?;

    let topics: Vec<String> = topics_config
        .topics
        .iter()
        .filter(|t| t.enabled)
        .map(|t| t.name.clone())
        .collect();

    if topics.is_empty() {
        return Err("No topics configured. Please add topics in Settings.".to_string());
    }

    tracing::info!("Researching {} topics: {:?}", topics.len(), topics);

    // Create research agent and run research
    let mut agent = ResearchAgent::new(api_key, Some(settings.model));
    let result = agent.run_research(topics).await?;

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

fn save_briefing_to_db(briefing: &serde_json::Value) -> Result<(), String> {
    let conn = db::get_connection()
        .map_err(|e| format!("Database connection failed: {}", e))?;

    let title = briefing.get("title")
        .and_then(|v| v.as_str())
        .unwrap_or("Daily Briefing");

    let cards = briefing.get("cards")
        .map(|v| serde_json::to_string(v).unwrap_or_default())
        .unwrap_or_else(|| "[]".to_string());

    let today = Local::now().format("%Y-%m-%d").to_string();

    conn.execute(
        "INSERT INTO briefings (date, title, cards) VALUES (?1, ?2, ?3)",
        rusqlite::params![today, title, cards],
    ).map_err(|e| format!("Failed to insert briefing: {}", e))?;

    Ok(())
}

// ============================================================================
// Topics commands
// ============================================================================

#[tauri::command]
pub fn get_topics() -> Result<Vec<Topic>, String> {
    let config = read_topics()?;
    Ok(config.topics)
}

#[tauri::command]
pub fn add_topic(name: String, description: Option<String>) -> Result<Topic, String> {
    let mut config = read_topics()?;

    // Check if topic already exists
    if config.topics.iter().any(|t| t.name.to_lowercase() == name.to_lowercase()) {
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

    config.topics.push(topic.clone());
    write_topics(&config)?;

    Ok(topic)
}

#[tauri::command]
pub fn update_topic(
    id: String,
    name: Option<String>,
    description: Option<String>,
    enabled: Option<bool>,
) -> Result<Topic, String> {
    let mut config = read_topics()?;

    let topic = config.topics.iter_mut()
        .find(|t| t.id == id)
        .ok_or_else(|| format!("Topic with id '{}' not found", id))?;

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

    let updated_topic = topic.clone();
    write_topics(&config)?;

    Ok(updated_topic)
}

#[tauri::command]
pub fn delete_topic(id: String) -> Result<(), String> {
    let mut config = read_topics()?;

    let original_len = config.topics.len();
    config.topics.retain(|t| t.id != id);

    if config.topics.len() == original_len {
        return Err(format!("Topic with id '{}' not found", id));
    }

    write_topics(&config)?;
    Ok(())
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
// API Key commands - Using OS keychain for secure storage
// ============================================================================

const KEYRING_SERVICE: &str = "claudius";
const KEYRING_USER: &str = "anthropic_api_key";

/// Read API key from OS keychain
fn read_api_key_from_keychain() -> Option<String> {
    let entry = keyring::Entry::new(KEYRING_SERVICE, KEYRING_USER).ok()?;

    match entry.get_password() {
        Ok(key) => {
            tracing::debug!("Successfully read API key from keychain");
            Some(key)
        }
        Err(keyring::Error::NoEntry) => {
            tracing::debug!("No API key found in keychain");
            None
        }
        Err(e) => {
            tracing::warn!("Failed to read API key from keychain: {}", e);
            None
        }
    }
}

/// Write API key to OS keychain
fn write_api_key_to_keychain(api_key: &str) -> Result<(), String> {
    let entry = keyring::Entry::new(KEYRING_SERVICE, KEYRING_USER)
        .map_err(|e| format!("Failed to create keychain entry: {}", e))?;

    entry.set_password(api_key)
        .map_err(|e| format!("Failed to save API key to keychain: {}", e))?;

    tracing::info!("Successfully saved API key to keychain");
    Ok(())
}

/// Delete API key from OS keychain
fn delete_api_key_from_keychain() -> Result<(), String> {
    let entry = keyring::Entry::new(KEYRING_SERVICE, KEYRING_USER)
        .map_err(|e| format!("Failed to access keychain: {}", e))?;

    match entry.delete_credential() {
        Ok(()) => {
            tracing::info!("Successfully deleted API key from keychain");
            Ok(())
        }
        Err(keyring::Error::NoEntry) => {
            // Already deleted, that's fine
            Ok(())
        }
        Err(e) => {
            Err(format!("Failed to delete API key from keychain: {}", e))
        }
    }
}

/// Get the API key for use in research (returns full key, not masked)
pub fn get_api_key_for_research() -> Option<String> {
    read_api_key_from_keychain()
}

#[tauri::command]
pub fn get_api_key() -> Result<Option<String>, String> {
    // Return masked version with dots for security - typical password field style
    if let Some(key) = read_api_key_from_keychain() {
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

    write_api_key_to_keychain(&api_key)
}

#[tauri::command]
pub fn has_api_key() -> Result<bool, String> {
    Ok(read_api_key_from_keychain().is_some())
}

#[tauri::command]
pub fn clear_api_key() -> Result<(), String> {
    delete_api_key_from_keychain()
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

    // Only return the most recent briefing for today
    let mut stmt = conn.prepare(
        "SELECT id, date, title, cards, research_time_ms, model_used, total_tokens
         FROM briefings
         WHERE date LIKE ?1
         ORDER BY id DESC
         LIMIT 1"
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
