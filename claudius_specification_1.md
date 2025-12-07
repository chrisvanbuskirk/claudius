# Claudius: A Local AI Research Briefing Agent

## Overview

**Claudius** is a macOS application that generates personalized daily research briefings using the Anthropic Agent SDK. It works offline, runs on your machine, and synthesizes relevant research based on your interests, calendar, emails, GitHub activity, and other personal context.

Think of it as a self-hosted, privacy-first alternative to OpenAI's ChatGPT Pulse—but built entirely on Claude and running locally on your Mac.

### Core Principle

**Proactive, personalized research delivered asynchronously.**

Instead of asking an AI for information, Claudius researches topics you care about overnight and presents curated briefing cards each morning. It learns from your feedback, respects your privacy (everything stays local), and lets you control exactly what it researches.

---

## Why Claudius?

### The Problem

- **ChatGPT Pulse** is expensive ($200/month), cloud-dependent, and limited to OpenAI's ecosystem
- **Generic news aggregators** aren't personalized to your work or interests
- **Manual research** is time-consuming (reading 20 sources, synthesizing insights)
- **Privacy concerns** with cloud-based briefing systems

### The Solution

A self-contained, open-source macOS app that:
- Runs overnight, generates briefings by morning
- Uses your calendar, email, GitHub to personalize research
- Costs ~$5/month in API calls (vs. $200 for Pulse)
- Keeps all data local (SQLite on your Mac)
- Open-source, community-driven, hackable
- **Integrates with Claude Desktop as an MCP Server** — query your briefings directly from Claude

---

## Architecture

```
┌─────────────────────────────────────────┐
│          Claudius (Tauri App)           │
├─────────────────────────────────────────┤
│                                         │
│  Frontend (React)                       │
│  ├── Briefing cards display             │
│  ├── Settings (interests, MCPs)         │
│  ├── Search/filter briefings            │
│  └── Feedback UI (thumbs up/down)       │
│                                         │
├─────────────────────────────────────────┤
│  Backend (Rust/Tauri)                   │
│  ├── Scheduler (nightly research)       │
│  ├── Agent SDK integration              │
│  ├── MCP server orchestration           │
│  ├── SQLite persistence                 │
│  └── CLI IPC bridge                     │
│                                         │
├─────────────────────────────────────────┤
│  Agent SDK (Anthropic)                  │
│  ├── Web search                         │
│  ├── Tool calling                       │
│  ├── Model selection (Sonnet default)   │
│  └── Context management                 │
│                                         │
├─────────────────────────────────────────┤
│  MCP Servers (External)                 │
│  ├── Google Calendar                    │
│  ├── Gmail                              │
│  ├── GitHub                             │
│  ├── Firecrawl (web scraping)           │
│  ├── Perplexity Search                  │
│  ├── Local Filesystem                   │
│  └── Memory (persistent learning)       │
│                                         │
├─────────────────────────────────────────┤
│  CLI Companion (Node.js)                │
│  ├── Manage interests                   │
│  ├── Configure MCP servers              │
│  ├── Trigger research                   │
│  ├── Query briefings                    │
│  └── Control app settings               │
└─────────────────────────────────────────┘
```

### Component Breakdown

**Tauri Backend (Rust)**
- HTTP server for IPC with React frontend
- Scheduler (cron-like, runs nightly by default)
- Invokes Agent SDK to run research
- Stores briefings in SQLite
- Manages MCP server connections

**React Frontend**
- Card-based UI (inspired by Pulse)
- Settings panel for interests/MCPs
- Search and historical briefing browser
- Feedback mechanism (thumbs up/down, reasons)

**CLI Tool (Node.js/TypeScript)**
- Shares SQLite DB and config with Tauri app
- Allows headless research (`claudius research --now`)
- Manages interests, MCPs, configuration
- No GUI dependency—power users can run headless

**Agent SDK Integration**
- Drives the actual research
- Coordinates web search + MCP context
- Synthesizes into structured briefing cards
- Learns from feedback over time

---

## Features

### Core Features (MVP)

- **Scheduled Research**: Runs nightly at configurable time (default 6 AM)
- **Personalized Briefings**: Generates 3-5 briefing cards based on your interests
- **MCP Integration**: Pulls context from Calendar, Gmail, GitHub, local notes
- **Card-Based UI**: Each briefing is a scannable card with title, summary, sources
- **Feedback Loop**: Thumbs up/down on cards; agent learns preferences
- **Local Storage**: All briefings stored in SQLite on your Mac
- **CLI + UI**: Both command-line and visual interface

### Phase 2 Features

