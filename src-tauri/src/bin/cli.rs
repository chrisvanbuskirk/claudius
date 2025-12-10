// Claudius CLI - Command-line interface for the AI Research Agent
//
// Usage: claudius <command> [options]

use clap::{Parser, Subcommand};
use colored::Colorize;
use comfy_table::{Table, ContentArrangement, presets::UTF8_FULL};
use chrono::Utc;
use uuid::Uuid;

use claudius::{
    db, research_state,
    Topic, ResearchAgent, BriefingCard,
    read_api_key, write_api_key, delete_api_key, has_api_key, validate_api_key,
    read_settings, write_settings,
    read_mcp_servers, write_mcp_servers, MCPServer, MCPServersConfig,
    Briefing, get_config_dir,
};

const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Helper to safely serialize JSON for output. Returns error JSON if serialization fails.
fn to_json<T: serde::Serialize>(value: &T) -> String {
    serde_json::to_string_pretty(value).unwrap_or_else(|e| {
        format!("{{\"error\": \"JSON serialization failed: {}\"}}", e)
    })
}

#[derive(Parser)]
#[command(
    name = "claudius",
    version = VERSION,
    about = "AI Research Agent CLI - Get daily briefings on topics you care about",
    long_about = None
)]
struct Cli {
    /// Output as JSON instead of formatted text
    #[arg(long, global = true)]
    json: bool,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Manage research topics
    Topics {
        #[command(subcommand)]
        action: TopicAction,
    },

    /// View and manage briefings
    Briefings {
        #[command(subcommand)]
        action: BriefingAction,
    },

    /// Run and manage research
    Research {
        #[command(subcommand)]
        action: ResearchAction,
    },

    /// Manage MCP servers
    Mcp {
        #[command(subcommand)]
        action: McpAction,
    },

    /// Manage configuration
    Config {
        #[command(subcommand)]
        action: ConfigAction,
    },
}

// ============================================================================
// Topics Commands
// ============================================================================

#[derive(Subcommand)]
enum TopicAction {
    /// List all topics
    List,
    /// Add a new topic
    Add {
        /// Topic name
        name: String,
        /// Optional description
        #[arg(short, long)]
        description: Option<String>,
    },
    /// Remove a topic
    Remove {
        /// Topic ID or name
        id: String,
    },
    /// Enable a topic
    Enable {
        /// Topic ID or name
        id: String,
    },
    /// Disable a topic
    Disable {
        /// Topic ID or name
        id: String,
    },
}

// ============================================================================
// Briefings Commands
// ============================================================================

#[derive(Subcommand)]
enum BriefingAction {
    /// List recent briefings
    List {
        /// Maximum number of briefings to show
        #[arg(short, long, default_value = "10")]
        limit: i32,
    },
    /// Show a specific briefing
    Show {
        /// Briefing ID
        id: i64,
    },
    /// Search briefings
    Search {
        /// Search query
        query: String,
    },
    /// Export a briefing
    Export {
        /// Briefing ID
        id: i64,
        /// Output format (markdown or json)
        #[arg(short, long, default_value = "markdown")]
        format: String,
    },
}

// ============================================================================
// Research Commands
// ============================================================================

#[derive(Subcommand)]
enum ResearchAction {
    /// Run research now
    Now {
        /// Only research a specific topic
        #[arg(short, long)]
        topic: Option<String>,
        /// Show verbose output
        #[arg(short, long)]
        verbose: bool,
    },
    /// Show research status
    Status,
    /// View research logs
    Logs {
        /// Maximum number of logs to show
        #[arg(short, long, default_value = "20")]
        limit: i64,
        /// Show only errors
        #[arg(short, long)]
        errors: bool,
    },
}

// ============================================================================
// MCP Commands
// ============================================================================

#[derive(Subcommand)]
enum McpAction {
    /// List MCP servers
    List,
    /// Add a new MCP server
    Add {
        /// Server name
        name: String,
        /// Command to run
        #[arg(short, long)]
        command: String,
        /// Command arguments
        #[arg(short, long)]
        args: Option<String>,
        /// Environment variables (KEY=VALUE format)
        #[arg(short, long)]
        env: Option<Vec<String>>,
    },
    /// Remove an MCP server
    Remove {
        /// Server ID or name
        id: String,
    },
    /// Enable an MCP server
    Enable {
        /// Server ID or name
        id: String,
    },
    /// Disable an MCP server
    Disable {
        /// Server ID or name
        id: String,
    },
    /// Test an MCP server connection
    Test {
        /// Server ID or name
        name: String,
    },
}

