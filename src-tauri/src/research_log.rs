//! Research logging module for Claudius.
//!
//! Provides structured error types, API error parsing, and database logging
//! for research operations.

use crate::db::get_connection;
use serde::{Deserialize, Serialize};
use tracing::{debug, error, info};

// ============================================================================
// Error Types
// ============================================================================

/// Categorized error codes for research operations.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ErrorCode {
    // API errors (user action may be required)
    InvalidApiKey,
    BudgetExceeded,
    RateLimited,
    ApiOverloaded,

    // Tool errors (usually transient)
    ToolExecutionFailed,
    McpConnectionFailed,
    McpToolFailed,

    // Network errors
    NetworkError,
    Timeout,

    // Data errors
    ParseError,
    InvalidResponse,

    // Internal errors
    InternalError,
    Unknown,
}

impl ErrorCode {
    /// Returns true if this error requires user action to resolve.
    pub fn requires_user_action(&self) -> bool {
        matches!(
            self,
            ErrorCode::InvalidApiKey | ErrorCode::BudgetExceeded
        )
    }

    /// Get a user-friendly message for this error code.
    pub fn user_message(&self) -> &'static str {
        match self {
            ErrorCode::InvalidApiKey => {
                "Your API key is invalid or has been revoked. Please check your API key in Settings."
            }
            ErrorCode::BudgetExceeded => {
                "Your Anthropic API budget has been exceeded. Please add credits to your account at console.anthropic.com."
            }
            ErrorCode::RateLimited => {
                "Too many requests. The research will automatically retry. If this persists, try again later."
            }
            ErrorCode::ApiOverloaded => {
                "The Anthropic API is currently overloaded. Research will retry automatically."
            }
            ErrorCode::ToolExecutionFailed => {
                "A research tool failed to execute. Some results may be incomplete."
            }
            ErrorCode::McpConnectionFailed => {
                "Failed to connect to an MCP server. Check your MCP server configuration in Settings."
            }
            ErrorCode::McpToolFailed => {
                "An MCP tool call failed. Some results may be incomplete."
            }
            ErrorCode::NetworkError => {
                "Network error occurred. Please check your internet connection."
            }
            ErrorCode::Timeout => {
                "The request timed out. Please try again."
            }
            ErrorCode::ParseError => {
                "Failed to parse the response. This is usually a temporary issue."
            }
            ErrorCode::InvalidResponse => {
                "Received an invalid response from the API."
            }
            ErrorCode::InternalError => {
                "An internal error occurred. Please try again or report this issue."
            }
            ErrorCode::Unknown => {
                "An unknown error occurred. Please try again."
            }
        }
    }

    /// Convert to string for database storage.
    pub fn as_str(&self) -> &'static str {
        match self {
            ErrorCode::InvalidApiKey => "invalid_api_key",
            ErrorCode::BudgetExceeded => "budget_exceeded",
            ErrorCode::RateLimited => "rate_limited",
            ErrorCode::ApiOverloaded => "api_overloaded",
            ErrorCode::ToolExecutionFailed => "tool_execution_failed",
            ErrorCode::McpConnectionFailed => "mcp_connection_failed",
            ErrorCode::McpToolFailed => "mcp_tool_failed",
            ErrorCode::NetworkError => "network_error",
            ErrorCode::Timeout => "timeout",
            ErrorCode::ParseError => "parse_error",
            ErrorCode::InvalidResponse => "invalid_response",
            ErrorCode::InternalError => "internal_error",
            ErrorCode::Unknown => "unknown",
        }
    }
}

/// A structured research error with user-friendly messaging.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResearchError {
    pub code: ErrorCode,
    pub message: String,
    pub user_message: String,
    pub requires_user_action: bool,
    pub details: Option<String>,
}

impl ResearchError {
    pub fn new(code: ErrorCode, message: impl Into<String>) -> Self {
        let user_message = code.user_message().to_string();
        let requires_user_action = code.requires_user_action();
        Self {
            code,
            message: message.into(),
            user_message,
            requires_user_action,
            details: None,
        }
    }

