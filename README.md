<p align="center">
  <img src="src-tauri/icons/128x128.png" alt="Claudius Icon" width="128" height="128">
</p>

<h1 align="center">Claudius</h1>

<p align="center">
  <strong>AI Research Agent</strong><br>
  <em>Desktop app + CLI for personalized daily briefings powered by Claude</em>
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

A native desktop application and CLI that generates personalized daily research briefings using Claude. Configure your topics of interest, and Claudius will research them overnight, delivering curated briefing cards by morning.

## Overview

Claudius is a **DIY research tool for power users** who want to harness frontier AI models for automated intelligence gathering. Built for developers, researchers, and AI enthusiasts who have their own API keys and want full control over their research pipeline.

**What makes Claudius different:**
- **Bring your own keys** - Works with Anthropic, OpenAI, Brave, Firecrawl, and other APIs you already have
- **Two research modes** - Standard mode for quick daily updates, Deep Research mode for comprehensive multi-source analysis
- **Fully local** - All data stays on your machine in SQLite; no cloud accounts or subscriptions
- **Extensible via MCP** - Add any MCP-compatible server to expand research capabilities
- **Desktop + CLI** - Use the app for browsing, CLI for automation and scripting

Claudius is a self-hosted, privacy-first research assistant available as both a **desktop app** and **command-line interface**. It:
- Runs overnight and generates briefings by morning (or on-demand via CLI)
- Uses Claude's agentic research capabilities to gather information on your topics
- Keeps all data local (SQLite database, no cloud storage)
- Supports macOS (Apple Silicon & Intel), Windows, and Linux
- Works headless via CLI for automation (cron, Shortcuts, scripts)
- Integrates with Claude Desktop as an MCP Server

## Features

- **Agentic Research**: Claude uses tools to actively research your topics (GitHub, web fetching, and more)
- **Two Research Modes**: Standard mode (Brave/Perplexity search) or Deep Research mode (Firecrawl for comprehensive extraction)
- **MCP Server Support**: Extend research capabilities with any MCP-compatible server (Brave Search, Firecrawl, etc.)
- **AI-Generated Images**: Optional DALL-E integration generates unique header images for each briefing card
- **Condensed Briefings**: Option to combine all topics into a single comprehensive daily briefing
- **Smart Deduplication**: Automatically avoids repeating recent topics unless there's significant new information
- **Daily Briefings**: Wake up to curated research cards with summaries and sources
- **Per-Card Chat**: Chat with Claude about any briefing card for deeper exploration
- **Print Support**: Print individual briefing cards with optimized formatting
- **Bookmarks**: Save important cards for later reference (bookmarked cards are never auto-deleted)
- **Storage Management**: Auto-delete old briefings after a configurable retention period, or manually delete individual cards
- **Privacy First**: All data stays on your machine - no cloud storage required
- **Auto-Update**: Automatic update detection with in-app notifications and one-click install
- **Desktop App**: Native app built with Tauri 2.0 for macOS, Windows, and Linux
- **CLI**: Full command-line interface for power users and automation
- **Claude Desktop Integration**: MCP server lets Claude access your briefings

## How It Works: Agentic Architecture

Claudius implements Claude's **agentic tool-use pattern** in native Rust. This is the same pattern used by the Claude Agent SDK, but built directly on the Anthropic Messages API for maximum performance and minimal dependencies.

### The Agentic Loop

```
┌─────────────────────────────────────────────────────────────────┐
│  1. User configures topics: "AI News", "Rust Updates"           │
└─────────────────────────────────────────────────────────────────┘
                              ↓
┌─────────────────────────────────────────────────────────────────┐
│  2. Claudius sends prompt + available tools to Claude API       │
│     Tools: fetch_webpage, brave_search, github_search, etc.     │
└─────────────────────────────────────────────────────────────────┘
                              ↓
┌─────────────────────────────────────────────────────────────────┐
│  3. Claude responds with tool_use: "I want to search for..."    │
└─────────────────────────────────────────────────────────────────┘
                              ↓
┌─────────────────────────────────────────────────────────────────┐
│  4. Claudius executes the tool and returns tool_result          │
└─────────────────────────────────────────────────────────────────┘
                              ↓
┌─────────────────────────────────────────────────────────────────┐
│  5. Claude processes results, may request more tools            │
│     (Loop continues until Claude has enough information)        │
└─────────────────────────────────────────────────────────────────┘
                              ↓
┌─────────────────────────────────────────────────────────────────┐
│  6. Claude synthesizes findings into briefing cards             │
└─────────────────────────────────────────────────────────────────┘
```

