//! Research agent for Claudius with tool_use support.
//!
//! Handles calling the Anthropic API to research topics and generate briefings.
//! Supports tool calling for external data sources via MCP servers and built-in tools.

use crate::mcp_client::{load_mcp_servers, McpClient};
use crate::research_log::{parse_api_error, ErrorCode, ResearchError, ResearchLogger};
use regex::Regex;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashSet;
use std::time::{Duration, Instant};
use tracing::{error, info, warn};

/// Maximum number of tool use iterations to prevent infinite loops.
const MAX_TOOL_ITERATIONS: usize = 10;

/// A single briefing card containing research on a topic.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BriefingCard {
    pub title: String,
    pub summary: String,
    pub sources: Vec<String>,
    pub suggested_next: Option<String>,
    pub relevance: String,
    pub topic: String,
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
#[derive(Debug, Serialize)]
struct AnthropicRequest {
    model: String,
    max_tokens: u32,
    messages: Vec<Message>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tools: Option<Vec<Tool>>,
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
            let owner = input.get("owner").and_then(|v| v.as_str()).ok_or("Missing owner")?;
            let repo = input.get("repo").and_then(|v| v.as_str()).ok_or("Missing repo")?;
            let activity_type = input.get("activity_type").and_then(|v| v.as_str()).ok_or("Missing activity_type")?;