// ============================================================================
// Config Commands
// ============================================================================

#[derive(Subcommand)]
enum ConfigAction {
    /// Show current configuration
    Show,
    /// Set a configuration value
    Set {
        /// Configuration key
        key: String,
        /// Value to set
        value: String,
    },
    /// Manage API key
    #[command(name = "api-key")]
    ApiKey {
        #[command(subcommand)]
        action: ApiKeyAction,
    },
}

#[derive(Subcommand)]
enum ApiKeyAction {
    /// Check if API key is set
    Show,
    /// Set the API key
    Set {
        /// Your Anthropic API key
        key: String,
    },
    /// Clear the API key
    Clear,
}

// ============================================================================
// Main
// ============================================================================

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    // Initialize tracing for verbose output
    tracing_subscriber::fmt()
        .with_target(false)
        .init();

    let result = match cli.command {
        Commands::Topics { action } => handle_topics(action, cli.json).await,
        Commands::Briefings { action } => handle_briefings(action, cli.json).await,
        Commands::Research { action } => handle_research(action, cli.json).await,
        Commands::Mcp { action } => handle_mcp(action, cli.json).await,
        Commands::Config { action } => handle_config(action, cli.json).await,
    };

    if let Err(e) = result {
        eprintln!("{} {}", "Error:".red().bold(), e);
        std::process::exit(1);
    }
}

// ============================================================================
// Topics Handlers
// ============================================================================

async fn handle_topics(action: TopicAction, json: bool) -> Result<(), String> {
    let conn = db::get_connection()
        .map_err(|e| format!("Database connection failed: {}", e))?;

    match action {
        TopicAction::List => {
            let topics = db::get_all_topics(&conn)?;

            if json {
                println!("{}", to_json(&serde_json::json!({
                    "topics": topics
                })));
            } else if topics.is_empty() {
                println!("{}", "No topics configured.".yellow());
                println!("Add a topic with: claudius topics add <name>");
            } else {
                let mut table = Table::new();
                table.load_preset(UTF8_FULL);
                table.set_content_arrangement(ContentArrangement::Dynamic);
                table.set_header(vec!["Name", "Status", "Description", "ID"]);

                for topic in &topics {
                    let status = if topic.enabled {
                        "✓ enabled".green().to_string()
                    } else {
                        "○ disabled".dimmed().to_string()
                    };
                    let desc = topic.description.as_deref().unwrap_or("-");
                    let short_id = if topic.id.len() >= 8 { &topic.id[..8] } else { &topic.id };
                    table.add_row(vec![&topic.name, &status, desc, short_id]);
                }

                println!("{table}");
                println!("\n{} topics total", topics.len());
            }
        }

        TopicAction::Add { name, description } => {
            // Check if topic already exists
            if db::topic_name_exists(&conn, &name)? {
                return Err(format!("Topic '{}' already exists", name));
            }

            let now = Utc::now().to_rfc3339();
            let topic = Topic {
                id: Uuid::new_v4().to_string(),
                name: name.clone(),
                description,
                enabled: true,
                created_at: now.clone(),
                updated_at: now,
            };

            let sort_order = db::get_next_sort_order(&conn)?;
            db::insert_topic(&conn, &topic, sort_order)?;

            if json {
                println!("{}", to_json(&topic));
            } else {
                println!("{} Added topic '{}'", "✓".green(), name);
            }
        }

        TopicAction::Remove { id } => {
            let topic = find_topic(&conn, &id)?;
            db::delete_topic(&conn, &topic.id)?;

            if json {
                println!("{}", serde_json::json!({ "deleted": topic.id }));
            } else {
                println!("{} Removed topic '{}'", "✓".green(), topic.name);
            }
        }

        TopicAction::Enable { id } => {
            let mut topic = find_topic(&conn, &id)?;
            topic.enabled = true;
            topic.updated_at = Utc::now().to_rfc3339();
            db::update_topic(&conn, &topic)?;

            if json {
                println!("{}", to_json(&topic));
            } else {
                println!("{} Enabled topic '{}'", "✓".green(), topic.name);
            }
        }

        TopicAction::Disable { id } => {
            let mut topic = find_topic(&conn, &id)?;
            topic.enabled = false;
            topic.updated_at = Utc::now().to_rfc3339();
            db::update_topic(&conn, &topic)?;

            if json {
                println!("{}", to_json(&topic));
            } else {
                println!("{} Disabled topic '{}'", "✓".green(), topic.name);
            }
        }
    }

    Ok(())
}

