//! MCP (Model Context Protocol) client for Claudius.
//!
//! This module handles communication with MCP servers to provide dynamic tool
//! capabilities for the research agent. It spawns server processes and communicates
//! via JSON-RPC 2.0 over stdio.

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::io::{BufRead, BufReader, Write};
use std::process::{Child, Command, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;
use tracing::{debug, info, warn};

/// Counter for generating unique JSON-RPC request IDs.
static REQUEST_ID: AtomicU64 = AtomicU64::new(1);

/// MCP server configuration as stored in the config file.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpServerConfig {
    pub id: String,
    pub name: String,
    pub enabled: bool,
    pub config: Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_used: Option<String>,
}

/// Tool definition from MCP server.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpTool {
    pub name: String,
    pub description: Option<String>,
    #[serde(rename = "inputSchema")]
    pub input_schema: Value,
}

/// A running MCP server connection.
pub struct McpConnection {
    pub server_name: String,
    pub server_id: String,
    child: Child,
    tools: Vec<McpTool>,
}

impl McpConnection {
    /// Get the available tools from this server.
    #[allow(dead_code)]
    pub fn tools(&self) -> &[McpTool] {
        &self.tools
    }
}

impl Drop for McpConnection {
    fn drop(&mut self) {
        // Clean up the child process - kill and wait to avoid zombie processes
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

/// MCP Client that manages connections to multiple MCP servers.
pub struct McpClient {
    connections: Vec<McpConnection>,
    /// Maps tool names to server index for routing tool calls.
    tool_routes: HashMap<String, usize>,
}

impl McpClient {
    /// Create a new MCP client by connecting to all enabled MCP servers.
    pub async fn connect(servers: Vec<McpServerConfig>) -> Result<Self, String> {
        let mut connections = Vec::new();
        let mut tool_routes = HashMap::new();

        for server in servers.into_iter().filter(|s| s.enabled) {
            match Self::connect_to_server(&server).await {
                Ok(conn) => {
                    let server_idx = connections.len();
                    // Register all tools from this server
                    for tool in &conn.tools {
                        // Prefix tool name with server name to avoid conflicts
                        let prefixed_name = format!("{}_{}", conn.server_name.replace(' ', "_").to_lowercase(), tool.name);
                        tool_routes.insert(prefixed_name, server_idx);
                        // Also register without prefix for direct calls
                        tool_routes.insert(tool.name.clone(), server_idx);
                    }
                    info!(
                        "Connected to MCP server '{}' with {} tools",
                        conn.server_name,
                        conn.tools.len()
                    );
                    connections.push(conn);
                }
                Err(e) => {
                    warn!("Failed to connect to MCP server '{}': {}", server.name, e);
                    // Continue with other servers
                }
            }
        }

        Ok(Self {
            connections,
            tool_routes,
        })
    }

    /// Connect to a single MCP server.
    async fn connect_to_server(server: &McpServerConfig) -> Result<McpConnection, String> {
        // Extract command and args from config
        let command = server.config.get("command")
            .and_then(|v| v.as_str())
            .ok_or_else(|| format!("MCP server '{}' missing 'command' in config", server.name))?;

        let args: Vec<&str> = server.config.get("args")
            .and_then(|v| v.as_array())
            .map(|arr| arr.iter().filter_map(|v| v.as_str()).collect())
            .unwrap_or_default();

        // Get environment variables
        let env: HashMap<String, String> = server.config.get("env")
            .and_then(|v| v.as_object())
            .map(|obj| {
                obj.iter()
                    .filter_map(|(k, v)| v.as_str().map(|s| (k.clone(), s.to_string())))
                    .collect()
            })
            .unwrap_or_default();

        info!("Starting MCP server '{}': {} {:?}", server.name, command, args);

        // Spawn the server process
        let mut cmd = Command::new(command);
        cmd.args(&args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        // Add environment variables
        for (key, value) in &env {
            cmd.env(key, value);
        }

        let mut child = cmd
            .spawn()
            .map_err(|e| format!("Failed to spawn MCP server '{}': {}", server.name, e))?;

        // Initialize the connection
        let stdin = child.stdin.as_mut()
            .ok_or_else(|| "Failed to get stdin".to_string())?;
        let stdout = child.stdout.take()
            .ok_or_else(|| "Failed to get stdout".to_string())?;

        // Send initialize request
        let init_request = json!({
            "jsonrpc": "2.0",
            "id": REQUEST_ID.fetch_add(1, Ordering::SeqCst),
            "method": "initialize",
            "params": {
                "protocolVersion": "2024-11-05",
                "capabilities": {
                    "roots": { "listChanged": true }
                },
                "clientInfo": {
                    "name": "claudius",
                    "version": env!("CARGO_PKG_VERSION")
                }
            }
        });

        Self::send_request(stdin, &init_request)?;

        // Read initialize response
        let mut reader = BufReader::new(stdout);
        let _init_response = Self::read_response(&mut reader)?;

        // Send initialized notification
        let initialized = json!({
            "jsonrpc": "2.0",
            "method": "notifications/initialized"
        });
        let stdin = child.stdin.as_mut().unwrap();
        Self::send_request(stdin, &initialized)?;

        // Request tools list
        let tools_request = json!({
            "jsonrpc": "2.0",
            "id": REQUEST_ID.fetch_add(1, Ordering::SeqCst),
            "method": "tools/list",
            "params": {}
        });
        Self::send_request(stdin, &tools_request)?;

        let tools_response = Self::read_response(&mut reader)?;

        // Parse tools from response
        let tools: Vec<McpTool> = tools_response
            .get("result")
            .and_then(|r| r.get("tools"))
            .and_then(|t| serde_json::from_value(t.clone()).ok())
            .unwrap_or_default();

        debug!("MCP server '{}' provides tools: {:?}",
            server.name,
            tools.iter().map(|t| &t.name).collect::<Vec<_>>()
        );

        // Reattach stdout for future reads
        child.stdout = Some(reader.into_inner());

        Ok(McpConnection {
            server_name: server.name.clone(),
            server_id: server.id.clone(),
            child,
            tools,
        })
    }

    /// Send a JSON-RPC request to the server.
    fn send_request(stdin: &mut impl Write, request: &Value) -> Result<(), String> {
        let request_str = serde_json::to_string(request)
            .map_err(|e| format!("Failed to serialize request: {}", e))?;

        debug!("MCP request: {}", request_str);

        writeln!(stdin, "{}", request_str)
            .map_err(|e| format!("Failed to write to MCP server: {}", e))?;

        stdin.flush()
            .map_err(|e| format!("Failed to flush to MCP server: {}", e))?;

        Ok(())
    }

    /// Read a JSON-RPC response from the server.
    fn read_response(reader: &mut BufReader<impl std::io::Read>) -> Result<Value, String> {
        let mut line = String::new();
        reader.read_line(&mut line)
            .map_err(|e| format!("Failed to read from MCP server: {}", e))?;

        debug!("MCP response: {}", line.trim());

        serde_json::from_str(&line)
            .map_err(|e| format!("Failed to parse MCP response: {}", e))
    }

    /// Get all available tools from all connected servers.
    pub fn get_all_tools(&self) -> Vec<McpToolWithServer> {
        let mut tools = Vec::new();

        for conn in &self.connections {
            for tool in &conn.tools {
                tools.push(McpToolWithServer {
                    server_name: conn.server_name.clone(),
                    server_id: conn.server_id.clone(),
                    tool: tool.clone(),
                });
            }
        }

        tools
    }

    /// Check if a tool is available.
    #[allow(dead_code)]
    pub fn has_tool(&self, tool_name: &str) -> bool {
        self.tool_routes.contains_key(tool_name)
    }

    /// Call a tool on the appropriate MCP server.
    pub fn call_tool(&mut self, tool_name: &str, arguments: Value) -> Result<Value, String> {
        // Find which server has this tool
        let server_idx = self.tool_routes.get(tool_name)
            .ok_or_else(|| format!("Unknown tool: {}", tool_name))?;

        let conn = self.connections.get_mut(*server_idx)
            .ok_or_else(|| "Server connection not found".to_string())?;

        // Find the actual tool name (might be prefixed)
        let actual_tool_name = conn.tools.iter()
            .find(|t| t.name == tool_name || format!("{}_{}", conn.server_name.replace(' ', "_").to_lowercase(), t.name) == tool_name)
            .map(|t| t.name.clone())
            .ok_or_else(|| format!("Tool '{}' not found on server", tool_name))?;

        info!("Calling MCP tool '{}' on server '{}'", actual_tool_name, conn.server_name);

        // Send tools/call request
        let request = json!({
            "jsonrpc": "2.0",
            "id": REQUEST_ID.fetch_add(1, Ordering::SeqCst),
            "method": "tools/call",
            "params": {
                "name": actual_tool_name,
                "arguments": arguments
            }
        });

        let call_start = std::time::Instant::now();

        let stdin = conn.child.stdin.as_mut()
            .ok_or_else(|| "Server stdin not available".to_string())?;
        Self::send_request(stdin, &request)?;

        let stdout = conn.child.stdout.take()
            .ok_or_else(|| "Server stdout not available".to_string())?;
        let mut reader = BufReader::new(stdout);

        let response = Self::read_response(&mut reader)?;
        conn.child.stdout = Some(reader.into_inner());

        let call_duration = call_start.elapsed();
        if call_duration > Duration::from_secs(30) {
            warn!(
                "MCP tool '{}' on server '{}' took {:.1}s (slow)",
                actual_tool_name, conn.server_name, call_duration.as_secs_f64()
            );
        } else {
            debug!(
                "MCP tool '{}' completed in {:.1}s",
                actual_tool_name, call_duration.as_secs_f64()
            );
        }

        // Check for error
        if let Some(error) = response.get("error") {
            let error_msg = error.get("message")
                .and_then(|m| m.as_str())
                .unwrap_or("Unknown MCP error");
            return Err(format!("MCP tool error: {}", error_msg));
        }

        // Extract result content
        let result = response.get("result")
            .cloned()
            .unwrap_or(json!(null));

        // If result has content array, extract text
        if let Some(content) = result.get("content").and_then(|c| c.as_array()) {
            let text_parts: Vec<&str> = content.iter()
                .filter_map(|c| {
                    if c.get("type").and_then(|t| t.as_str()) == Some("text") {
                        c.get("text").and_then(|t| t.as_str())
                    } else {
                        None
                    }
                })
                .collect();

            if !text_parts.is_empty() {
                return Ok(json!(text_parts.join("\n")));
            }
        }

        Ok(result)
    }

    /// Get the number of connected servers.
    pub fn server_count(&self) -> usize {
        self.connections.len()
    }

    /// Get the total number of available tools.
    pub fn tool_count(&self) -> usize {
        self.connections.iter().map(|c| c.tools.len()).sum()
    }
}

/// Tool information including which server it comes from.
#[derive(Debug, Clone)]
pub struct McpToolWithServer {
    pub server_name: String,
    #[allow(dead_code)]
    pub server_id: String,
    pub tool: McpTool,
}

impl McpToolWithServer {
    /// Convert to Anthropic API tool format.
    #[allow(dead_code)]
    pub fn to_anthropic_tool(&self) -> Value {
        json!({
            "name": self.tool.name,
            "description": self.tool.description.clone().unwrap_or_else(||
                format!("Tool from {} MCP server", self.server_name)
            ),
            "input_schema": self.tool.input_schema
        })
    }
}

/// Read MCP server configurations from the config file.
pub fn load_mcp_servers() -> Result<Vec<McpServerConfig>, String> {
    let home = dirs::home_dir()
        .ok_or_else(|| "Could not find home directory".to_string())?;

    let config_path = home.join(".claudius").join("mcp-servers.json");

    if !config_path.exists() {
        debug!("No MCP servers config file found");
        return Ok(Vec::new());
    }

    let content = std::fs::read_to_string(&config_path)
        .map_err(|e| format!("Failed to read MCP servers config: {}", e))?;

    #[derive(Deserialize)]
    struct ConfigFile {
        servers: Vec<McpServerConfig>,
    }

    let config: ConfigFile = serde_json::from_str(&content)
        .map_err(|e| format!("Failed to parse MCP servers config: {}", e))?;

    Ok(config.servers)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mcp_tool_to_anthropic() {
        let tool = McpToolWithServer {
            server_name: "github".to_string(),
            server_id: "abc123".to_string(),
            tool: McpTool {
                name: "search_repositories".to_string(),
                description: Some("Search GitHub repositories".to_string()),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "query": { "type": "string" }
                    },
                    "required": ["query"]
                }),
            },
        };

        let anthropic = tool.to_anthropic_tool();
        assert_eq!(anthropic["name"], "search_repositories");
        assert_eq!(anthropic["description"], "Search GitHub repositories");
        assert!(anthropic["input_schema"]["properties"]["query"].is_object());
    }

    #[test]
    fn test_load_mcp_servers_missing_file() {
        // This test just verifies the function doesn't panic on missing file
        // In CI the file won't exist
        let result = load_mcp_servers();
        assert!(result.is_ok());
    }
}