- **Claudius as MCP Server**: Query briefings directly from Claude Desktop/Claude.ai
- **MCP Marketplace**: Searchable registry of vetted MCP servers with one-click install, curated starter sets, and community ratings
- **Follow-up Chat**: Tap a card to ask clarifying questions
- **Multi-day Research**: "Give me a deep-dive on this topic"
- **Export**: Download briefings as PDF, Markdown, HTML
- **Sharing**: Export a briefing to share with team
- **Smart Scheduling**: Adjust research based on your calendar (more research on slow days)
- **Collaborations**: Sync interests with team (internal Autoliv use)

### Phase 3 Features

- **iOS/iPad Companion App**: Browse briefings on the go
- **Browser Extension**: Save articles for Claudius to research
- **Slack Integration**: Post daily briefing to channel
- **Custom MCP Servers**: Build internal MCPs (e.g., Autoliv news, JIRA)
- **Reasoning Mode**: Deep analysis using extended thinking

---

## MCP Servers (Initial Set)

### Personal Context (High Priority)

1. **Google Calendar** 
   - Why: Understand your week, upcoming travel, meetings
   - Use: "You have a trip to Detroit next week, here are relevant logistics/news"
   - Command: `mcp-google-calendar`

2. **Gmail**
   - Why: Surface important messages, decisions made in email
   - Use: "Your team decided on Swift 6 migration strategy. Here are new resources"
   - Command: `mcp-gmail`

3. **GitHub**
   - Why: Track PR activity, issues, releases in your org + personal projects
   - Use: "SwiftUI modernization in progress. Here are related best practices"
   - Command: `mcp-github`

### Research & Discovery

4. **Firecrawl**
   - Why: Deep web scraping for research depth, not just headlines
   - Use: Full-text extraction from articles, reports, documentation
   - Command: `mcp-firecrawl`

5. **Perplexity Search**
   - Why: Better search synthesis than generic web search
   - Use: "Here's what's new in Swift 6 based on latest sources"
   - Command: `mcp-perplexity-search`

6. **Fetch** (Official MCP)
   - Why: Convert web content to LLM-friendly format
   - Use: Process research URLs into clean text
   - Command: `npx @modelcontextprotocol/server-fetch`

### Knowledge Base

7. **Local Filesystem**
   - Why: Personal notes, documentation, saved research
   - Use: Cross-reference with research (e.g., "relates to your Q4 strategy doc")
   - Command: `npx @modelcontextprotocol/server-filesystem --path ~/.claudius/knowledge`

8. **Memory** (Official MCP)
   - Why: Persistent learning about your interests, preferences
   - Use: Remember "user likes technical deep-dives, not news aggregation"
   - Command: `npx @modelcontextprotocol/server-memory`

### Optional (Phase 2+)

9. **Slack** - Get team context and recent decisions
10. **Linear/Jira** - See active tickets and roadmap
11. **Stripe/Financial** - Business metrics if relevant
12. **Notion** - Query shared knowledge base
13. **X/Twitter** - Real-time tweet search and trending topics
14. **Grok** - Leverage Grok's real-time web context as a research tool

---

## MCP Marketplace & Discovery

One of Claudius' key strengths is flexibility—researchers have different needs and preferred data sources. The **MCP Marketplace** makes it easy for users to discover, install, and configure MCP servers without manual setup.

### Marketplace Overview

The MCP Marketplace is a searchable registry of vetted MCP servers, built directly into Claudius. Users can:
- Browse recommended servers by category
- Search for specific tools
- View requirements, authentication, and documentation
- Install with one click
- Test connections before enabling
- Track which servers are installed/enabled

### UI Experience

#### Home Tab → "MCP Servers" Section

- **Browse** button opens full marketplace modal
- Shows installed + enabled MCPs at a glance
- Quick toggles to enable/disable active servers

#### Marketplace Modal (Phase 2)

**Layout:**
- Left sidebar: **Categories** (Research, Personal Context, Productivity, AI/Analysis, Specialized)
- Main area: Searchable grid of MCP server cards
- Right panel: Detailed view of selected server

**MCP Server Card:**
```
┌─────────────────────────────────────────┐
│ Google Calendar        [Installed] [✓]  │
├─────────────────────────────────────────┤
│ Sync calendar events for context        │
│                                         │
│ Requirements:                           │
│ • Google OAuth connection              │
│ • Read calendar permissions            │
│                                         │
│ Status: Enabled (last tested 2h ago)   │
│                                         │
│ [Configure] [Documentation] [GitHub]   │
└─────────────────────────────────────────┘
```