fn find_topic(conn: &rusqlite::Connection, id_or_name: &str) -> Result<Topic, String> {
    // Try by ID first
    if let Some(topic) = db::get_topic_by_id(conn, id_or_name)? {
        return Ok(topic);
    }

    // Try by name (case-insensitive)
    let topics = db::get_all_topics(conn)?;
    for topic in topics {
        if topic.name.to_lowercase() == id_or_name.to_lowercase() {
            return Ok(topic);
        }
        // Also try partial ID match
        if topic.id.starts_with(id_or_name) {
            return Ok(topic);
        }
    }

    Err(format!("Topic '{}' not found", id_or_name))
}

// ============================================================================
// Briefings Handlers
// ============================================================================

async fn handle_briefings(action: BriefingAction, json: bool) -> Result<(), String> {
    let conn = db::get_connection()
        .map_err(|e| format!("Database connection failed: {}", e))?;

    match action {
        BriefingAction::List { limit } => {
            let briefings = get_briefings(&conn, limit)?;

            if json {
                let output: Vec<serde_json::Value> = briefings.iter().map(|b| {
                    let cards: Vec<BriefingCard> = serde_json::from_str(&b.cards).unwrap_or_default();
                    serde_json::json!({
                        "id": b.id,
                        "date": b.date,
                        "title": b.title,
                        "card_count": cards.len(),
                        "model_used": b.model_used,
                        "research_time_ms": b.research_time_ms,
                    })
                }).collect();
                println!("{}", to_json(&serde_json::json!({
                    "briefings": output
                })));
            } else if briefings.is_empty() {
                println!("{}", "No briefings found.".yellow());
                println!("Run research with: claudius research now");
            } else {
                let mut table = Table::new();
                table.load_preset(UTF8_FULL);
                table.set_content_arrangement(ContentArrangement::Dynamic);
                table.set_header(vec!["ID", "Date", "Title", "Cards", "Duration"]);

                for b in &briefings {
                    let cards: Vec<BriefingCard> = serde_json::from_str(&b.cards).unwrap_or_default();
                    let duration = b.research_time_ms
                        .map(|ms| format!("{}s", ms / 1000))
                        .unwrap_or("-".to_string());
                    table.add_row(vec![
                        &b.id.to_string(),
                        &b.date[..10], // Just date part
                        &b.title,
                        &cards.len().to_string(),
                        &duration,
                    ]);
                }

                println!("{table}");
            }
        }

        BriefingAction::Show { id } => {
            let briefing = get_briefing(&conn, id)?;
            let cards: Vec<BriefingCard> = serde_json::from_str(&briefing.cards)
                .map_err(|e| format!("Failed to parse cards: {}", e))?;

            if json {
                println!("{}", to_json(&serde_json::json!({
                    "id": briefing.id,
                    "date": briefing.date,
                    "title": briefing.title,
                    "cards": cards,
                    "model_used": briefing.model_used,
                    "research_time_ms": briefing.research_time_ms,
                    "total_tokens": briefing.total_tokens,
                })));
            } else {
                println!("{}", briefing.title.bold());
                println!("{}", briefing.date.dimmed());
                println!();

                for (i, card) in cards.iter().enumerate() {
                    println!("{}. {}", i + 1, card.title.cyan().bold());
                    if !card.topic.is_empty() {
                        println!("   Topic: {}", card.topic.dimmed());
                    }
                    println!();
                    println!("   {}", card.summary);
                    println!();
                    if !card.detailed_content.is_empty() {
                        println!("   {}", "Details:".yellow());
                        println!("   {}", card.detailed_content);
                        println!();
                    }
                    if !card.sources.is_empty() {
                        println!("   {}", "Sources:".dimmed());
                        for source in &card.sources {
                            println!("   - {}", source);
                        }
                        println!();
                    }
                    println!("{}", "─".repeat(60).dimmed());
                    println!();
                }

                if let Some(ms) = briefing.research_time_ms {
                    println!("Research completed in {}s", ms / 1000);
                }
            }
        }

        BriefingAction::Search { query } => {
            let briefings = search_briefings(&conn, &query)?;

            if json {
                println!("{}", to_json(&serde_json::json!({
                    "query": query,
                    "results": briefings,
                })));
            } else if briefings.is_empty() {
                println!("{}", format!("No briefings found matching '{}'", query).yellow());
            } else {
                println!("Found {} briefings matching '{}':\n", briefings.len(), query);
                for b in &briefings {
                    println!("  {} {} - {}",
                        b.id.to_string().cyan(),
                        b.date[..10].dimmed(),
                        b.title
                    );
                }
            }
        }

        BriefingAction::Export { id, format } => {
            let briefing = get_briefing(&conn, id)?;
            let cards: Vec<BriefingCard> = serde_json::from_str(&briefing.cards)
                .map_err(|e| format!("Failed to parse cards: {}", e))?;

            match format.as_str() {
                "json" => {
                    println!("{}", to_json(&serde_json::json!({
                        "id": briefing.id,
                        "date": briefing.date,
                        "title": briefing.title,
                        "cards": cards,
                    })));
                }
                "markdown" | "md" => {
                    println!("# {}", briefing.title);
                    println!("\n*{}*\n", briefing.date);

                    for card in &cards {
                        println!("## {}", card.title);
                        if !card.topic.is_empty() {
                            println!("\n**Topic:** {}\n", card.topic);
                        }
                        println!("{}\n", card.summary);
                        if !card.detailed_content.is_empty() {
                            println!("### Details\n");
                            println!("{}\n", card.detailed_content);
                        }
                        if !card.sources.is_empty() {
                            println!("### Sources\n");
                            for source in &card.sources {
                                println!("- {}", source);
                            }
                            println!();
                        }
                        println!("---\n");
                    }
                }
                _ => return Err(format!("Unknown format: {}. Use 'markdown' or 'json'", format)),
            }
        }
    }

    Ok(())
}

