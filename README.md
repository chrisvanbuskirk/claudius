# Claudius

A local AI research briefing agent for macOS that generates personalized daily research briefings using the Anthropic Agent SDK.

## Overview

Claudius is a self-hosted, privacy-first alternative to cloud-based briefing systems. It:
- Runs overnight and generates briefings by morning
- Uses your calendar, email, and GitHub to personalize research
- Keeps all data local (SQLite on your Mac)
- Integrates with Claude Desktop as an MCP Server

## Project Structure

```
claudius/
├── packages/
│   ├── cli/           # @claudius/cli - Command-line interface
│   ├── shared/        # @claudius/shared - Database, types, utilities
│   ├── frontend/      # @claudius/frontend - React UI (Vite + Tailwind)
│   └── mcp-server/    # @claudius/mcp-server - MCP server for Claude Desktop
├── src-tauri/         # Rust backend for Tauri desktop app
├── agent/             # Python AI agent using Anthropic SDK
└── ~/.claudius/       # Config & data (created at runtime)
    ├── interests.json
    ├── mcp-servers.json
    ├── preferences.json
    └── claudius.db
```

## Prerequisites

- Node.js >= 18.0.0
- npm >= 9.0.0
- Rust (for Tauri development)
- Python 3.9+ (for agent)
- Anthropic API key

## Quick Start

1. **Clone and install:**
```bash
git clone https://github.com/chrisvanbuskirk/claudius.git
cd claudius
npm install
```

2. **Set up environment:**
```bash
cp .env.example .env
# Add your ANTHROPIC_API_KEY to .env
```

3. **Install Python dependencies:**
```bash
cd agent && pip install -r requirements.txt && cd ..
```

4. **Initialize config:**
```bash
npm run cli -- config init
```

5. **Add your interests:**
```bash
npm run cli -- interests add "Swift development"
npm run cli -- interests add "Machine learning"
```

6. **Run research:**
```bash
npm run cli -- research --now
```

## Development

### CLI Development
```bash
cd packages/cli
npm run build
npm run start -- --help
```

### Frontend Development (React)
```bash
npm run dev
# Opens at http://localhost:5173
```

### Tauri Desktop App
```bash
# Requires Rust and Tauri CLI
npm run dev:tauri
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

Add Claudius as an MCP server in `~/.claude/claude.json`:

```json
{
  "mcpServers": {
    "claudius": {
      "command": "claudius-mcp",
      "env": {}
    }
  }
}
```

Then ask Claude: "What did Claudius research today?"

## Tech Stack

| Component | Technology |
|-----------|------------|
| CLI | TypeScript, Commander.js |
| Frontend | React 18, Vite, Tailwind CSS |
| Desktop | Tauri 2, Rust |
| Database | SQLite (better-sqlite3, rusqlite) |
| Agent | Python, Anthropic SDK |
| MCP Server | TypeScript, @modelcontextprotocol/sdk |

## License

MIT

## Author

Chris Van Buskirk
