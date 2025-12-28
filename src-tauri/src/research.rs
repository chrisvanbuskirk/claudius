//! Research agent for Claudius with tool_use support.
//!
//! Handles calling the Anthropic API to research topics and generate briefings.
//! Supports tool calling for external data sources via MCP servers and built-in tools.
#![allow(dead_code)]

use crate::mcp_client::{load_mcp_servers, McpClient};
use crate::research_log::{parse_api_error, ErrorCode, ResearchError, ResearchLogger};
use crate::research_state;
use chrono::Datelike;
use regex::Regex;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashSet;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tauri::Emitter;
use tracing::{debug, error, info, warn};

/// Maximum number of tool use iterations to prevent infinite loops.
const MAX_TOOL_ITERATIONS: usize = 10;

/// Claude's built-in web search tool type identifier.
/// This version string may change with API updates.
const WEB_SEARCH_TOOL_TYPE: &str = "web_search_20250305";

/// Maximum number of web searches per topic to control costs (~$0.01/search).
const WEB_SEARCH_MAX_USES: u32 = 10;

/// A single briefing card containing research on a topic.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BriefingCard {
    pub title: String,
    pub summary: String,
    pub detailed_content: String, // Full research text (2-3 paragraphs)
    pub sources: Vec<String>,
    pub suggested_next: Option<String>,
    pub relevance: String,
    pub topic: String,
    // Image generation fields (DALL-E)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image_prompt: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image_style: Option<String>, // Legacy field, not used with DALL-E
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image_path: Option<String>,
}

/// Result of a research operation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResearchResult {
    pub date: String,
    pub title: String,
    pub cards: Vec<BriefingCard>,
    pub research_time_ms: u64,
    pub model_used: String,
    pub total_tokens: u32,
}

// ============================================================================
// Research Progress Events for Real-Time Tracking
// ============================================================================

/// Event emitted when research starts
#[derive(Serialize, Clone)]
pub struct ResearchStartedEvent {
    timestamp: String,
    total_topics: usize,
    topics: Vec<String>,
}

/// Event emitted when MCP server connects successfully
#[derive(Serialize, Clone)]
#[allow(dead_code)]
pub struct McpConnectedEvent {
    timestamp: String,
    server_name: String,
    tool_count: usize,
    tools: Vec<String>,
}

/// Event emitted when MCP server fails to connect
#[derive(Serialize, Clone)]
#[allow(dead_code)]
pub struct McpConnectionFailedEvent {
    timestamp: String,
    server_name: String,
    error: String,
}

/// Event emitted when starting research for a topic
#[derive(Serialize, Clone)]
pub struct TopicStartedEvent {
    timestamp: String,
    topic_name: String,
    topic_index: usize,
    total_topics: usize,
}

/// Event emitted when Claude is thinking/reasoning
#[derive(Serialize, Clone)]
#[allow(dead_code)]
pub struct ThinkingEvent {
    timestamp: String,
    topic_name: String,
    phase: String, // "initial_research" | "tool_calling" | "synthesis"
}

/// Event emitted after tool execution
#[derive(Serialize, Clone)]
#[allow(dead_code)]
pub struct ToolExecutedEvent {
    timestamp: String,
    topic_name: String,
    tool_name: String,
    tool_type: String, // "mcp" | "brave_search" | "builtin"
    status: String,    // "success" | "error"
    error: Option<String>,
}

/// Event emitted when topic research completes
#[derive(Serialize, Clone)]
pub struct TopicCompletedEvent {
    timestamp: String,
    topic_name: String,
    topic_index: usize,
    cards_generated: usize,
}

/// Event emitted when saving to database
#[derive(Serialize, Clone)]
#[allow(dead_code)]
pub struct SavingEvent {
    timestamp: String,
    total_cards: usize,
}

/// Event emitted when research completes
#[derive(Serialize, Clone)]
pub struct CompletedEvent {
    timestamp: String,
    total_topics: usize,
    total_cards: usize,
    duration_ms: u128,
    success: bool,
    error: Option<String>,
}

/// Event emitted when synthesis starts
#[derive(Serialize, Clone)]
pub struct SynthesisStartedEvent {
    timestamp: String,
    research_content_length: usize,
}

/// Event emitted when synthesis completes
#[derive(Serialize, Clone)]
pub struct SynthesisCompletedEvent {
    timestamp: String,
    cards_generated: usize,
    duration_ms: u128,
}

/// Event emitted when research is cancelled
#[derive(Serialize, Clone)]
pub struct CancelledEvent {
    pub timestamp: String,
    pub reason: String,
    pub phase: String,
    pub topics_completed: usize,
    pub total_topics: usize,
}

/// Event emitted as a heartbeat during long operations
#[derive(Serialize, Clone)]
pub struct HeartbeatEvent {
    pub timestamp: String,
    pub phase: String,
    pub topic_index: Option<usize>,
    pub message: String,
}

/// Event emitted when Claude uses built-in web search
#[derive(Serialize, Clone)]
pub struct WebSearchEvent {
    pub timestamp: String,
    pub topic_name: String,
    pub search_query: Option<String>,
    pub status: String, // "started" | "completed"
}

/// Helper to get current timestamp in RFC3339 format
fn get_timestamp() -> String {
    chrono::Utc::now().to_rfc3339()
}

// ============================================================================
// Anthropic API Types with Tool Support
// ============================================================================

/// Tool definition for Anthropic API.
#[derive(Debug, Clone, Serialize)]
struct Tool {
    name: String,
    description: String,
    input_schema: serde_json::Value,
}

/// Anthropic API message request with tools.
/// Note: `tools` uses serde_json::Value to support both regular tools and server tools (like web_search)
#[derive(Debug, Serialize)]
struct AnthropicRequest {
    model: String,
    max_tokens: u32,
    messages: Vec<Message>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tools: Option<Vec<serde_json::Value>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    system: Option<String>,
}

/// A message in the conversation.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct Message {
    role: String,
    content: MessageContent,
}

/// Message content can be a string or array of content blocks.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
enum MessageContent {
    Text(String),
    Blocks(Vec<ContentBlock>),
}

/// A content block in a message.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
enum ContentBlock {
    #[serde(rename = "text")]
    Text { text: String },
    #[serde(rename = "tool_use")]
    ToolUse {
        id: String,
        name: String,
        input: serde_json::Value,
    },
    #[serde(rename = "tool_result")]
    ToolResult {
        tool_use_id: String,
        content: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        is_error: Option<bool>,
    },
}

/// Anthropic API response.
#[derive(Debug, Deserialize)]
struct AnthropicResponse {
    content: Vec<ResponseContentBlock>,
    usage: Usage,
    stop_reason: Option<String>,
}

/// Content block in API response (slightly different structure for deserialization).
#[derive(Debug, Deserialize)]
struct ResponseContentBlock {
    #[serde(rename = "type")]
    content_type: String,
    #[serde(default)]
    text: Option<String>,
    #[serde(default)]
    id: Option<String>,
    #[serde(default)]
    name: Option<String>,
    #[serde(default)]
    input: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
struct Usage {
    input_tokens: u32,
    output_tokens: u32,
}

/// Response from Claude for briefing cards.
#[derive(Debug, Deserialize)]
struct BriefingResponse {
    cards: Vec<BriefingCard>,
}

// ============================================================================
// Tool Definitions
// ============================================================================

fn get_research_tools() -> Vec<Tool> {
    vec![
        Tool {
            name: "get_github_activity".to_string(),
            description: "Get recent activity from a GitHub repository including recent commits, PRs, and issues. Use this when researching topics related to open source projects or specific GitHub repositories.".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "owner": {
                        "type": "string",
                        "description": "The GitHub repository owner (username or organization)"
                    },
                    "repo": {
                        "type": "string",
                        "description": "The GitHub repository name"
                    },
                    "activity_type": {
                        "type": "string",
                        "enum": ["commits", "pulls", "issues", "releases"],
                        "description": "Type of activity to fetch"
                    }
                },
                "required": ["owner", "repo", "activity_type"]
            }),
        },
        Tool {
            name: "fetch_webpage".to_string(),
            description: "Fetch and extract text content from a webpage URL. Use this to get current information from news sites, documentation, or other web sources.".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "url": {
                        "type": "string",
                        "description": "The URL of the webpage to fetch"
                    }
                },
                "required": ["url"]
            }),
        },
    ]
}

// ============================================================================
// Tool Execution
// ============================================================================

/// Execute a tool and return the result.
async fn execute_tool(
    client: &Client,
    tool_name: &str,
    input: &serde_json::Value,
    github_token: Option<&str>,
) -> Result<String, String> {
    match tool_name {
        "get_github_activity" => {
            let owner = input
                .get("owner")
                .and_then(|v| v.as_str())
                .ok_or("Missing owner")?;
            let repo = input
                .get("repo")
                .and_then(|v| v.as_str())
                .ok_or("Missing repo")?;
            let activity_type = input
                .get("activity_type")
                .and_then(|v| v.as_str())
                .ok_or("Missing activity_type")?;

            execute_github_activity(client, owner, repo, activity_type, github_token).await
        }
        "fetch_webpage" => {
            let url = input
                .get("url")
                .and_then(|v| v.as_str())
                .ok_or("Missing url")?;
            execute_fetch_webpage(client, url).await
        }
        _ => Err(format!("Unknown tool: {}", tool_name)),
    }
}

