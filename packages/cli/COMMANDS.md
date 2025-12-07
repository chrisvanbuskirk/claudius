# Claudius CLI Commands Reference

## Interests Management

### `claudius interests add <topic>`
Add a new research interest.

**Options:**
- `-d, --depth <depth>` - Research depth (1-5, default: 3)

**Example:**
```bash
claudius interests add "quantum computing" --depth 4
```

### `claudius interests list`
List all configured research interests.

### `claudius interests remove <topic>`
Remove a research interest.

### `claudius interests block <topic>`
Block a topic from being researched.

### `claudius interests blocked`
List all blocked topics.

### `claudius interests set-depth <topic> <depth>`
Update the research depth for a topic (1-5).

---

## MCP Server Management

### `claudius mcp list`
List all MCP servers.

**Options:**
- `--enabled` - Show only enabled servers
- `--installed` - Show only installed servers

### `claudius mcp add <name>`
Add a new MCP server (interactive).

### `claudius mcp enable <name>`
Enable an MCP server.

### `claudius mcp disable <name>`
Disable an MCP server.

### `claudius mcp test <name>`
Test connection to an MCP server.

### `claudius mcp remove <name>`
Remove an MCP server.

### `claudius mcp configure <name>`
Configure an MCP server (interactive).

---

## Research Control

### `claudius research`
Trigger research on your interests.

**Options:**
- `--now` - Run research immediately (default: schedule)
- `--topic <topic>` - Research specific topic (overrides interests)
- `--depth <depth>` - Research depth 1-5 (default: 3)
- `--model <model>` - AI model to use (default: claude-opus-4)
- `--output <path>` - Output path for briefing
- `-v, --verbose` - Verbose output

**Examples:**
```bash
# Run immediate research on all interests
claudius research --now

# Research specific topic with high depth
claudius research --topic "AI safety" --depth 5 --now

# Save to specific file
claudius research --now --output ./briefing.md
```

---

## Briefings Management

### `claudius briefings list`
List all generated briefings.

**Options:**
- `--last <days>` - Show briefings from last N days (default: 30)

### `claudius briefings search <query>`
Search briefings by content.

**Options:**
- `--since <date>` - Only search briefings since date (YYYY-MM-DD)

**Example:**
```bash
claudius briefings search "machine learning" --since 2024-01-01
```

### `claudius briefings export <id>`
Export a briefing to file.

**Options:**
- `--format <format>` - Export format: markdown, pdf, json (default: markdown)

**Example:**
```bash
claudius briefings export brief-001 --format pdf
```

### `claudius briefings cleanup`
Delete old briefings.

**Options:**
- `--older-than <days>` - Delete briefings older than N days (default: 90)

---

## Configuration

### `claudius config show`
Display current configuration.

### `claudius config edit`
Open configuration directory in default editor.

### `claudius config set <key> <value>`
Set a configuration value.

**Key format:** `<file>.<path>.<to>.<key>`

**Examples:**
```bash
claudius config set settings.model claude-opus-4
claudius config set settings.defaultDepth 4
claudius config set settings.scheduleInterval "0 8 * * *"
```

### `claudius config init`
Initialize configuration files and directory.

---

## Database Management

### `claudius db stats`
Show database statistics.

Displays:
- Total briefings
- Total insights
- Database size
- Date range
- Most researched topics

### `claudius db export`
Export all database data.

**Options:**
- `--format <format>` - Export format: json, csv, sql (default: json)

### `claudius db reset`
Reset database (WARNING: deletes all data).

Includes confirmation prompt for safety.

---

## Global Options

All commands support:
- `--help` - Show help for command
- `--version` - Show CLI version

**Examples:**
```bash
claudius --version
claudius interests --help
claudius research --help
```

---

## Configuration Files

Located in `~/.claudius/`:

### `interests.json`
```json
{
  "interests": [
    {
      "topic": "AI Research",
      "depth": 4,
      "addedAt": "2024-12-06T10:00:00.000Z"
    }
  ],
  "blocked": ["politics", "sports"]
}
```

### `mcp.json`
```json
{
  "servers": [
    {
      "name": "filesystem",
      "command": "npx",
      "args": ["-y", "@modelcontextprotocol/server-filesystem", "/path"],
      "enabled": true,
      "addedAt": "2024-12-06T10:00:00.000Z"
    }
  ]
}
```

### `settings.json`
```json
{
  "model": "claude-opus-4",
  "defaultDepth": 3,
  "scheduleInterval": "0 8 * * *",
  "outputFormat": "markdown"
}
```
