//! Chat module for briefing conversations.
//!
//! Provides simple chat functionality using the Anthropic API.
//! Users can chat about briefings with Claude, using the briefing content as context.

use chrono::{Datelike, Local};
use regex::Regex;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::time::Duration;
use tracing::{error, info};

use crate::db::{self, ChatMessage};
use crate::mcp_client::{load_mcp_servers, McpClient};
use serde_json::json;
use tauri::Emitter;

// ============================================================================
// Event Types
// ============================================================================

/// Event emitted when chat starts tool execution.
#[derive(Clone, serde::Serialize)]
pub struct ChatToolStartEvent {
    pub tool_name: String,
    pub briefing_id: i64,
    pub card_index: i32,
}

/// Event emitted when chat tool execution completes.
#[derive(Clone, serde::Serialize)]
pub struct ChatToolCompleteEvent {
    pub tool_name: String,
    pub success: bool,
    pub briefing_id: i64,
    pub card_index: i32,
}

// ============================================================================
// API Structures
// ============================================================================

/// Tool definition for chat.
#[derive(Debug, Clone, Serialize)]
struct Tool {
    name: String,
    description: String,
    input_schema: serde_json::Value,
}

/// Anthropic API message request with tool support.
#[derive(Debug, Serialize)]
struct ChatRequest {
    model: String,
    max_tokens: u32,
    messages: Vec<Message>,
    system: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    tools: Option<Vec<serde_json::Value>>,
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

/// A content block in a message (for serialization).
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
struct ChatResponse {
    content: Vec<ResponseContentBlock>,
    usage: Usage,
    stop_reason: Option<String>,
}

/// Content block in API response (for deserialization).
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

// ============================================================================
// Tool Constants
// ============================================================================

/// Claude's built-in web search tool type.
const WEB_SEARCH_TOOL_TYPE: &str = "web_search_20250305";

/// Maximum number of web searches per chat turn.
const WEB_SEARCH_MAX_USES: u32 = 3;

// ============================================================================
// Tool Definitions
// ============================================================================

/// Get built-in tools available for chat.
fn get_chat_tools() -> Vec<Tool> {
    vec![
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
    ]
}

/// Get all tools as JSON values for API request.
///
/// Combines built-in tools with MCP tools and optionally Claude's web_search.
fn get_tools_json(
    mcp_client: &Option<McpClient>,
    enable_web_search: bool,
) -> Vec<serde_json::Value> {
    let tools = get_chat_tools();
    let mut tools_json: Vec<serde_json::Value> = tools
        .iter()
        .map(|t| {
            json!({
                "name": t.name,
                "description": t.description,
                "input_schema": t.input_schema
            })
        })
        .collect();

    // Add MCP tools
    if let Some(ref client) = mcp_client {
        for mcp_tool in client.get_all_tools() {
            tools_json.push(json!({
                "name": mcp_tool.tool.name,
                "description": mcp_tool.tool.description.clone().unwrap_or_else(||
                    format!("Tool from {} MCP server", mcp_tool.server_name)
                ),
                "input_schema": mcp_tool.tool.input_schema
            }));
        }
    }

    // Add Claude's built-in web search tool if enabled
    if enable_web_search {
        tools_json.push(json!({
            "type": WEB_SEARCH_TOOL_TYPE,
            "name": "web_search",
            "max_uses": WEB_SEARCH_MAX_USES
        }));
        info!("Added web_search tool to chat request");
    }

    tools_json
}

/// Names of built-in tools (to differentiate from MCP tools).
fn get_builtin_tool_names() -> std::collections::HashSet<String> {
    get_chat_tools().into_iter().map(|t| t.name).collect()
}

// ============================================================================
// Chat Functions
// ============================================================================

/// Build the system prompt for chat, including specific card context and date.
fn build_system_prompt(
    briefing_title: &str,
    briefing_cards: &str,
    card_index: i32,
    has_tools: bool,
) -> String {
    // Parse the cards JSON and extract the specific card's content
    let card_content = extract_card_content(briefing_cards, card_index);

    // Add date context like research.rs does
    let now = Local::now();
    let current_date = now.format("%B %d, %Y").to_string();
    let current_year = now.format("%Y").to_string();
    let _prev_year = (now.year() - 1).to_string();

    let tool_context = if has_tools {
        "\n\nYou have access to tools to fetch real-time information. If the user asks about current events, weather, prices, latest news, or other time-sensitive information, use your tools to get up-to-date data. When searching, include the current date/year in queries to get recent results."
    } else {
        ""
    };

    format!(
        r#"You are a helpful assistant discussing a research briefing card with the user.

Today's date is {date}. The current year is {year}. The briefing was generated recently.

The user is viewing a specific card from a briefing titled "{title}".

Here is the card content:
{content}

Help the user understand this card, answer questions about it, provide additional context, or discuss related topics. Be concise but thorough. If the user asks about something not covered in the card, you can draw on your general knowledge but make it clear when you're going beyond the card content.{tools}"#,
        date = current_date,
        year = current_year,
        title = briefing_title,
        content = card_content,
        tools = tool_context
    )
}

/// Extract readable content from a specific card in the briefing cards JSON.
fn extract_card_content(cards_json: &str, card_index: i32) -> String {
    #[derive(Deserialize)]
    struct Card {
        title: Option<String>,
        summary: Option<String>,
        detailed_content: Option<String>,
        relevance: Option<String>,
        topic: Option<String>,
    }

    let cards: Vec<Card> = serde_json::from_str(cards_json).unwrap_or_default();

    if cards.is_empty() {
        return "No briefing cards available.".to_string();
    }

    // Get the specific card by index
    let card_idx = card_index as usize;
    if card_idx >= cards.len() {
        return format!(
            "Card {} not found (briefing has {} cards).",
            card_index,
            cards.len()
        );
    }

    let card = &cards[card_idx];
    let title = card.title.as_deref().unwrap_or("Untitled");
    let summary = card.summary.as_deref().unwrap_or("");
    let details = card.detailed_content.as_deref().unwrap_or("");
    let relevance = card.relevance.as_deref().unwrap_or("medium");
    let topic = card.topic.as_deref().unwrap_or("General");

    let mut content = format!(
        "Title: {}\nTopic: {}\nRelevance: {}\nSummary: {}",
        title, topic, relevance, summary
    );

    if !details.is_empty() {
        content.push_str(&format!("\n\nDetails:\n{}", details));
    }

    content
}

/// Build the messages array for the API call.
fn build_messages(history: &[ChatMessage], new_message: &str) -> Vec<Message> {
    let mut messages = Vec::new();

    // Add history (limit to last 20 messages to manage context window)
    let max_history = 20;
    let start_idx = history.len().saturating_sub(max_history);

    for msg in &history[start_idx..] {
        messages.push(Message {
            role: msg.role.clone(),
            content: MessageContent::Text(msg.content.clone()),
        });
    }

    // Add the new user message
    messages.push(Message {
        role: "user".to_string(),
        content: MessageContent::Text(new_message.to_string()),
    });

    messages
}

/// Maximum number of tool iterations to prevent infinite loops.
const MAX_TOOL_ITERATIONS: u32 = 5;

/// Send a chat message and get a response from Claude.
///
/// This function:
/// 1. Loads the briefing for context
/// 2. Loads existing chat history for this specific card
/// 3. Initializes MCP client for tool support
/// 4. Calls the Anthropic API with tools in an agentic loop
/// 5. Saves both user message and assistant response to the database
/// 6. Returns the assistant's message
pub async fn send_chat_message(
    api_key: &str,
    model: &str,
    briefing_id: i64,
    card_index: i32,
    user_message: &str,
    enable_web_search: bool,
    app_handle: Option<&tauri::AppHandle>,
) -> Result<(ChatMessage, i32), String> {
    // Get database connection
    let conn = db::get_connection().map_err(|e| format!("Database connection failed: {}", e))?;

    // Load briefing for context
    let briefing = load_briefing(&conn, briefing_id)?;

    // Load existing chat history for this specific card
    let history = db::get_chat_messages(&conn, briefing_id, card_index)?;
    info!(
        "Loaded {} messages from chat history for briefing {} card {}",
        history.len(),
        briefing_id,
        card_index
    );

    // Initialize MCP client for tools
    let mut mcp_client: Option<McpClient> = match load_mcp_servers() {
        Ok(servers) => {
            let enabled_servers: Vec<_> = servers.into_iter().filter(|s| s.enabled).collect();
            if enabled_servers.is_empty() {
                info!("No MCP servers configured for chat");
                None
            } else {
                info!(
                    "Connecting to {} MCP servers for chat...",
                    enabled_servers.len()
                );
                match McpClient::connect(enabled_servers).await {
                    Ok(client) => {
                        let tool_count = client.get_all_tools().len();
                        info!("MCP client connected with {} tools", tool_count);
                        for tool in client.get_all_tools() {
                            info!(
                                "  - MCP tool available: {} (from {})",
                                tool.tool.name, tool.server_name
                            );
                        }
                        Some(client)
                    }
                    Err(e) => {
                        error!("Failed to connect MCP client: {}", e);
                        None
                    }
                }
            }
        }
        Err(e) => {
            info!("No MCP servers available: {}", e);
            None
        }
    };

    // Build tools JSON
    let tools_json = get_tools_json(&mcp_client, enable_web_search);
    let has_tools = !tools_json.is_empty();

    info!(
        "Chat tools configured: {} total (built-in: 2, MCP: {}, web_search: {})",
        tools_json.len(),
        tools_json.len() - 2 - if enable_web_search { 1 } else { 0 },
        enable_web_search
    );

    // Build system prompt with specific card context and tool awareness
    let system_prompt =
        build_system_prompt(&briefing.title, &briefing.cards, card_index, has_tools);

    // Build messages array (will be mutated during agentic loop)
    let mut messages = build_messages(&history, user_message);

    // Create HTTP client
    let http_client = Client::builder()
        .timeout(Duration::from_secs(120)) // Longer timeout for tool calls
        .connect_timeout(Duration::from_secs(10))
        .build()
        .map_err(|e| format!("Failed to build HTTP client: {}", e))?;

    info!(
        "Sending chat message for briefing {} card {} (tools: {}, web_search: {})",
        briefing_id, card_index, has_tools, enable_web_search
    );

    let mut total_tokens: u32 = 0;
    let mut iterations: u32 = 0;
    let builtin_tools = get_builtin_tool_names();
    let final_text: String;

    // Agentic loop - continue until Claude finishes or max iterations
    loop {
        iterations += 1;
        if iterations > MAX_TOOL_ITERATIONS {
            error!("Max tool iterations ({}) exceeded", MAX_TOOL_ITERATIONS);
            return Err(format!(
                "Max tool iterations ({}) exceeded",
                MAX_TOOL_ITERATIONS
            ));
        }

        // Create API request
        let request = ChatRequest {
            model: model.to_string(),
            max_tokens: 2048,
            messages: messages.clone(),
            system: system_prompt.clone(),
            tools: if has_tools {
                Some(tools_json.clone())
            } else {
                None
            },
        };

        // Send request to Anthropic API
        let response = http_client
            .post("https://api.anthropic.com/v1/messages")
            .header("x-api-key", api_key)
            .header("anthropic-version", "2023-06-01")
            .header("content-type", "application/json")
            .json(&request)
            .send()
            .await
            .map_err(|e| format!("HTTP request failed: {}", e))?;

        // Check for errors
        if !response.status().is_success() {
            let status = response.status().as_u16();
            let body = response.text().await.unwrap_or_default();
            error!("Chat API error {}: {}", status, body);
            return Err(format!("API error {}: {}", status, body));
        }

        // Parse response
        let chat_response: ChatResponse = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse response: {}", e))?;

        let tokens = chat_response.usage.input_tokens + chat_response.usage.output_tokens;
        total_tokens += tokens;

        info!(
            "Chat API iteration {}: {} tokens, stop_reason: {:?}",
            iterations, tokens, chat_response.stop_reason
        );

        // Check for tool use requests
        let tool_uses: Vec<_> = chat_response
            .content
            .iter()
            .filter(|c| c.content_type == "tool_use")
            .collect();

        // If no tool calls or stop_reason is end_turn, we're done
        if tool_uses.is_empty() || chat_response.stop_reason.as_deref() == Some("end_turn") {
            // Extract final text response
            final_text = chat_response
                .content
                .iter()
                .filter_map(|block| {
                    if block.content_type == "text" {
                        block.text.clone()
                    } else {
                        None
                    }
                })
                .collect::<Vec<_>>()
                .join("\n");

            info!(
                "Chat complete after {} iterations, {} total tokens",
                iterations, total_tokens
            );
            break;
        }

        // Execute tool calls
        info!("Executing {} tool calls", tool_uses.len());
        let mut tool_results: Vec<ContentBlock> = Vec::new();

        for tool_use in &tool_uses {
            let tool_id = tool_use.id.as_deref().unwrap_or("unknown");
            let tool_name = tool_use.name.as_deref().unwrap_or("unknown");
            let tool_input = tool_use.input.clone().unwrap_or(json!({}));

            info!("Executing tool: {} ({})", tool_name, tool_id);

            // Emit tool start event
            if let Some(app) = app_handle {
                let _ = app.emit(
                    "chat:tool_start",
                    ChatToolStartEvent {
                        tool_name: tool_name.to_string(),
                        briefing_id,
                        card_index,
                    },
                );
            }

            let result = execute_chat_tool(
                &http_client,
                &mut mcp_client,
                &builtin_tools,
                tool_name,
                &tool_input,
            )
            .await;

            let (content, is_error) = match &result {
                Ok(output) => {
                    info!("Tool {} succeeded: {} chars", tool_name, output.len());
                    (output.clone(), None)
                }
                Err(e) => {
                    error!("Tool {} failed: {}", tool_name, e);
                    (format!("Error: {}", e), Some(true))
                }
            };

            // Emit tool complete event
            if let Some(app) = app_handle {
                let _ = app.emit(
                    "chat:tool_complete",
                    ChatToolCompleteEvent {
                        tool_name: tool_name.to_string(),
                        success: result.is_ok(),
                        briefing_id,
                        card_index,
                    },
                );
            }

            tool_results.push(ContentBlock::ToolResult {
                tool_use_id: tool_id.to_string(),
                content,
                is_error,
            });
        }

        // Add assistant's response (with tool_use blocks) to messages
        let assistant_blocks: Vec<ContentBlock> = chat_response
            .content
            .iter()
            .map(|c| {
                if c.content_type == "text" {
                    ContentBlock::Text {
                        text: c.text.clone().unwrap_or_default(),
                    }
                } else if c.content_type == "tool_use" {
                    ContentBlock::ToolUse {
                        id: c.id.clone().unwrap_or_default(),
                        name: c.name.clone().unwrap_or_default(),
                        input: c.input.clone().unwrap_or(json!({})),
                    }
                } else {
                    ContentBlock::Text {
                        text: String::new(),
                    }
                }
            })
            .collect();

        messages.push(Message {
            role: "assistant".to_string(),
            content: MessageContent::Blocks(assistant_blocks),
        });

        // Add tool results as user message
        messages.push(Message {
            role: "user".to_string(),
            content: MessageContent::Blocks(tool_results),
        });
    }

    // Save user message to database
    let _user_id =
        db::insert_chat_message(&conn, briefing_id, card_index, "user", user_message, None)?;

    // Save assistant response to database
    let assistant_id = db::insert_chat_message(
        &conn,
        briefing_id,
        card_index,
        "assistant",
        &final_text,
        Some(total_tokens as i32),
    )?;

    // Get the saved assistant message
    let assistant_message = db::get_chat_message_by_id(&conn, assistant_id)?
        .ok_or("Failed to retrieve saved message")?;

    Ok((assistant_message, total_tokens as i32))
}

/// Load a briefing from the database.
fn load_briefing(conn: &rusqlite::Connection, briefing_id: i64) -> Result<BriefingData, String> {
    let mut stmt = conn
        .prepare("SELECT id, title, cards FROM briefings WHERE id = ?1")
        .map_err(|e| format!("Failed to prepare statement: {}", e))?;

    stmt.query_row([briefing_id], |row| {
        Ok(BriefingData {
            id: row.get(0)?,
            title: row.get(1)?,
            cards: row.get(2)?,
        })
    })
    .map_err(|e| format!("Failed to load briefing: {}", e))
}

/// Minimal briefing data for chat context.
struct BriefingData {
    #[allow(dead_code)]
    id: i64,
    title: String,
    cards: String,
}

/// Get chat history for a specific card in a briefing.
pub fn get_chat_history(briefing_id: i64, card_index: i32) -> Result<Vec<ChatMessage>, String> {
    let conn = db::get_connection().map_err(|e| format!("Database connection failed: {}", e))?;

    db::get_chat_messages(&conn, briefing_id, card_index)
}

/// Clear chat history for a specific card in a briefing.
pub fn clear_chat_history(briefing_id: i64, card_index: i32) -> Result<usize, String> {
    let conn = db::get_connection().map_err(|e| format!("Database connection failed: {}", e))?;

    db::delete_chat_messages(&conn, briefing_id, card_index)
}

// ============================================================================
// Tool Execution
// ============================================================================

/// Execute a chat tool and return the result.
///
/// Routes to built-in tools or MCP client based on tool name.
async fn execute_chat_tool(
    http_client: &Client,
    mcp_client: &mut Option<McpClient>,
    builtin_tools: &HashSet<String>,
    tool_name: &str,
    tool_input: &serde_json::Value,
) -> Result<String, String> {
    // Check if it's a built-in tool
    if builtin_tools.contains(tool_name) {
        return execute_builtin_tool(http_client, tool_name, tool_input).await;
    }

    // Try MCP client
    if let Some(ref mut client) = mcp_client {
        // Check if the tool exists in MCP
        let has_tool = client
            .get_all_tools()
            .into_iter()
            .any(|t| t.tool.name == tool_name);

        if has_tool {
            info!("Calling MCP tool '{}'", tool_name);
            // call_tool returns serde_json::Value, convert to String
            let result = client.call_tool(tool_name, tool_input.clone())?;
            // Convert Value to readable string
            if let Some(s) = result.as_str() {
                return Ok(s.to_string());
            } else {
                return Ok(
                    serde_json::to_string_pretty(&result).unwrap_or_else(|_| result.to_string())
                );
            }
        }
    }

    Err(format!("Unknown tool: {}", tool_name))
}

/// Execute a built-in tool.
async fn execute_builtin_tool(
    client: &Client,
    tool_name: &str,
    input: &serde_json::Value,
) -> Result<String, String> {
    match tool_name {
        "fetch_webpage" => {
            let url = input
                .get("url")
                .and_then(|v| v.as_str())
                .ok_or("Missing url parameter")?;
            execute_fetch_webpage(client, url).await
        }
        "get_github_activity" => {
            let owner = input
                .get("owner")
                .and_then(|v| v.as_str())
                .ok_or("Missing owner parameter")?;
            let repo = input
                .get("repo")
                .and_then(|v| v.as_str())
                .ok_or("Missing repo parameter")?;
            let activity_type = input
                .get("activity_type")
                .and_then(|v| v.as_str())
                .ok_or("Missing activity_type parameter")?;

            // Try to get GitHub token from environment
            let github_token = std::env::var("GITHUB_TOKEN").ok();
            execute_github_activity(client, owner, repo, activity_type, github_token.as_deref())
                .await
        }
        _ => Err(format!("Unknown built-in tool: {}", tool_name)),
    }
}

/// Fetch and extract text content from a webpage.
async fn execute_fetch_webpage(client: &Client, url: &str) -> Result<String, String> {
    // Validate URL
    if !url.starts_with("http://") && !url.starts_with("https://") {
        return Err("URL must start with http:// or https://".to_string());
    }

    info!("Fetching webpage: {}", url);

    let response = client
        .get(url)
        .header("User-Agent", "Claudius-Chat-Agent")
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

    // Extract text content from HTML
    let text = extract_text_from_html(&html);

    // Truncate if too long
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

    info!("Fetching GitHub {}: {}/{}", activity_type, owner, repo);

    let mut request = client
        .get(&endpoint)
        .header("User-Agent", "Claudius-Chat-Agent")
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
    format_github_activity(&data, activity_type)
}

/// Format GitHub API response into readable text.
fn format_github_activity(data: &serde_json::Value, activity_type: &str) -> Result<String, String> {
    let items = data.as_array().ok_or("Expected array response")?;

    if items.is_empty() {
        return Ok(format!("No {} found.", activity_type));
    }

    let mut result = format!("Recent {} ({} items):\n\n", activity_type, items.len());

    for item in items {
        match activity_type {
            "commits" => {
                let sha = item
                    .get("sha")
                    .and_then(|v| v.as_str())
                    .unwrap_or("unknown");
                let message = item
                    .get("commit")
                    .and_then(|c| c.get("message"))
                    .and_then(|m| m.as_str())
                    .unwrap_or("No message");
                let author = item
                    .get("commit")
                    .and_then(|c| c.get("author"))
                    .and_then(|a| a.get("name"))
                    .and_then(|n| n.as_str())
                    .unwrap_or("Unknown");
                let date = item
                    .get("commit")
                    .and_then(|c| c.get("author"))
                    .and_then(|a| a.get("date"))
                    .and_then(|d| d.as_str())
                    .unwrap_or("");

                result.push_str(&format!(
                    "- {} ({}) by {} on {}\n",
                    message.lines().next().unwrap_or(message),
                    &sha[..7.min(sha.len())],
                    author,
                    date
                ));
            }
            "pulls" | "issues" => {
                let number = item.get("number").and_then(|v| v.as_u64()).unwrap_or(0);
                let title = item
                    .get("title")
                    .and_then(|v| v.as_str())
                    .unwrap_or("No title");
                let state = item
                    .get("state")
                    .and_then(|v| v.as_str())
                    .unwrap_or("unknown");
                let user = item
                    .get("user")
                    .and_then(|u| u.get("login"))
                    .and_then(|l| l.as_str())
                    .unwrap_or("unknown");

                result.push_str(&format!(
                    "- #{} [{}] {} (by {})\n",
                    number, state, title, user
                ));
            }
            "releases" => {
                let name = item
                    .get("name")
                    .and_then(|v| v.as_str())
                    .or_else(|| item.get("tag_name").and_then(|v| v.as_str()))
                    .unwrap_or("Unnamed");
                let tag = item.get("tag_name").and_then(|v| v.as_str()).unwrap_or("");
                let date = item
                    .get("published_at")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");

                result.push_str(&format!("- {} ({}) - {}\n", name, tag, date));
            }
            _ => {}
        }
    }

    Ok(result)
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_chat_tools() {
        let tools = get_chat_tools();
        assert_eq!(tools.len(), 2);

        let tool_names: Vec<_> = tools.iter().map(|t| t.name.as_str()).collect();
        assert!(tool_names.contains(&"fetch_webpage"));
        assert!(tool_names.contains(&"get_github_activity"));
    }

    #[test]
    fn test_get_builtin_tool_names() {
        let names = get_builtin_tool_names();
        assert!(names.contains("fetch_webpage"));
        assert!(names.contains("get_github_activity"));
        assert_eq!(names.len(), 2);
    }

    #[test]
    fn test_get_tools_json_without_mcp() {
        let tools = get_tools_json(&None, false);
        // Should have 2 built-in tools
        assert_eq!(tools.len(), 2);

        // Check tool structure
        let fetch_tool = tools
            .iter()
            .find(|t| t.get("name").and_then(|n| n.as_str()) == Some("fetch_webpage"));
        assert!(fetch_tool.is_some());
    }

    #[test]
    fn test_get_tools_json_with_web_search() {
        let tools = get_tools_json(&None, true);
        // Should have 2 built-in tools + web_search
        assert_eq!(tools.len(), 3);

        // Check web_search is included
        let web_search = tools
            .iter()
            .find(|t| t.get("name").and_then(|n| n.as_str()) == Some("web_search"));
        assert!(web_search.is_some());

        // Verify web_search has correct type
        let ws = web_search.unwrap();
        assert_eq!(
            ws.get("type").and_then(|t| t.as_str()),
            Some(WEB_SEARCH_TOOL_TYPE)
        );
    }

    #[test]
    fn test_extract_text_from_html() {
        let html = r#"
            <html>
            <head><style>body { color: red; }</style></head>
            <body>
                <script>alert('test');</script>
                <h1>Hello World</h1>
                <p>This is a &lt;test&gt; with &amp; entities.</p>
            </body>
            </html>
        "#;

        let text = extract_text_from_html(html);
        assert!(text.contains("Hello World"));
        assert!(text.contains("This is a <test> with & entities"));
        assert!(!text.contains("alert"));
        assert!(!text.contains("color: red"));
    }

    #[test]
    fn test_extract_card_content() {
        let cards_json = r#"[
            {"title": "First Card", "summary": "Summary 1", "topic": "Tech", "relevance": "high"},
            {"title": "Second Card", "summary": "Summary 2", "topic": "Science", "relevance": "medium"}
        ]"#;

        let content = extract_card_content(cards_json, 0);
        assert!(content.contains("First Card"));
        assert!(content.contains("Summary 1"));
        assert!(content.contains("Tech"));

        let content2 = extract_card_content(cards_json, 1);
        assert!(content2.contains("Second Card"));
        assert!(content2.contains("Science"));
    }