/// Fetch GitHub activity (commits, PRs, issues, or releases).
async fn execute_github_activity(
    client: &Client,
    owner: &str,
    repo: &str,
    activity_type: &str,
    github_token: Option<&str>,
) -> Result<String, String> {
    let endpoint = match activity_type {
        "commits" => format!(
            "https://api.github.com/repos/{}/{}/commits?per_page=10",
            owner, repo
        ),
        "pulls" => format!(
            "https://api.github.com/repos/{}/{}/pulls?state=all&per_page=10",
            owner, repo
        ),
        "issues" => format!(
            "https://api.github.com/repos/{}/{}/issues?state=all&per_page=10",
            owner, repo
        ),
        "releases" => format!(
            "https://api.github.com/repos/{}/{}/releases?per_page=5",
            owner, repo
        ),
        _ => return Err(format!("Unknown activity type: {}", activity_type)),
    };

    let mut request = client
        .get(&endpoint)
        .header("User-Agent", "Claudius-Research-Agent")
        .header("Accept", "application/vnd.github.v3+json");

    if let Some(token) = github_token {
        request = request.header("Authorization", format!("Bearer {}", token));
    }

    let response = request
        .send()
        .await
        .map_err(|e| format!("GitHub API request failed: {}", e))?;

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        return Err(format!("GitHub API error {}: {}", status, body));
    }

    let data: serde_json::Value = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse GitHub response: {}", e))?;

    // Format the response based on activity type
    let formatted = match activity_type {
        "commits" => format_github_commits(&data),
        "pulls" => format_github_pulls(&data),
        "issues" => format_github_issues(&data),
        "releases" => format_github_releases(&data),
        _ => data.to_string(),
    };

    Ok(formatted)
}

fn format_github_commits(data: &serde_json::Value) -> String {
    let commits = data.as_array().map(|arr| {
        arr.iter()
            .take(10)
            .filter_map(|c| {
                let sha = c.get("sha")?.as_str()?.get(..7)?;
                let message = c.get("commit")?.get("message")?.as_str()?;
                let author = c.get("commit")?.get("author")?.get("name")?.as_str()?;
                let date = c.get("commit")?.get("author")?.get("date")?.as_str()?;
                Some(format!(
                    "- {} by {} ({}): {}",
                    sha,
                    author,
                    &date[..10],
                    message.lines().next().unwrap_or("")
                ))
            })
            .collect::<Vec<_>>()
            .join("\n")
    });
    commits.unwrap_or_else(|| "No commits found".to_string())
}

fn format_github_pulls(data: &serde_json::Value) -> String {
    let pulls = data.as_array().map(|arr| {
        arr.iter()
            .take(10)
            .filter_map(|p| {
                let number = p.get("number")?.as_i64()?;
                let title = p.get("title")?.as_str()?;
                let state = p.get("state")?.as_str()?;
                let user = p.get("user")?.get("login")?.as_str()?;
                Some(format!("- #{} [{}] by {}: {}", number, state, user, title))
            })
            .collect::<Vec<_>>()
            .join("\n")
    });
    pulls.unwrap_or_else(|| "No pull requests found".to_string())
}

fn format_github_issues(data: &serde_json::Value) -> String {
    let issues = data.as_array().map(|arr| {
        arr.iter()
            .take(10)
            .filter_map(|i| {
                let number = i.get("number")?.as_i64()?;
                let title = i.get("title")?.as_str()?;
                let state = i.get("state")?.as_str()?;
                let user = i.get("user")?.get("login")?.as_str()?;
                Some(format!("- #{} [{}] by {}: {}", number, state, user, title))
            })
            .collect::<Vec<_>>()
            .join("\n")
    });
    issues.unwrap_or_else(|| "No issues found".to_string())
}

fn format_github_releases(data: &serde_json::Value) -> String {
    let releases = data.as_array().map(|arr| {
        arr.iter()
            .take(5)
            .filter_map(|r| {
                let tag = r.get("tag_name")?.as_str()?;
                let name = r.get("name")?.as_str().unwrap_or(tag);
                let date = r.get("published_at")?.as_str()?;
                let prerelease = r.get("prerelease")?.as_bool().unwrap_or(false);
                let suffix = if prerelease { " (prerelease)" } else { "" };
                Some(format!("- {} - {}{} ({})", tag, name, suffix, &date[..10]))
            })
            .collect::<Vec<_>>()
            .join("\n")
    });
    releases.unwrap_or_else(|| "No releases found".to_string())
}

/// Fetch and extract text content from a webpage.
async fn execute_fetch_webpage(client: &Client, url: &str) -> Result<String, String> {
    // Validate URL
    if !url.starts_with("http://") && !url.starts_with("https://") {
        return Err("URL must start with http:// or https://".to_string());
    }

    let response = client
        .get(url)
        .header("User-Agent", "Claudius-Research-Agent")
        .send()
        .await
        .map_err(|e| format!("Failed to fetch URL: {}", e))?;

    if !response.status().is_success() {
        return Err(format!("HTTP error: {}", response.status()));
    }

    let html = response
        .text()
        .await
        .map_err(|e| format!("Failed to read response: {}", e))?;

    // Extract text content from HTML (simple extraction)
    let text = extract_text_from_html(&html);

    // Truncate if too long (use character count, not byte index to avoid UTF-8 panic)
    let max_chars = 8000;
    let char_count = text.chars().count();
    if char_count > max_chars {
        let truncated: String = text.chars().take(max_chars).collect();
        Ok(format!(
            "{}...\n\n[Content truncated, {} total characters]",
            truncated, char_count
        ))
    } else {
        Ok(text)
    }
}

/// Simple HTML text extraction (removes tags, scripts, styles).
fn extract_text_from_html(html: &str) -> String {
    // Remove script and style tags with content
    let without_scripts = Regex::new(r"(?is)<script[^>]*>.*?</script>")
        .ok()
        .map(|re| re.replace_all(html, "").to_string())
        .unwrap_or_else(|| html.to_string());

    let without_styles = Regex::new(r"(?is)<style[^>]*>.*?</style>")
        .ok()
        .map(|re| re.replace_all(&without_scripts, "").to_string())
        .unwrap_or_else(|| without_scripts.clone());

    // Remove HTML tags
    let without_tags = Regex::new(r"<[^>]+>")
        .ok()
        .map(|re| re.replace_all(&without_styles, " ").to_string())
        .unwrap_or_else(|| without_styles.clone());

    // Decode common HTML entities
    let decoded = without_tags
        .replace("&nbsp;", " ")
        .replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#39;", "'");

    // Clean up whitespace
    Regex::new(r"\s+")
        .ok()
        .map(|re| re.replace_all(&decoded, " ").trim().to_string())
        .unwrap_or_else(|| decoded.trim().to_string())
}

// ============================================================================
// Research Agent
// ============================================================================

/// Research agent that calls Anthropic API with tool support.
pub struct ResearchAgent {
    client: Client,
    api_key: String,
    model: String,
    github_token: Option<String>,
    mcp_client: Option<McpClient>,
    /// Names of built-in tools (to differentiate from MCP tools)
    builtin_tools: HashSet<String>,
    /// Cancellation token for aborting research
    cancellation_token: Option<Arc<AtomicBool>>,
    /// Enable Claude's built-in web search ($0.01/search)
    enable_web_search: bool,
    /// Research mode: "standard" or "firecrawl"
    research_mode: String,
}

impl ResearchAgent {
    /// Create a new research agent.
    pub fn new(
        api_key: String,
        model: Option<String>,
        enable_web_search: bool,
        research_mode: String,
    ) -> Self {
        // Try to read GitHub token from environment or config
        let github_token = std::env::var("GITHUB_TOKEN").ok().or_else(|| {
            // Try to read from ~/.claudius/.env
            let home = dirs::home_dir()?;
            let env_path = home.join(".claudius").join(".env");
            let content = std::fs::read_to_string(env_path).ok()?;
            content
                .lines()
                .find(|line| line.starts_with("GITHUB_TOKEN="))
                .map(|line| line.trim_start_matches("GITHUB_TOKEN=").trim().to_string())
        });

        // Track built-in tool names
        let builtin_tools: HashSet<String> = get_research_tools()
            .iter()
            .map(|t| t.name.clone())
            .collect();

        if enable_web_search {
            tracing::info!(
                "Web search enabled - Claude will use built-in web search ($0.01/search)"
            );
        }

        Self {
            client: Client::builder()
                .timeout(Duration::from_secs(180)) // 3 min total request timeout
                .connect_timeout(Duration::from_secs(30)) // 30s to establish connection
                .pool_idle_timeout(Duration::from_secs(60)) // Close idle connections after 60s
                .build()
                .expect("Failed to build HTTP client"),
            api_key,
            model: model.unwrap_or_else(|| "claude-haiku-4-5-20251001".to_string()),
            github_token,
            mcp_client: None,
            builtin_tools,
            cancellation_token: None,
            enable_web_search,
            research_mode,
        }
    }

    /// Set the cancellation token for this agent
    pub fn set_cancellation_token(&mut self, token: Arc<AtomicBool>) {
        self.cancellation_token = Some(token);
    }

