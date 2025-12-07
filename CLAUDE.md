# Claudius - Development Guide

This file provides context for AI assistants (like Claude Code) working on this project.

## Project Overview

Claudius is a macOS desktop application that delivers personalized daily research briefings using the Anthropic Agent SDK. It runs a multi-agent research system that gathers information from configured topics and presents digestible briefing cards.

## Architecture

```
claudius/
├── agent/                    # Python research agent (Anthropic Agent SDK)
│   ├── research.py          # Main research logic
│   ├── briefing.py          # Briefing generation
│   ├── config.py            # Agent configuration
│   └── mcp_client.py        # MCP server client
├── packages/
│   ├── frontend/            # React + Vite + Tailwind (Tauri webview)
│   │   ├── src/
│   │   │   ├── components/  # Reusable React components
│   │   │   ├── pages/       # Route pages (Home, History, Settings)
│   │   │   ├── hooks/       # Custom React hooks
│   │   │   └── types/       # TypeScript types
│   │   └── vitest.config.ts # Test configuration
│   ├── shared/              # Shared TypeScript types and database utilities
│   │   ├── src/
│   │   │   ├── db/          # Database operations (sql.js)
│   │   │   ├── config/      # Configuration management
│   │   │   └── types/       # Shared type definitions
│   │   └── __tests__/       # Unit tests (75+ tests)
│   ├── cli/                 # Command-line interface
│   └── mcp-server/          # MCP server for Claude Desktop integration
├── src-tauri/               # Rust backend (Tauri 2.0)
│   ├── src/
│   │   ├── main.rs          # Entry point
│   │   ├── commands.rs      # Tauri commands (IPC)
│   │   └── db.rs            # Rust database layer
│   └── tauri.conf.json      # Tauri configuration
└── .github/workflows/       # CI/CD workflows
    ├── ci.yml               # Build, test, lint on push/PR
    └── release.yml          # Release builds on tags
```

## Tech Stack

- **Frontend**: React 18, TypeScript, Vite, Tailwind CSS, Lucide icons
- **Desktop**: Tauri 2.0 (Rust)
- **Agent**: Python with Anthropic Agent SDK
- **Database**: SQLite via sql.js (browser) and rusqlite (Rust)
- **Testing**: Vitest, React Testing Library
- **Build**: npm workspaces monorepo

## Development Commands

```bash
# Install dependencies
npm install

# Run Tauri desktop app in dev mode
cargo tauri dev
# OR
npm run dev:tauri

# Run frontend only (browser mode with mock data)
npm run dev

# Build production app
cargo tauri build

# Run all tests
npm test

# Run tests with coverage
npm run test:coverage

# Run specific package tests
npm test -w @claudius/shared
npm test -w @claudius/frontend

# Type check all packages
npm run build

# Python agent tests
cd agent && pytest
```

## Key Files

| File | Purpose |
|------|---------|
| `src-tauri/tauri.conf.json` | Tauri app configuration, permissions |
| `src-tauri/src/main.rs` | Rust entry point, command registration |
| `src-tauri/src/commands.rs` | IPC commands called from frontend |
| `packages/frontend/src/App.tsx` | React router, main layout |
| `packages/frontend/src/hooks/useTauri.ts` | Tauri IPC bridge with mock data fallback |
| `packages/shared/src/db/` | Database operations (briefings, feedback) |
| `packages/shared/src/db/schema.ts` | SQLite schema definition |
| `agent/research.py` | Main research agent logic |

## Environment Variables

Required in `.env` (copy from `.env.example`):
- `ANTHROPIC_API_KEY` - Required for Claude API access

Optional:
- `GITHUB_TOKEN` - For GitHub MCP server
- `FIRECRAWL_API_KEY` - For web scraping capabilities

## Database Schema

SQLite database (`~/.claudius/claudius.db`) with tables:
- `briefings` - Generated research briefings (id, date, title, cards JSON)
- `briefing_cards` - Individual briefing cards (foreign key to briefings)
- `feedback` - User feedback on briefings/cards (rating, reason)
- `feedback_patterns` - Aggregated feedback patterns by topic
- `research_logs` - Audit log of research operations

## Testing

Tests are organized by package:

```
packages/shared/__tests__/
├── db/
│   ├── index.test.ts      # Database lifecycle tests
│   ├── briefings.test.ts  # CRUD operations for briefings
│   └── feedback.test.ts   # Feedback operations
└── config/
    └── manager.test.ts    # Configuration management

packages/frontend/src/
└── *.test.tsx             # Component tests (to be added)
```

Run tests:
```bash
npm test                    # All tests
npm test -w @claudius/shared # Shared package only
```

## Code Style

- **TypeScript**: Strict mode, prefer explicit types
- **React**: Functional components with hooks, avoid class components
- **Rust**: Follow rustfmt conventions (`cargo fmt`)
- **Python**: PEP 8, use type hints, run `black` for formatting

## Git Workflow

- `main` - Protected branch, requires PR approval
- `develop` - Integration branch, merge features here
- Feature branches from `develop`, named `feature/<description>`

## Common Tasks

### Adding a new Tauri command

1. Add command function in `src-tauri/src/commands.rs`:
```rust
#[tauri::command]
pub async fn my_command(arg: String) -> Result<String, String> {
    Ok(format!("Hello {}", arg))
}
```

2. Register in `src-tauri/src/main.rs`:
```rust
.invoke_handler(tauri::generate_handler![
    commands::my_command,  // Add here
])
```

3. Call from frontend:
```typescript
import { invoke } from '@tauri-apps/api/core';
const result = await invoke<string>('my_command', { arg: 'world' });
```

### Adding a new page

1. Create component in `packages/frontend/src/pages/NewPage.tsx`
2. Add route in `packages/frontend/src/App.tsx`:
```tsx
<Route path="/new" element={<NewPage />} />
```
3. Add nav link in `packages/frontend/src/components/Sidebar.tsx`

### Adding a database operation

1. Add function in `packages/shared/src/db/` (e.g., `briefings.ts`)
2. Export from `packages/shared/src/db/index.ts`
3. Add tests in `packages/shared/__tests__/db/`
4. If needed in Tauri, mirror in `src-tauri/src/db.rs`

### Modifying database schema

1. Update `packages/shared/src/db/schema.ts`
2. Update `src-tauri/src/schema.sql` (if applicable)
3. Consider adding migration logic for existing databases

## Troubleshooting

### Blank screen in Tauri app
- Ensure using `HashRouter` not `BrowserRouter` (Tauri uses file:// protocol)
- Check browser console for errors via Tauri's devtools

### Tests failing with database errors
- Each test should use a fresh temp database path
- Call `closeDatabase()` in `afterEach` to reset singleton state

### TypeScript errors after schema changes
- Rebuild shared package: `npm run build -w @claudius/shared`
- Restart TypeScript server in your editor

## CI/CD

GitHub Actions runs on push to `main`/`develop` and on PRs:

1. **Lint & Type Check** - TypeScript compilation
2. **Unit Tests** - Vitest for all packages (75+ tests)
3. **Python Tests** - pytest for agent
4. **Build Tauri** - macOS (universal), Windows, Ubuntu

Release workflow triggers on version tags (`v*`) and creates draft releases with installers.