**Server Details Panel:**
- Full description
- Setup instructions
- Required vs optional authentication
- Common use cases (with examples)
- Community ratings (⭐ 4.8/5 from 320 users)
- Tags: #research #calendar #personal-context
- Last updated: "Updated 3 days ago"
- Maintenance status: "Actively maintained" / "Community-maintained"

### Curated Collections

**Use-Case Starter Sets** — pre-selected MCP combinations:

1. **"Researcher Starter"** (for analysts, knowledge workers)
   - Google Calendar, Gmail, Firecrawl, Perplexity, Local Filesystem, Memory
   - "Get your calendar context + deep research on topics you care about"

2. **"Developer Starter"** (for engineers like Chris)
   - GitHub, Linear, Slack, Local Filesystem, Memory, X/Twitter
   - "Track your repos, tickets, team decisions, and dev community trends"

3. **"Productivity Starter"** (for busy professionals)
   - Google Calendar, Gmail, Slack, Notion, Memory
   - "Stay on top of meetings, email decisions, team updates, and knowledge base"

4. **"Data Scientist Starter"** (Phase 2)
   - Stripe, database connectors, memory, analytics MCPs
   - "Track metrics, query databases, synthesize business insights"

Each collection shows:
- Included MCPs
- What you'll be able to research
- Estimated setup time
- One-click "Install All" button

### CLI Commands for MCP Management

```bash
# Marketplace search & discovery
claudius marketplace list                    # List all servers
claudius marketplace search "calendar"       # Search by name/keyword
claudius marketplace list --category research --sort popular
claudius marketplace show google-calendar   # Detailed info
claudius marketplace collections            # Show starter sets
claudius marketplace install-set researcher # Install "Researcher Starter"

# Server management (existing + enhanced)
claudius mcp add google-calendar             # Install + launch guided config
claudius mcp configure google-calendar       # Edit config (show saved values)
claudius mcp test github                     # Test connection, show diagnostics
claudius mcp enable/disable github           # Toggle without removing
claudius mcp list                            # Show all + status
claudius mcp list --installed                # Only installed
claudius mcp list --enabled                  # Only active
```

### One-Click Installation Flow

**Scenario: User clicks "Add" on Google Calendar card**

1. **Verification Screen**
   ```
   📅 Google Calendar
   ├─ Status: Not installed
   ├─ Setup Time: ~2 minutes
   ├─ Requirements:
   │  └─ Google OAuth (one-time)
   └─ [Install] [Learn more]
   ```

2. **Auth Prompt** (if needed)
   - Opens OAuth flow for Google
   - Clear explanation: "Claudius needs permission to read your calendar for research context"
   - Returns to Claudius with token stored

3. **Configuration**
   ```
   Google Calendar Config
   ├─ Calendar(s) to sync: [Primary] [Work] [Personal] ✓
   ├─ How far ahead to look: [This week] [Next 30 days] [All]
   ├─ Event types to include: [Meetings] [Travel] [Deadlines]
   └─ [Test connection] [Save & enable]
   ```

4. **Test Connection**
   - Claudius queries: "Can I access your calendar?"
   - Shows: "✓ Found 12 events for next 30 days"
   - Or shows error with troubleshooting steps

5. **Enable**
   - Server automatically enabled after successful test
   - Shows confirmation: "Google Calendar enabled. Will be used in next research run."

### Metadata for Each MCP Server

The marketplace registry stores/fetches:

```json
{
  "id": "google-calendar",
  "name": "Google Calendar",
  "description": "Sync events, meetings, travel for research context",
  "category": ["personal-context"],
  "tags": ["calendar", "scheduling", "events"],
  "documentation": "https://github.com/anthropics/mcp-servers/tree/main/src/google-calendar",
  "repository": "https://github.com/anthropics/mcp-servers",
  "command": "npx @anthropics/mcp-google-calendar",
  "requirements": {
    "auth": "google_oauth",
    "optional": false,
    "scopes": ["calendar.readonly"]
  },
  "setupTime": "2-3 minutes",
  "rating": 4.8,
  "ratingCount": 320,
  "maintenance": "actively-maintained",
  "lastUpdated": "2025-12-01",
  "usageCount": 15420,
  "testimonials": [
    {
      "user": "Chris Van Buskirk",
      "text": "Essential for calendar-aware research briefs",
      "verified": true
    }
  ],
  "useCases": [
    "Get briefings relevant to your upcoming meetings",
    "Surface travel-related research before trips",
    "Research topics related to scheduled deep work blocks"
  ]
}
```

### Community Ratings & Feedback

Users can rate MCPs post-install:

```
Google Calendar
⭐⭐⭐⭐⭐ 4.8/5 (320 ratings)

Most helpful: "Sets up in 2 minutes, data is always current"
Concerns: "Needs extra permission scopes for shared calendars"

[Help me rate this] [Report issue]
```