    /// Check if cancellation has been requested
    fn check_cancellation(&self) -> Result<(), String> {
        if let Some(ref token) = self.cancellation_token {
            if token.load(Ordering::Relaxed) {
                return Err("Research cancelled by user".to_string());
            }
        }
        Ok(())
    }

    /// Check cancellation and emit cancelled event if cancelled
    fn check_cancellation_with_event(
        &self,
        app_handle: Option<&tauri::AppHandle>,
        phase: &str,
        topics_completed: usize,
        total_topics: usize,
    ) -> Result<(), String> {
        if let Some(ref token) = self.cancellation_token {
            if token.load(Ordering::Relaxed) {
                // Emit cancelled event
                if let Some(app) = app_handle {
                    let _ = app.emit(
                        "research:cancelled",
                        CancelledEvent {
                            timestamp: get_timestamp(),
                            reason: "User cancelled research".to_string(),
                            phase: phase.to_string(),
                            topics_completed,
                            total_topics,
                        },
                    );
                }
                return Err("Research cancelled by user".to_string());
            }
        }
        Ok(())
    }

    /// Initialize MCP connections to configured servers.
    pub async fn init_mcp(&mut self) -> Result<(), String> {
        match load_mcp_servers() {
            Ok(servers) => {
                let enabled_count = servers.iter().filter(|s| s.enabled).count();
                if enabled_count == 0 {
                    info!("No enabled MCP servers configured");
                    return Ok(());
                }

                info!("Connecting to {} enabled MCP servers...", enabled_count);
                match McpClient::connect(servers).await {
                    Ok(client) => {
                        info!(
                            "MCP connected: {} servers, {} tools available",
                            client.server_count(),
                            client.tool_count()
                        );
                        self.mcp_client = Some(client);
                        Ok(())
                    }
                    Err(e) => {
                        warn!("Failed to connect to MCP servers: {}", e);
                        // Don't fail research, just continue without MCP
                        Ok(())
                    }
                }
            }
            Err(e) => {
                warn!("Failed to load MCP server config: {}", e);
                // Don't fail research, just continue without MCP
                Ok(())
            }
        }
    }

    /// Get all available tools (built-in + MCP), filtered by research_mode.
    fn get_all_tools(&self) -> Vec<Tool> {
        let mut tools = Vec::new();

        // Firecrawl tool names to filter
        let firecrawl_tools = [
            "firecrawl_search",
            "firecrawl_scrape",
            "firecrawl_extract",
            "firecrawl_map",
            "firecrawl_crawl",
        ];
        // Standard search tools to exclude in firecrawl mode
        let standard_search_tools = [
            "brave_search",
            "brave_web_search",
            "perplexity_ask",
            "fetch_webpage",
        ];

        // Add built-in tools (filtered by mode)
        for tool in get_research_tools() {
            // In firecrawl mode, exclude the built-in fetch_webpage
            if self.research_mode == "firecrawl"
                && standard_search_tools.contains(&tool.name.as_str())
            {
                tracing::debug!("Excluding built-in tool '{}' in firecrawl mode", tool.name);
                continue;
            }
            tools.push(tool);
        }

        // Add MCP tools (filtered by mode)
        if let Some(ref mcp_client) = self.mcp_client {
            for mcp_tool in mcp_client.get_all_tools() {
                let tool_name = &mcp_tool.tool.name;

                // Filter based on research mode
                let dominated_by_firecrawl =
                    firecrawl_tools.iter().any(|ft| tool_name.contains(ft));
                let is_standard_search = standard_search_tools
                    .iter()
                    .any(|st| tool_name.contains(st));

                if self.research_mode == "firecrawl" {
                    // In firecrawl mode, exclude standard search tools
                    if is_standard_search {
                        tracing::debug!("Excluding tool '{}' in firecrawl mode", tool_name);
                        continue;
                    }
                } else {
                    // In standard mode, exclude firecrawl tools
                    if dominated_by_firecrawl {
                        tracing::debug!("Excluding tool '{}' in standard mode", tool_name);
                        continue;
                    }
                }

                tools.push(Tool {
                    name: mcp_tool.tool.name.clone(),
                    description: mcp_tool.tool.description.clone().unwrap_or_else(|| {
                        format!("Tool from {} MCP server", mcp_tool.server_name)
                    }),
                    input_schema: mcp_tool.tool.input_schema.clone(),
                });
            }
        }

        tracing::info!(
            "Research mode '{}': {} tools available",
            self.research_mode,
            tools.len()
        );
        tools
    }

    /// Get all tools as JSON values for API request, including web_search if enabled.
    fn get_tools_json(&self) -> Vec<serde_json::Value> {
        let tools = self.get_all_tools();
        let mut tools_json: Vec<serde_json::Value> = tools
            .iter()
            .map(|t| {
                serde_json::json!({
                    "name": t.name,
                    "description": t.description,
                    "input_schema": t.input_schema
                })
            })
            .collect();

        // Add Claude's built-in web search tool if enabled
        if self.enable_web_search {
            tools_json.push(serde_json::json!({
                "type": WEB_SEARCH_TOOL_TYPE,
                "name": "web_search",
                "max_uses": WEB_SEARCH_MAX_USES
            }));
            tracing::debug!("Added web_search tool to request");
        }

        tools_json
    }

    /// Check if a tool is a built-in tool.
    fn is_builtin_tool(&self, name: &str) -> bool {
        self.builtin_tools.contains(name)
    }