    pub fn with_details(mut self, details: impl Into<String>) -> Self {
        self.details = Some(details.into());
        self
    }
}

impl std::fmt::Display for ResearchError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for ResearchError {}

// ============================================================================
// API Error Parsing
// ============================================================================

/// Anthropic API error response structure.
#[derive(Debug, Deserialize)]
struct AnthropicApiError {
    #[serde(rename = "type")]
    error_type: Option<String>,
    error: Option<AnthropicErrorDetail>,
}

#[derive(Debug, Deserialize)]
struct AnthropicErrorDetail {
    #[serde(rename = "type")]
    error_type: Option<String>,
    message: Option<String>,
}

/// Parse an HTTP status code and response body into a structured error.
pub fn parse_api_error(status_code: u16, body: &str) -> ResearchError {
    // Try to parse as JSON first
    if let Ok(api_error) = serde_json::from_str::<AnthropicApiError>(body) {
        let error_type = api_error
            .error
            .as_ref()
            .and_then(|e| e.error_type.as_deref())
            .unwrap_or("");

        let message = api_error
            .error
            .as_ref()
            .and_then(|e| e.message.as_deref())
            .unwrap_or(body);

        // Match on error type from API
        let code = match error_type {
            "authentication_error" => ErrorCode::InvalidApiKey,
            "invalid_api_key" => ErrorCode::InvalidApiKey,
            "rate_limit_error" => ErrorCode::RateLimited,
            "overloaded_error" => ErrorCode::ApiOverloaded,
            "invalid_request_error" => {
                // Check for budget-related messages
                if message.to_lowercase().contains("credit")
                    || message.to_lowercase().contains("budget")
                    || message.to_lowercase().contains("billing")
                {
                    ErrorCode::BudgetExceeded
                } else {
                    ErrorCode::InvalidResponse
                }
            }
            _ => match status_code {
                401 => ErrorCode::InvalidApiKey,
                402 => ErrorCode::BudgetExceeded,
                429 => ErrorCode::RateLimited,
                500..=599 => ErrorCode::ApiOverloaded,
                _ => ErrorCode::Unknown,
            },
        };

        return ResearchError::new(code, message).with_details(body.to_string());
    }

    // Fallback to status code-based detection
    let code = match status_code {
        401 => ErrorCode::InvalidApiKey,
        402 => ErrorCode::BudgetExceeded,
        429 => ErrorCode::RateLimited,
        408 => ErrorCode::Timeout,
        500..=599 => ErrorCode::ApiOverloaded,
        _ => ErrorCode::Unknown,
    };

    ResearchError::new(code, format!("HTTP {}: {}", status_code, body))
        .with_details(body.to_string())
}

// ============================================================================
// Log Entry Types
// ============================================================================

/// Type of log entry.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LogType {
    ToolCall,
    ApiRequest,
    McpCall,
    Error,
}

impl LogType {
    pub fn as_str(&self) -> &'static str {
        match self {
            LogType::ToolCall => "tool_call",
            LogType::ApiRequest => "api_request",
            LogType::McpCall => "mcp_call",
            LogType::Error => "error",
        }
    }
}

/// A research log entry to be stored in the database.
#[derive(Debug, Clone)]
pub struct ResearchLogEntry {
    pub briefing_id: Option<i64>,
    pub log_type: LogType,
    pub topic: Option<String>,
    pub tool_name: Option<String>,
    pub input_summary: Option<String>,
    pub output_summary: Option<String>,
    pub duration_ms: Option<i64>,
    pub tokens_used: Option<i64>,
    pub success: bool,
    pub error_code: Option<ErrorCode>,
    pub error_message: Option<String>,
}

impl ResearchLogEntry {
    /// Create a new log entry for a successful operation.
    pub fn success(log_type: LogType) -> Self {
        Self {
            briefing_id: None,
            log_type,
            topic: None,
            tool_name: None,
            input_summary: None,
            output_summary: None,
            duration_ms: None,
            tokens_used: None,
            success: true,
            error_code: None,
            error_message: None,
        }
    }

