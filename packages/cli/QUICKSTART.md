# Claudius CLI - Quick Start Guide

## Installation

```bash
# Navigate to CLI package
cd packages/cli

# Install dependencies
npm install

# Build TypeScript
npm run build

# Link globally (makes 'claudius' command available)
npm link
```

## First Steps

### 1. Initialize Configuration

```bash
claudius config init
```

This creates `~/.claudius/` with:
- `interests.json` - Your research interests
- `mcp.json` - MCP server configurations
- `settings.json` - General settings
- `claudius.db` - SQLite database (created on first research)

### 2. Add Your Interests

```bash
# Add topics you want to track
claudius interests add "artificial intelligence" --depth 4
claudius interests add "climate technology" --depth 3
claudius interests add "quantum computing" --depth 5

# View your interests
claudius interests list
```

**Depth levels:**
- 1: Quick overview
- 2: Brief summary
- 3: Standard depth (default)
- 4: Detailed analysis
- 5: Comprehensive deep dive

### 3. Configure Settings (Optional)

```bash
# View current config
claudius config show

# Set preferred AI model
claudius config set settings.model claude-opus-4

# Set default research depth
claudius config set settings.defaultDepth 4

# Set daily schedule (cron format)
claudius config set settings.scheduleInterval "0 8 * * *"
```

### 4. Run Research

```bash
# Run immediate research on all interests
claudius research --now

# Research specific topic
claudius research --topic "machine learning" --now

# Verbose output
claudius research --now --verbose

# Save to custom location
claudius research --now --output ./my-briefing.md
```

### 5. View Briefings

```bash
# List recent briefings
claudius briefings list --last 7

# Search briefings
claudius briefings search "quantum"

# Export a briefing
claudius briefings export brief-001 --format markdown
```

## Common Workflows

### Daily Research Routine

```bash
# Morning: run research on all interests
claudius research --now

# View latest briefing
claudius briefings list --last 1
```

### Managing Interests

```bash
# Add new interest
claudius interests add "space exploration" --depth 3

# Change depth of existing interest
claudius interests set-depth "space exploration" 5

# Remove interest
claudius interests remove "old topic"

# Block unwanted topics
claudius interests block "celebrity news"
```

### Database Management

```bash
# View statistics
claudius db stats

# Export all data
claudius db export --format json

# Clean up old briefings
claudius briefings cleanup --older-than 90
```

## MCP Server Setup (Advanced)

MCP (Model Context Protocol) servers extend research capabilities.

```bash
# List MCP servers
claudius mcp list

# Add a server (interactive)
claudius mcp add filesystem

# Enable/disable servers
claudius mcp enable filesystem
claudius mcp disable unused-server

# Test connection
claudius mcp test filesystem
```

## Configuration Files

### Edit manually

```bash
# Location
cd ~/.claudius

# Edit interests
vim interests.json

# Edit MCP servers
vim mcp.json

# Edit settings
vim settings.json
```

### Or use CLI

```bash
# Open in default editor
claudius config edit

# Set specific values
claudius config set settings.model claude-opus-4
claudius config set settings.outputFormat markdown
```

## Tips

1. **Start simple**: Add 2-3 interests with default depth
2. **Experiment with depth**: Try different levels to find your preference
3. **Use blocking**: Block topics you're not interested in
4. **Check stats**: Use `claudius db stats` to see your research history
5. **Export briefings**: Save important briefings in multiple formats
6. **Automate**: Set up cron for automated daily research

## Troubleshooting

### Command not found

```bash
# Re-link the package
cd packages/cli
npm link
```

### TypeScript errors

```bash
# Rebuild
npm run build
```

### Config issues

```bash
# Reinitialize config
claudius config init
```

### Database issues

```bash
# View database stats
claudius db stats

# Export data before reset
claudius db export --format json

# Reset if needed (WARNING: deletes all data)
claudius db reset
```

## Help

```bash
# General help
claudius --help

# Command-specific help
claudius interests --help
claudius research --help
claudius briefings --help
```

## What's Next?

- Set up automated scheduling for daily briefings
- Configure MCP servers for enhanced research
- Explore different AI models and depth levels
- Export and share interesting briefings
- Integrate with other tools via database export

For complete command reference, see [COMMANDS.md](./COMMANDS.md)

For architecture details, see [STRUCTURE.md](./STRUCTURE.md)
