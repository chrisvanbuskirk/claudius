// Claudius Library - Shared code between Tauri app and CLI
//
// This module exports the pure Rust components that can be used
// without Tauri dependencies.

// Core modules (pure Rust, no Tauri dependencies)
pub mod config;
pub mod db;
pub mod dedup;
pub mod research;
pub mod research_state;
pub mod mcp_client;
pub mod research_log;
pub mod chat;
pub mod housekeeping;

// Re-export key types for convenience
pub use db::{Topic, ChatMessage};
pub use research::{ResearchAgent, ResearchResult, BriefingCard};
pub use chat::{send_chat_message, get_chat_history, clear_chat_history};
pub use research_state::ResearchState;
pub use config::{
    Briefing, MCPServer, MCPServersConfig, ResearchSettings,
    read_api_key, write_api_key, delete_api_key, has_api_key, validate_api_key,
    read_settings, write_settings,
    read_mcp_servers, write_mcp_servers,
    get_config_dir, ensure_config_dir,
};