    /// Run research on the given topics and generate a briefing.
    pub async fn run_research(
        &mut self,
        topics: Vec<String>,
        app_handle: Option<tauri::AppHandle>,
        condense_briefings: bool,
        past_cards_context: Option<String>,
    ) -> Result<ResearchResult, String> {
        let start_time = Instant::now();
        info!("Starting research on {} topics", topics.len());

        if topics.is_empty() {
            return Err("No topics provided for research".to_string());
        }

        // Emit research:started event and update phase
        research_state::set_phase("Starting research...");

        // Debug logging to file
        let log_path = dirs::home_dir()
            .unwrap()
            .join(".claudius")
            .join("research-debug.log");
        let _ = std::fs::write(
            &log_path,
            format!("{}: RESEARCH STARTED\n", chrono::Local::now()),
        );

        if let Some(app) = &app_handle {
            let _ = std::fs::OpenOptions::new()
                .append(true)
                .open(&log_path)
                .and_then(|mut f| {
                    std::io::Write::write_all(
                        &mut f,
                        format!(
                            "{}: Emitting research:started event\n",
                            chrono::Local::now()
                        )
                        .as_bytes(),
                    )
                });
            let _ = app.emit(
                "research:started",
                ResearchStartedEvent {
                    timestamp: get_timestamp(),
                    total_topics: topics.len(),
                    topics: topics.clone(),
                },
            );
        }

        // Initialize MCP connections in a separate thread to avoid blocking Tauri's async runtime.
        // The MCP client uses blocking I/O (std::io::BufReader::read_line) which would block
        // the entire async runtime if run directly. Using std::thread::spawn ensures the blocking
        // I/O runs on a completely separate OS thread.
        let _ = std::fs::OpenOptions::new()
            .append(true)
            .open(&log_path)
            .and_then(|mut f| {
                std::io::Write::write_all(
                    &mut f,
                    format!(
                        "{}: STARTING MCP INIT (std::thread)\n",
                        chrono::Local::now()
                    )
                    .as_bytes(),
                )
            });

        // Use a oneshot channel to get the result from the thread
        let (tx, rx) = tokio::sync::oneshot::channel();
        let log_path_clone = log_path.clone();

        std::thread::spawn(move || {
            let _ = std::fs::OpenOptions::new()
                .append(true)
                .open(&log_path_clone)
                .and_then(|mut f| {
                    std::io::Write::write_all(
                        &mut f,
                        format!("{}: MCP thread started\n", chrono::Local::now()).as_bytes(),
                    )
                });

            // Create a new tokio runtime for this thread
            let rt = match tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
            {
                Ok(rt) => rt,
                Err(e) => {
                    let _ = tx.send(Err(format!("Failed to create runtime: {}", e)));
                    return;
                }
            };

            let result = rt.block_on(async {
                let _ = std::fs::OpenOptions::new()
                    .append(true)
                    .open(&log_path_clone)
                    .and_then(|mut f| {
                        std::io::Write::write_all(
                            &mut f,
                            format!("{}: Loading MCP servers config\n", chrono::Local::now())
                                .as_bytes(),
                        )
                    });

                match load_mcp_servers() {
                    Ok(servers) => {
                        let enabled_count = servers.iter().filter(|s| s.enabled).count();
                        let _ = std::fs::OpenOptions::new()
                            .append(true)
                            .open(&log_path_clone)
                            .and_then(|mut f| {
                                std::io::Write::write_all(
                                    &mut f,
                                    format!(
                                        "{}: Found {} enabled MCP servers\n",
                                        chrono::Local::now(),
                                        enabled_count
                                    )
                                    .as_bytes(),
                                )
                            });

                        if enabled_count == 0 {
                            return Ok(None);
                        }

                        let _ = std::fs::OpenOptions::new()
                            .append(true)
                            .open(&log_path_clone)
                            .and_then(|mut f| {
                                std::io::Write::write_all(
                                    &mut f,
                                    format!(
                                        "{}: Connecting to MCP servers...\n",
                                        chrono::Local::now()
                                    )
                                    .as_bytes(),
                                )
                            });

                        match McpClient::connect(servers).await {
                            Ok(client) => {
                                let _ = std::fs::OpenOptions::new()
                                    .append(true)
                                    .open(&log_path_clone)
                                    .and_then(|mut f| {
                                        std::io::Write::write_all(
                                            &mut f,
                                            format!(
                                                "{}: MCP connect returned {} tools\n",
                                                chrono::Local::now(),
                                                client.tool_count()
                                            )
                                            .as_bytes(),
                                        )
                                    });
                                Ok(Some(client))
                            }
                            Err(e) => {
                                let _ = std::fs::OpenOptions::new()
                                    .append(true)
                                    .open(&log_path_clone)
                                    .and_then(|mut f| {
                                        std::io::Write::write_all(
                                            &mut f,
                                            format!(
                                                "{}: MCP connect error: {}\n",
                                                chrono::Local::now(),
                                                e
                                            )
                                            .as_bytes(),
                                        )
                                    });
                                Err(e)
                            }
                        }
                    }
                    Err(e) => {
                        let _ = std::fs::OpenOptions::new()
                            .append(true)
                            .open(&log_path_clone)
                            .and_then(|mut f| {
                                std::io::Write::write_all(
                                    &mut f,
                                    format!(
                                        "{}: Failed to load MCP config: {}\n",
                                        chrono::Local::now(),
                                        e
                                    )
                                    .as_bytes(),
                                )
                            });
                        Err(e)
                    }
                }
            });

            let _ = tx.send(result);
        });

        // Wait for the MCP init thread with a timeout
        let mcp_result = match tokio::time::timeout(Duration::from_secs(120), rx).await {
            Ok(Ok(result)) => result,
            Ok(Err(_)) => Err("MCP init channel closed unexpectedly".to_string()),
            Err(_) => Err("MCP initialization timed out after 120 seconds".to_string()),
        };

        match mcp_result {
            Ok(Some(client)) => {
                info!(
                    "MCP connected: {} servers, {} tools",
                    client.server_count(),
                    client.tool_count()
                );
                let _ = std::fs::OpenOptions::new()
                    .append(true)
                    .open(&log_path)
                    .and_then(|mut f| {
                        std::io::Write::write_all(
                            &mut f,
                            format!(
                                "{}: MCP INIT SUCCESS - {} tools\n",
                                chrono::Local::now(),
                                client.tool_count()
                            )
                            .as_bytes(),
                        )
                    });
                self.mcp_client = Some(client);
            }
            Ok(None) => {
                info!("No MCP servers enabled");
                let _ = std::fs::OpenOptions::new()
                    .append(true)
                    .open(&log_path)
                    .and_then(|mut f| {
                        std::io::Write::write_all(
                            &mut f,
                            format!("{}: NO MCP SERVERS ENABLED\n", chrono::Local::now())
                                .as_bytes(),
                        )
                    });
            }
            Err(e) => {
                warn!("MCP initialization failed: {}", e);
                let _ = std::fs::OpenOptions::new()
                    .append(true)
                    .open(&log_path)
                    .and_then(|mut f| {
                        std::io::Write::write_all(
                            &mut f,
                            format!("{}: MCP INIT FAILED: {}\n", chrono::Local::now(), e)
                                .as_bytes(),
                        )
                    });
            }
        }

        let _ = std::fs::OpenOptions::new()
            .append(true)
            .open(&log_path)
            .and_then(|mut f| {
                std::io::Write::write_all(
                    &mut f,
                    format!("{}: MCP INIT COMPLETE\n", chrono::Local::now()).as_bytes(),
                )
            });

        // Validate Firecrawl mode - fail early if Firecrawl MCP is not configured
        if self.research_mode == "firecrawl" {
            let has_firecrawl = self
                .mcp_client
                .as_ref()
                .map(|client| {
                    client
                        .get_all_tools()
                        .iter()
                        .any(|t| t.tool.name.contains("firecrawl"))
                })
                .unwrap_or(false);

            if !has_firecrawl {
                return Err(
                    "Deep Research mode requires Firecrawl MCP server to be configured. \
                     Please add Firecrawl in Settings → MCP Servers, or switch to Standard mode."
                        .to_string(),
                );
            }
        }

        // Step 1: Research each topic with tool support
        let mut research_content = String::new();
        let mut total_tokens: u32 = 0;
        let mut topic_stats: Vec<(String, usize)> = Vec::new(); // Track (topic_name, cards_generated)

        let mut topics_completed_count = 0;
        for (i, topic) in topics.iter().enumerate() {
            // Check for cancellation before each topic
            self.check_cancellation_with_event(
                app_handle.as_ref(),
                "researching",
                topics_completed_count,
                topics.len(),
            )?;

            info!("Researching topic {}/{}: {}", i + 1, topics.len(), topic);

            // Update phase and emit research:topic_started event
            research_state::set_phase(&format!(
                "Researching topic {}/{}: {}",
                i + 1,
                topics.len(),
                topic
            ));
            if let Some(app) = &app_handle {
                let _ = app.emit(
                    "research:topic_started",
                    TopicStartedEvent {
                        timestamp: get_timestamp(),
                        topic_name: topic.clone(),
                        topic_index: i,
                        total_topics: topics.len(),
                    },
                );
            }

            match self
                .research_topic_with_tools(topic, app_handle.as_ref(), i)
                .await
            {
                Ok((content, tokens)) => {
                    research_content.push_str(&format!(
                        "\n## Topic {}: {}\n{}\n",
                        i + 1,
                        topic,
                        content
                    ));
                    total_tokens += tokens;
                    topic_stats.push((topic.clone(), 0)); // Will be updated after synthesis
                }
                Err(e) => {
                    error!("Error researching topic '{}': {}", topic, e);
                    research_content.push_str(&format!(
                        "\n## Topic {}: {}\nError: Could not research this topic.\n",
                        i + 1,
                        topic
                    ));
                    topic_stats.push((topic.clone(), 0));
                }
            }

            // Emit research:topic_completed event
            if let Some(app) = &app_handle {
                let _ = app.emit(
                    "research:topic_completed",
                    TopicCompletedEvent {
                        timestamp: get_timestamp(),
                        topic_name: topic.clone(),
                        topic_index: i,
                        cards_generated: 0, // Will be known after synthesis
                    },
                );
            }

            topics_completed_count += 1;
        }

        // Check for cancellation before synthesis
        self.check_cancellation_with_event(
            app_handle.as_ref(),
            "synthesizing",
            topics_completed_count,
            topics.len(),
        )?;

        // Step 2: Synthesize into briefing cards
        info!(
            "Synthesizing research into briefing cards (condensed: {})",
            condense_briefings
        );
        let (cards, synthesis_tokens) = self
            .synthesize_briefing(
                &research_content,
                app_handle.as_ref(),
                condense_briefings,
                past_cards_context.as_deref(),
            )
            .await
            .map_err(|e| {
                let _ = ResearchLogger::log_api_error("synthesis", &e);
                e.message
            })?;
        total_tokens += synthesis_tokens;

        let research_time_ms = start_time.elapsed().as_millis() as u64;

        let result = ResearchResult {
            date: chrono::Local::now().format("%Y-%m-%dT%H:%M:%S").to_string(),
            title: format!(
                "Daily Briefing - {}",
                chrono::Local::now().format("%B %d, %Y")
            ),
            cards,
            research_time_ms,
            model_used: self.model.clone(),
            total_tokens,
        };

        info!(
            "Research complete: {} cards, {}ms, {} tokens",
            result.cards.len(),
            result.research_time_ms,
            result.total_tokens
        );

        // Note: research:completed event is emitted by the caller (commands.rs)
        // AFTER saving to database and generating images, to avoid premature UI refresh.
        // Only update the phase here for CLI progress display.
        research_state::set_phase(&format!(
            "Research complete: {} cards in {:.1}s",
            result.cards.len(),
            result.research_time_ms as f64 / 1000.0
        ));

        Ok(result)
    }

