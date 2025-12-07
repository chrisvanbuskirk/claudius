# Claudius CLI

Command-line interface for Claudius - AI-powered personalized research briefing system.

## Installation

```bash
cd packages/cli
npm install
npm run build
npm link  # Make 'claudius' command available globally
```

## Development

```bash
npm run dev  # Watch mode for development
```

## Commands

### Interest Management

Manage your research interests:

```bash
claudius interests add <topic> [--depth <1-5>]  # Add interest
claudius interests list                          # List all interests
claudius interests remove <topic>                # Remove interest
claudius interests block <topic>                 # Block topic
claudius interests blocked                       # List blocked topics
claudius interests set-depth <topic> <depth>     # Set research depth
```

### MCP Server Management

Manage Model Context Protocol servers:

```bash
claudius mcp list [--enabled] [--installed]  # List MCP servers
claudius mcp add <name>                      # Add MCP server
claudius mcp enable <name>                   # Enable server
claudius mcp disable <name>                  # Disable server
claudius mcp test <name>                     # Test connection
claudius mcp remove <name>                   # Remove server
claudius mcp configure <name>                # Configure server
```

### Research Control

Trigger research on your interests:

```bash
claudius research [options]

Options:
  --now                Run research immediately
  --topic <topic>      Research specific topic
  --depth <1-5>        Research depth (default: 3)
  --model <model>      AI model to use (default: claude-opus-4)
  --output <path>      Output path for briefing
  -v, --verbose        Verbose output
```

### Briefing Queries

Query and manage research briefings:

```bash
claudius briefings list [--last <days>]           # List briefings
claudius briefings search <query> [--since <date>] # Search briefings
claudius briefings export <id> [--format <format>] # Export briefing
claudius briefings cleanup [--older-than <days>]   # Delete old briefings
```

### Configuration

Manage Claudius configuration:

```bash
claudius config show               # Show current config
claudius config edit               # Open config in editor
claudius config set <key> <value>  # Set config value
claudius config init               # Initialize config files
```

### Database

Database management commands:

```bash
claudius db stats                    # Show database statistics
claudius db export [--format <fmt>]  # Export all data
claudius db reset                    # Reset database (with confirmation)
```

## Configuration

Configuration files are stored in `~/.claudius/`:

- `interests.json` - Your research interests and blocked topics
- `mcp.json` - MCP server configurations
- `settings.json` - General settings (model, depth, schedule, etc.)
- `claudius.db` - SQLite database for briefings and insights

## Architecture

```
src/
├── index.ts              # Main entry point
├── commands/             # Command modules
│   ├── interests.ts      # Interest management
│   ├── mcp.ts            # MCP server management
│   ├── research.ts       # Research control
│   ├── briefings.ts      # Briefing queries
│   ├── config.ts         # Configuration
│   └── db.ts             # Database commands
└── utils/                # Utility modules
    ├── config.ts         # Config file utilities
    └── output.ts         # Output formatting
```

Each command module exports a `register*Commands(program)` function that sets up its commands on the main Commander program.

## Implementation Status

Most commands have placeholder implementations that log "Not implemented yet". The structure is fully in place and ready for implementation:

- Config file loading/saving is fully implemented
- Output formatting utilities are ready to use
- All command interfaces are defined and functional
- Database path and config directory setup is complete

Next steps would be to:
1. Implement actual research logic
2. Set up SQLite database schema
3. Integrate with AI models
4. Add MCP server communication
5. Implement briefing generation