fn get_briefings(conn: &rusqlite::Connection, limit: i32) -> Result<Vec<Briefing>, String> {
    let mut stmt = conn.prepare(
        "SELECT id, date, title, cards, research_time_ms, model_used, total_tokens
         FROM briefings
         ORDER BY date DESC
         LIMIT ?1"
    ).map_err(|e| format!("Failed to prepare statement: {}", e))?;

    let briefings = stmt.query_map([limit], |row| {
        Ok(Briefing {
            id: row.get(0)?,
            date: row.get(1)?,
            title: row.get(2)?,
            cards: row.get(3)?,
            research_time_ms: row.get(4)?,
            model_used: row.get(5)?,
            total_tokens: row.get(6)?,
        })
    }).map_err(|e| format!("Query failed: {}", e))?
    .collect::<Result<Vec<_>, _>>()
    .map_err(|e| format!("Failed to collect results: {}", e))?;

    Ok(briefings)
}

fn get_briefing(conn: &rusqlite::Connection, id: i64) -> Result<Briefing, String> {
    let mut stmt = conn.prepare(
        "SELECT id, date, title, cards, research_time_ms, model_used, total_tokens
         FROM briefings
         WHERE id = ?1"
    ).map_err(|e| format!("Failed to prepare statement: {}", e))?;

    stmt.query_row([id], |row| {
        Ok(Briefing {
            id: row.get(0)?,
            date: row.get(1)?,
            title: row.get(2)?,
            cards: row.get(3)?,
            research_time_ms: row.get(4)?,
            model_used: row.get(5)?,
            total_tokens: row.get(6)?,
        })
    }).map_err(|e| format!("Briefing not found: {}", e))
}