    /// Research a single topic using Claude with tool support.
    async fn research_topic_with_tools(
        &mut self,
        topic: &str,
        app_handle: Option<&tauri::AppHandle>,
        topic_index: usize,
    ) -> Result<(String, u32), String> {
        // Build dynamic system prompt based on available tools
        let tools = self.get_all_tools();
        let tool_descriptions: Vec<String> = tools
            .iter()
            .map(|t| format!("- {}: {}", t.name, t.description))
            .collect();

        // Get current date components for research context
        let now = chrono::Local::now();
        let current_date = now.format("%B %d, %Y").to_string();
        let _current_month = now.format("%B").to_string();
        let current_year = now.format("%Y").to_string();
        let prev_year = (now.year() - 1).to_string();
        let month_year = now.format("%B %Y").to_string();

        // Build mode-specific tool usage instructions
        let tool_usage_instructions = if self.research_mode == "firecrawl" {
            format!(
                r#"CRITICAL SEARCH TOOL USAGE (Firecrawl Deep Research Mode):
- For COMPLEX topics requiring multi-page research, USE firecrawl_agent - it autonomously researches across multiple sources and synthesizes findings. This is your most powerful tool for in-depth research.
- For quick searches, use firecrawl_search to find {} articles - it searches AND extracts content in one call
- Use specific search queries like "[topic] {}" or "[topic] {} latest news"
- firecrawl_search returns full page content, not just URLs - analyze the results directly
- Use firecrawl_scrape to get full content from specific URLs you want to analyze deeply
- Use firecrawl_extract for structured data extraction with custom prompts (great for extracting specific facts)
- Use firecrawl_map to discover related pages on a website
- Use get_github_activity for open source projects to see recent commits, PRs, and releases from {}

IMPORTANT: firecrawl_agent is ideal for comprehensive research on complex topics - it will autonomously explore multiple sources and provide synthesized findings. Use it when depth matters.

Firecrawl tools handle JavaScript-heavy sites and provide clean markdown content. Use them aggressively for comprehensive research."#,
                month_year,
                month_year,
                current_year,
                month_year
            )
        } else {
            format!(
                r#"CRITICAL SEARCH TOOL USAGE:
- If you have access to brave_search or perplexity search tools, USE THEM FIRST to find {} articles and information
- Use specific search queries like "[topic] {}" or "[topic] {} latest news"
- Search tools will give you current URLs and content - these are your primary source for {} information
- After getting search results, use fetch_webpage to read the most promising URLs in full
- Use get_github_activity for open source projects to see recent commits, PRs, and releases from {}

When using fetch_webpage directly (without search):
- Target URLs likely to have {} content: TechCrunch, The Verge, Hacker News, company blogs, official documentation
- Prioritize URLs with "/{}" or "{}" in the path"#,
                month_year,
                month_year,
                current_year,
                month_year,
                month_year,
                month_year,
                current_year.to_lowercase(),
                month_year.to_lowercase().replace(" ", "-")
            )
        };

        let system_prompt = format!(
            r#"You are a research assistant gathering information on topics of interest.

IMPORTANT: Today's date is {}. You must focus on finding information from {} and late {}. Any information from {} or earlier is outdated and should be avoided unless it provides essential background context.

You have access to the following tools to fetch real-time data:
{}

{}

After gathering current information, provide a comprehensive research summary based on {} data."#,
            current_date,
            month_year,
            current_year,
            prev_year,
            tool_descriptions.join("\n"),
            tool_usage_instructions,
            month_year
        );

        let user_prompt = format!(
            r#"Research the following topic and provide:
1. Key recent developments from {} (ideally within the last 24-48 hours, or at minimum from late {})
2. Why this might be relevant to someone interested in this topic
3. Actionable insights or next steps
4. Credible sources with dates (MUST be from {}, preferably {})

Topic: {}

CRITICAL: Use the available tools aggressively to fetch current {} information. Do NOT rely solely on your training data, as it may be outdated. If you can't find {} information after trying multiple sources, explicitly state this limitation.

Provide a concise but informative research summary (2-3 paragraphs) based on current {} data."#,
            month_year,
            current_year,
            current_year,
            month_year,
            topic,
            month_year,
            month_year,
            month_year
        );
        let mut messages = vec![Message {
            role: "user".to_string(),
            content: MessageContent::Text(user_prompt),
        }];

        let mut total_tokens: u32 = 0;
        let mut iterations = 0;
        let mut last_heartbeat = Instant::now();
        const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(30);

        // Agentic loop - keep going until Claude stops calling tools
        loop {
            // Check for cancellation at each iteration
            self.check_cancellation()?;

            // Emit heartbeat if enough time has passed
            if last_heartbeat.elapsed() >= HEARTBEAT_INTERVAL {
                if let Some(app) = app_handle {
                    let _ = app.emit(
                        "research:heartbeat",
                        HeartbeatEvent {
                            timestamp: get_timestamp(),
                            phase: "researching".to_string(),
                            topic_index: Some(topic_index),
                            message: format!(
                                "Still researching '{}' (iteration {})",
                                topic, iterations
                            ),
                        },
                    );
                }
                last_heartbeat = Instant::now();
            }

            iterations += 1;
            if iterations > MAX_TOOL_ITERATIONS {
                warn!(
                    "Reached max tool iterations ({}), stopping",
                    MAX_TOOL_ITERATIONS
                );
                break;
            }

            let request = AnthropicRequest {
                model: self.model.clone(),
                max_tokens: 2048,
                messages: messages.clone(),
                tools: Some(self.get_tools_json()),
                system: Some(system_prompt.to_string()),
            };

            info!(
                "Calling Claude API (iteration {}/{}) for topic: {}",
                iterations, MAX_TOOL_ITERATIONS, topic
            );
            let api_start = Instant::now();
            let response = match self.send_request(&request).await {
                Ok(r) => r,
                Err(e) => {
                    // Log the API error
                    let _ = ResearchLogger::log_api_error(topic, &e);
                    return Err(e.message);
                }
            };
            let api_duration = api_start.elapsed().as_millis() as i64;
            let tokens = response.usage.input_tokens + response.usage.output_tokens;
            total_tokens += tokens;

            info!(
                "Claude API responded in {}ms ({} tokens, stop_reason: {:?})",
                api_duration, tokens, response.stop_reason
            );

            // Log successful API request
            let _ = ResearchLogger::log_api_request(topic, tokens as i64, api_duration);

            // Check for web_search usage in response (server_tool_use blocks)
            // Claude's built-in web_search returns server_tool_use and web_search_tool_result blocks
            let web_search_uses: Vec<_> = response
                .content
                .iter()
                .filter(|c| {
                    c.content_type == "server_tool_use"
                        || c.content_type == "web_search_tool_result"
                })
                .collect();

            if !web_search_uses.is_empty() {
                for block in &web_search_uses {
                    if block.content_type == "server_tool_use" {
                        // Extract search query from input if available
                        let search_query = block
                            .input
                            .as_ref()
                            .and_then(|i| i.get("query"))
                            .and_then(|q| q.as_str())
                            .map(|s| s.to_string());

                        if let Some(name) = &block.name {
                            info!(
                                "🔍 Web search initiated: tool={}, query={:?}",
                                name, search_query
                            );
                        }

                        // Emit web search started event
                        if let Some(app) = app_handle {
                            let _ = app.emit(
                                "research:web_search",
                                WebSearchEvent {
                                    timestamp: get_timestamp(),
                                    topic_name: topic.to_string(),
                                    search_query: search_query.clone(),
                                    status: "started".to_string(),
                                },
                            );
                        }
                    } else if block.content_type == "web_search_tool_result" {
                        info!("🔍 Web search completed for topic: {}", topic);

                        // Emit web search completed event
                        if let Some(app) = app_handle {
                            let _ = app.emit(
                                "research:web_search",
                                WebSearchEvent {
                                    timestamp: get_timestamp(),
                                    topic_name: topic.to_string(),
                                    search_query: None,
                                    status: "completed".to_string(),
                                },
                            );
                        }

                        // Log the web search tool result
                        let _ = ResearchLogger::log_tool_call(
                            topic,
                            "web_search",
                            "built-in web search",
                            "Web search completed (result in response)",
                            api_duration,
                        );
                    }
                }
            }

            // Check if Claude wants to use tools
            let tool_uses: Vec<_> = response
                .content
                .iter()
                .filter(|c| c.content_type == "tool_use")
                .collect();

            if tool_uses.is_empty() || response.stop_reason.as_deref() == Some("end_turn") {
                info!(
                    "No more tool calls requested - research complete for topic: {}",
                    topic
                );
                // No more tool calls, extract the text response
                let text_content: String = response
                    .content
                    .iter()
                    .filter_map(|c| {
                        if c.content_type == "text" {
                            c.text.clone()
                        } else {
                            None
                        }
                    })
                    .collect::<Vec<_>>()
                    .join("\n");

                return Ok((text_content, total_tokens));
            }

            // Build assistant message with tool uses
            // Filter out empty text blocks - Claude API rejects "text content blocks must be non-empty"
            let assistant_blocks: Vec<ContentBlock> = response
                .content
                .iter()
                .filter_map(|c| {
                    if c.content_type == "text" {
                        let text = c.text.clone().unwrap_or_default();
                        // Only include non-empty text blocks
                        if !text.is_empty() {
                            Some(ContentBlock::Text { text })
                        } else {
                            None
                        }
                    } else if c.content_type == "tool_use" {
                        Some(ContentBlock::ToolUse {
                            id: c.id.clone().unwrap_or_default(),
                            name: c.name.clone().unwrap_or_default(),
                            input: c.input.clone().unwrap_or(json!({})),
                        })
                    } else {
                        // Skip unknown content types instead of creating empty text blocks
                        None
                    }
                })
                .collect();

            messages.push(Message {
                role: "assistant".to_string(),
                content: MessageContent::Blocks(assistant_blocks),
            });

            // Execute tools and build results
            info!("Claude requested {} tool call(s)", tool_uses.len());
            let mut tool_results: Vec<ContentBlock> = Vec::new();
            let empty_input = json!({});
            for tool_use in tool_uses {
                let tool_name = tool_use.name.as_deref().unwrap_or("");
                let tool_id = tool_use.id.as_deref().unwrap_or("");
                let tool_input = tool_use.input.as_ref().unwrap_or(&empty_input);
                let input_str = serde_json::to_string(tool_input).unwrap_or_default();

                info!("Executing tool: {}", tool_name);
                debug!("Tool input: {}", tool_input);

                let tool_start = Instant::now();

                // Route to built-in tools or MCP client
                let is_mcp_tool = !self.is_builtin_tool(tool_name);
                let mcp_server_name: Option<String> = if is_mcp_tool {
                    // Find which server this tool belongs to
                    self.mcp_client.as_ref().and_then(|client| {
                        client
                            .get_all_tools()
                            .into_iter()
                            .find(|t| t.tool.name == tool_name)
                            .map(|t| t.server_name)
                    })
                } else {
                    None
                };

                let result = if self.is_builtin_tool(tool_name) {
                    // Execute built-in tool
                    execute_tool(
                        &self.client,
                        tool_name,
                        tool_input,
                        self.github_token.as_deref(),
                    )
                    .await
                } else if let Some(ref mut mcp_client) = self.mcp_client {
                    // Execute MCP tool
                    mcp_client
                        .call_tool(tool_name, tool_input.clone())
                        .map(|v| {
                            if let Some(s) = v.as_str() {
                                s.to_string()
                            } else {
                                serde_json::to_string_pretty(&v).unwrap_or_default()
                            }
                        })
                } else {
                    Err(format!("Unknown tool: {}", tool_name))
                };

                let tool_duration = tool_start.elapsed().as_millis() as i64;

                let (content, is_error) = match result {
                    Ok(output) => {
                        info!(
                            "Tool {} completed in {}ms (output: {} chars)",
                            tool_name,
                            tool_duration,
                            output.len()
                        );
                        // Log successful tool call - use MCP logging if it's an MCP tool
                        if is_mcp_tool {
                            let server_name = mcp_server_name.as_deref().unwrap_or("unknown");
                            let _ = ResearchLogger::log_mcp_call(
                                topic,
                                server_name,
                                tool_name,
                                &input_str,
                                &output,
                                tool_duration,
                            );
                        } else {
                            let _ = ResearchLogger::log_tool_call(
                                topic,
                                tool_name,
                                &input_str,
                                &output,
                                tool_duration,
                            );
                        }
                        (output, None)
                    }
                    Err(e) => {
                        error!("Tool {} failed: {}", tool_name, e);
                        // Log failed tool call - use appropriate error code for MCP tools
                        let err = if is_mcp_tool {
                            ResearchError::new(ErrorCode::McpToolFailed, &e)
                        } else {
                            ResearchError::new(ErrorCode::ToolExecutionFailed, &e)
                        };

                        if is_mcp_tool {
                            let server_name = mcp_server_name.as_deref().unwrap_or("unknown");
                            let _ = ResearchLogger::log_mcp_error(topic, server_name, &err);
                        } else {
                            let _ = ResearchLogger::log_tool_error(
                                topic,
                                tool_name,
                                &input_str,
                                &err,
                                tool_duration,
                            );
                        }
                        (format!("Error: {}", e), Some(true))
                    }
                };

                tool_results.push(ContentBlock::ToolResult {
                    tool_use_id: tool_id.to_string(),
                    content,
                    is_error,
                });
            }

            // Add tool results as user message
            messages.push(Message {
                role: "user".to_string(),
                content: MessageContent::Blocks(tool_results),
            });
        }

        // If we exit the loop due to max iterations, extract any text we have
        Ok((
            "Research completed (max iterations reached)".to_string(),
            total_tokens,
        ))
    }