### Available Tools

| Tool | Source | Description |
|------|--------|-------------|
| `fetch_webpage` | Built-in | Fetches and parses web page content |
| `brave_search` | MCP Server | Real-time web search (recommended) |
| `perplexity` | MCP Server | AI-powered search validation |
| `firecrawl_search` | MCP Server | Search with content extraction (Deep Research mode) |
| `firecrawl_scrape` | MCP Server | Full page extraction, handles JS (Deep Research mode) |
| `firecrawl_extract` | MCP Server | Structured data extraction with LLM (Deep Research mode) |
| `github_search` | MCP Server | Search GitHub repos, issues, PRs |
| `github_get_repo` | MCP Server | Get repository details |
| `web_search` | Claude API | Claude's native web search ($0.01/search) |

### Why This Matters

- **Autonomous Research**: Claude decides what to search, what URLs to fetch, and when it has enough information
- **Dynamic Tool Selection**: Claude chooses the best tool for each query (search vs direct fetch)
- **Iterative Refinement**: Claude can search, read results, then search again with better queries
- **No SDK Required**: Native Rust implementation means no Python/Node.js dependencies

The research agent implementation lives in `src-tauri/src/research.rs` and handles the full agentic loop, tool execution, and synthesis phases.

## Screenshots

<p align="center">
  <img src="screenshots/home.jpg" alt="Home - Daily Briefings" width="800">
</p>
<p align="center"><em>Home screen with daily briefing cards featuring AI-generated images and glassmorphism design</em></p>

<p align="center">
  <img src="screenshots/topic.jpg" alt="Topics Configuration" width="800">
</p>
<p align="center"><em>Configure research topics with descriptions and enable/disable toggles</em></p>

<p align="center">
  <img src="screenshots/research.jpg" alt="Research Settings" width="800">
</p>
<p align="center"><em>Research settings: model selection, research mode, and image generation</em></p>

<p align="center">
  <img src="screenshots/mcps.jpg" alt="MCP Servers" width="800">
</p>
<p align="center"><em>MCP server configuration for enhanced research capabilities</em></p>

<p align="center">
  <img src="screenshots/chat.jpg" alt="Chat with Cards" width="800">
</p>
<p align="center"><em>Chat with Claude about any briefing card for deeper exploration</em></p>

<p align="center">
  <img src="screenshots/bookmarks.jpg" alt="Bookmarks" width="800">
</p>
<p align="center"><em>Save and organize your favorite briefing cards</em></p>

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
│       ├── updater.rs    # Auto-update functionality
│       └── db.rs         # SQLite database layer
└── ~/.claudius/       # Config & data (created at runtime)
    ├── .env              # API key (ANTHROPIC_API_KEY)
    ├── mcp-servers.json
    ├── preferences.json
    └── claudius.db       # SQLite: briefings, topics, feedback, logs
