// Claudius Library - Shared code between Tauri app and CLI
//
// This module exports the pure Rust components that can be used
// without Tauri dependencies.

// Core modules (pure Rust, no Tauri dependencies)
pub mod chat;
pub mod config;
pub mod db;
pub mod dedup;
pub mod housekeeping;
pub mod image_gen;
pub mod mcp_client;
pub mod research;
pub mod research_log;
pub mod research_state;

// Re-export key types for convenience
pub use chat::{clear_chat_history, get_chat_history, send_chat_message};
pub use config::{
    delete_api_key, delete_openai_api_key, ensure_config_dir, get_config_dir, has_api_key,
    has_openai_api_key, read_api_key, read_mcp_servers, read_openai_api_key, read_settings,
    validate_api_key, validate_openai_api_key, write_api_key, write_mcp_servers,
    write_openai_api_key, write_settings, Briefing, MCPServer, MCPServersConfig, ResearchSettings,
};
pub use db::{ChatMessage, Topic};
pub use research::{BriefingCard, ResearchAgent, ResearchResult};
pub use research_state::ResearchState;
