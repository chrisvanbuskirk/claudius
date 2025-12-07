# Claudius - AI Research Assistant

## Project Overview

Claudius is a macOS desktop application that delivers personalized daily research briefings using the Anthropic Agent SDK. It runs a multi-agent research system that gathers information from configured topics and presents digestible briefing cards.

## Architecture

```
claudius/
├── agent/                    # Python research agent (Anthropic Agent SDK)
├── packages/
│   ├── frontend/            # React + Vite + Tailwind (Tauri webview)
│   ├── shared/              # Shared TypeScript types and database utilities
│   ├── cli/                 # Command-line interface
│   └── mcp-server/          # MCP server for tool integrations
├── src-tauri/               # Rust backend (Tauri 2.0)
└── package.json             # npm workspaces root
```

## Tech Stack

- **Frontend**: React 18, TypeScript, Vite, Tailwind CSS
- **Desktop**: Tauri 2.0 (Rust)
- **Agent**: Python with Anthropic Agent SDK
- **Database**: SQLite via sql.js (browser-compatible)
- **Build**: npm workspaces monorepo

## Development Commands

```bash
# Install dependencies
npm install

# Run Tauri desktop app in dev mode
cargo tauri dev

# Build production app
cargo tauri build

# Run frontend only (browser)
npm run dev -w @claudius/frontend

# Build all packages
npm run build

# Type check
npm run typecheck
```

## Key Files

- `src-tauri/tauri.conf.json` - Tauri configuration
- `src-tauri/src/main.rs` - Rust entry point
- `packages/frontend/src/App.tsx` - React app entry
- `packages/frontend/src/hooks/useTauri.ts` - Tauri IPC bridge with mock data fallback
- `packages/shared/src/db/` - Database operations
- `agent/research.py` - Main research agent logic

## Environment Variables

Copy `.env.example` to `.env` and configure:
- `ANTHROPIC_API_KEY` - Required for Claude API
- `GITHUB_TOKEN` - Optional, for GitHub MCP server
- `FIRECRAWL_API_KEY` - Optional, for web scraping

## Database Schema

SQLite database with tables:
- `topics` - Research topics/interests
- `briefings` - Generated research briefings
- `feedback` - User feedback on briefings
- `mcp_servers` - MCP server configurations
- `settings` - App settings

## Testing

Tests are organized by package:
- `packages/shared/__tests__/` - Database and utility tests
- `packages/frontend/__tests__/` - React component tests
- `packages/cli/__tests__/` - CLI command tests
- `agent/tests/` - Python agent tests

Run tests:
```bash
# JavaScript/TypeScript tests
npm test

# Python tests
cd agent && pytest
```

## Code Style

- TypeScript: Follow existing patterns, use strict mode
- React: Functional components with hooks
- Rust: Follow rustfmt conventions
- Python: Follow PEP 8, use type hints

## Git Workflow

- `main` - Protected, requires PR with 1 approval
- `develop` - Integration branch for features
- Feature branches from `develop`

## Common Tasks

### Adding a new Tauri command
1. Add command in `src-tauri/src/commands.rs`
2. Register in `src-tauri/src/main.rs`
3. Add TypeScript types in `packages/frontend/src/types/`
4. Use via `invoke()` in frontend

### Adding a new page
1. Create component in `packages/frontend/src/pages/`
2. Add route in `packages/frontend/src/App.tsx`
3. Add nav item in `packages/frontend/src/components/Sidebar.tsx`

### Modifying database schema
1. Update `packages/shared/src/db/schema.ts`
2. Update `src-tauri/src/schema.sql`
3. Add migration if needed