Ratings are stored locally + optionally submitted to a community feedback server (privacy-respecting, no PII).

### Testing & Diagnostics

Each MCP has a **Test** function that runs diagnostic checks:

```bash
$ claudius mcp test github

Testing GitHub MCP...
├─ ✓ Command found: /usr/local/bin/mcp-github
├─ ✓ Environment variables set (GITHUB_TOKEN)
├─ ✓ Can establish connection
├─ ✓ Sample query successful: Found 5 repos
├─ ✓ Response time: 245ms
├─ ⚠ Warning: Token expires in 30 days
└─ All checks passed!

Ready to use. GitHub will be included in next research.
```

### Smart Defaults & Recommendations

Marketplace shows recommendations:

- **"Popular with developers"** — GitHub, VS Code, Linear
- **"Popular with mobile devs"** — GitHub, X/Twitter (for dev community), Slack
- **"Trending this week"** — Shows MCPs gaining usage
- **"Recommended for your interests"** (Phase 2+) — Based on user's interests, suggests relevant MCPs

### Error Handling & Help

If MCP installation/config fails:

```
❌ Google Calendar connection failed

Possible causes:
1. OAuth token expired → [Re-authenticate]
2. Network issue → Check your connection
3. API quota exceeded → Try again in 1 hour
4. Invalid scopes → [View setup guide]

[Troubleshooting] [Report issue] [Contact support]
```

### MCP Marketplace Data Sources

**Registry hosted at:** `https://registry.claudius.sh` (or similar)

Includes:
- Official Anthropic MCPs
- Vetted community MCPs (GitHub, filtered by quality/maintenance)
- User submissions (with review process)
- Auto-discovered MCPs from GitHub topic tags

### Extensibility (Phase 2+)

- **User can add custom MCP sources** (e.g., internal company MCPs)
  ```bash
  claudius marketplace add-source https://internal.company.com/mcp-registry
  ```
- **Slack integration**: Post "MCP of the week" updates to team channel
- **MCP scoring**: Algorithm rewards well-documented, actively-maintained servers

---

## Research Workflow

### Daily Research Cycle

```
6:00 AM (Default)
  ├─ Load interests from ~/.claudius/interests.json
  ├─ Query MCP servers for context:
  │  ├─ Calendar: What's happening this week?
  │  ├─ Gmail: Recent important decisions?
  │  ├─ GitHub: What's changed in my repos?
  │  └─ Local notes: What am I working on?
  │
  ├─ Invoke Agent SDK with research brief:
  │  "Given this context, research these topics..."
  │  ├─ Uses web search (Perplexity)
  │  ├─ Deep scrapes promising URLs (Firecrawl)
  │  ├─ Cross-references with user knowledge base
  │  └─ Synthesizes insights
  │
  ├─ Generate structured briefings:
  │  [
  │    {
  │      "title": "Swift 6 adoption rate...",
  │      "summary": "...",
  │      "sources": ["link1", "link2"],
  │      "suggestedNext": "Read full RFC on concurrency",
  │      "relevance": "high" // based on feedback history
  │    },
  │    ...
  │  ]
  │
  ├─ Query Memory MCP: Did user like this type of briefing before?
  │  └─ Adjust relevance scoring
  │
  ├─ Store briefing in SQLite
  │  ├─ briefing (id, date, title, cards JSON, research_time_ms)
  │  └─ research_logs (server, query, duration, tokens_used)
  │
  └─ Notify user (macOS notification or silent storage)
```

### Feedback Loop

```
User views briefing in UI:
  ├─ Thumbs up on card → Records positive signal
  ├─ Thumbs down + reason → Records negative signal + context
  ├─ "Why did I get this?" → Shows reasoning from research
  └─ "Research more on this" → Adds topic to queue

Agent learns (fed into next research run):
  ├─ "User gave 5 thumbs up to Swift + Kotlin topics"
  ├─ "User hides generic news, prefers technical deep-dives"
  ├─ "Related topics: multiplatform dev, Android, iOS"
  └─ Adjusts research priorities for next cycle
```

### Manual Research (On-Demand)

```bash
# Trigger research immediately
claudius research --now

# Research specific topic
claudius research --now --topic "Swift 6 migration" --depth deep

# Use different model
claudius research --now --model opus

# Save to file instead of DB
claudius research --now --output ~/Downloads/research.md
```

---

## Data Storage

### Configuration (JSON)

**Location**: `~/.claudius/`

```
~/.claudius/
├── interests.json          # Topics to research
├── mcp-servers.json        # MCP server config
├── preferences.json        # App settings
├── knowledge/              # Personal notes/docs (optional)
│   ├── swift-notes.md
│   ├── autoliv-context.md
│   └── q4-strategy.md
└── claudius.db             # SQLite database
```