            execute_github_activity(client, owner, repo, activity_type, github_token).await
        }
        "fetch_webpage" => {
            let url = input.get("url").and_then(|v| v.as_str()).ok_or("Missing url")?;
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
        "commits" => format!("https://api.github.com/repos/{}/{}/commits?per_page=10", owner, repo),
        "pulls" => format!("https://api.github.com/repos/{}/{}/pulls?state=all&per_page=10", owner, repo),
        "issues" => format!("https://api.github.com/repos/{}/{}/issues?state=all&per_page=10", owner, repo),
        "releases" => format!("https://api.github.com/repos/{}/{}/releases?per_page=5", owner, repo),
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
                Some(format!("- {} by {} ({}): {}", sha, author, &date[..10], message.lines().next().unwrap_or("")))
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

    // Truncate if too long
    let max_len = 8000;
    if text.len() > max_len {
        Ok(format!("{}...\n\n[Content truncated, {} total characters]", &text[..max_len], text.len()))
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
}

impl ResearchAgent {
    /// Create a new research agent.
    pub fn new(api_key: String, model: Option<String>) -> Self {
        // Try to read GitHub token from environment or config
        let github_token = std::env::var("GITHUB_TOKEN").ok().or_else(|| {
            // Try to read from ~/.claudius/.env
            let home = dirs::home_dir()?;
            let env_path = home.join(".claudius").join(".env");
            let content = std::fs::read_to_string(env_path).ok()?;
            content.lines()
                .find(|line| line.starts_with("GITHUB_TOKEN="))
                .map(|line| line.trim_start_matches("GITHUB_TOKEN=").trim().to_string())
        });

        // Track built-in tool names
        let builtin_tools: HashSet<String> = get_research_tools()
            .iter()
            .map(|t| t.name.clone())
            .collect();

        Self {
            client: Client::builder()
                .timeout(Duration::from_secs(120))
                .build()
                .expect("Failed to build HTTP client"),
            api_key,
            model: model.unwrap_or_else(|| "claude-haiku-4-5-20251001".to_string()),
            github_token,
            mcp_client: None,
            builtin_tools,
        }
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

    /// Get all available tools (built-in + MCP).
    fn get_all_tools(&self) -> Vec<Tool> {
        let mut tools = get_research_tools();

        // Add MCP tools
        if let Some(ref mcp_client) = self.mcp_client {
            for mcp_tool in mcp_client.get_all_tools() {
                tools.push(Tool {
                    name: mcp_tool.tool.name.clone(),
                    description: mcp_tool.tool.description.clone().unwrap_or_else(||
                        format!("Tool from {} MCP server", mcp_tool.server_name)
                    ),
                    input_schema: mcp_tool.tool.input_schema.clone(),
                });
            }
        }

        tools
    }

    /// Check if a tool is a built-in tool.
    fn is_builtin_tool(&self, name: &str) -> bool {
        self.builtin_tools.contains(name)
    }

    /// Run research on the given topics and generate a briefing.
    pub async fn run_research(&mut self, topics: Vec<String>) -> Result<ResearchResult, String> {
        let start_time = Instant::now();
        info!("Starting research on {} topics", topics.len());

        if topics.is_empty() {
            return Err("No topics provided for research".to_string());
        }

        // Initialize MCP connections (non-blocking, continues without MCP if it fails)
        if let Err(e) = self.init_mcp().await {
            warn!("MCP initialization failed (continuing without MCP): {}", e);
        }

        // Step 1: Research each topic with tool support
        let mut research_content = String::new();
        let mut total_tokens: u32 = 0;

        for (i, topic) in topics.iter().enumerate() {
            info!("Researching topic {}/{}: {}", i + 1, topics.len(), topic);

            match self.research_topic_with_tools(topic).await {
                Ok((content, tokens)) => {
                    research_content.push_str(&format!("\n## Topic {}: {}\n{}\n", i + 1, topic, content));
                    total_tokens += tokens;
                }
                Err(e) => {
                    error!("Error researching topic '{}': {}", topic, e);
                    research_content.push_str(&format!(
                        "\n## Topic {}: {}\nError: Could not research this topic.\n",
                        i + 1, topic
                    ));
                }
            }
        }

        // Step 2: Synthesize into briefing cards
        info!("Synthesizing research into briefing cards");
        let (cards, synthesis_tokens) = self.synthesize_briefing(&research_content).await
            .map_err(|e| {
                let _ = ResearchLogger::log_api_error("synthesis", &e);
                e.message
            })?;
        total_tokens += synthesis_tokens;

        let research_time_ms = start_time.elapsed().as_millis() as u64;

        let result = ResearchResult {
            date: chrono::Local::now().format("%Y-%m-%dT%H:%M:%S").to_string(),
            title: format!("Daily Briefing - {}", chrono::Local::now().format("%B %d, %Y")),
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

        Ok(result)
    }

    /// Research a single topic using Claude with tool support.
    async fn research_topic_with_tools(&mut self, topic: &str) -> Result<(String, u32), String> {
        // Build dynamic system prompt based on available tools
        let tools = self.get_all_tools();
        let tool_descriptions: Vec<String> = tools.iter()
            .map(|t| format!("- {}: {}", t.name, t.description))
            .collect();

        let system_prompt = format!(
            r#"You are a research assistant gathering information on topics of interest.
You have access to the following tools to fetch real-time data:
{}

Use these tools when they would provide valuable, current information about the topic.
For example, if researching "Rust programming", you might fetch recent activity from rust-lang/rust.
If researching a news topic, you might fetch relevant news articles.

After gathering information, provide a comprehensive research summary."#,
            tool_descriptions.join("\n")
        );

        let user_prompt = format!(
            r#"Research the following topic and provide:
1. Key recent developments (last 24-48 hours if available, otherwise recent news)
2. Why this might be relevant to someone interested in this topic
3. Actionable insights or next steps
4. Any credible sources you're aware of

Topic: {}

Use the available tools if they would help gather current information. Then provide a concise but informative research summary (2-3 paragraphs)."#,
            topic
        );
        let mut messages = vec![Message {
            role: "user".to_string(),
            content: MessageContent::Text(user_prompt),
        }];

        let mut total_tokens: u32 = 0;
        let mut iterations = 0;

        // Agentic loop - keep going until Claude stops calling tools
        loop {
            iterations += 1;
            if iterations > MAX_TOOL_ITERATIONS {
                warn!("Reached max tool iterations ({}), stopping", MAX_TOOL_ITERATIONS);
                break;
            }

            let request = AnthropicRequest {
                model: self.model.clone(),
                max_tokens: 2048,
                messages: messages.clone(),
                tools: Some(tools.clone()),
                system: Some(system_prompt.to_string()),
            };

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

            // Log successful API request
            let _ = ResearchLogger::log_api_request(topic, tokens as i64, api_duration);

            // Check if Claude wants to use tools
            let tool_uses: Vec<_> = response.content.iter()
                .filter(|c| c.content_type == "tool_use")
                .collect();

            if tool_uses.is_empty() || response.stop_reason.as_deref() == Some("end_turn") {
                // No more tool calls, extract the text response
                let text_content: String = response.content.iter()
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
            let assistant_blocks: Vec<ContentBlock> = response.content.iter()
                .map(|c| {
                    if c.content_type == "text" {
                        ContentBlock::Text { text: c.text.clone().unwrap_or_default() }
                    } else if c.content_type == "tool_use" {
                        ContentBlock::ToolUse {
                            id: c.id.clone().unwrap_or_default(),
                            name: c.name.clone().unwrap_or_default(),
                            input: c.input.clone().unwrap_or(json!({})),
                        }
                    } else {
                        ContentBlock::Text { text: "".to_string() }
                    }
                })
                .collect();

            messages.push(Message {
                role: "assistant".to_string(),
                content: MessageContent::Blocks(assistant_blocks),
            });

            // Execute tools and build results
            let mut tool_results: Vec<ContentBlock> = Vec::new();
            let empty_input = json!({});
            for tool_use in tool_uses {
                let tool_name = tool_use.name.as_ref().map(|s| s.as_str()).unwrap_or("");
                let tool_id = tool_use.id.as_ref().map(|s| s.as_str()).unwrap_or("");
                let tool_input = tool_use.input.as_ref().unwrap_or(&empty_input);
                let input_str = serde_json::to_string(tool_input).unwrap_or_default();

                info!("Executing tool: {} with input: {}", tool_name, tool_input);

                let tool_start = Instant::now();

                // Route to built-in tools or MCP client
                let is_mcp_tool = !self.is_builtin_tool(tool_name);
                let mcp_server_name: Option<String> = if is_mcp_tool {
                    // Find which server this tool belongs to
                    self.mcp_client.as_ref().and_then(|client| {
                        client.get_all_tools()
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
                    ).await
                } else if let Some(ref mut mcp_client) = self.mcp_client {
                    // Execute MCP tool
                    mcp_client.call_tool(tool_name, tool_input.clone())
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
        Ok(("Research completed (max iterations reached)".to_string(), total_tokens))
    }

    /// Send a request to the Anthropic API.
    async fn send_request(&self, request: &AnthropicRequest) -> Result<AnthropicResponse, ResearchError> {
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
                let err = ResearchError::new(ErrorCode::NetworkError, format!("HTTP request failed: {}", e));
                error!("Network error: {}", e);
                err
            })?;

        if !response.status().is_success() {
            let status = response.status().as_u16();
            let body = response.text().await.unwrap_or_default();
            let err = parse_api_error(status, &body);

            // Log the error
            error!("API error {}: {} (code: {:?})", status, err.message, err.code);
            if err.requires_user_action {
                error!("USER ACTION REQUIRED: {}", err.user_message);
            }

            return Err(err);
        }

        response
            .json()
            .await
            .map_err(|e| {
                ResearchError::new(ErrorCode::ParseError, format!("Failed to parse response: {}", e))
            })
    }

    /// Synthesize research results into briefing cards.
    async fn synthesize_briefing(
        &self,
        research_content: &str,
    ) -> Result<(Vec<BriefingCard>, u32), ResearchError> {
        let prompt = format!(
            r#"You are a research assistant creating a personalized daily briefing.
Synthesize the following research results into clear, actionable briefing cards.

{}

Generate briefing cards following these guidelines:

1. **Relevance**: Only include cards with medium or higher relevance
2. **Limit**: Maximum 10 cards total
3. **Priority**: Prioritize timely, actionable information

For each card, provide:
- **Title**: Clear, specific title (max 60 chars)
- **Summary**: Key findings and why it matters (2-4 sentences)
- **Sources**: List of source URLs (if available, otherwise empty array)
- **Suggested Next**: Optional next action or follow-up
- **Relevance**: "high", "medium", or "low"
- **Topic**: The original topic this relates to

Return ONLY valid JSON in this exact format:
{{
  "cards": [
    {{
      "title": "Card title",
      "summary": "Card summary with key findings and relevance.",
      "sources": ["https://example.com/source1"],
      "suggested_next": "Optional next action",
      "relevance": "high",
      "topic": "Original topic name"
    }}
  ]
}}

Return the JSON response now:"#,
            research_content
        );

        let request = AnthropicRequest {
            model: self.model.clone(),
            max_tokens: 4096,
            messages: vec![Message {
                role: "user".to_string(),
                content: MessageContent::Text(prompt),
            }],
            tools: None,
            system: None,
        };

        let response = self.send_request(&request).await?;

        let content = response.content.iter()
            .filter_map(|c| c.text.clone())
            .collect::<Vec<_>>()
            .join("\n");

        let tokens = response.usage.input_tokens + response.usage.output_tokens;

        // Parse the JSON response
        let cards = parse_briefing_response(&content)
            .map_err(|e| ResearchError::new(ErrorCode::ParseError, e))?;

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

    // Parse JSON
    let briefing_response: BriefingResponse = serde_json::from_str(json_str)
        .map_err(|e| format!("Failed to parse briefing JSON: {}. Response: {}", e, json_str))?;

    Ok(briefing_response.cards)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_briefing_response() {
        let response = r#"{"cards": [{"title": "Test", "summary": "Test summary", "sources": [], "suggested_next": null, "relevance": "high", "topic": "Test Topic"}]}"#;
        let cards = parse_briefing_response(response).unwrap();
        assert_eq!(cards.len(), 1);
        assert_eq!(cards[0].title, "Test");
    }

    #[test]
    fn test_parse_briefing_response_with_markdown() {
        let response = r#"```json
{"cards": [{"title": "Test", "summary": "Test summary", "sources": [], "suggested_next": null, "relevance": "high", "topic": "Test Topic"}]}
```"#;
        let cards = parse_briefing_response(response).unwrap();
        assert_eq!(cards.len(), 1);
    }

    #[test]
    fn test_parse_briefing_response_multiple_cards() {
        let response = r#"{"cards": [
            {"title": "Card 1", "summary": "Summary 1", "sources": ["https://example.com"], "suggested_next": "Read more", "relevance": "high", "topic": "Topic A"},
            {"title": "Card 2", "summary": "Summary 2", "sources": [], "suggested_next": null, "relevance": "medium", "topic": "Topic B"},
            {"title": "Card 3", "summary": "Summary 3", "sources": ["https://source1.com", "https://source2.com"], "suggested_next": "Follow up", "relevance": "low", "topic": "Topic A"}
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
{"cards": [{"title": "AI Developments", "summary": "Major advances in AI", "sources": [], "suggested_next": null, "relevance": "high", "topic": "Artificial Intelligence"}]}
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
    fn test_briefing_card_serialization() {
        let card = BriefingCard {
            title: "Test Title".to_string(),
            summary: "Test summary with details".to_string(),
            sources: vec!["https://example.com".to_string()],
            suggested_next: Some("Follow up action".to_string()),
            relevance: "high".to_string(),
            topic: "Test Topic".to_string(),
        };

        let json = serde_json::to_string(&card).unwrap();
        assert!(json.contains("Test Title"));
        assert!(json.contains("Test summary"));
        assert!(json.contains("https://example.com"));

        // Deserialize back
        let parsed: BriefingCard = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.title, card.title);
        assert_eq!(parsed.sources, card.sources);
    }

    #[test]
    fn test_research_result_serialization() {
        let result = ResearchResult {
            date: "2025-01-15".to_string(),
            title: "Daily Briefing - January 15, 2025".to_string(),
            cards: vec![
                BriefingCard {
                    title: "Card 1".to_string(),
                    summary: "Summary 1".to_string(),
                    sources: vec![],
                    suggested_next: None,
                    relevance: "high".to_string(),
                    topic: "Topic 1".to_string(),
                }
            ],
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
        let agent = ResearchAgent::new("test-api-key".to_string(), None);
        assert_eq!(agent.model, "claude-haiku-4-5-20251001");

        let agent_custom = ResearchAgent::new(
            "test-api-key".to_string(),
            Some("claude-opus-4-5-20251101".to_string())
        );
        assert_eq!(agent_custom.model, "claude-opus-4-5-20251101");
    }

    #[tokio::test]
    async fn test_run_research_empty_topics() {
        let mut agent = ResearchAgent::new("test-api-key".to_string(), None);
        let result = agent.run_research(vec![]).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("No topics provided"));
    }

    #[test]
    fn test_extract_text_from_html() {
        let html = r#"<html><head><title>Test</title></head><body><p>Hello World</p></body></html>"#;
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
}