```

## Data Storage

All Claudius data is stored locally in `~/.claudius/`:

| File/Directory | Contents |
|----------------|----------|
| `.env` | Your Anthropic and OpenAI API keys |
| `mcp-servers.json` | MCP server configurations and API keys |
| `preferences.json` | App settings (schedule, model preferences, research mode, etc.) |
| `claudius.db` | SQLite database with briefings, topics, bookmarks, chat messages, and research logs |
| `images/` | DALL-E generated header images for briefing cards (if enabled) |

**Note:** The `.env` file contains your API keys in plaintext with restricted file permissions (owner read/write only on Unix systems). Keep this file secure and do not share it.

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

### Alternative: Claude Web Search

Claude also offers a **built-in web search tool** that doesn't require any MCP server setup. This can be enabled directly in Settings.

| Option | Cost | Setup Required |
|--------|------|----------------|
| **Claude Web Search** | $0.01 per search | None - built-in |
| **Brave Search MCP** | Free tier (2,000/month) | MCP server config |
| **Perplexity MCP** | Pay-as-you-go | MCP server config |

**To enable Claude Web Search:**
1. Open Claudius Settings → Research
2. Toggle "Enable Claude Web Search"
3. Run research - Claude will automatically use web search when needed

**Cost Considerations:**
- Each web search costs approximately $0.01
- A typical research session may use 3-10 searches per topic
- For heavy usage, consider using Brave Search MCP (free tier: 2,000 queries/month)

You can use Claude Web Search alongside MCP servers - Claude will intelligently choose the best tool for each query.

## Research Modes

Claudius supports two research modes, configurable in Settings:

### Standard Mode (Default)

Uses Brave Search and/or Perplexity for real-time web search, combined with built-in page fetching. This is fast and cost-effective for daily briefings.

**Tool Priority:**
1. `brave_search` or `perplexity_search` - Primary real-time web search
2. `fetch_webpage` - Reads promising URLs discovered by search
3. `get_github_activity` - For open source project activity
4. Claude's built-in `web_search` (if enabled, $0.01/search)

**Best for:** Daily news, quick updates, monitoring topics

### Deep Research Mode (Firecrawl)

Uses [Firecrawl](https://www.firecrawl.dev/) MCP for comprehensive web extraction. Firecrawl can crawl entire sites, extract structured data, and handle JavaScript-heavy pages that simple fetching would miss.

**Tool Priority:**
1. `firecrawl_search` - Search with built-in content extraction
2. `firecrawl_extract` - Deep structured extraction with LLM prompts
3. `firecrawl_scrape` - Full page content extraction (handles JS-heavy sites)
4. `firecrawl_map` - Discover related URLs on a site
5. `get_github_activity` - Still available for GitHub activity

**Best for:** In-depth research, complex topics, sites with dynamic content

### Setting Up Firecrawl

1. **Get API Key**: Sign up at [firecrawl.dev](https://www.firecrawl.dev/) (free tier available)

2. **Configure MCP Server**: Add to your `~/.claudius/mcp-servers.json`:
```json
{
  "firecrawl": {
    "command": "npx",
    "args": ["-y", "firecrawl-mcp"],
    "env": {
      "FIRECRAWL_API_KEY": "your-api-key-here"
    },
    "enabled": true
  }
}
```

3. **Select Mode**: In Settings → Research, select "Deep Research (Firecrawl)"

**Note:** When Deep Research mode is selected, Standard mode tools (Brave, Perplexity, fetch_webpage) are automatically excluded to prevent tool confusion.

### Firecrawl Agent Rate Limiting

Firecrawl provides a powerful `firecrawl_agent` tool that can autonomously crawl and analyze multiple pages. However, this tool is credit-intensive:

| Usage | Cost |
|-------|------|
| First 5 calls/day | **Free** |
| After 5 calls/day | 200-600 credits per call |

**Rate Limiting (Enabled by Default):**

Claudius automatically limits `firecrawl_agent` to 5 calls per day to keep you within the free tier. When the limit is reached, Claude is instructed to use the cheaper alternatives (`firecrawl_search`, `firecrawl_scrape`, `firecrawl_extract`).

**To disable rate limiting** (if you want unlimited agent calls and are willing to pay credits):
1. Open Settings → Research
2. Select "Deep Research (Firecrawl)" mode
3. Toggle off "Limit Firecrawl Agent (5/day)"

**Note:** The daily count resets at midnight local time. You can check today's usage in the research logs.

## AI-Generated Images (DALL-E)

Claudius can generate unique header images for each briefing card using OpenAI's DALL-E 3 API.

### Setup

1. **Get OpenAI API Key**: Sign up at [platform.openai.com](https://platform.openai.com/)
2. **Add Key in Settings**: Settings → API Keys → OpenAI API Key
3. **Enable Image Generation**: Settings → Research → Enable Image Generation

### How It Works

- During synthesis, Claude generates a short visual description for each card (e.g., "futuristic circuit board with glowing pathways")
- After research completes, DALL-E generates a 1792x1024 landscape image for each card
- Images are stored locally in `~/.claudius/images/`
- Images display as headers on briefing cards

### Cost

- DALL-E 3 costs approximately $0.04-0.08 per image
- A typical research session with 5-7 cards costs ~$0.30-0.50 for images
- Disable in Settings if you prefer text-only briefings

## Condensed Briefings

By default, Claudius generates one briefing card per topic. Enable **Condensed Briefings** to combine all topics into a single comprehensive daily briefing.

### Standard Mode (Multiple Cards)
- One card per topic (e.g., 5 topics = 5 cards)
- Each card focuses on a specific topic area
- Quick scanning of individual topics

### Condensed Mode (Single Card)
- All topics combined into one comprehensive briefing
- Cross-topic analysis and connections highlighted
- Longer, more narrative format (400+ words)
- Better for reading as a "daily digest"

**Enable in:** Settings → Research → Condensed Briefings

## Smart Deduplication

Claudius automatically tracks recent briefings and avoids repeating the same information:

- Compares new research against the last 3 days of briefings
- Uses similarity scoring to detect duplicate topics
- Only generates new cards when there's significant new information
- Configurable threshold in Settings (default: 70% similarity = duplicate)

This prevents your briefings from becoming repetitive when topics don't have daily updates.

## Installation

### Download (Recommended)

Download the latest release from [GitHub Releases](https://github.com/chrisvanbuskirk/claudius/releases):

| Platform | File |
|----------|------|
| macOS (Apple Silicon) | `Claudius_x.x.x_aarch64.dmg` |
| Linux (Debian/Ubuntu) | `Claudius_x.x.x_amd64.deb` |
| Linux (Fedora/RHEL) | `Claudius-x.x.x-1.x86_64.rpm` |
| Linux (AppImage) | `Claudius_x.x.x_amd64.AppImage` |

### Homebrew (macOS)

```bash
brew tap chrisvanbuskirk/claudius
brew install --cask claudius
```

Update with: `brew upgrade --cask claudius`

### Auto-Updates

Claudius automatically checks for updates on startup:

1. **Detection**: When a new version is available, you'll see an in-app banner
2. **Background Download**: Updates download automatically in the background
3. **Notification**: A native notification appears when the download completes
4. **One-Click Install**: Click "Restart to Update" to apply the update

Updates are cryptographically signed and verified before installation.

## Quick Start

### For End Users

1. Install Claudius using one of the methods above
2. Launch the app
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

The Rust CLI provides full access to Claudius features from the terminal. Install it from Settings → "Install CLI" in the desktop app, which creates a symlink to `/usr/local/bin/claudius`.

### Topics
```bash
claudius topics list              # List all research topics
claudius topics add "AI News"     # Add a new topic
claudius topics add "Rust" --description "Rust programming language updates"
claudius topics remove <id|name>  # Remove a topic
claudius topics enable <id|name>  # Enable a topic
claudius topics disable <id|name> # Disable a topic
```

### Research
```bash
claudius research now             # Run research immediately (shows live progress)
claudius research now --topic "AI News"  # Research specific topic only
claudius research now --verbose   # Show topics being researched
claudius research status          # Check if research is running
claudius research logs            # View recent research logs
claudius research logs --errors   # View only error logs
```

### Briefings
```bash
claudius briefings list           # List recent briefings
claudius briefings list --limit 5 # Limit results
claudius briefings show <id>      # Show full briefing with cards
claudius briefings search "Claude" # Search briefings
claudius briefings export <id>    # Export as markdown
claudius briefings export <id> --format json  # Export as JSON
```

### MCP Servers
```bash
claudius mcp list                 # List configured MCP servers
claudius mcp add "Brave" --command "npx" --args "-y @anthropic/mcp-server-brave-search"
claudius mcp remove <id|name>     # Remove server
claudius mcp enable <id|name>     # Enable server
claudius mcp disable <id|name>    # Disable server
claudius mcp test <name>          # Test server connection
```

### Configuration
```bash
claudius config show              # Show all settings
claudius config set model claude-sonnet-4-5-20250929  # Change model
claudius config api-key show      # Check if API key is set
claudius config api-key set <key> # Set API key
claudius config api-key clear     # Remove API key
```

### Housekeeping
```bash
claudius housekeeping status      # Show storage stats (briefings, cards, db size)
claudius housekeeping run         # Run cleanup based on retention settings
claudius housekeeping run --dry-run  # Preview what would be deleted
claudius housekeeping optimize    # Optimize database (VACUUM)
```

### JSON Output
Add `--json` to any command for machine-readable output:
```bash
claudius topics list --json
claudius briefings list --json
claudius research status --json
```

### Automation & Scheduling

The CLI enables flexible scheduling without keeping the app running. Your briefings are saved to the shared database, so they appear in the desktop app whenever you open it.

**Cron (Unix/macOS/Linux):**
```bash
# Edit crontab
crontab -e