#### `interests.json`

```json
{
  "topics": [
    "Swift 6 migration",
    "iOS 18 features",
    "Android modularization",
    "mobile team collaboration",
    "Autoliv safety innovations"
  ],
  "keywords": {
    "include": ["SwiftUI", "Kotlin", "Jetpack", "async/await"],
    "exclude": ["gaming", "cryptocurrency", "web3"]
  },
  "sources": {
    "github": ["steipete", "realm", "AutolivOrg"],
    "rss": []
  },
  "research_depth": {
    "default": "balanced",
    "Swift": "deep",
    "news": "surface"
  }
}
```

#### `mcp-servers.json`

```json
{
  "servers": [
    {
      "name": "calendar",
      "type": "stdio",
      "command": "mcp-google-calendar",
      "enabled": true,
      "config": {
        "scopes": ["readonly"]
      }
    },
    {
      "name": "github",
      "type": "stdio",
      "command": "mcp-github",
      "enabled": true,
      "config": {
        "token": "${GITHUB_TOKEN}",
        "orgs": ["AutolivOrg"]
      }
    },
    {
      "name": "firecrawl",
      "type": "stdio",
      "command": "mcp-firecrawl",
      "enabled": true,
      "config": {
        "api_key": "${FIRECRAWL_API_KEY}"
      }
    }
  ]
}
```

#### `preferences.json`

```json
{
  "app": {
    "theme": "dark",
    "notifications": true
  },
  "research": {
    "schedule": "0 6 * * *",
    "default_model": "claude-sonnet-4-5-20250929",
    "max_tokens": 2000,
    "temperature": 0.7,
    "num_briefings": 4
  },
  "storage": {
    "db_path": "~/.claudius/claudius.db",
    "briefing_retention_days": 90
  }
}
```

### SQLite Database

**Schema:**

```sql
-- Briefings
CREATE TABLE briefings (
  id INTEGER PRIMARY KEY,
  date DATE NOT NULL,
  title TEXT,
  cards JSON NOT NULL,
  research_time_ms INTEGER,
  model_used TEXT,
  total_tokens INTEGER,
  created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Cards detail (denormalized from cards JSON for quick queries)
CREATE TABLE briefing_cards (
  id INTEGER PRIMARY KEY,
  briefing_id INTEGER NOT NULL,
  card_index INTEGER,
  title TEXT,
  summary TEXT,
  sources JSON,
  relevance_score FLOAT,
  FOREIGN KEY (briefing_id) REFERENCES briefings(id)
);

-- User feedback on cards
CREATE TABLE feedback (
  id INTEGER PRIMARY KEY,
  briefing_id INTEGER NOT NULL,
  card_index INTEGER,
  rating INTEGER,           -- -1 (dislike), 0 (neutral), 1 (like)
  reason TEXT,              -- "too much news", "not technical enough", etc.
  created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
  FOREIGN KEY (briefing_id) REFERENCES briefings(id)
);

-- Research logs for debugging/analytics
CREATE TABLE research_logs (
  id INTEGER PRIMARY KEY,
  briefing_id INTEGER NOT NULL,
  mcp_server TEXT,
  query TEXT,
  result_tokens INTEGER,
  duration_ms INTEGER,
  error_message TEXT,
  created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
  FOREIGN KEY (briefing_id) REFERENCES briefings(id)
);

-- Feedback patterns (for agent learning)
CREATE TABLE feedback_patterns (
  id INTEGER PRIMARY KEY,
  topic TEXT,
  positive_count INTEGER DEFAULT 0,
  negative_count INTEGER DEFAULT 0,
  last_updated TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);
```

---

## CLI Commands

### Interest Management

```bash
# Add interest
claudius interests add "Swift 6 migration"
claudius interests add "Android Compose" --depth deep

# List interests
claudius interests list

# Remove
claudius interests remove "Swift 6 migration"

# Block topics
claudius interests block "cryptocurrency"
claudius interests block "web3"

# View blocks
claudius interests blocked

# Set research depth for topic
claudius interests set-depth "Swift" deep
claudius interests set-depth "news" surface
```

### MCP Server Management

