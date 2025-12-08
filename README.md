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

| Home | Settings | Menu Bar |
|:----:|:--------:|:--------:|
| View daily briefings | Configure topics & schedule | Quick access popover |

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
    ├── topics.json
    ├── mcp-servers.json
    ├── settings.json
    └── claudius.db
```

## Data Storage

All Claudius data is stored locally in `~/.claudius/`:

| File | Contents |
|------|----------|
| `.env` | Your Anthropic API key (stored as `ANTHROPIC_API_KEY=sk-ant-...`) |
| `topics.json` | Your configured research topics |
| `mcp-servers.json` | MCP server configurations and API keys |
| `settings.json` | App settings (schedule, model preferences, etc.) |
| `claudius.db` | SQLite database with briefings, feedback, and research logs |

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

```bash
# Interest management
claudius interests add <topic>
claudius interests list
claudius interests remove <topic>

# Research
claudius research --now
claudius research --now --topic "Swift 6" --depth deep

# Briefings
claudius briefings list
claudius briefings search "Swift"
claudius briefings export <id> --format markdown

# MCP Servers
claudius mcp list
claudius mcp add <name>
claudius mcp enable <name>

# Configuration
claudius config show
claudius config init
claudius config set research.schedule "0 6 * * *"
```

## Claude Desktop Integration

Claudius includes an MCP server that lets Claude Desktop access your briefings. Add it to `~/Library/Application Support/Claude/claude_desktop_config.json` (macOS):

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
| CLI | TypeScript, Commander.js |
| Frontend | React 18, Vite, Tailwind CSS |
| Desktop | Tauri 2.0, Rust |
| Database | SQLite (sql.js, rusqlite) |
| Research Agent | Rust, Anthropic API (tool_use) |
| MCP Client | Rust, JSON-RPC 2.0 over stdio |
| MCP Server | TypeScript, @modelcontextprotocol/sdk |
| Testing | Vitest, React Testing Library, Cargo Test |
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
│     - Loads topics from ~/.claudius/topics.json                 │
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

A sample configuration is provided in [`mcp-servers.example.json`](mcp-servers.example.json). Copy it to your config directory:

```bash
cp mcp-servers.example.json ~/.claudius/mcp-servers.json
```

**Pre-configured servers (disabled by default):**

| Server | Package | API Key Required |
|--------|---------|------------------|
| [Brave Search](https://brave.com/search/api/) | `@modelcontextprotocol/server-brave-search` | Yes - `BRAVE_API_KEY` |
| [Perplexity](https://docs.perplexity.ai/guides/mcp-server) | `@perplexity-ai/mcp-server` | Yes - `PERPLEXITY_API_KEY` |
| [GitHub](https://github.com/modelcontextprotocol/servers) | `@modelcontextprotocol/server-github` | Yes - `GITHUB_PERSONAL_ACCESS_TOKEN` |
| [Fetch](https://www.npmjs.com/package/@modelcontextprotocol/server-fetch) | `@modelcontextprotocol/server-fetch` | No |
| [Memory](https://www.npmjs.com/package/@modelcontextprotocol/server-memory) | `@modelcontextprotocol/server-memory` | No |

**To enable a server:**
1. Edit `~/.claudius/mcp-servers.json`
2. Add your API key(s) to the `env` section
3. Set `"enabled": true`
4. Restart Claudius or run research

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
