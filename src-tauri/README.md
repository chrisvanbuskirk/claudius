# Claudius Tauri Backend

This directory contains the Rust backend for the Claudius desktop application built with Tauri.

## Structure

```
src-tauri/
├── Cargo.toml           # Rust dependencies and project configuration
├── build.rs             # Tauri build script
├── tauri.conf.json      # Tauri configuration
├── icons/               # Application icons (to be added)
└── src/
    ├── main.rs          # Entry point, sets up Tauri app
    ├── db.rs            # SQLite database initialization and connection
    ├── schema.sql       # Database schema
    ├── commands.rs      # Tauri commands (IPC handlers)
    └── scheduler.rs     # Background research scheduler
```

## Commands

The following Tauri commands are exposed to the frontend:

### Briefing Management
- `get_briefings(limit: Option<i32>)` - Get recent briefings
- `get_briefing(id: i64)` - Get a specific briefing
- `search_briefings(query: String)` - Search briefings by text

### Feedback
- `add_feedback(briefing_id: i64, card_index: i32, rating: i32, reason: Option<String>)` - Add user feedback

### User Interests
- `get_interests()` - Get user's research interests
- `add_interest(topic: String)` - Add a new interest
- `remove_interest(topic: String)` - Remove an interest

### Preferences
- `get_preferences()` - Get user preferences
- `update_preferences(preferences: Value)` - Update preferences

### Research
- `trigger_research()` - Manually trigger a research run

## Database

The SQLite database is stored at `~/.claudius/claudius.db` with the following tables:

- `briefings` - Stores generated briefings with metadata
- `feedback` - Stores user feedback on briefing cards

## Configuration

User configuration is stored at `~/.claudius/config.json` with:

```json
{
  "interests": ["topic1", "topic2"],
  "preferences": {
    "schedule": "0 6 * * *",
    "briefingLength": "medium",
    "notificationsEnabled": true
  }
}
```

## Building

```bash
# Check for errors
cargo check

# Build for development
cargo build

# Build for production
cargo build --release
```

## Running

The Tauri app is typically run through the npm workspace:

```bash
npm run tauri dev
```

## Dependencies

- **tauri** - Core Tauri framework
- **tauri-plugin-shell** - Shell command execution
- **rusqlite** - SQLite database
- **tokio** - Async runtime
- **serde/serde_json** - JSON serialization
- **chrono** - Date/time handling
- **dirs** - Cross-platform directory paths
