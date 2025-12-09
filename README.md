<p align="center">
  <img src="src-tauri/icons/128x128.png" alt="Claudius Icon" width="128" height="128">
</p>

<h1 align="center">Claudius</h1>

<p align="center">
  <strong>AI Research Assistant</strong><br>
  <em>Personalized daily briefings powered by Claude</em>
</p>

<p align="center">
  <a href="https://github.com/chrisvanbuskirk/claudius/actions/workflows/ci.yml">
    <img src="https://github.com/chrisvanbuskirk/claudius/actions/workflows/ci.yml/badge.svg" alt="CI">
  </a>
  <a href="https://github.com/chrisvanbuskirk/claudius/releases">
    <img src="https://img.shields.io/github/v/release/chrisvanbuskirk/claudius?include_prereleases" alt="Release">
  </a>
  <a href="LICENSE">
    <img src="https://img.shields.io/badge/license-MIT-blue.svg" alt="License">
  </a>
</p>

---

A native desktop application that generates personalized daily research briefings using Claude. Configure your topics of interest, and Claudius will research them overnight, delivering curated briefing cards by morning.

## Overview

Claudius is a self-hosted, privacy-first research assistant. It:
- Runs overnight and generates briefings by morning
- Uses Claude's agentic research capabilities to gather information on your topics
- Keeps all data local (SQLite database, no cloud storage)
- Supports macOS (Apple Silicon & Intel), Windows, and Linux
- Integrates with Claude Desktop as an MCP Server

## Features

- **Agentic Research**: Claude uses tools to actively research your topics (GitHub, web fetching, and more)
- **MCP Server Support**: Extend research capabilities with any MCP-compatible server (Brave Search, etc.)
- **Daily Briefings**: Wake up to curated research cards with summaries and sources
- **Feedback Learning**: Thumbs up/down on cards helps refine future research
- **Privacy First**: All data stays on your machine - no cloud storage required
- **Desktop App**: Native app built with Tauri 2.0 for macOS, Windows, and Linux
- **CLI**: Full command-line interface for power users
- **Claude Desktop Integration**: MCP server lets Claude access your briefings

## Screenshots

<p align="center">
  <img src="screenshots/home.png" alt="Home - Daily Briefings" width="800">
</p>
<p align="center"><em>Home screen with daily briefing cards featuring glassmorphism design</em></p>

<p align="center">
  <img src="screenshots/topics.png" alt="Topics Configuration" width="800">
</p>
<p align="center"><em>Configure research topics with descriptions and enable/disable toggles</em></p>

<p align="center">
  <img src="screenshots/research_settings.png" alt="Research Settings" width="800">
</p>
<p align="center"><em>Research settings: schedule, model selection, and manual trigger</em></p>

<p align="center">
  <img src="screenshots/mcps.png" alt="MCP Servers" width="800">
</p>
<p align="center"><em>MCP server configuration for enhanced research capabilities</em></p>

## Project Structure

```
claudius/
├── packages/
│   ├── cli/           # @claudius/cli - Command-line interface
│   ├── shared/        # @claudius/shared - Database, types, utilities
│   ├── frontend/      # @claudius/frontend - React UI (Vite + Tailwind)
│   └── mcp-server/    # @claudius/mcp-server - MCP server for Claude Desktop
├── src-tauri/         # Rust backend (Tauri + Research Agent)
│   └── src/
│       ├── main.rs       # App entry point
│       ├── commands.rs   # Tauri IPC commands
│       ├── research.rs   # Anthropic API research agent with tool_use
│       ├── mcp_client.rs # MCP server client (JSON-RPC 2.0)
│       ├── scheduler.rs  # Cron-based research scheduler
│       └── db.rs         # SQLite database layer
└── ~/.claudius/       # Config & data (created at runtime)
    ├── .env              # API key (ANTHROPIC_API_KEY)
    ├── mcp-servers.json
    ├── preferences.json
    └── claudius.db       # SQLite: briefings, topics, feedback, logs
```

## Data Storage

All Claudius data is stored locally in `~/.claudius/`:

| File | Contents |
|------|----------|
| `.env` | Your Anthropic API key (stored as `ANTHROPIC_API_KEY=sk-ant-...`) |
| `mcp-servers.json` | MCP server configurations and API keys |
| `preferences.json` | App settings (schedule, model preferences, etc.) |
| `claudius.db` | SQLite database with briefings, topics, feedback, and research logs |

**Note:** The `.env` file contains your API key in plaintext with restricted file permissions (owner read/write only on Unix systems). Keep this file secure and do not share it.