# Run research at 6:30 AM every weekday
30 6 * * 1-5 /usr/local/bin/claudius research now

# Run twice daily (morning and evening)
30 6,18 * * * /usr/local/bin/claudius research now

# Run every 4 hours
0 */4 * * * /usr/local/bin/claudius research now

# Run housekeeping weekly (Sunday at midnight)
0 0 * * 0 /usr/local/bin/claudius housekeeping run
```

**macOS Shortcuts:**
1. Open Shortcuts app
2. Create new shortcut with "Run Shell Script" action
3. Enter: `/usr/local/bin/claudius research now`
4. Trigger via: time of day, Focus mode, location, or manually from menu bar

**launchd (macOS - survives reboots):**
```xml
<!-- Save as ~/Library/LaunchAgents/com.claudius.research.plist -->
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>com.claudius.research</string>
    <key>ProgramArguments</key>
    <array>
        <string>/usr/local/bin/claudius</string>
        <string>research</string>
        <string>now</string>
    </array>
    <key>StartCalendarInterval</key>
    <dict>
        <key>Hour</key>
        <integer>6</integer>
        <key>Minute</key>
        <integer>30</integer>
    </dict>
</dict>
</plist>
```
Then load with: `launchctl load ~/Library/LaunchAgents/com.claudius.research.plist`

**Why use CLI scheduling?**
- Works even when your Mac wakes from sleep
- More flexible triggers (location, Focus modes, events)
- Can run on headless servers or in CI/CD pipelines
- Briefings appear in the desktop app whenever you open it

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
│  1. User triggers research (button click or CLI)                 │
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

| Server | Package | API Key Required | Mode |
|--------|---------|------------------|------|
| [Brave Search](https://brave.com/search/api/) | `@brave/brave-search-mcp-server` | Yes - `BRAVE_API_KEY` | Standard |
| [Perplexity](https://docs.perplexity.ai/guides/mcp-server) | `@anthropic-ai/perplexity-search` | Yes - `PERPLEXITY_API_KEY` | Standard |
| [Firecrawl](https://www.firecrawl.dev/) | `firecrawl-mcp` | Yes - `FIRECRAWL_API_KEY` | Deep Research |
| [GitHub](https://github.com/modelcontextprotocol/servers) | `@modelcontextprotocol/server-github` | Yes - `GITHUB_PERSONAL_ACCESS_TOKEN` | Both |
| [Fetch](https://www.npmjs.com/package/@modelcontextprotocol/server-fetch) | `@modelcontextprotocol/server-fetch` | No | Standard |

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