```bash
# Marketplace discovery
claudius marketplace list                          # List all available MCPs
claudius marketplace search "calendar"             # Search by name/keyword
claudius marketplace list --category research     # Filter by category
claudius marketplace list --sort popular          # Sort by popularity
claudius marketplace show google-calendar         # View detailed MCP info
claudius marketplace collections                  # Show starter sets
claudius marketplace install-set researcher       # Install curated collection

# Server management
claudius mcp list                                 # List all installed MCPs
claudius mcp list --enabled                       # Show only active
claudius mcp list --installed                     # Show only installed
claudius mcp add gmail --command "mcp-gmail"      # Install from command
claudius mcp enable calendar                      # Activate server
claudius mcp disable slack                        # Deactivate server
claudius mcp configure github                     # Edit config for installed MCP
claudius mcp test github                          # Test connection & diagnostics
claudius mcp remove github                        # Uninstall server
```

### Research Control

```bash
# Trigger research immediately
claudius research --now

# Research specific topic
claudius research --now --topic "Swift 6" --depth deep

# Use different model
claudius research --now --model opus

# Save as markdown
claudius research --now --output ~/Downloads/brief.md

# Verbose logging
claudius research --now --verbose
```

### Briefing Queries

```bash
# List recent briefings
claudius briefings list

# Show last 7 days
claudius briefings list --last 7d

# Search briefings
claudius briefings search "Swift"
claudius briefings search "Swift" --since 2025-01-01

# Export briefing
claudius briefings export 5 --format markdown
claudius briefings export 5 --format pdf

# Delete old briefings
claudius briefings cleanup --older-than 90d
```

### Feedback & Learning

```bash
# View feedback patterns
claudius feedback summary

# Show what user liked
claudius feedback liked --topic Swift

# Show what user disliked
claudius feedback disliked --since 7d
```

### Configuration

```bash
# Show config
claudius config show

# Edit config
claudius config edit

# Change setting
claudius config set research.schedule "0 6 * * *"
claudius config set research.default_model "claude-opus-4-20250514"
claudius config set research.num_briefings 5
```

### Database

```bash
# Show stats
claudius db stats

# Export all briefings
claudius db export --format json > all-briefings.json

# Reset database (WARNING)
# Reset database (WARNING)
claudius db reset
```

---

## Claudius as an MCP Server

One of Claudius' most powerful features is that it can expose itself as an MCP (Model Context Protocol) server. This means your research briefings and learned preferences become available directly within Claude Desktop and Claude.ai.

### Why This Matters

Instead of copy-pasting research from Claudius into Claude, you can ask Claude directly:
- "What did my Claudius agent find today?"
- "Summarize my briefings from this week"
- "Which research topics did I engage with most?"
- "Show me sources for the Swift 6 briefing"

Claude queries Claudius in real-time, can follow up with questions, and integrates the research into your actual work.

### Configuration

**Claude Desktop Setup** (`~/.claude/claude.json`):

```json
{
  "mcpServers": {
    "claudius": {
      "command": "claudius",
      "args": ["mcp", "--serve"],
      "env": {
        "CLAUDIUS_DB": "~/.claudius/claudius.db"
      }
    }
  }
}
```

Restart Claude Desktop. Claudius MCP is now available.

### Available MCP Tools

**Queries:**
- `get_briefings(date_range)` - Retrieve briefings from a date range (e.g., "last 7 days", "today")
- `search_briefings(query)` - Full-text search across all briefing titles and summaries
- `get_briefing_detail(briefing_id)` - Get full briefing with all sources and metadata

**Insights:**
- `get_feedback_patterns()` - Show what topics user likes/dislikes (trending topics)
- `get_sources(briefing_id)` - Get full list of sources for a specific briefing
- `get_research_stats()` - Briefing count, average research time, tokens used

**Management:**
- `get_interests()` - List currently configured research interests
- `get_mcp_servers()` - List enabled MCP servers Claudius uses

### Example Usage in Claude

```
You: "What did Claudius research about Swift this week?"

Claude (via Claudius MCP):
→ Queries: search_briefings("Swift")
→ Returns 3 briefings from last 7 days
→ Shows summaries and sources

Claude: "I found 3 briefings about Swift 6:
1. Swift 6 Concurrency Changes Stabilizing (Jan 10)
2. SwiftUI 6.0 Release Notes (Jan 8)
3. Migration Guide: Swift 5 → Swift 6 (Jan 5)

Want me to dive deeper into any of these?"

You: "Tell me more about concurrency changes"

Claude:
→ Queries: get_briefing_detail(id=1)
→ Returns full briefing with all sources
→ Can cite sources: "According to your research [source], ..."
```

### How It Works

When Claude asks Claudius for data:

```
Claude Desktop / Claude.ai
    ↓
Calls Claudius MCP Server
    ↓
Claudius reads from local SQLite
    ↓
Returns structured JSON
    ↓
Claude synthesizes + responds to user
```

**Zero network calls for data retrieval** — everything stays local. Only Claude API calls go to Anthropic.

