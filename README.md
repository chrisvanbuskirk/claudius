# Claudius

[![CI](https://github.com/chrisvanbuskirk/claudius/actions/workflows/ci.yml/badge.svg)](https://github.com/chrisvanbuskirk/claudius/actions/workflows/ci.yml)

A local AI research briefing agent for macOS that generates personalized daily research briefings using the Anthropic Agent SDK.

## Overview

Claudius is a self-hosted, privacy-first alternative to cloud-based briefing systems. It:
- Runs overnight and generates briefings by morning
- Uses your calendar, email, and GitHub to personalize research
- Keeps all data local (SQLite on your Mac)
- Integrates with Claude Desktop as an MCP Server

## Features

- **Personalized Research**: Configure topics of interest and Claudius researches them for you
- **Daily Briefings**: Wake up to curated research cards with summaries and sources
- **Feedback Learning**: Thumbs up/down on cards helps refine future research
- **Privacy First**: All data stays on your Mac - no cloud storage required
- **Desktop App**: Native macOS app built with Tauri (supports Apple Silicon & Intel)
- **CLI**: Full command-line interface for power users
- **Claude Desktop Integration**: MCP server lets Claude access your briefings

## Screenshots

*Coming soon*

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
# Run all tests
npm test

# Run with coverage
npm run test:coverage

# Run Python tests
cd agent && pytest
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
| Desktop | Tauri 2.0, Rust |
| Database | SQLite (sql.js, rusqlite) |
| Agent | Python, Anthropic SDK |
| MCP Server | TypeScript, @modelcontextprotocol/sdk |
| Testing | Vitest, React Testing Library |
| CI/CD | GitHub Actions |

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