    /// Send a request to the Anthropic API.
    async fn send_request(
        &self,
        request: &AnthropicRequest,
    ) -> Result<AnthropicResponse, ResearchError> {
        let response = self
            .client
            .post("https://api.anthropic.com/v1/messages")
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
            .header("content-type", "application/json")
            .json(request)
            .send()
            .await
            .map_err(|e| {
                let err = ResearchError::new(
                    ErrorCode::NetworkError,
                    format!("HTTP request failed: {}", e),
                );
                error!("Network error: {}", e);
                err
            })?;

        if !response.status().is_success() {
            let status = response.status().as_u16();
            let body = response.text().await.unwrap_or_default();
            let err = parse_api_error(status, &body);

            // Log the error
            error!(
                "API error {}: {} (code: {:?})",
                status, err.message, err.code
            );
            if err.requires_user_action {
                error!("USER ACTION REQUIRED: {}", err.user_message);
            }

            return Err(err);
        }

        response.json().await.map_err(|e| {
            ResearchError::new(
                ErrorCode::ParseError,
                format!("Failed to parse response: {}", e),
            )
        })
    }

    /// Synthesize research results into briefing cards.
    async fn synthesize_briefing(
        &self,
        research_content: &str,
        app_handle: Option<&tauri::AppHandle>,
        condense_briefings: bool,
        past_cards_context: Option<&str>,
    ) -> Result<(Vec<BriefingCard>, u32), ResearchError> {
        // Build the deduplication context if available
        let dedup_instruction = if let Some(context) = past_cards_context {
            if !context.is_empty() {
                format!(
                    r#"

DEDUPLICATION INSTRUCTIONS:
{}

When generating cards, avoid creating cards that duplicate these previously covered topics unless there is SIGNIFICANT NEW information. If a topic was recently covered and there's only minor updates, either:
1. Skip that topic entirely
2. Briefly mention "Continuing from previous coverage..." with only the new developments
"#,
                    context
                )
            } else {
                String::new()
            }
        } else {
            String::new()
        };

        // Adjust content requirements based on research mode
        let is_deep_research = self.research_mode == "firecrawl";
        let (min_words_condensed, min_paragraphs_condensed) = if is_deep_research {
            (800, "8-12")  // Deep research: more comprehensive
        } else {
            (400, "5-7")   // Standard: normal length
        };
        let (min_words_standard, min_paragraphs_standard) = if is_deep_research {
            (350, "4-6")   // Deep research: more detailed per card
        } else {
            (150, "2-3")   // Standard: normal length
        };
        let depth_instruction = if is_deep_research {
            "\n**DEEP RESEARCH MODE**: You have access to comprehensive web extraction. Provide EXTRA detail, analysis, and insights. Include more sources, deeper technical analysis, and thorough coverage. Users are paying premium credits for this depth - deliver exceptional value."
        } else {
            ""
        };

        let prompt = if condense_briefings {
            // Condensed mode: one comprehensive card combining all topics
            format!(
                r#"You are a research assistant creating a personalized daily briefing.
Synthesize ALL the following research into ONE comprehensive briefing card that tells a cohesive story.
{}
CRITICAL: ONLY include information from the RESEARCH CONTENT below.
Do NOT add topics from the deduplication list - that list is ONLY to help you avoid repeating old content.
{}
{}

Create a SINGLE comprehensive briefing card following these guidelines:

1. **Combine all topics** into one unified narrative
2. **Identify connections** and cross-cutting themes between topics
3. **Prioritize** the most significant developments

For the single card, provide:
- **Title**: A headline summarizing today's key developments (max 80 chars)
- **Summary**: Overview of all topics covered (3-4 sentences)
- **Detailed Content**: COMPREHENSIVE analysis using MARKDOWN formatting (minimum {} words, {} full paragraphs)
  - Use **bold text** for section headers and key terms (e.g., **Key Themes**, **Implications**)
  - Use bullet points or numbered lists for multiple items
  - Weave together insights from ALL research topics
  - Identify patterns, connections, and overarching themes
  - Include context, implications, and technical details
  - Provide actionable takeaways and what to watch for
  - This is the user's "daily read" - make it engaging and insightful
- **Sources**: Combined list of all source URLs
- **Suggested Next**: Key action or focus area based on the briefing
- **Relevance**: "high" (single briefing is always high priority)
- **Topic**: "Daily Briefing"
- **Image Prompt**: SHORT visual description (max 8 words, plain text only, no quotes or special characters).
  Examples: "futuristic city skyline at sunset", "abstract flowing data streams"

Return ONLY valid JSON in this exact format:
{{
  "cards": [
    {{
      "title": "Your Daily Briefing: Key Developments",
      "summary": "Overview covering all topics researched today with the most important findings.",
      "detailed_content": "**Key Themes**\\n\\nOpening paragraph introduces key themes and sets the stage for the briefing.\\n\\n**Topic Area One**\\n\\nThis section covers the first major topic area with **key findings** highlighted.\\n\\n- Important point one\\n- Important point two\\n\\n**Topic Area Two**\\n\\nSubsequent sections cover each major topic area, weaving them together into a coherent narrative.\\n\\n**Implications**\\n\\nAnalysis explores implications, connections between topics, and deeper insights.\\n\\n**Key Takeaways**\\n\\nConcluding section summarizes key takeaways and what to watch for going forward.",
      "sources": ["https://example.com/source1", "https://example.com/source2"],
      "suggested_next": "Key action or focus area",
      "relevance": "high",
      "topic": "Daily Briefing",
      "image_prompt": "abstract network of connected glowing nodes"
    }}
  ]
}}

Return the JSON response now:"#,
                depth_instruction, dedup_instruction, research_content, min_words_condensed, min_paragraphs_condensed
            )
        } else {
            // Standard mode: multiple cards
            format!(
                r#"You are a research assistant creating a personalized daily briefing.
Synthesize the following research results into clear, actionable briefing cards.
{}
CRITICAL: ONLY create cards for topics that appear in the RESEARCH CONTENT below. 
Do NOT create cards for topics mentioned in the deduplication list - that list is ONLY to help you avoid repeating old content.

CARD QUALITY GUIDELINES:
- Prefer fewer, stronger cards over many weak ones
- You MAY create multiple cards for a single topic IF there are genuinely distinct sub-themes or developments worth separating
- Each card must be substantial and stand on its own - no filler cards
- If in doubt, consolidate into fewer comprehensive cards rather than splitting thin content
{}
{}

Generate briefing cards following these guidelines:

1. **ONLY use topics from the research content** - never invent or add topics not researched
2. **Relevance**: Only include cards with medium or higher relevance
3. **Limit**: Maximum 10 cards total
4. **Priority**: Prioritize timely, actionable information

For each card, provide:
- **Title**: Clear, specific title (max 60 chars)
- **Summary**: Brief overview (2-4 sentences) - what the user sees by default
- **Detailed Content**: COMPREHENSIVE research analysis using MARKDOWN formatting (minimum {} words, {} full paragraphs)
  - Use **bold** for key terms, important findings, and emphasis
  - Use bullet points or numbered lists when presenting multiple items
  - Include context, implications, technical details, and deeper insights
  - This should be substantially longer and more detailed than the summary
  - Think of this as the "full story" while summary is the "headline"
- **Sources**: List of source URLs (if available, otherwise empty array)
- **Suggested Next**: Optional next action or follow-up
- **Relevance**: "high", "medium", or "low"
- **Topic**: The original topic this relates to
- **Image Prompt**: SHORT visual description (max 8 words, plain text only, no quotes or special characters).
  Examples: "robot hand reaching toward human hand", "stock market charts with upward arrows"

IMPORTANT: The detailed_content must be significantly more comprehensive than the summary.
The summary is what users see at a glance. The detailed_content is what they read when they want the full analysis.

Return ONLY valid JSON in this exact format:
{{
  "cards": [
    {{
      "title": "Card title",
      "summary": "Brief overview with key findings and why it matters to the user.",
      "detailed_content": "**Context and Background**\\n\\nFirst paragraph provides context and background information about the topic, explaining the current situation and recent developments.\\n\\n**Key Findings**\\n\\nSecond paragraph dives into the technical details, implications, and analysis of what this means:\\n\\n- Important finding or data point\\n- Another key insight from the research\\n- Relevant quote or statistic\\n\\n**Looking Ahead**\\n\\nThird paragraph discusses future implications, what to watch for, and how this connects to broader trends or related topics.",
      "sources": ["https://example.com/source1"],
      "suggested_next": "Optional next action",
      "relevance": "high",
      "topic": "Original topic name",
      "image_prompt": "futuristic circuit board with glowing pathways"
    }}
  ]
}}

Return the JSON response now:"#,
                depth_instruction, dedup_instruction, research_content, min_words_standard, min_paragraphs_standard
            )
        };

        let request = AnthropicRequest {
            model: self.model.clone(),
            max_tokens: 16384, // Large enough for many cards with detailed_content + image fields
            messages: vec![Message {
                role: "user".to_string(),
                content: MessageContent::Text(prompt),
            }],
            tools: None,
            system: None,
        };

        // Update phase and emit synthesis:started event
        research_state::set_phase("Synthesizing briefing cards...");
        if let Some(app) = app_handle {
            let _ = app.emit(
                "research:synthesis_started",
                SynthesisStartedEvent {
                    timestamp: get_timestamp(),
                    research_content_length: research_content.len(),
                },
            );
        }

        info!(
            "Calling Claude API for synthesis (research content: {} chars)",
            research_content.len()
        );
        let synthesis_start = Instant::now();
        let response = self.send_request(&request).await?;
        let synthesis_duration = synthesis_start.elapsed().as_millis();

        let content = response
            .content
            .iter()
            .filter_map(|c| c.text.clone())
            .collect::<Vec<_>>()
            .join("\n");

        let tokens = response.usage.input_tokens + response.usage.output_tokens;

        info!(
            "Synthesis API responded in {}ms ({} tokens)",
            synthesis_duration, tokens
        );

        // Parse the JSON response
        let cards = parse_briefing_response(&content)
            .map_err(|e| ResearchError::new(ErrorCode::ParseError, e))?;

        info!(
            "Successfully generated {} briefing cards from synthesis",
            cards.len()
        );

        // Update phase and emit synthesis:completed event
        research_state::set_phase(&format!("Synthesis complete: {} cards", cards.len()));
        if let Some(app) = app_handle {
            let _ = app.emit(
                "research:synthesis_completed",
                SynthesisCompletedEvent {
                    timestamp: get_timestamp(),
                    cards_generated: cards.len(),
                    duration_ms: synthesis_duration,
                },
            );
        }

        Ok((cards, tokens))
    }
}

