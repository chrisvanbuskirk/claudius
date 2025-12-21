// Configuration helpers - shared between Tauri app and CLI
//
// This module provides pure Rust functions for reading/writing
// configuration files. No Tauri dependencies.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

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
    pub model: String,
    pub research_depth: String,
    pub max_sources_per_topic: i32,
    pub enable_notifications: bool,
    #[serde(default = "default_notification_sound")]
    pub notification_sound: bool,
    #[serde(default)]
    pub enable_web_search: bool,
    #[serde(default)]
    pub retention_days: Option<i32>,  // None = never delete
    #[serde(default)]
    pub condense_briefings: bool,  // Combine all topics into one comprehensive card
    #[serde(default = "default_dedup_days")]
    pub dedup_days: i32,  // Days to look back for duplicates
    #[serde(default = "default_dedup_threshold")]
    pub dedup_threshold: f64,  // Similarity threshold (0.0-1.0)
}

fn default_notification_sound() -> bool {
    true
}

fn default_dedup_days() -> i32 {
    14
}

fn default_dedup_threshold() -> f64 {
    0.75
}

impl Default for ResearchSettings {
    fn default() -> Self {
        Self {
            model: "claude-haiku-4-5-20251001".to_string(),
            research_depth: "medium".to_string(),
            max_sources_per_topic: 10,
            enable_notifications: true,
            notification_sound: true,
            enable_web_search: false,
            retention_days: None,
            condense_briefings: false,
            dedup_days: default_dedup_days(),
            dedup_threshold: default_dedup_threshold(),
        }
    }
}

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

pub fn get_config_dir() -> PathBuf {
    let home = dirs::home_dir().expect("Could not find home directory");
    home.join(".claudius")
}

pub fn ensure_config_dir() -> Result<PathBuf, String> {
    let config_dir = get_config_dir();
    std::fs::create_dir_all(&config_dir)
        .map_err(|e| format!("Failed to create config directory: {}", e))?;
    Ok(config_dir)
}

pub fn get_mcp_servers_path() -> PathBuf {
    get_config_dir().join("mcp-servers.json")
}

pub fn get_preferences_path() -> PathBuf {
    get_config_dir().join("preferences.json")
}

pub fn get_env_file_path() -> PathBuf {
    get_config_dir().join(".env")
}

pub fn get_logs_dir() -> PathBuf {
    get_config_dir().join("logs")
}

// ============================================================================
// MCP Servers
// ============================================================================

pub fn read_mcp_servers() -> Result<MCPServersConfig, String> {
    let path = get_mcp_servers_path();
    if !path.exists() {
        return Ok(MCPServersConfig { servers: vec![] });
    }
    let content = std::fs::read_to_string(&path)
        .map_err(|e| format!("Failed to read MCP servers: {}", e))?;
    serde_json::from_str(&content)
        .map_err(|e| format!("Failed to parse MCP servers: {}", e))
}

pub fn write_mcp_servers(config: &MCPServersConfig) -> Result<(), String> {
    ensure_config_dir()?;
    let path = get_mcp_servers_path();
    let content = serde_json::to_string_pretty(&config)
        .map_err(|e| format!("Failed to serialize MCP servers: {}", e))?;
    std::fs::write(&path, content)
        .map_err(|e| format!("Failed to write MCP servers: {}", e))
}

// ============================================================================
// Research Settings
// ============================================================================

pub fn read_settings() -> Result<ResearchSettings, String> {
    let path = get_preferences_path();
    if !path.exists() {
        return Ok(ResearchSettings::default());
    }
    let content = std::fs::read_to_string(&path)
        .map_err(|e| format!("Failed to read settings: {}", e))?;
    serde_json::from_str(&content)
        .map_err(|e| format!("Failed to parse settings: {}", e))
}

pub fn write_settings(settings: &ResearchSettings) -> Result<(), String> {
    ensure_config_dir()?;
    let path = get_preferences_path();
    let content = serde_json::to_string_pretty(&settings)
        .map_err(|e| format!("Failed to serialize settings: {}", e))?;
    std::fs::write(&path, content)
        .map_err(|e| format!("Failed to write settings: {}", e))
}

// ============================================================================
// API Key
// ============================================================================

pub fn read_api_key() -> Option<String> {
    let env_path = get_env_file_path();

    if !env_path.exists() {
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
                        return Some(key.to_string());
                    }
                }
            }
            None
        }
        Err(_) => None,
    }
}

pub fn write_api_key(api_key: &str) -> Result<(), String> {
    ensure_config_dir()?;
    let env_path = get_env_file_path();

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

    Ok(())
}

pub fn delete_api_key() -> Result<(), String> {
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

pub fn has_api_key() -> bool {
    read_api_key().is_some()
}

pub fn validate_api_key(api_key: &str) -> Result<(), String> {
    if api_key.is_empty() {
        return Err("API key cannot be empty".to_string());
    }

    if !api_key.starts_with("sk-ant-") {
        return Err("Invalid API key format. Anthropic API keys start with 'sk-ant-'".to_string());
    }

    Ok(())
}