fn search_briefings(conn: &rusqlite::Connection, query: &str) -> Result<Vec<Briefing>, String> {
    let search_pattern = format!("%{}%", query);

    let mut stmt = conn.prepare(
        "SELECT id, date, title, cards, research_time_ms, model_used, total_tokens
         FROM briefings
         WHERE title LIKE ?1 OR cards LIKE ?1
         ORDER BY date DESC
         LIMIT 50"
    ).map_err(|e| format!("Failed to prepare statement: {}", e))?;

    let briefings = stmt.query_map([&search_pattern], |row| {
        Ok(Briefing {
            id: row.get(0)?,
            date: row.get(1)?,
            title: row.get(2)?,
            cards: row.get(3)?,
            research_time_ms: row.get(4)?,
            model_used: row.get(5)?,
            total_tokens: row.get(6)?,
        })
    }).map_err(|e| format!("Query failed: {}", e))?
    .collect::<Result<Vec<_>, _>>()
    .map_err(|e| format!("Failed to collect results: {}", e))?;

    Ok(briefings)
}

// ============================================================================
// Research Handlers
// ============================================================================

async fn handle_research(action: ResearchAction, json: bool) -> Result<(), String> {
    match action {
        ResearchAction::Now { topic, verbose } => {
            // Check for API key
            let api_key = require_api_key()?;

            // Get settings
            let settings = read_settings().unwrap_or_default();

            // Get topics
            let conn = db::get_connection()
                .map_err(|e| format!("Database connection failed: {}", e))?;
            let all_topics = db::get_all_topics(&conn)?;

            let topics: Vec<String> = if let Some(ref specific_topic) = topic {
                // Find the specific topic
                let found = all_topics.iter().find(|t| {
                    t.name.to_lowercase() == specific_topic.to_lowercase()
                });
                match found {
                    Some(t) => vec![t.name.clone()],
                    None => return Err(format!("Topic '{}' not found", specific_topic)),
                }
            } else {
                // Get all enabled topics
                all_topics.iter()
                    .filter(|t| t.enabled)
                    .map(|t| t.name.clone())
                    .collect()
            };

            if topics.is_empty() {
                return Err("No topics to research. Add topics with: claudius topics add <name>".to_string());
            }

            if !json {
                println!("{} Starting research on {} topic(s)...", "→".cyan(), topics.len());
                if verbose {
                    for t in &topics {
                        println!("  • {}", t);
                    }
                }
                println!();
            }

            // Create research agent and run in background for progress tracking
            let mut agent = ResearchAgent::new(
                api_key,
                Some(settings.model.clone()),
                settings.enable_web_search,
            );

            let start = std::time::Instant::now();

            // Spawn research on a background task
            let research_handle = tokio::spawn(async move {
                agent.run_research(topics, None).await
            });

            // Poll for progress updates (only in non-JSON mode)
            let mut last_phase = String::new();
            if !json {
                loop {
                    tokio::time::sleep(std::time::Duration::from_millis(500)).await;

                    let state = research_state::get_state();

                    // Print phase changes
                    if state.current_phase != last_phase && !state.current_phase.is_empty() {
                        // Clear the line and print new phase
                        print!("\r{} {}                    ", "→".cyan(), state.current_phase);
                        use std::io::Write;
                        std::io::stdout().flush().ok();
                        last_phase = state.current_phase.clone();
                    }

                    // Check if research is done
                    if research_handle.is_finished() {
                        println!(); // New line after progress
                        break;
                    }
                }
            }

            // Get the result
            let result = research_handle.await
                .map_err(|e| format!("Research task failed: {}", e))?
                .map_err(|e| e)?;
            let duration = start.elapsed();

            // Save to database
            let cards_json = serde_json::to_string(&result.cards)
                .map_err(|e| format!("Failed to serialize cards: {}", e))?;

            conn.execute(
                "INSERT INTO briefings (date, title, cards, research_time_ms, model_used, total_tokens)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                rusqlite::params![
                    result.date,
                    result.title,
                    cards_json,
                    result.research_time_ms as i64,
                    result.model_used,
                    result.total_tokens as i64,
                ],
            ).map_err(|e| format!("Failed to save briefing: {}", e))?;

            if json {
                println!("{}", to_json(&serde_json::json!({
                    "status": "completed",
                    "title": result.title,
                    "cards": result.cards.len(),
                    "duration_ms": duration.as_millis(),
                    "model": result.model_used,
                    "tokens": result.total_tokens,
                })));
            } else {
                println!("{} Research completed!", "✓".green().bold());
                println!();
                println!("  {} briefing cards generated", result.cards.len().to_string().cyan());
                println!("  Duration: {}s", duration.as_secs());
                println!("  Model: {}", result.model_used.dimmed());
                println!();
                println!("View with: claudius briefings list");
            }
        }

        ResearchAction::Status => {
            let state = research_state::get_state();

            if json {
                let started_at = state.started_at.map(|t| {
                    chrono::DateTime::<chrono::Utc>::from(t).to_rfc3339()
                });
                println!("{}", to_json(&serde_json::json!({
                    "is_running": state.is_running,
                    "current_phase": state.current_phase,
                    "started_at": started_at,
                    "is_cancelled": research_state::is_cancelled(),
                })));
            } else if state.is_running {
                println!("{} Research is running", "●".yellow());
                println!("  Phase: {}", state.current_phase.cyan());
                if let Some(started) = state.started_at {
                    let elapsed = std::time::SystemTime::now()
                        .duration_since(started)
                        .unwrap_or_default();
                    println!("  Running for: {}s", elapsed.as_secs());
                }
            } else {
                println!("{} No research currently running", "○".dimmed());
            }
        }

        ResearchAction::Logs { limit, errors } => {
            use claudius::research_log::ResearchLogger;

            let logs = if errors {
                ResearchLogger::get_actionable_errors(limit)?
            } else {
                ResearchLogger::get_logs(None, limit)?
            };

            if json {
                println!("{}", to_json(&serde_json::json!({
                    "logs": logs
                })));
            } else if logs.is_empty() {
                println!("{}", "No research logs found.".dimmed());
            } else {
                for log in &logs {
                    let type_color = if !log.success {
                        log.log_type.red()
                    } else {
                        log.log_type.normal()
                    };
                    let message = log.error_message.as_deref()
                        .or(log.output_summary.as_deref())
                        .unwrap_or(&log.log_type);
                    let topic_info = log.topic.as_ref()
                        .map(|t| format!(" [{}]", t))
                        .unwrap_or_default();
                    println!("[{}] {}{} {}",
                        log.created_at[..19].dimmed(),
                        type_color,
                        topic_info.cyan(),
                        message
                    );
                }
            }
        }
    }

    Ok(())
}