## Prerequisites

**For End Users (downloading releases):**
- macOS, Windows, or Linux
- Anthropic API key

**For Development:**
- Node.js >= 18.0.0
- npm >= 9.0.0
- Rust (for Tauri development)
- Anthropic API key

## Recommended: MCP Servers for Enhanced Research

**⚡ HIGHLY RECOMMENDED**: While Claudius works with just the Anthropic API key, adding MCP servers dramatically improves research quality by enabling real-time web search instead of relying solely on Claude's training data.

### Top Recommendations

| Server | Cost | Why Use It | Documentation |
|--------|------|------------|---------------|
| **Brave Search** | **Free tier available** | Real-time web search for current news, articles, and documentation | [Setup Guide](https://github.com/brave/brave-search-mcp-server) |
| **Perplexity** | Pay-as-you-go | AI-powered search that validates and enriches research | [Setup Guide](https://docs.perplexity.ai/guides/mcp-server) |

### How Research Works With MCP Servers

**Without MCP servers**: Claude relies on training data (may be outdated) and can only fetch individual URLs you configure.

**With Brave Search + Perplexity**: Claude actively searches the web for current information, discovers relevant URLs, and validates findings with AI search. This is especially critical for topics like:
- Technology news and product launches
- Software releases and updates
- Industry trends and events
- Recent research papers

### Quick Setup

1. **Get API Keys**:
   - Brave Search: Sign up at https://brave.com/search/api/ (free tier: 2,000 queries/month)
   - Perplexity: Sign up at https://www.perplexity.ai/settings/api (pay-as-you-go pricing)

2. **Configure in Claudius**:
   - Open Claudius Settings
   - Navigate to "MCP Servers"
   - Add your API keys for Brave Search and Perplexity
   - Enable both servers
   - Click "Save"

3. **Run Research**: Your next research run will automatically use these search tools for up-to-date information.

**Note**: The included example configuration at `mcp-servers.example.json` shows all available MCP servers, including GitHub, Fetch, and Memory servers.

## Quick Start

### For End Users

1. Download the latest release from [GitHub Releases](https://github.com/chrisvanbuskirk/claudius/releases)
2. Install and launch the app
3. Add your Anthropic API key in Settings
4. Configure your research topics
5. Click "Run Research Now" or wait for the scheduled run

### For Development

1. **Clone and install:**
```bash
git clone https://github.com/chrisvanbuskirk/claudius.git
cd claudius
npm install
npm run build  # Build all packages including MCP server
```

2. **Run the desktop app:**
```bash
npm run dev:tauri
```

3. **Add your API key in the Settings page, then configure topics and run research.**

4. **(Optional) Set up Claude Desktop integration** - See [Claude Desktop Integration](#claude-desktop-integration) below.

## Development

### Desktop App (Tauri)
```bash
# Requires Rust and Tauri CLI
# Install: https://tauri.app/v2/guides/getting-started/prerequisites

npm run dev:tauri
```

### Frontend Only (Browser)
```bash
npm run dev
# Opens at http://localhost:5173
```

### CLI Development
```bash
cd packages/cli
npm run build
npm run start -- --help
```

### Running Tests
```bash
# Run all tests (TypeScript + Rust)
npm test

# Run with coverage
npm run test:coverage

# Run Rust tests only
cd src-tauri && cargo test
```

### Build All
```bash
npm run build
```

## CLI Commands

> **Note:** The CLI is under development and not yet functional. Use the desktop app for all features.

```bash
# Coming soon...
claudius interests list
claudius research --now
claudius briefings list
```

## Claude Desktop Integration

> **Note:** This feature requires cloning the repo and building from source. It is not available with the standalone app download.

Claudius includes an MCP server that lets Claude Desktop access your briefings.

**Setup (requires Node.js):**

1. Clone and build the repo:
```bash
git clone https://github.com/chrisvanbuskirk/claudius.git
cd claudius
npm install && npm run build
```

2. Add to `~/Library/Application Support/Claude/claude_desktop_config.json` (macOS):

```json
{
  "mcpServers": {
    "claudius": {
      "command": "node",
      "args": ["/path/to/claudius/packages/mcp-server/dist/index.js"]
    }
  }
}
```

Replace `/path/to/claudius` with the actual path where you cloned the repo.

**Available Tools:**
| Tool | Description |
|------|-------------|
| `get_briefings` | Get recent briefings (last 7 days) |
| `search_briefings` | Search briefings by keyword |
| `get_briefing_detail` | Get full details of a specific briefing |
| `get_interests` | Get configured research topics |
| `get_research_stats` | Get total briefings, avg time, tokens used |

Then ask Claude: "What did Claudius research today?" or "Search my briefings for Swift"

## Tech Stack

| Component | Technology |
|-----------|------------|
| Frontend | React 18, TypeScript, Vite, Tailwind CSS, Framer Motion |
| Desktop | Tauri 2.0, Rust |
| Database | SQLite (rusqlite) |
| Research Agent | Rust, Anthropic API (tool_use), reqwest |
| MCP Client | Rust, JSON-RPC 2.0 over stdio |
| MCP Server | TypeScript, @modelcontextprotocol/sdk |
| Testing | Vitest, Cargo Test |
| CI/CD | GitHub Actions |

## Research Agent Architecture

Claudius uses an agentic research system built in Rust that leverages Claude's `tool_use` capability for intelligent information gathering.

### How It Works

```
┌─────────────────────────────────────────────────────────────────┐
│                    RESEARCH FLOW                                 │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  1. User triggers research (button click or scheduler)          │
│                                                                 │
│  2. Agent initializes:                                          │
│     - Loads topics from SQLite database                         │
│     - Connects to MCP servers from ~/.claudius/mcp-servers.json │
│     - Discovers available tools (built-in + MCP)                │
│                                                                 │
│  3. For each topic, enters agentic loop:                        │
│     ┌─────────────────────────────────────────────────────┐     │
│     │ POST api.anthropic.com/v1/messages                  │     │
│     │   - model: claude-sonnet-4-20250514                 │     │
│     │   - tools: [github_*, fetch_webpage, MCP tools...]  │     │
│     │   - messages: conversation history                  │     │
│     └──────────────────────┬──────────────────────────────┘     │
│                            │                                    │
│     ┌──────────────────────▼──────────────────────────┐         │
│     │ Claude responds with tool_use or final answer   │         │
│     └──────────────────────┬──────────────────────────┘         │
│                            │                                    │
│         If tool_use:       │        If end_turn:                │
│     ┌──────────────────────┴─────┐                              │
│     │ Route to appropriate tool: │   Parse JSON briefing cards  │
│     │  - Built-in → HTTP client  │   Save to SQLite database    │
│     │  - MCP → JSON-RPC to server│                              │
│     └─────────────┬──────────────┘                              │
│                   │                                             │
│     Append result to conversation, loop back (max 10 turns)     │
│                                                                 │
│  4. Return briefing cards to frontend                           │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

### Built-in Tools

The research agent includes these tools by default:

| Tool | Description |
|------|-------------|
| `github_search` | Search GitHub repositories |
| `github_get_repo` | Get repository details |
| `github_list_commits` | List recent commits |
| `github_list_pull_requests` | List open pull requests |
| `fetch_webpage` | Fetch and extract text from any URL |

### MCP Server Integration

You can extend the agent's capabilities by adding MCP servers in Settings. When research runs:

1. Agent spawns each enabled MCP server as a child process
2. Performs JSON-RPC 2.0 handshake via stdio
3. Discovers available tools via `tools/list`
4. Presents all tools (built-in + MCP) to Claude
5. Routes Claude's tool calls to the appropriate handler

MCP servers are configured via the Settings page in the desktop app. A sample configuration is also provided in [`mcp-servers.example.json`](mcp-servers.example.json) for reference.

**Recommended MCP servers:**

| Server | Package | API Key Required |
|--------|---------|------------------|
| [Brave Search](https://brave.com/search/api/) | `@modelcontextprotocol/server-brave-search` | Yes - `BRAVE_API_KEY` |
| [Perplexity](https://docs.perplexity.ai/guides/mcp-server) | `@perplexity-ai/mcp-server` | Yes - `PERPLEXITY_API_KEY` |
| [GitHub](https://github.com/modelcontextprotocol/servers) | `@modelcontextprotocol/server-github` | Yes - `GITHUB_PERSONAL_ACCESS_TOKEN` |
| [Fetch](https://www.npmjs.com/package/@modelcontextprotocol/server-fetch) | `@modelcontextprotocol/server-fetch` | No |

**To add an MCP server:**
1. Open Claudius Settings → MCP Servers
2. Click "Add Server" and configure the command, args, and environment variables
3. Enable the server and save
4. Run research to use the new tools

## Contributing

1. Fork the repository
2. Create a feature branch from `develop`
3. Make your changes
4. Run tests: `npm test`
5. Submit a pull request to `develop`

See [CLAUDE.md](CLAUDE.md) for development guidelines.

## License

MIT

## Author

Chris Van Buskirk