/// Parse Claude's response into BriefingCard objects.
fn parse_briefing_response(response: &str) -> Result<Vec<BriefingCard>, String> {
    // Try to extract JSON from response (Claude might wrap it in markdown)
    // Use (?s) flag for DOTALL mode to match across newlines
    let json_str = if let Some(captures) = Regex::new(r"(?s)```(?:json)?\s*(\{.*\})\s*```")
        .ok()
        .and_then(|re| re.captures(response))
    {
        captures.get(1).map(|m| m.as_str()).unwrap_or(response)
    } else if let Some(captures) = Regex::new(r"(?s)(\{.*\})")
        .ok()
        .and_then(|re| re.captures(response))
    {
        captures.get(1).map(|m| m.as_str()).unwrap_or(response)
    } else {
        response
    };

    // Parse JSON - if it fails, try to provide helpful error message
    match serde_json::from_str::<BriefingResponse>(json_str) {
        Ok(briefing_response) => Ok(briefing_response.cards),
        Err(e) => {
            // Check if response looks truncated (EOF errors)
            let error_msg = e.to_string();
            if error_msg.contains("EOF") {
                // Try to fix truncated JSON by closing the array and object
                let fixed_attempt = format!("{}\n]\n}}", json_str.trim_end_matches(','));
                if let Ok(briefing_response) =
                    serde_json::from_str::<BriefingResponse>(&fixed_attempt)
                {
                    warn!(
                        "Recovered {} cards from truncated response",
                        briefing_response.cards.len()
                    );
                    return Ok(briefing_response.cards);
                }

                Err(format!(
                    "Response was truncated (likely hit max_tokens limit). Increase max_tokens in synthesis call. \
                    Error: {}. Response length: {} chars. Last 200 chars: ...{}",
                    error_msg,
                    json_str.len(),
                    &json_str[json_str.len().saturating_sub(200)..]
                ))
            } else {
                Err(format!(
                    "Failed to parse briefing JSON: {}. Response length: {} chars. First 500 chars: {}...",
                    error_msg,
                    json_str.len(),
                    &json_str[..json_str.len().min(500)]
                ))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_briefing_response() {
        let response = r#"{"cards": [{"title": "Test", "summary": "Test summary", "detailed_content": "Detailed test content", "sources": [], "suggested_next": null, "relevance": "high", "topic": "Test Topic"}]}"#;
        let cards = parse_briefing_response(response).unwrap();
        assert_eq!(cards.len(), 1);
        assert_eq!(cards[0].title, "Test");
    }

    #[test]
    fn test_parse_briefing_response_with_markdown() {
        let response = r#"```json
{"cards": [{"title": "Test", "summary": "Test summary", "detailed_content": "Detailed test content", "sources": [], "suggested_next": null, "relevance": "high", "topic": "Test Topic"}]}
```"#;
        let cards = parse_briefing_response(response).unwrap();
        assert_eq!(cards.len(), 1);
    }

    #[test]
    fn test_parse_briefing_response_multiple_cards() {
        let response = r#"{"cards": [
            {"title": "Card 1", "summary": "Summary 1", "detailed_content": "Detailed content 1", "sources": ["https://example.com"], "suggested_next": "Read more", "relevance": "high", "topic": "Topic A"},
            {"title": "Card 2", "summary": "Summary 2", "detailed_content": "Detailed content 2", "sources": [], "suggested_next": null, "relevance": "medium", "topic": "Topic B"},
            {"title": "Card 3", "summary": "Summary 3", "detailed_content": "Detailed content 3", "sources": ["https://source1.com", "https://source2.com"], "suggested_next": "Follow up", "relevance": "low", "topic": "Topic A"}
        ]}"#;
        let cards = parse_briefing_response(response).unwrap();
        assert_eq!(cards.len(), 3);
        assert_eq!(cards[0].title, "Card 1");
        assert_eq!(cards[0].sources.len(), 1);
        assert_eq!(cards[0].suggested_next, Some("Read more".to_string()));
        assert_eq!(cards[1].relevance, "medium");
        assert_eq!(cards[2].sources.len(), 2);
    }

    #[test]
    fn test_parse_briefing_response_with_markdown_code_block() {
        let response = r#"Here is the briefing:
```json
{"cards": [{"title": "AI Developments", "summary": "Major advances in AI", "detailed_content": "Detailed AI content", "sources": [], "suggested_next": null, "relevance": "high", "topic": "Artificial Intelligence"}]}
```
That's the summary!"#;
        let cards = parse_briefing_response(response).unwrap();
        assert_eq!(cards.len(), 1);
        assert_eq!(cards[0].title, "AI Developments");
        assert_eq!(cards[0].topic, "Artificial Intelligence");
    }

    #[test]
    fn test_parse_briefing_response_empty_cards() {
        let response = r#"{"cards": []}"#;
        let cards = parse_briefing_response(response).unwrap();
        assert_eq!(cards.len(), 0);
    }

    #[test]
    fn test_parse_briefing_response_invalid_json() {
        let response = r#"{"cards": invalid}"#;
        let result = parse_briefing_response(response);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_briefing_response_with_markdown_content() {
        // Test that detailed_content with markdown formatting is parsed correctly
        let response = r#"{"cards": [{"title": "Test Card", "summary": "Brief summary", "detailed_content": "**Context and Background**\n\nFirst paragraph with **bold text** and context.\n\n**Key Findings**\n\n- Finding one\n- Finding two\n- Finding three\n\n**Looking Ahead**\n\nConclusion with implications.", "sources": ["https://example.com"], "suggested_next": null, "relevance": "high", "topic": "Test Topic"}]}"#;
        let cards = parse_briefing_response(response).unwrap();
        assert_eq!(cards.len(), 1);
        assert!(cards[0]
            .detailed_content
            .contains("**Context and Background**"));
        assert!(cards[0].detailed_content.contains("- Finding one"));
        assert!(cards[0].detailed_content.contains("**Key Findings**"));
    }

    #[test]
    fn test_parse_briefing_response_condensed_with_markdown() {
        // Test condensed briefing format with markdown sections
        let response = r#"{"cards": [{"title": "Your Daily Briefing", "summary": "Overview of all topics", "detailed_content": "**Key Themes**\n\nIntroduction to themes.\n\n**Topic Area One**\n\nFirst topic coverage with **highlights**.\n\n- Point one\n- Point two\n\n**Topic Area Two**\n\nSecond topic coverage.\n\n**Implications**\n\nAnalysis of implications.\n\n**Key Takeaways**\n\nSummary of takeaways.", "sources": ["https://source1.com", "https://source2.com"], "suggested_next": "Focus area", "relevance": "high", "topic": "Daily Briefing"}]}"#;
        let cards = parse_briefing_response(response).unwrap();
        assert_eq!(cards.len(), 1);
        assert_eq!(cards[0].topic, "Daily Briefing");
        assert!(cards[0].detailed_content.contains("**Key Themes**"));
        assert!(cards[0].detailed_content.contains("**Topic Area One**"));
        assert!(cards[0].detailed_content.contains("**Implications**"));
        assert!(cards[0].detailed_content.contains("**Key Takeaways**"));
    }

    #[test]
    fn test_briefing_card_serialization() {
        let card = BriefingCard {
            title: "Test Title".to_string(),
            summary: "Test summary with details".to_string(),
            sources: vec!["https://example.com".to_string()],
            suggested_next: Some("Follow up action".to_string()),
            relevance: "high".to_string(),
            topic: "Test Topic".to_string(),
            detailed_content: "Detailed test content".to_string(),
            image_prompt: Some("futuristic technology concept".to_string()),
            image_style: Some("illustration".to_string()),
            image_path: None,
        };

        let json = serde_json::to_string(&card).unwrap();
        assert!(json.contains("Test Title"));
        assert!(json.contains("Test summary"));
        assert!(json.contains("https://example.com"));
        assert!(json.contains("futuristic technology"));

        // Deserialize back
        let parsed: BriefingCard = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.title, card.title);
        assert_eq!(parsed.sources, card.sources);
        assert_eq!(parsed.image_prompt, card.image_prompt);
    }

    #[test]
    fn test_research_result_serialization() {
        let result = ResearchResult {
            date: "2025-01-15".to_string(),
            title: "Daily Briefing - January 15, 2025".to_string(),
            cards: vec![BriefingCard {
                title: "Card 1".to_string(),
                summary: "Summary 1".to_string(),
                sources: vec![],
                suggested_next: None,
                relevance: "high".to_string(),
                topic: "Topic 1".to_string(),
                detailed_content: "Detailed content 1".to_string(),
                image_prompt: None,
                image_style: None,
                image_path: None,
            }],
            research_time_ms: 1500,
            model_used: "claude-haiku-4-5-20251001".to_string(),
            total_tokens: 2500,
        };

        let json = serde_json::to_string(&result).unwrap();
        assert!(json.contains("2025-01-15"));
        assert!(json.contains("Daily Briefing"));
        assert!(json.contains("1500"));
        assert!(json.contains("2500"));

        let parsed: ResearchResult = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.date, result.date);
        assert_eq!(parsed.cards.len(), 1);
        assert_eq!(parsed.research_time_ms, 1500);
    }

    #[test]
    fn test_research_agent_creation() {
        let agent = ResearchAgent::new(
            "test-api-key".to_string(),
            None,
            false,
            "standard".to_string(),
        );
        assert_eq!(agent.model, "claude-haiku-4-5-20251001");
        assert!(!agent.enable_web_search);
        assert_eq!(agent.research_mode, "standard");

        let agent_custom = ResearchAgent::new(
            "test-api-key".to_string(),
            Some("claude-opus-4-5-20251101".to_string()),
            false,
            "firecrawl".to_string(),
        );
        assert_eq!(agent_custom.model, "claude-opus-4-5-20251101");
        assert_eq!(agent_custom.research_mode, "firecrawl");
    }

    #[test]
    fn test_research_agent_with_web_search() {
        let agent = ResearchAgent::new(
            "test-api-key".to_string(),
            None,
            true,
            "standard".to_string(),
        );
        assert!(agent.enable_web_search);

        // Test that get_tools_json includes web_search when enabled
        let tools = agent.get_tools_json();
        let has_web_search = tools
            .iter()
            .any(|t| t.get("type").and_then(|v| v.as_str()) == Some(WEB_SEARCH_TOOL_TYPE));
        assert!(
            has_web_search,
            "web_search tool should be included when enabled"
        );
    }

    #[tokio::test]
    async fn test_run_research_empty_topics() {
        let mut agent = ResearchAgent::new(
            "test-api-key".to_string(),
            None,
            false,
            "standard".to_string(),
        );
        let result = agent.run_research(vec![], None, false, None).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("No topics provided"));
    }

    #[test]
    fn test_extract_text_from_html() {
        let html =
            r#"<html><head><title>Test</title></head><body><p>Hello World</p></body></html>"#;
        let text = extract_text_from_html(html);
        assert!(text.contains("Hello World"));
        assert!(!text.contains("<p>"));
    }

    #[test]
    fn test_extract_text_from_html_with_scripts() {
        let html = r#"<html><script>alert('test');</script><body><p>Content</p></body></html>"#;
        let text = extract_text_from_html(html);
        assert!(text.contains("Content"));
        assert!(!text.contains("alert"));
    }

    #[test]
    fn test_format_github_commits() {
        let data = json!([
            {
                "sha": "abc123def456789",
                "commit": {
                    "message": "Fix bug in parser",
                    "author": {
                        "name": "John Doe",
                        "date": "2025-01-15T10:00:00Z"
                    }
                }
            }
        ]);
        let formatted = format_github_commits(&data);
        assert!(formatted.contains("abc123d"));
        assert!(formatted.contains("John Doe"));
        assert!(formatted.contains("Fix bug"));
    }

    #[test]
    fn test_format_github_pulls() {
        let data = json!([
            {
                "number": 123,
                "title": "Add new feature",
                "state": "open",
                "user": { "login": "contributor" }
            }
        ]);
        let formatted = format_github_pulls(&data);
        assert!(formatted.contains("#123"));
        assert!(formatted.contains("open"));
        assert!(formatted.contains("Add new feature"));
    }

    #[test]
    fn test_get_research_tools() {
        let tools = get_research_tools();
        assert_eq!(tools.len(), 2);
        assert!(tools.iter().any(|t| t.name == "get_github_activity"));
        assert!(tools.iter().any(|t| t.name == "fetch_webpage"));
    }

    #[test]
    fn test_tool_filtering_standard_mode() {
        // In standard mode, firecrawl tools should be excluded
        let agent = ResearchAgent::new(
            "test-api-key".to_string(),
            None,
            false,
            "standard".to_string(),
        );

        // Without MCP client, should only have built-in tools
        let tools = agent.get_all_tools();
        assert_eq!(tools.len(), 2); // get_github_activity and fetch_webpage
        assert!(tools.iter().any(|t| t.name == "fetch_webpage"));
        assert!(tools.iter().any(|t| t.name == "get_github_activity"));
    }

    #[test]
    fn test_tool_filtering_firecrawl_mode() {
        // In firecrawl mode, fetch_webpage should be excluded from built-in tools
        let agent = ResearchAgent::new(
            "test-api-key".to_string(),
            None,
            false,
            "firecrawl".to_string(),
        );

        // Without MCP client, fetch_webpage should be excluded
        let tools = agent.get_all_tools();
        assert_eq!(tools.len(), 1); // Only get_github_activity
        assert!(tools.iter().any(|t| t.name == "get_github_activity"));
        assert!(
            !tools.iter().any(|t| t.name == "fetch_webpage"),
            "fetch_webpage should be excluded in firecrawl mode"
        );
    }

    #[test]
    fn test_research_mode_stored_correctly() {
        let agent_standard =
            ResearchAgent::new("key".to_string(), None, false, "standard".to_string());
        assert_eq!(agent_standard.research_mode, "standard");

        let agent_firecrawl =
            ResearchAgent::new("key".to_string(), None, false, "firecrawl".to_string());
        assert_eq!(agent_firecrawl.research_mode, "firecrawl");
    }
}