    #[test]
    fn test_extract_card_content_invalid_index() {
        let cards_json = r#"[{"title": "Only Card", "summary": "Only one"}]"#;

        let content = extract_card_content(cards_json, 5);
        assert!(content.contains("not found"));
    }

    #[test]
    fn test_extract_card_content_empty() {
        let content = extract_card_content("[]", 0);
        assert!(content.contains("No briefing cards available"));
    }

    #[test]
    fn test_build_system_prompt_with_tools() {
        let prompt = build_system_prompt("Test Briefing", "[]", 0, true);
        assert!(prompt.contains("Today's date is"));
        assert!(prompt.contains("tools to fetch real-time information"));
    }

    #[test]
    fn test_build_system_prompt_without_tools() {
        let prompt = build_system_prompt("Test Briefing", "[]", 0, false);
        assert!(prompt.contains("Today's date is"));
        assert!(!prompt.contains("tools to fetch real-time information"));
    }

    #[test]
    fn test_format_github_activity_commits() {
        let data = serde_json::json!([
            {
                "sha": "abc1234567890",
                "commit": {
                    "message": "Fix bug in parser",
                    "author": {
                        "name": "John Doe",
                        "date": "2025-01-01T12:00:00Z"
                    }
                }
            }
        ]);

        let result = format_github_activity(&data, "commits").unwrap();
        assert!(result.contains("Fix bug in parser"));
        assert!(result.contains("abc1234"));
        assert!(result.contains("John Doe"));
    }

