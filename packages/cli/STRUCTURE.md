# CLI Structure Overview

## File Organization

```
packages/cli/
├── package.json                    # Package configuration with ES modules
├── tsconfig.json                   # TypeScript config extending base
├── README.md                       # Usage documentation
├── .gitignore                      # Git ignore rules
├── src/
│   ├── index.ts                    # Main entry point with shebang
│   ├── commands/                   # Command modules
│   │   ├── interests.ts            # Interest management commands
│   │   ├── mcp.ts                  # MCP server management commands
│   │   ├── research.ts             # Research trigger command
│   │   ├── briefings.ts            # Briefing query commands
│   │   ├── config.ts               # Configuration commands
│   │   └── db.ts                   # Database management commands
│   └── utils/                      # Shared utilities
│       ├── config.ts               # Config file I/O utilities
│       └── output.ts               # Output formatting utilities
└── dist/                           # Compiled output (generated)
    └── (compiled .js files)
```

## Command Registration Pattern

Each command module follows this pattern:

```typescript
import { Command } from 'commander';
import { success, error, info, warn, table, spinner } from '../utils/output.js';
import { loadConfig, saveConfig } from '../utils/config.js';

export function register*Commands(program: Command): void {
  const cmd = program
    .command('cmdname')
    .description('Command description');

  cmd
    .command('subcommand')
    .description('Subcommand description')
    .option('--flag', 'Flag description')
    .action((args, options) => {
      // Implementation
    });
}
```

## Configuration Storage

All configuration is stored in `~/.claudius/`:

- `interests.json` - User interests and blocked topics
- `mcp.json` - MCP server configurations  
- `settings.json` - General settings
- `claudius.db` - SQLite database for briefings

## Key Design Decisions

1. **ES Modules**: Uses `"type": "module"` for modern JavaScript
2. **Commander.js**: Industry-standard CLI parsing with subcommands
3. **Modular Commands**: Each command group in separate file for maintainability
4. **Functional Utilities**: Shared formatting and config functions
5. **Placeholder Implementations**: Structure in place, ready for implementation
6. **Type Safety**: Full TypeScript with strict mode enabled

## Building and Running

```bash
# Build TypeScript to JavaScript
npm run build

# Watch mode for development
npm run dev

# Run the CLI
node dist/index.js <command>

# Or install globally
npm link
claudius <command>
```

## Next Implementation Steps

1. Set up SQLite database schema and migrations
2. Implement research logic with AI model integration
3. Add MCP server communication layer
4. Create briefing generation pipeline
5. Add interactive prompts for complex commands (using inquirer)
6. Implement export formats (markdown, PDF, JSON)
7. Add scheduling/cron integration for automated research
