# Claudius - Development Guide

This file provides context for AI assistants (like Claude Code) working on this project.

## Project Overview

Claudius is a macOS desktop application that delivers personalized daily research briefings using Claude. It runs a native Rust research agent that calls the Anthropic API to gather information on configured topics and presents digestible briefing cards. The app is fully self-contained - no external dependencies like Python required for end users.

## Architecture

```
claudius/
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
├── src-tauri/               # Rust backend (Tauri 2.0 + Research Agent)
│   ├── src/
│   │   ├── main.rs          # Entry point, app setup
│   │   ├── commands.rs      # Tauri commands (IPC)
│   │   ├── research.rs      # Research agent (calls Anthropic API)
│   │   ├── scheduler.rs     # Cron-based research scheduler
│   │   ├── notifications.rs # Desktop notifications
│   │   ├── tray.rs          # System tray integration
│   │   └── db.rs            # Rust database layer
│   └── tauri.conf.json      # Tauri configuration
└── .github/workflows/       # CI/CD workflows
    ├── ci.yml               # Build, test, lint on push/PR
    └── release.yml          # Release builds on tags
```

## Tech Stack

- **Frontend**: React 18, TypeScript, Vite, Tailwind CSS, Lucide icons
- **Desktop**: Tauri 2.0 (Rust)
- **Research Agent**: Rust (built-in), calls Anthropic API via reqwest
- **Database**: SQLite via sql.js (browser) and rusqlite (Rust)
- **Testing**: Vitest, React Testing Library, Cargo Test

## Claude Models

**IMPORTANT**: This project uses Claude 4.5 models exclusively. Do NOT use Claude 3.x model IDs.

| Model | Model ID | Use Case |
|-------|----------|----------|
| Haiku 4.5 | `claude-haiku-4-5-20241022` | Default for research (fastest, cheapest) |
| Sonnet 4.5 | `claude-sonnet-4-5-20250929` | Balanced quality/cost |
| Opus 4.5 | `claude-opus-4-5-20251101` | Highest quality |

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

# Run Rust tests
cd src-tauri && cargo test

# Type check all packages
npm run build
```

## Key Files

| File | Purpose |
|------|---------|
| `src-tauri/tauri.conf.json` | Tauri app configuration, permissions |
| `src-tauri/src/main.rs` | Rust entry point, command registration |
| `src-tauri/src/commands.rs` | IPC commands called from frontend |
| `src-tauri/src/research.rs` | Research agent (Anthropic API client) |
| `src-tauri/src/scheduler.rs` | Cron-based research scheduler |
| `packages/frontend/src/App.tsx` | React router, main layout |
| `packages/frontend/src/hooks/useTauri.ts` | Tauri IPC bridge with mock data fallback |
| `packages/shared/src/db/` | Database operations (briefings, feedback) |
| `packages/shared/src/db/schema.ts` | SQLite schema definition |

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
3. **Rust Tests** - cargo test for research agent
4. **Build Tauri** - macOS (universal), Windows, Ubuntu

Release workflow triggers on version tags (`v*`) and creates draft releases with installers.