    #[test]
    fn test_format_github_activity_issues() {
        let data = serde_json::json!([
            {
                "number": 42,
                "title": "Bug report",
                "state": "open",
                "user": { "login": "reporter" }
            }
        ]);

        let result = format_github_activity(&data, "issues").unwrap();
        assert!(result.contains("#42"));
        assert!(result.contains("Bug report"));
        assert!(result.contains("[open]"));
        assert!(result.contains("reporter"));
    }

    #[test]
    fn test_format_github_activity_empty() {
        let data = serde_json::json!([]);
        let result = format_github_activity(&data, "commits").unwrap();
        assert!(result.contains("No commits found"));
    }

    #[test]
    fn test_message_content_serialization() {
        let text_content = MessageContent::Text("Hello".to_string());
        let json = serde_json::to_string(&text_content).unwrap();
        assert_eq!(json, "\"Hello\"");

        let blocks_content = MessageContent::Blocks(vec![ContentBlock::Text {
            text: "Test".to_string(),
        }]);
        let json = serde_json::to_string(&blocks_content).unwrap();
        assert!(json.contains("\"type\":\"text\""));
    }

    #[test]
    fn test_content_block_tool_result() {
        let block = ContentBlock::ToolResult {
            tool_use_id: "tool_123".to_string(),
            content: "Success".to_string(),
            is_error: None,
        };
        let json = serde_json::to_string(&block).unwrap();
        assert!(json.contains("tool_result"));
        assert!(json.contains("tool_123"));
        assert!(!json.contains("is_error")); // Should be skipped when None
    }

    #[test]
    fn test_content_block_tool_result_with_error() {
        let block = ContentBlock::ToolResult {
            tool_use_id: "tool_456".to_string(),
            content: "Failed".to_string(),
            is_error: Some(true),
        };
        let json = serde_json::to_string(&block).unwrap();
        assert!(json.contains("is_error"));
        assert!(json.contains("true"));
    }
}