    /// Create a new log entry for a failed operation.
    pub fn failure(log_type: LogType, error: &ResearchError) -> Self {
        Self {
            briefing_id: None,
            log_type,
            topic: None,
            tool_name: None,
            input_summary: None,
            output_summary: None,
            duration_ms: None,
            tokens_used: None,
            success: false,
            error_code: Some(error.code.clone()),
            error_message: Some(error.message.clone()),
        }
    }

    pub fn with_briefing_id(mut self, id: i64) -> Self {
        self.briefing_id = Some(id);
        self
    }

    pub fn with_topic(mut self, topic: impl Into<String>) -> Self {
        self.topic = Some(topic.into());
        self
    }

    pub fn with_tool(mut self, name: impl Into<String>) -> Self {
        self.tool_name = Some(name.into());
        self
    }

    pub fn with_input(mut self, input: impl Into<String>) -> Self {
        let input = input.into();
        // Truncate to reasonable size
        self.input_summary = Some(truncate_string(&input, 500));
        self
    }

    pub fn with_output(mut self, output: impl Into<String>) -> Self {
        let output = output.into();
        // Truncate to reasonable size
        self.output_summary = Some(truncate_string(&output, 1000));
        self
    }

    pub fn with_duration_ms(mut self, ms: i64) -> Self {
        self.duration_ms = Some(ms);
        self
    }

    pub fn with_tokens(mut self, tokens: i64) -> Self {
        self.tokens_used = Some(tokens);
        self
    }
}

/// Truncate a string to a maximum length, adding "..." if truncated.
fn truncate_string(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len.saturating_sub(3)])
    }
}

// ============================================================================
// Database Operations
// ============================================================================

/// Research logger that writes to the database.
pub struct ResearchLogger;

impl ResearchLogger {
    /// Write a log entry to the database.
    pub fn log(entry: &ResearchLogEntry) -> Result<i64, String> {
        let conn = get_connection().map_err(|e| format!("Failed to open database: {}", e))?;

        let user_action_required = entry
            .error_code
            .as_ref()
            .map(|c| c.requires_user_action())
            .unwrap_or(false);

        conn.execute(
            r#"INSERT INTO research_logs
               (briefing_id, log_type, topic, tool_name, input_summary, output_summary,
                duration_ms, tokens_used, success, error_code, error_message, user_action_required)
               VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)"#,
            rusqlite::params![
                entry.briefing_id,
                entry.log_type.as_str(),
                entry.topic,
                entry.tool_name,
                entry.input_summary,
                entry.output_summary,
                entry.duration_ms,
                entry.tokens_used,
                if entry.success { 1 } else { 0 },
                entry.error_code.as_ref().map(|c| c.as_str()),
                entry.error_message,
                if user_action_required { 1 } else { 0 },
            ],
        )
        .map_err(|e| format!("Failed to insert log: {}", e))?;

        let id = conn.last_insert_rowid();

        if entry.success {
            debug!(
                "Logged {} for {:?}: {}",
                entry.log_type.as_str(),
                entry.topic,
                entry.tool_name.as_deref().unwrap_or("N/A")
            );
        } else {
            info!(
                "Logged error for {:?}: {:?} - {}",
                entry.topic,
                entry.error_code,
                entry.error_message.as_deref().unwrap_or("Unknown")
            );
        }

