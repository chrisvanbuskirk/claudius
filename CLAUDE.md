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
├── src-tauri/               # Rust backend (Tauri 2.0 + Research Agent + CLI)
│   ├── src/
│   │   ├── lib.rs           # Shared library (db, research, mcp_client, config)
│   │   ├── main.rs          # Tauri app entry point
│   │   ├── bin/cli.rs       # CLI binary entry point
│   │   ├── commands.rs      # Tauri commands (IPC)
│   │   ├── research.rs      # Research agent (calls Anthropic API)
│   │   ├── research_state.rs # Global research state (for CLI progress)
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

## UI Notes

### Commented Out: Thumbs Up/Down Buttons

The thumbs up/down feedback buttons on briefing cards are **commented out** (not deleted). They were removed because:
- Unlike ChatGPT/Claude Desktop, this app doesn't use feedback for RLHF training
- The feedback data wasn't being used for anything meaningful
- Bookmarks already serve the "I want to keep this" purpose

If you want to re-enable them, search for "Thumbs handlers commented out" in:
- `packages/frontend/src/components/BriefingCard.tsx`
- `packages/frontend/src/pages/HomePage.tsx`
- `packages/frontend/src/pages/HistoryPage.tsx`
- `packages/frontend/src/pages/BookmarksPage.tsx`

The feedback database table and Tauri commands still exist if needed.

## Claude Models

**IMPORTANT**: This project uses Claude 4.5 models. See https://platform.claude.com/docs/en/about-claude/models/overview

| Model | Model ID | Use Case |
|-------|----------|----------|
| Haiku 4.5 | `claude-haiku-4-5-20251001` | Default for research (fastest, cheapest) |
| Sonnet 4.5 | `claude-sonnet-4-5-20250929` | Balanced quality/cost |
| Opus 4.5 | `claude-opus-4-5-20251101` | Highest quality |

## Research Agent: MCP Integration & Dynamic Dates

### Critical Research Quality Factors

The research agent's effectiveness depends heavily on **real-time web search capabilities**. Without MCP servers like Brave Search and Perplexity, Claude can only use its training data (which may be outdated) or fetch individual URLs via `fetch_webpage`.

**Recommended Setup**: Enable both Brave Search (free tier) and Perplexity (pay-as-you-go) MCP servers for optimal research quality.

### Dynamic Date Context

Research prompts (`src-tauri/src/research.rs`) calculate dates dynamically at runtime to ensure Claude always searches for current information:

```rust
let now = chrono::Local::now();
let current_date = now.format("%B %d, %Y").to_string();      // "December 9, 2025"
let current_month = now.format("%B").to_string();             // "December"
let current_year = now.format("%Y").to_string();              // "2025"
let prev_year = (now.year() - 1).to_string();                 // "2024"
let month_year = now.format("%B %Y").to_string();            // "December 2025"
```

These variables are interpolated into the system prompt to:
1. State the current date explicitly: "Today's date is {current_date}"
2. Require information from {month_year} and late {current_year}
3. Mark {prev_year} content as outdated
4. Format search queries: "[topic] {month_year}"

**Why This Matters**: Claude needs explicit date context because its training data has a cutoff. Without stating "Today is December 9, 2025", Claude may return outdated information or not realize it should be searching for December 2025 content.

### MCP Tool Priority

The research agent prioritizes tools in this order:

1. **Search Tools** (if available):
   - `brave_search`: Primary real-time web search
   - `perplexity`: AI-powered search for validation
   - Queries formatted as: `"[topic] {month_year}"` or `"[topic] {current_year} latest news"`

2. **Content Fetching**:
   - `fetch_webpage`: Reads promising URLs discovered by search
   - Can also be used directly with URLs likely to have current content

3. **GitHub Activity** (for open source topics):
   - `github_search`, `github_get_repo`, `github_list_commits`, etc.
   - Gets recent activity from {month_year}

4. **Graceful Degradation**:
   - If MCP servers fail to initialize, research continues with built-in tools
   - Claude explicitly states when current information is unavailable

### System Prompt Architecture

The system prompt (lines 750-784 in `research.rs`) follows this structure:

```
1. Date Context
   "Today's date is {current_date}"
   "Focus on {month_year} and late {current_year}"
   "{prev_year} or earlier is outdated"

2. Tool Descriptions
   Lists all available tools (built-in + MCP)

3. CRITICAL SEARCH TOOL USAGE
   "USE brave_search or perplexity FIRST"
   "Search for '[topic] {month_year}'"
   "These are your primary source"

4. Fetch Strategy
   "After search, use fetch_webpage on promising URLs"
   "If no search, target URLs with '/{current_year}' or '{month_year}'"
```

### User Prompt Architecture

The user prompt (lines 786-806) specifies:

```
Research: {topic}

Provide:
1. Key developments from {month_year} (last 24-48 hours)
2. Relevance and actionable insights
3. Sources dated {current_year}, preferably {month_year}

CRITICAL: Use tools aggressively for {month_year} info
Do NOT rely solely on training data
If you can't find {month_year} info, state this explicitly
```

### Event Flow for Synthesis Phase

The research agent emits progress events throughout the lifecycle:

```
research:started          → Research begins (total topics)
research:topic_started    → Per-topic research starts
research:topic_completed  → Per-topic research done (cards generated)
research:synthesis_started → Synthesis of all research begins
research:synthesis_completed → Synthesis done (cards generated, duration)
research:saving           → Saving to database
research:completed        → Full research session done
```

**Synthesis Phase** (lines 1078-1113): After completing all topic research, the agent calls Claude again to synthesize all research content into cohesive briefing cards. This phase typically takes 60-90 seconds and now has dedicated progress events so users know synthesis is happening.

- **Build**: npm workspaces monorepo

## Rust CLI

The CLI is a standalone Rust binary (`src-tauri/src/bin/cli.rs`) that shares code with the Tauri app via `lib.rs`. It provides full access to Claudius features without needing the GUI.

### Architecture

```
Cargo.toml:
  [lib]              → src/lib.rs (shared code: db, research, mcp_client, config)
  [[bin]] ClaudiusApp → src/main.rs (Tauri app, named to avoid case conflict with CLI)
  [[bin]] claudius   → src/bin/cli.rs (CLI)
```

Note: macOS has a case-insensitive filesystem, so "Claudius" and "claudius" would collide. The Tauri app binary is named `ClaudiusApp` in dev mode, but production builds use `productName: "Claudius"` from tauri.conf.json (via Info.plist).

Both binaries share:
- `research.rs` - Research agent (passes `app_handle: None` for CLI)
- `research_state.rs` - Global state for progress tracking
- `db.rs` - SQLite database operations
- `mcp_client.rs` - MCP server connections
- Config helpers (`read_api_key`, `read_settings`, etc.)

### CLI Progress Feedback

The CLI shows real-time progress during research by polling `research_state::get_state()`:

```rust
// research.rs sets phase at each step:
research_state::set_phase("Starting research...");
research_state::set_phase(&format!("Researching topic {}/{}: {}", i, total, topic));
research_state::set_phase("Synthesizing briefing cards...");
research_state::set_phase(&format!("Research complete: {} cards in {:.1}s", cards, secs));

// cli.rs polls in a loop while research runs:
loop {
    let state = research_state::get_state();
    if state.current_phase != last_phase {
        print!("\r→ {}...", state.current_phase);
    }
    if research_handle.is_finished() { break; }
    tokio::time::sleep(Duration::from_millis(500)).await;
}
```

### Installation

The CLI is bundled in the app and installed via symlink:
- **macOS**: `/usr/local/bin/claudius` → `/Applications/Claudius.app/Contents/MacOS/claudius`
- Users install from Settings → "Install CLI"
- Symlink means CLI auto-updates when app updates

### Key Commands

```bash
claudius topics list          # Manage research topics
claudius research now         # Run research (shows live progress)
claudius briefings show <id>  # View briefing cards
claudius mcp test <name>      # Test MCP server connection
claudius config api-key set   # Set API key
```

All commands support `--json` for machine-readable output.

### Scheduling Research (Automation)

The desktop app does not include built-in scheduling. To automate daily briefings, use the CLI with system scheduling tools. This approach is more reliable as it ensures the computer is awake when research runs.

**macOS (launchd)** - Create `~/Library/LaunchAgents/com.claudius.research.plist`:

```xml
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
        <integer>7</integer>
        <key>Minute</key>
        <integer>0</integer>
    </dict>
    <key>StandardOutPath</key>
    <string>/tmp/claudius-research.log</string>
    <key>StandardErrorPath</key>
    <string>/tmp/claudius-research.log</string>
</dict>
</plist>
```

Load with: `launchctl load ~/Library/LaunchAgents/com.claudius.research.plist`

**Linux/macOS (cron)** - Run `crontab -e` and add:

```cron
# Run research daily at 7:00 AM
0 7 * * * /usr/local/bin/claudius research now >> /tmp/claudius-research.log 2>&1
```

Note: Ensure the CLI is installed (Settings → Install CLI) and your API key is configured (`claudius config api-key set`).

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
- `topics` - Research topics (id, name, description, enabled, sort_order)
- `feedback` - User feedback on briefings/cards (rating, reason)
- `research_logs` - Audit log of research operations

Topics were migrated from JSON to SQLite for consistency. On first run after the migration, existing `~/.claudius/interests.json` is automatically migrated and renamed to `interests.json.migrated`.

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
- DO NOT PUSH UPDATES TO THE MAIN BRANCH