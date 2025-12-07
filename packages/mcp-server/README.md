# Claudius MCP Server

This MCP server exposes Claudius briefings to Claude Desktop.

## Setup

Add to `~/.claude/claude.json`:

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

## Available Tools

- `get_briefings` - Get recent briefings
- `search_briefings` - Search by keyword
- `get_briefing_detail` - Get full briefing
- `get_feedback_patterns` - See liked/disliked topics
- `get_interests` - See research interests
- `get_research_stats` - Get statistics
