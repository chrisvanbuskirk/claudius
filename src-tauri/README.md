# Claudius Tauri Backend

This directory contains the Rust backend for the Claudius desktop application built with Tauri.

## Structure

```
src-tauri/
├── Cargo.toml           # Rust dependencies and project configuration
├── build.rs             # Tauri build script
├── tauri.conf.json      # Tauri configuration
├── icons/               # Application icons
└── src/
    ├── main.rs          # Entry point, sets up Tauri app
    ├── db.rs            # SQLite database initialization and connection
    ├── schema.sql       # Database schema
    ├── commands.rs      # Tauri commands (IPC handlers)
    ├── research.rs      # Research agent (Anthropic API + tool use)
    ├── research_state.rs # Research state management
    ├── research_log.rs  # Research audit logging
    ├── mcp_client.rs    # MCP server client (JSON-RPC 2.0)
    ├── scheduler.rs     # Background research scheduler
    ├── notifications.rs # Desktop notifications
    └── tray.rs          # System tray integration
```

## Commands

The following Tauri commands are exposed to the frontend:

### Briefing Management
- `get_briefings(limit: Option<i32>)` - Get recent briefings
- `get_briefing(id: i64)` - Get a specific briefing
- `search_briefings(query: String)` - Search briefings by text

### Feedback
- `add_feedback(briefing_id: i64, card_index: i32, rating: i32, reason: Option<String>)` - Add user feedback

### User Interests
- `get_interests()` - Get user's research interests
- `add_interest(topic: String)` - Add a new interest
- `remove_interest(topic: String)` - Remove an interest

### Preferences
- `get_preferences()` - Get user preferences
- `update_preferences(preferences: Value)` - Update preferences

### Research
- `trigger_research()` - Manually trigger a research run
- `cancel_research()` - Cancel an in-progress research run
- `get_research_state()` - Get current research state

### Settings
- `get_settings()` - Get research settings
- `save_settings(settings)` - Save research settings (includes `enable_web_search`)

## Research Agent

The research agent (`research.rs`) uses the Anthropic API with `tool_use` to gather information on configured topics.

### Tool Architecture

The agent provides Claude with multiple tool types:

| Tool Type | Examples | Cost | Notes |
|-----------|----------|------|-------|
| **Built-in** | `github_search`, `fetch_webpage` | Free | Always available |
| **MCP Tools** | `brave_web_search`, `perplexity_search` | Varies | Configured in Settings |
| **Server Tools** | `web_search` | $0.01/search | Claude's built-in web search |

### Tool Selection Behavior

The agent uses `tool_choice: auto` (Anthropic API default), which means **Claude decides which tools to use**. When multiple similar tools are available (e.g., `brave_web_search` MCP and built-in `web_search`), Claude intelligently selects based on:

1. Tool names and descriptions
2. Task requirements
3. What it determines will work best

**Practical implications:**
- If Brave Search MCP is enabled, Claude typically prefers it (free)
- Built-in `web_search` is used when no MCP search tools are available
- This is cost-optimal: Claude uses free tools when available

### Web Search Options

| Option | Cost | Setup |
|--------|------|-------|
| **Brave Search MCP** | Free (2,000/month) | Configure in Settings → MCP Servers |
| **Built-in Web Search** | $0.01/search | Toggle in Settings → Research |

**Recommendation**: Configure Brave Search MCP for free searches. Enable built-in web search only as a fallback for users without MCP servers.

### Progress Events

The agent emits Tauri events for frontend progress tracking:

| Event | Description |
|-------|-------------|
| `research:started` | Research session begins |
| `research:topic_started` | Starting research on a topic |
| `research:topic_completed` | Topic research complete |
| `research:web_search` | Claude used built-in web search |
| `research:synthesis_started` | Synthesizing all research |
| `research:synthesis_completed` | Synthesis complete |
| `research:saving` | Saving to database |
| `research:completed` | Full session complete |
| `research:cancelled` | User cancelled |
| `research:heartbeat` | Keep-alive during long operations |

## Database

The SQLite database is stored at `~/.claudius/claudius.db` with the following tables:

- `briefings` - Stores generated briefings with metadata
- `briefing_cards` - Individual briefing cards (foreign key to briefings)
- `topics` - User's research topics (migrated from JSON)
- `feedback` - Stores user feedback on briefing cards
- `feedback_patterns` - Aggregated feedback patterns by topic
- `research_logs` - Audit log of research operations

## Configuration Files

Configuration is stored in `~/.claudius/`:

| File | Purpose |
|------|---------|
| `.env` | API keys (`ANTHROPIC_API_KEY=sk-ant-...`) |
| `mcp-servers.json` | MCP server configurations |
| `preferences.json` | Research settings (schedule, model, web search toggle) |
| `claudius.db` | SQLite database |
| `interests.json.migrated` | Backup of pre-migration topics |

## Building

```bash
# Check for errors
cargo check

# Build for development
cargo build

# Build for production
cargo build --release
```

## Running

The Tauri app is typically run through the npm workspace:

```bash
npm run tauri dev
```

## Dependencies

| Crate | Purpose |
|-------|---------|
| `tauri` | Core Tauri 2.0 framework |
| `tauri-plugin-shell` | Shell command execution (MCP servers) |
| `tauri-plugin-notification` | Desktop notifications |
| `rusqlite` | SQLite database |
| `reqwest` | HTTP client (Anthropic API) |
| `tokio` | Async runtime |
| `tokio-cron-scheduler` | Scheduled research runs |
| `serde/serde_json` | JSON serialization |
| `chrono` | Date/time handling |
| `dirs` | Cross-platform directory paths |
| `tracing` | Structured logging |
| `uuid` | Unique identifiers |

## Environment Variables

| Variable | Required | Description |
|----------|----------|-------------|
| `ANTHROPIC_API_KEY` | Yes | Claude API key (stored in `~/.claudius/.env`) |
| `GITHUB_TOKEN` | No | For GitHub MCP server |
| `BRAVE_API_KEY` | No | For Brave Search MCP server |

## API Reference

### Anthropic API Usage

The research agent calls the Anthropic Messages API with:
- **Model**: Configurable (default: `claude-haiku-4-5-20251001`)
- **Max tokens**: 8192
- **Tools**: Built-in + MCP + optional `web_search_20250305`
- **Tool choice**: `auto` (Claude decides which tools to use)

See [Anthropic Tool Use Docs](https://docs.anthropic.com/en/docs/agents-and-tools/tool-use/overview) for details on the `tool_choice` parameter and tool selection behavior.