fn require_api_key() -> Result<String, String> {
    read_api_key().ok_or_else(|| {
        format!(
            "{}\n\n{}\n  {}\n\n{}\n  {}",
            "Error: No API key configured.".red().bold(),
            "Set your Anthropic API key with:",
            "claudius config api-key set <YOUR_KEY>".cyan(),
            "Or create ~/.claudius/.env with:",
            "ANTHROPIC_API_KEY=sk-ant-...".dimmed()
        )
    })
}

// ============================================================================
// MCP Handlers
// ============================================================================

async fn handle_mcp(action: McpAction, json: bool) -> Result<(), String> {
    match action {
        McpAction::List => {
            let config = read_mcp_servers()?;

            if json {
                println!("{}", to_json(&serde_json::json!({
                    "servers": config.servers
                })));
            } else if config.servers.is_empty() {
                println!("{}", "No MCP servers configured.".yellow());
                println!("Add a server with: claudius mcp add <name> --command <cmd>");
            } else {
                let mut table = Table::new();
                table.load_preset(UTF8_FULL);
                table.set_content_arrangement(ContentArrangement::Dynamic);
                table.set_header(vec!["Name", "Status", "Command", "ID"]);

                for server in &config.servers {
                    let status = if server.enabled {
                        "✓ enabled".green().to_string()
                    } else {
                        "○ disabled".dimmed().to_string()
                    };
                    let command = server.config.get("command")
                        .and_then(|v| v.as_str())
                        .unwrap_or("-");
                    let short_id = if server.id.len() >= 8 { &server.id[..8] } else { &server.id };
                    table.add_row(vec![&server.name, &status, command, short_id]);
                }

                println!("{table}");
                println!("\n{} servers total", config.servers.len());
            }
        }

        McpAction::Add { name, command, args, env } => {
            let mut config = read_mcp_servers()?;

            // Build config object
            let mut server_config = serde_json::json!({
                "command": command,
            });

            if let Some(args_str) = args {
                let args_vec: Vec<&str> = args_str.split_whitespace().collect();
                server_config["args"] = serde_json::json!(args_vec);
            }

            if let Some(env_vars) = env {
                let mut env_map = serde_json::Map::new();
                for var in env_vars {
                    if let Some((key, value)) = var.split_once('=') {
                        env_map.insert(key.to_string(), serde_json::json!(value));
                    }
                }
                if !env_map.is_empty() {
                    server_config["env"] = serde_json::Value::Object(env_map);
                }
            }

            let server = MCPServer {
                id: Uuid::new_v4().to_string(),
                name: name.clone(),
                enabled: true,
                config: server_config,
                last_used: None,
            };

            config.servers.push(server.clone());
            write_mcp_servers(&config)?;

            if json {
                println!("{}", to_json(&server));
            } else {
                println!("{} Added MCP server '{}'", "✓".green(), name);
            }
        }

        McpAction::Remove { id } => {
            let mut config = read_mcp_servers()?;
            let server = find_mcp_server(&config, &id)?;
            let name = server.name.clone();

            config.servers.retain(|s| s.id != server.id);
            write_mcp_servers(&config)?;

            if json {
                println!("{}", serde_json::json!({ "deleted": server.id }));
            } else {
                println!("{} Removed MCP server '{}'", "✓".green(), name);
            }
        }

        McpAction::Enable { id } => {
            let mut config = read_mcp_servers()?;
            let server = find_mcp_server_mut(&mut config, &id)?;
            server.enabled = true;
            let name = server.name.clone();
            let server_clone = server.clone();
            write_mcp_servers(&config)?;

            if json {
                println!("{}", to_json(&server_clone));
            } else {
                println!("{} Enabled MCP server '{}'", "✓".green(), name);
            }
        }

        McpAction::Disable { id } => {
            let mut config = read_mcp_servers()?;
            let server = find_mcp_server_mut(&mut config, &id)?;
            server.enabled = false;
            let name = server.name.clone();
            let server_clone = server.clone();
            write_mcp_servers(&config)?;

            if json {
                println!("{}", to_json(&server_clone));
            } else {
                println!("{} Disabled MCP server '{}'", "✓".green(), name);
            }
        }

        McpAction::Test { name } => {
            let config = read_mcp_servers()?;
            let server = find_mcp_server(&config, &name)?;

            if !json {
                println!("{} Testing MCP server '{}'...", "→".cyan(), server.name);
            }

            // Convert MCPServer to McpServerConfig for the client
            let server_config = claudius::mcp_client::McpServerConfig {
                id: server.id.clone(),
                name: server.name.clone(),
                enabled: true, // Force enabled for testing
                config: server.config.clone(),
                last_used: None,
            };

            match claudius::mcp_client::McpClient::connect(vec![server_config]).await {
                Ok(client) => {
                    let tools = client.get_all_tools();

                    if json {
                        println!("{}", to_json(&serde_json::json!({
                            "status": "success",
                            "server": server.name,
                            "tools": tools.len(),
                        })));
                    } else {
                        println!("{} Connection successful!", "✓".green());
                        println!("  Available tools: {}", tools.len());
                        for tool in &tools {
                            println!("    • {}", tool.tool.name);
                        }
                    }
                }
                Err(e) => {
                    if json {
                        println!("{}", to_json(&serde_json::json!({
                            "status": "error",
                            "server": server.name,
                            "error": e,
                        })));
                    } else {
                        println!("{} Connection failed: {}", "✗".red(), e);
                    }
                }
            }
        }
    }

    Ok(())
}