        Ok(id)
    }

    /// Log a successful tool call.
    pub fn log_tool_call(
        topic: &str,
        tool_name: &str,
        input: &str,
        output: &str,
        duration_ms: i64,
    ) -> Result<i64, String> {
        Self::log(
            &ResearchLogEntry::success(LogType::ToolCall)
                .with_topic(topic)
                .with_tool(tool_name)
                .with_input(input)
                .with_output(output)
                .with_duration_ms(duration_ms),
        )
    }

    /// Log a failed tool call.
    pub fn log_tool_error(
        topic: &str,
        tool_name: &str,
        input: &str,
        error: &ResearchError,
        duration_ms: i64,
    ) -> Result<i64, String> {
        let mut entry = ResearchLogEntry::failure(LogType::ToolCall, error)
            .with_topic(topic)
            .with_tool(tool_name)
            .with_input(input)
            .with_duration_ms(duration_ms);
        entry.error_message = Some(error.message.clone());
        Self::log(&entry)
    }

    /// Log an API request.
    pub fn log_api_request(
        topic: &str,
        tokens: i64,
        duration_ms: i64,
    ) -> Result<i64, String> {
        Self::log(
            &ResearchLogEntry::success(LogType::ApiRequest)
                .with_topic(topic)
                .with_tokens(tokens)
                .with_duration_ms(duration_ms),
        )
    }

    /// Log an API error.
    pub fn log_api_error(topic: &str, error: &ResearchError) -> Result<i64, String> {
        Self::log(
            &ResearchLogEntry::failure(LogType::ApiRequest, error)
                .with_topic(topic),
        )
    }

    /// Log an MCP tool call.
    pub fn log_mcp_call(
        topic: &str,
        server_name: &str,
        tool_name: &str,
        input: &str,
        output: &str,
        duration_ms: i64,
    ) -> Result<i64, String> {
        Self::log(
            &ResearchLogEntry::success(LogType::McpCall)
                .with_topic(topic)
                .with_tool(format!("{}:{}", server_name, tool_name))
                .with_input(input)
                .with_output(output)
                .with_duration_ms(duration_ms),
        )
    }

    /// Log an MCP error.
    pub fn log_mcp_error(
        topic: &str,
        server_name: &str,
        error: &ResearchError,
    ) -> Result<i64, String> {
        Self::log(
            &ResearchLogEntry::failure(LogType::McpCall, error)
                .with_topic(topic)
                .with_tool(server_name),
        )
    }

    /// Get recent logs, optionally filtered by briefing_id.
    pub fn get_logs(
        briefing_id: Option<i64>,
        limit: i64,
    ) -> Result<Vec<ResearchLogRecord>, String> {
        let conn = get_connection().map_err(|e| format!("Failed to open database: {}", e))?;

        let query = if briefing_id.is_some() {
            r#"SELECT id, briefing_id, log_type, topic, tool_name, input_summary, output_summary,
                      duration_ms, tokens_used, success, error_code, error_message,
                      user_action_required, created_at
               FROM research_logs
               WHERE briefing_id = ?1
               ORDER BY created_at DESC
               LIMIT ?2"#
        } else {
            r#"SELECT id, briefing_id, log_type, topic, tool_name, input_summary, output_summary,
                      duration_ms, tokens_used, success, error_code, error_message,
                      user_action_required, created_at
               FROM research_logs
               ORDER BY created_at DESC
               LIMIT ?2"#
        };

        let mut stmt = conn.prepare(query).map_err(|e| format!("Failed to prepare query: {}", e))?;

        let params: Vec<Box<dyn rusqlite::ToSql>> = if let Some(bid) = briefing_id {
            vec![Box::new(bid), Box::new(limit)]
        } else {
            vec![Box::new(Option::<i64>::None), Box::new(limit)]
        };

        let rows = stmt
            .query_map(rusqlite::params_from_iter(params.iter().map(|p| p.as_ref())), |row| {
                Ok(ResearchLogRecord {
                    id: row.get(0)?,
                    briefing_id: row.get(1)?,
                    log_type: row.get(2)?,
                    topic: row.get(3)?,
                    tool_name: row.get(4)?,
                    input_summary: row.get(5)?,
                    output_summary: row.get(6)?,
                    duration_ms: row.get(7)?,
                    tokens_used: row.get(8)?,
                    success: row.get::<_, i32>(9)? == 1,
                    error_code: row.get(10)?,
                    error_message: row.get(11)?,
                    user_action_required: row.get::<_, i32>(12)? == 1,
                    created_at: row.get(13)?,
                })
            })
            .map_err(|e| format!("Failed to query logs: {}", e))?;

        let mut logs = Vec::new();
        for row in rows {
            logs.push(row.map_err(|e| format!("Failed to read log row: {}", e))?);
        }

        Ok(logs)
    }

    /// Get logs that require user action.
    pub fn get_actionable_errors(limit: i64) -> Result<Vec<ResearchLogRecord>, String> {
        let conn = get_connection().map_err(|e| format!("Failed to open database: {}", e))?;

        let mut stmt = conn
            .prepare(
                r#"SELECT id, briefing_id, log_type, topic, tool_name, input_summary, output_summary,
                          duration_ms, tokens_used, success, error_code, error_message,
                          user_action_required, created_at
                   FROM research_logs
                   WHERE user_action_required = 1
                   ORDER BY created_at DESC
                   LIMIT ?1"#,
            )
            .map_err(|e| format!("Failed to prepare query: {}", e))?;

        let rows = stmt
            .query_map([limit], |row| {
                Ok(ResearchLogRecord {
                    id: row.get(0)?,
                    briefing_id: row.get(1)?,
                    log_type: row.get(2)?,
                    topic: row.get(3)?,
                    tool_name: row.get(4)?,
                    input_summary: row.get(5)?,
                    output_summary: row.get(6)?,
                    duration_ms: row.get(7)?,
                    tokens_used: row.get(8)?,
                    success: row.get::<_, i32>(9)? == 1,
                    error_code: row.get(10)?,
                    error_message: row.get(11)?,
                    user_action_required: row.get::<_, i32>(12)? == 1,
                    created_at: row.get(13)?,
                })
            })
            .map_err(|e| format!("Failed to query logs: {}", e))?;

        let mut logs = Vec::new();
        for row in rows {
            logs.push(row.map_err(|e| format!("Failed to read log row: {}", e))?);
        }

        Ok(logs)
    }
}