### Benefits

- **Seamless Integration** - Ask Claude about your research without leaving chat
- **Privacy** - Briefing queries never leave your Mac
- **Context-Aware** - Claude has real-time access to your learned interests
- **Two-Way** - Claude can help interpret research, suggest follow-ups, save insights
- **Synergistic** - Claudius provides data, Claude provides reasoning

This transforms Claudius from a standalone briefing tool into a first-class citizen in your Claude workflow.

---

## Tauri UI

### Main Interface

**Home/Briefings Tab**
- Display today's briefing cards
- Each card shows:
  - Title
  - Summary (2-3 sentences)
  - Source links
  - Suggested next step/action
  - Thumbs up/down buttons
  - "Why did I get this?" info icon

**Card Interaction**
- Click to expand (full text, all sources)
- Thumbs up → adds to liked pattern
- Thumbs down → asks for reason
- "Save" → exports to Markdown/PDF
- "Research more" → queues for deep-dive

**Settings Tab**
- **Interests**: List with add/remove buttons
- **MCP Servers**: 
  - Browse Marketplace button (opens MCP discovery modal)
  - Installed MCPs with enable/disable toggles
  - Test connection & view config for each
  - Quick access to "Starter Sets" (Researcher, Developer, Productivity)
- **Research Settings**: 
  - Schedule (time picker, or cron expression)
  - Default model (dropdown: Sonnet/Haiku/Opus)
  - Depth (slider or Low/Medium/High)
  - Num briefings (slider 2-10)
- **Storage**: Show DB size, retention days, export button
- **MCP Marketplace Modal** (when clicking "Browse" or "Add"):
  - Category filters on left
  - Search bar
  - Grid of MCP cards with install/enable buttons
  - Detailed view on right side
  - One-click "Install All" for starter sets

**History Tab**
- Calendar view of briefings
- Search box (full-text search across briefings)
- Filter by topic
- Sort by date, relevance

**Feedback Tab**
- Show learned preferences
- "You liked these topics..." (chart)
- "You avoided these..." (chart)
- Reset learning

---

## Model Selection

### Defaults

**Default**: `claude-sonnet-4-5-20250929`

### Why Sonnet?

- **Strong synthesis** for multi-source research
- **Cost-effective** (~$0.15 per nightly run)
- **Fast enough** for scheduled research (no real-time constraint)
- **Maintained** and actively improved

### User Override

Users can change model per-run or globally:

```bash
# For this run
claudius research --now --model opus

# Set as default
claudius config set research.default_model opus
```

### Model Options

| Model | Use Case | Cost |
|-------|----------|------|
| **Sonnet 4.5** | Default, balanced | $0.15/run |
| **Haiku 4.5** | Quick testing, cost-sensitive | $0.03/run |
| **Opus 4** | Deep analysis, complex synthesis | $0.75/run |

---

## Installation & Distribution

### macOS (Tauri App)

```bash
# From Homebrew (Phase 1)
brew install claudius

# Launches Tauri app
open /Applications/Claudius.app
```

### CLI (Standalone)

```bash
# npm (works on Mac, Linux, Windows)
npm install -g claudius

# Use from terminal
claudius interests list
claudius research --now
```

### Setup (First Run)

1. User installs app
2. App asks for API keys:
   - Anthropic API key (required)
   - Google OAuth (for Calendar/Gmail)
   - GitHub token (optional)
   - Firecrawl API key (optional)
3. Creates `~/.claudius/` directory
4. Initializes SQLite database
5. Shows welcome briefing

---

## How to Build It

### Tech Stack

| Component | Tech | Why |
|-----------|------|-----|
| Backend | Rust (Tauri) | Cross-platform, low overhead, native OS integration |
| Frontend | React + TypeScript | Modern, maintainable, Tauri native |
| CLI | TypeScript/Node.js | Easier to write than Rust, reuses Anthropic SDK |
| Database | SQLite | Portable, no external deps, good for local storage |
| Agent Logic | Python (Agent SDK) | Native Agent SDK, easy to integrate |

### Development Approach

**Use Claude Code to build:**

1. **Phase 1: CLI + Agent Integration**
   - ✅ CLI commands structure
   - ✅ Config file parsing
   - ✅ SQLite setup
   - ✅ Agent SDK integration
   - ✅ MCP orchestration

2. **Phase 2: Tauri Backend + IPC**
   - ✅ Tauri project scaffold
   - ✅ HTTP/IPC bridge for CLI
   - ✅ Scheduler integration
   - ✅ Database access from Rust

3. **Phase 3: React Frontend**
   - ✅ Card UI components
   - ✅ Settings management
   - ✅ Briefing browser
   - ✅ Feedback interface