fn find_mcp_server(config: &MCPServersConfig, id_or_name: &str) -> Result<MCPServer, String> {
    for server in &config.servers {
        if server.id == id_or_name ||
           server.name.to_lowercase() == id_or_name.to_lowercase() ||
           server.id.starts_with(id_or_name) {
            return Ok(server.clone());
        }
    }
    Err(format!("MCP server '{}' not found", id_or_name))
}

fn find_mcp_server_mut<'a>(config: &'a mut MCPServersConfig, id_or_name: &str) -> Result<&'a mut MCPServer, String> {
    for server in &mut config.servers {
        if server.id == id_or_name ||
           server.name.to_lowercase() == id_or_name.to_lowercase() ||
           server.id.starts_with(id_or_name) {
            return Ok(server);
        }
    }
    Err(format!("MCP server '{}' not found", id_or_name))
}

// ============================================================================
// Config Handlers
// ============================================================================

async fn handle_config(action: ConfigAction, json: bool) -> Result<(), String> {
    match action {
        ConfigAction::Show => {
            let settings = read_settings().unwrap_or_default();
            let has_key = has_api_key();
            let config_dir = get_config_dir();

            if json {
                println!("{}", to_json(&serde_json::json!({
                    "config_dir": config_dir.display().to_string(),
                    "api_key_set": has_key,
                    "settings": settings,
                })));
            } else {
                println!("{}", "Configuration".bold());
                println!();
                println!("  Config directory: {}", config_dir.display().to_string().dimmed());
                println!("  API key: {}", if has_key { "✓ set".green().to_string() } else { "✗ not set".red().to_string() });
                println!();
                println!("{}", "Research Settings".bold());
                println!();
                println!("  Model: {}", settings.model.cyan());
                println!("  Schedule: {}", settings.schedule_cron);
                println!("  Research depth: {}", settings.research_depth);
                println!("  Max sources per topic: {}", settings.max_sources_per_topic);
                println!("  Notifications: {}", if settings.enable_notifications { "enabled" } else { "disabled" });
                println!("  Web search: {}", if settings.enable_web_search { "enabled" } else { "disabled" });
            }
        }

        ConfigAction::Set { key, value } => {
            let mut settings = read_settings().unwrap_or_default();

            match key.as_str() {
                "model" => settings.model = value.clone(),
                "schedule" | "schedule_cron" => settings.schedule_cron = value.clone(),
                "research_depth" | "depth" => settings.research_depth = value.clone(),
                "max_sources" | "max_sources_per_topic" => {
                    settings.max_sources_per_topic = value.parse()
                        .map_err(|_| "Invalid number for max_sources")?;
                }
                "notifications" | "enable_notifications" => {
                    settings.enable_notifications = value.parse()
                        .map_err(|_| "Invalid boolean for notifications")?;
                }
                "web_search" | "enable_web_search" => {
                    settings.enable_web_search = value.parse()
                        .map_err(|_| "Invalid boolean for web_search")?;
                }
                _ => return Err(format!("Unknown config key: {}", key)),
            }

            write_settings(&settings)?;

            if json {
                println!("{}", serde_json::json!({ "updated": key, "value": value }));
            } else {
                println!("{} Set {} = {}", "✓".green(), key, value);
            }
        }

        ConfigAction::ApiKey { action } => {
            match action {
                ApiKeyAction::Show => {
                    if has_api_key() {
                        if json {
                            println!("{}", serde_json::json!({ "api_key_set": true }));
                        } else {
                            println!("{} API key is configured", "✓".green());
                        }
                    } else {
                        if json {
                            println!("{}", serde_json::json!({ "api_key_set": false }));
                        } else {
                            println!("{} No API key configured", "✗".red());
                            println!("\nSet with: claudius config api-key set <YOUR_KEY>");
                        }
                    }
                }

                ApiKeyAction::Set { key } => {
                    validate_api_key(&key)?;
                    write_api_key(&key)?;

                    if json {
                        println!("{}", serde_json::json!({ "status": "success" }));
                    } else {
                        println!("{} API key saved", "✓".green());
                    }
                }

                ApiKeyAction::Clear => {
                    delete_api_key()?;

                    if json {
                        println!("{}", serde_json::json!({ "status": "cleared" }));
                    } else {
                        println!("{} API key cleared", "✓".green());
                    }
                }
            }
        }
    }

    Ok(())
}