/// A log record from the database.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResearchLogRecord {
    pub id: i64,
    pub briefing_id: Option<i64>,
    pub log_type: String,
    pub topic: Option<String>,
    pub tool_name: Option<String>,
    pub input_summary: Option<String>,
    pub output_summary: Option<String>,
    pub duration_ms: Option<i64>,
    pub tokens_used: Option<i64>,
    pub success: bool,
    pub error_code: Option<String>,
    pub error_message: Option<String>,
    pub user_action_required: bool,
    pub created_at: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_code_user_action() {
        assert!(ErrorCode::InvalidApiKey.requires_user_action());
        assert!(ErrorCode::BudgetExceeded.requires_user_action());
        assert!(!ErrorCode::RateLimited.requires_user_action());
        assert!(!ErrorCode::ToolExecutionFailed.requires_user_action());
    }

    #[test]
    fn test_parse_api_error_authentication() {
        let body = r#"{"type":"error","error":{"type":"authentication_error","message":"Invalid API key"}}"#;
        let error = parse_api_error(401, body);
        assert_eq!(error.code, ErrorCode::InvalidApiKey);
        assert!(error.requires_user_action);
    }

    #[test]
    fn test_parse_api_error_rate_limit() {
        let body = r#"{"type":"error","error":{"type":"rate_limit_error","message":"Rate limit exceeded"}}"#;
        let error = parse_api_error(429, body);
        assert_eq!(error.code, ErrorCode::RateLimited);
        assert!(!error.requires_user_action);
    }

    #[test]
    fn test_parse_api_error_fallback() {
        let body = "Internal Server Error";
        let error = parse_api_error(500, body);
        assert_eq!(error.code, ErrorCode::ApiOverloaded);
    }

    #[test]
    fn test_truncate_string() {
        assert_eq!(truncate_string("short", 10), "short");
        assert_eq!(truncate_string("this is a long string", 10), "this is...");
    }

    #[test]
    fn test_research_error_display() {
        let error = ResearchError::new(ErrorCode::RateLimited, "Too many requests");
        assert_eq!(format!("{}", error), "Too many requests");
    }
}