4. **Phase 4: Polish & Testing**
   - ✅ Notarization for macOS
   - ✅ CI/CD setup
   - ✅ User testing
   - ✅ Documentation

---

## Example Use Cases

### For Chris at Autoliv

**Daily Briefing Topics:**
- Swift 6 migration updates (from GitHub)
- Android team updates (from GitHub, Slack)
- Mobile safety research (web search)
- Your calendar context (meetings with teams)

**Morning Briefing Output:**
```
📱 Swift 6 Concurrency Changes Stabilizing
   Summary: Latest RFC feedback shows adoption blockers
   being addressed. Your Q1 migration timeline...
   [Thumbs up] [More info] [Research deeper]

🤖 Android Jetpack 2025 Releases
   Summary: Compose 1.8, WorkManager improvements.
   Related to your team's Q1 plans...
   [Thumbs up] [More info] [Research deeper]

🏢 Autoliv Q1 Safety Innovation Pipeline
   Summary: New projects approved. Connections to mobile
   team collaboration opportunities...
   [Thumbs up] [More info] [Research deeper]

👥 OneDevTeam Collaboration News
   Summary: iOS/Android knowledge sharing frameworks
   trending. Resources for your initiative...
   [Thumbs up] [More info] [Research deeper]
```

### For a Mobile Developer

**Research Interests:**
- New iOS/Android features
- Performance optimization trends
- Testing frameworks
- Architecture patterns

**Feedback Learning:**
- "User likes deep technical dives, not news aggregation"
- "User cares about iOS more than Android"
- "User ignores gaming/social apps, focuses on productivity"

---

## Open Source & Community

### GitHub Home

`https://github.com/chrisvanbuskirk/claudius`

### License

MIT - free to use, modify, build on

### Contributing

1. Fork repo
2. Create feature branch
3. Use Claude Code to build features
4. Submit PR with clear description

### Future Extensions

- **Team Collaboration**: Share Claudius configs with your iOS/Android team
- **Slack Bot**: Post daily briefing to team channel
- **Browser Extension**: Save articles for Claudius to research
- **iOS Companion**: Read briefings on the go
- **Custom MCPs**: Build internal MCPs for company data

---

## FAQ

**Q: Is this private?**  
A: Yes. Your briefings, preferences, and feedback live on your Mac in SQLite. Only your Anthropic API calls go to the cloud (same as any Claude user).

**Q: How much does it cost?**  
A: ~$5/month for nightly research (Sonnet model). Compare to ChatGPT Pulse at $200/month.

**Q: Can I use it at Autoliv?**  
A: Yes, with your own API keys. You could also build internal MCPs to access company data (with proper permissions).

**Q: Can I follow up on a briefing?**  
A: Phase 2 feature. For now, each briefing is standalone. You can manually ask Claude.ai follow-up questions.

**Q: What if I don't like a briefing?**  
A: Give thumbs down, agent learns for next time. You can also disable specific topics or MCPs.

**Q: Can I export my briefings?**  
A: Yes. CLI: `claudius briefings export`. UI: Click "Save" button. Formats: Markdown, PDF, JSON.

**Q: Is there a team version?**  
A: Not yet. Phase 2+ could support shared interest lists, collaborative feedback, team briefing posts.

---

## Getting Started

### For Users

1. Install Claudius (`brew install claudius`)
2. Get Anthropic API key (anthropic.com/console)
3. Run `claudius config init`
4. Add interests (`claudius interests add "Your topic"`)
5. Enable MCPs (Google Calendar, Gmail, GitHub)
6. Set schedule (`claudius config set research.schedule "0 6 * * *"`)
7. Wait for first briefing tomorrow morning, or run `claudius research --now`

### For Developers

1. Clone repo
2. Install deps: `npm install`
3. Set up env: `cp .env.example .env` + add your Anthropic API key
4. Build: `npm run build`
5. Dev: `npm run dev`
6. Test: `npm run test`

---

## Next Steps

1. **Build MVP (Phase 1)**
   - CLI structure
   - Agent SDK integration
   - SQLite persistence
   - Basic MCP support

2. **Add UI (Phase 2)**
   - Tauri scaffold
   - React briefing cards
   - Settings management

3. **Polish & Launch**
   - Notarize for macOS
   - Publish to Homebrew
   - Release on GitHub
   - Community feedback

4. **Expand**
   - Follow-up chat
   - iOS companion
   - Team features
   - Custom MCPs

---

**Author**: Chris (Mobile Solution Architect, Autoliv)  
**Status**: In Design  
**License**: MIT  
**Repository**: https://github.com/chrisvanbuskirk/claudius
