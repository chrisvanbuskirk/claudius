// Claudius CLI - Command-line interface for the AI Research Agent
//
// Usage: claudius <command> [options]

use chrono::Utc;
use clap::{Parser, Subcommand};
use colored::Colorize;
use comfy_table::{presets::UTF8_FULL, ContentArrangement, Table};
use scopeguard::defer;
use uuid::Uuid;

use claudius::{
    db, delete_api_key, get_config_dir, has_api_key, image_gen, read_api_key, read_mcp_servers,
    read_openai_api_key, read_settings, research_state, validate_api_key, write_api_key,
    write_mcp_servers, write_settings, Briefing, BriefingCard, MCPServer, MCPServersConfig,
    ResearchAgent, Topic,
};

const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Helper to safely serialize JSON for output. Returns error JSON if serialization fails.
fn to_json<T: serde::Serialize>(value: &T) -> String {
    serde_json::to_string_pretty(value)
        .unwrap_or_else(|e| format!("{{\"error\": \"JSON serialization failed: {}\"}}", e))
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

    /// Database housekeeping and cleanup
    Housekeeping {
        #[command(subcommand)]
        action: HousekeepingAction,
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
// Housekeeping Commands
// ============================================================================

#[derive(Subcommand)]
enum HousekeepingAction {
    /// Run cleanup based on retention settings
    Run {
        /// Dry run (show what would be deleted without deleting)
        #[arg(short, long)]
        dry_run: bool,
    },
    /// Show housekeeping status (briefing counts, etc.)
    Status,
    /// Optimize database (run VACUUM)
    Optimize,
}

// ============================================================================
// Main
// ============================================================================

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    // Initialize tracing for verbose output
    tracing_subscriber::fmt().with_target(false).init();

    let result = match cli.command {
        Commands::Topics { action } => handle_topics(action, cli.json).await,
        Commands::Briefings { action } => handle_briefings(action, cli.json).await,
        Commands::Research { action } => handle_research(action, cli.json).await,
        Commands::Mcp { action } => handle_mcp(action, cli.json).await,
        Commands::Config { action } => handle_config(action, cli.json).await,
        Commands::Housekeeping { action } => handle_housekeeping(action, cli.json).await,
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
    let conn = db::get_connection().map_err(|e| format!("Database connection failed: {}", e))?;

    match action {
        TopicAction::List => {
            let topics = db::get_all_topics(&conn)?;

            if json {
                println!(
                    "{}",
                    to_json(&serde_json::json!({
                        "topics": topics
                    }))
                );
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
                    let short_id = if topic.id.len() >= 8 {
                        &topic.id[..8]
                    } else {
                        &topic.id
                    };
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
    let conn = db::get_connection().map_err(|e| format!("Database connection failed: {}", e))?;

    match action {
        BriefingAction::List { limit } => {
            let briefings = get_briefings(&conn, limit)?;

            if json {
                let output: Vec<serde_json::Value> = briefings
                    .iter()
                    .map(|b| {
                        let cards: Vec<BriefingCard> =
                            serde_json::from_str(&b.cards).unwrap_or_default();
                        serde_json::json!({
                            "id": b.id,
                            "date": b.date,
                            "title": b.title,
                            "card_count": cards.len(),
                            "model_used": b.model_used,
                            "research_time_ms": b.research_time_ms,
                        })
                    })
                    .collect();
                println!(
                    "{}",
                    to_json(&serde_json::json!({
                        "briefings": output
                    }))
                );
            } else if briefings.is_empty() {
                println!("{}", "No briefings found.".yellow());
                println!("Run research with: claudius research now");
            } else {
                let mut table = Table::new();
                table.load_preset(UTF8_FULL);
                table.set_content_arrangement(ContentArrangement::Dynamic);
                table.set_header(vec!["ID", "Date", "Title", "Cards", "Duration"]);

                for b in &briefings {
                    let cards: Vec<BriefingCard> =
                        serde_json::from_str(&b.cards).unwrap_or_default();
                    let duration = b
                        .research_time_ms
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
                println!(
                    "{}",
                    to_json(&serde_json::json!({
                        "id": briefing.id,
                        "date": briefing.date,
                        "title": briefing.title,
                        "cards": cards,
                        "model_used": briefing.model_used,
                        "research_time_ms": briefing.research_time_ms,
                        "total_tokens": briefing.total_tokens,
                    }))
                );
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
                println!(
                    "{}",
                    to_json(&serde_json::json!({
                        "query": query,
                        "results": briefings,
                    }))
                );
            } else if briefings.is_empty() {
                println!(
                    "{}",
                    format!("No briefings found matching '{}'", query).yellow()
                );
            } else {
                println!(
                    "Found {} briefings matching '{}':\n",
                    briefings.len(),
                    query
                );
                for b in &briefings {
                    println!(
                        "  {} {} - {}",
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
                    println!(
                        "{}",
                        to_json(&serde_json::json!({
                            "id": briefing.id,
                            "date": briefing.date,
                            "title": briefing.title,
                            "cards": cards,
                        }))
                    );
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
                _ => {
                    return Err(format!(
                        "Unknown format: {}. Use 'markdown' or 'json'",
                        format
                    ))
                }
            }
        }
    }

    Ok(())
}

fn get_briefings(conn: &rusqlite::Connection, limit: i32) -> Result<Vec<Briefing>, String> {
    let mut stmt = conn
        .prepare(
            "SELECT id, date, title, cards, research_time_ms, model_used, total_tokens
         FROM briefings
         ORDER BY date DESC
         LIMIT ?1",
        )
        .map_err(|e| format!("Failed to prepare statement: {}", e))?;

    let briefings = stmt
        .query_map([limit], |row| {
            Ok(Briefing {
                id: row.get(0)?,
                date: row.get(1)?,
                title: row.get(2)?,
                cards: row.get(3)?,
                research_time_ms: row.get(4)?,
                model_used: row.get(5)?,
                total_tokens: row.get(6)?,
            })
        })
        .map_err(|e| format!("Query failed: {}", e))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| format!("Failed to collect results: {}", e))?;

    Ok(briefings)
}

fn get_briefing(conn: &rusqlite::Connection, id: i64) -> Result<Briefing, String> {
    let mut stmt = conn
        .prepare(
            "SELECT id, date, title, cards, research_time_ms, model_used, total_tokens
         FROM briefings
         WHERE id = ?1",
        )
        .map_err(|e| format!("Failed to prepare statement: {}", e))?;

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
    })
    .map_err(|e| format!("Briefing not found: {}", e))
}

fn search_briefings(conn: &rusqlite::Connection, query: &str) -> Result<Vec<Briefing>, String> {
    let search_pattern = format!("%{}%", query);

    let mut stmt = conn
        .prepare(
            "SELECT id, date, title, cards, research_time_ms, model_used, total_tokens
         FROM briefings
         WHERE title LIKE ?1 OR cards LIKE ?1
         ORDER BY date DESC
         LIMIT 50",
        )
        .map_err(|e| format!("Failed to prepare statement: {}", e))?;

    let briefings = stmt
        .query_map([&search_pattern], |row| {
            Ok(Briefing {
                id: row.get(0)?,
                date: row.get(1)?,
                title: row.get(2)?,
                cards: row.get(3)?,
                research_time_ms: row.get(4)?,
                model_used: row.get(5)?,
                total_tokens: row.get(6)?,
            })
        })
        .map_err(|e| format!("Query failed: {}", e))?
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
            let conn =
                db::get_connection().map_err(|e| format!("Database connection failed: {}", e))?;
            let all_topics = db::get_all_topics(&conn)?;

            let topics: Vec<String> = if let Some(ref specific_topic) = topic {
                // Find the specific topic
                let found = all_topics
                    .iter()
                    .find(|t| t.name.to_lowercase() == specific_topic.to_lowercase());
                match found {
                    Some(t) => vec![t.name.clone()],
                    None => return Err(format!("Topic '{}' not found", specific_topic)),
                }
            } else {
                // Get all enabled topics
                all_topics
                    .iter()
                    .filter(|t| t.enabled)
                    .map(|t| t.name.clone())
                    .collect()
            };

            if topics.is_empty() {
                return Err(
                    "No topics to research. Add topics with: claudius topics add <name>"
                        .to_string(),
                );
            }

            if !json {
                println!(
                    "{} Starting research on {} topic(s)...",
                    "→".cyan(),
                    topics.len()
                );
                if verbose {
                    for t in &topics {
                        println!("  • {}", t);
                    }
                }
                println!();
            }

            // Load past card fingerprints for deduplication
            let (past_cards_context, past_fingerprints) = if settings.dedup_days > 0 {
                match db::get_recent_card_fingerprints(&conn, settings.dedup_days) {
                    Ok(fingerprints) => {
                        let context = claudius::dedup::format_past_cards_for_prompt(&fingerprints);
                        if verbose && !json && !fingerprints.is_empty() {
                            println!(
                                "{} Loaded {} past cards for dedup",
                                "→".cyan(),
                                fingerprints.len()
                            );
                        }
                        (Some(context), fingerprints)
                    }
                    Err(e) => {
                        if verbose && !json {
                            eprintln!("{} Dedup unavailable: {}", "Warning:".yellow(), e);
                        }
                        (None, Vec::new())
                    }
                }
            } else {
                (None, Vec::new())
            };

            // Set running state BEFORE spawning to prevent race conditions
            let _cancellation_token = research_state::set_running("starting")
                .map_err(|e| format!("Cannot start research: {}", e))?;

            // RAII guard: ensure cleanup even if we panic or return early
            defer! {
                if let Err(e) = research_state::set_stopped() {
                    eprintln!("{} Failed to reset research state: {}", "Warning:".yellow(), e);
                }
            }

            // Create research agent and run in background for progress tracking
            let mut agent = ResearchAgent::new(
                api_key,
                Some(settings.model.clone()),
                settings.enable_web_search,
                settings.research_mode.clone(),
            );

            let start = std::time::Instant::now();
            let condense = settings.condense_briefings;
            let dedup_threshold = settings.dedup_threshold;

            // Spawn research on a background task
            let research_handle = tokio::spawn(async move {
                agent
                    .run_research(topics, None, condense, past_cards_context)
                    .await
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
                        print!(
                            "\r{} {}                    ",
                            "→".cyan(),
                            state.current_phase
                        );
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

            // Get the result - ensure cleanup happens regardless of success/failure
            let research_result = research_handle
                .await
                .map_err(|e| format!("Research task failed: {}", e))
                .and_then(|r| r);

            let duration = start.elapsed();

            // Note: cleanup is handled by defer! guard above (panic-safe)

            // Now handle the result
            let mut result = research_result?;

            // Apply post-synthesis deduplication filter (safety net)
            if !past_fingerprints.is_empty() && dedup_threshold > 0.0 {
                let original_count = result.cards.len();
                result.cards = claudius::dedup::filter_duplicates(
                    result.cards,
                    &past_fingerprints,
                    dedup_threshold,
                );
                let filtered_count = original_count - result.cards.len();
                if filtered_count > 0 && verbose && !json {
                    println!("{} Filtered {} duplicate cards", "→".cyan(), filtered_count);
                }
            }

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

            let briefing_id = conn.last_insert_rowid();

            // Generate images for cards that have image_prompt (if enabled and API key configured)
            if settings.enable_image_generation {
                if let Some(openai_key) = read_openai_api_key() {
                    if !json {
                        println!("{} Generating header images...", "→".cyan());
                    }

                    let mut images_generated = 0;
                    for (idx, card) in result.cards.iter_mut().enumerate() {
                        if let Some(ref prompt) = card.image_prompt {
                            if verbose && !json {
                                println!("  {} Generating image for card {}...", "→".dimmed(), idx);
                            }

                            match image_gen::generate_image(prompt, briefing_id, idx, &openai_key)
                                .await
                            {
                                image_gen::ImageGenResult::Success(path) => {
                                    card.image_path = Some(path.to_string_lossy().to_string());
                                    images_generated += 1;
                                    if verbose && !json {
                                        println!("    {} Image saved", "✓".green());
                                    }
                                }
                                image_gen::ImageGenResult::Disabled => {
                                    if verbose && !json {
                                        println!("    {} Image generation disabled", "○".dimmed());
                                    }
                                    break;
                                }
                                image_gen::ImageGenResult::NoApiKey => {
                                    if !json {
                                        println!(
                                            "    {} No OpenAI API key configured",
                                            "!".yellow()
                                        );
                                    }
                                    break;
                                }
                                image_gen::ImageGenResult::Failed(err) => {
                                    if verbose && !json {
                                        println!("    {} Failed: {}", "✗".red(), err);
                                    }
                                    // Continue with other cards
                                }
                            }
                        }
                    }

                    // Update briefing with image paths if any were generated
                    if images_generated > 0 {
                        let updated_cards_json = serde_json::to_string(&result.cards)
                            .map_err(|e| format!("Failed to serialize updated cards: {}", e))?;

                        conn.execute(
                            "UPDATE briefings SET cards = ?1 WHERE id = ?2",
                            rusqlite::params![updated_cards_json, briefing_id],
                        )
                        .map_err(|e| {
                            format!("Failed to update briefing with image paths: {}", e)
                        })?;

                        if !json {
                            println!("{} Generated {} images", "✓".green(), images_generated);
                        }
                    }
                } else if verbose && !json {
                    println!(
                        "{} Image generation enabled but no OpenAI API key configured",
                        "!".yellow()
                    );
                }
            }

            if json {
                println!(
                    "{}",
                    to_json(&serde_json::json!({
                        "status": "completed",
                        "title": result.title,
                        "cards": result.cards.len(),
                        "duration_ms": duration.as_millis(),
                        "model": result.model_used,
                        "tokens": result.total_tokens,
                    }))
                );
            } else {
                println!("{} Research completed!", "✓".green().bold());
                println!();
                println!(
                    "  {} briefing cards generated",
                    result.cards.len().to_string().cyan()
                );
                println!("  Duration: {}s", duration.as_secs());
                println!("  Model: {}", result.model_used.dimmed());
                println!();
                println!("View with: claudius briefings list");
            }

            // Try to refresh the desktop app if it's running
            // This uses the single-instance plugin to send a refresh signal
            if let Err(e) = std::process::Command::new("open")
                .args(["-a", "Claudius", "--args", "--refresh"])
                .stdout(std::process::Stdio::null())
                .stderr(std::process::Stdio::null())
                .spawn()
            {
                // Silently ignore if app isn't installed or can't be opened
                if verbose && !json {
                    println!("{} Could not refresh desktop app: {}", "!".dimmed(), e);
                }
            }
        }

        ResearchAction::Status => {
            let state = research_state::get_state();

            if json {
                let started_at = state
                    .started_at
                    .map(|t| chrono::DateTime::<chrono::Utc>::from(t).to_rfc3339());
                println!(
                    "{}",
                    to_json(&serde_json::json!({
                        "is_running": state.is_running,
                        "current_phase": state.current_phase,
                        "started_at": started_at,
                        "is_cancelled": research_state::is_cancelled(),
                    }))
                );
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
                println!(
                    "{}",
                    to_json(&serde_json::json!({
                        "logs": logs
                    }))
                );
            } else if logs.is_empty() {
                println!("{}", "No research logs found.".dimmed());
            } else {
                for log in &logs {
                    let type_color = if !log.success {
                        log.log_type.red()
                    } else {
                        log.log_type.normal()
                    };
                    let message = log
                        .error_message
                        .as_deref()
                        .or(log.output_summary.as_deref())
                        .unwrap_or(&log.log_type);
                    let topic_info = log
                        .topic
                        .as_ref()
                        .map(|t| format!(" [{}]", t))
                        .unwrap_or_default();
                    println!(
                        "[{}] {}{} {}",
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
                println!(
                    "{}",
                    to_json(&serde_json::json!({
                        "servers": config.servers
                    }))
                );
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
                    let command = server
                        .config
                        .get("command")
                        .and_then(|v| v.as_str())
                        .unwrap_or("-");
                    let short_id = if server.id.len() >= 8 {
                        &server.id[..8]
                    } else {
                        &server.id
                    };
                    table.add_row(vec![&server.name, &status, command, short_id]);
                }

                println!("{table}");
                println!("\n{} servers total", config.servers.len());
            }
        }

        McpAction::Add {
            name,
            command,
            args,
            env,
        } => {
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
                        println!(
                            "{}",
                            to_json(&serde_json::json!({
                                "status": "success",
                                "server": server.name,
                                "tools": tools.len(),
                            }))
                        );
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
                        println!(
                            "{}",
                            to_json(&serde_json::json!({
                                "status": "error",
                                "server": server.name,
                                "error": e,
                            }))
                        );
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
        if server.id == id_or_name
            || server.name.to_lowercase() == id_or_name.to_lowercase()
            || server.id.starts_with(id_or_name)
        {
            return Ok(server.clone());
        }
    }
    Err(format!("MCP server '{}' not found", id_or_name))
}

fn find_mcp_server_mut<'a>(
    config: &'a mut MCPServersConfig,
    id_or_name: &str,
) -> Result<&'a mut MCPServer, String> {
    for server in &mut config.servers {
        if server.id == id_or_name
            || server.name.to_lowercase() == id_or_name.to_lowercase()
            || server.id.starts_with(id_or_name)
        {
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
                println!(
                    "{}",
                    to_json(&serde_json::json!({
                        "config_dir": config_dir.display().to_string(),
                        "api_key_set": has_key,
                        "settings": settings,
                    }))
                );
            } else {
                println!("{}", "Configuration".bold());
                println!();
                println!(
                    "  Config directory: {}",
                    config_dir.display().to_string().dimmed()
                );
                println!(
                    "  API key: {}",
                    if has_key {
                        "✓ set".green().to_string()
                    } else {
                        "✗ not set".red().to_string()
                    }
                );
                println!();
                println!("{}", "Research Settings".bold());
                println!();
                println!("  Model: {}", settings.model.cyan());
                println!("  Research depth: {}", settings.research_depth);
                println!(
                    "  Max sources per topic: {}",
                    settings.max_sources_per_topic
                );
                println!(
                    "  Notifications: {}",
                    if settings.enable_notifications {
                        "enabled"
                    } else {
                        "disabled"
                    }
                );
                println!(
                    "  Web search: {}",
                    if settings.enable_web_search {
                        "enabled"
                    } else {
                        "disabled"
                    }
                );
            }
        }

        ConfigAction::Set { key, value } => {
            let mut settings = read_settings().unwrap_or_default();

            match key.as_str() {
                "model" => settings.model = value.clone(),
                "research_depth" | "depth" => settings.research_depth = value.clone(),
                "max_sources" | "max_sources_per_topic" => {
                    settings.max_sources_per_topic = value
                        .parse()
                        .map_err(|_| "Invalid number for max_sources")?;
                }
                "notifications" | "enable_notifications" => {
                    settings.enable_notifications = value
                        .parse()
                        .map_err(|_| "Invalid boolean for notifications")?;
                }
                "web_search" | "enable_web_search" => {
                    settings.enable_web_search = value
                        .parse()
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

        ConfigAction::ApiKey { action } => match action {
            ApiKeyAction::Show => {
                if has_api_key() {
                    if json {
                        println!("{}", serde_json::json!({ "api_key_set": true }));
                    } else {
                        println!("{} API key is configured", "✓".green());
                    }
                } else if json {
                    println!("{}", serde_json::json!({ "api_key_set": false }));
                } else {
                    println!("{} No API key configured", "✗".red());
                    println!("\nSet with: claudius config api-key set <YOUR_KEY>");
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
        },
    }

    Ok(())
}

/// Handle housekeeping subcommands
async fn handle_housekeeping(action: HousekeepingAction, json: bool) -> Result<(), String> {
    use claudius::db;
    use claudius::housekeeping;

    match action {
        HousekeepingAction::Run { dry_run } => {
            if dry_run {
                // Show what would be deleted
                let settings = read_settings()?;

                match settings.retention_days {
                    Some(days) => {
                        let conn = db::get_connection()
                            .map_err(|e| format!("Database connection failed: {}", e))?;
                        let count = db::count_cleanup_candidates(&conn, days)?;

                        if json {
                            println!(
                                "{}",
                                serde_json::json!({
                                    "dry_run": true,
                                    "retention_days": days,
                                    "would_delete": count
                                })
                            );
                        } else if count > 0 {
                            println!(
                                "{} {} briefing(s) would be deleted (older than {} days)",
                                "Preview:".yellow(),
                                count,
                                days
                            );
                            println!("\nRun without --dry-run to delete");
                        } else {
                            println!("{} No briefings to clean up", "✓".green());
                        }
                    }
                    None => {
                        if json {
                            println!(
                                "{}",
                                serde_json::json!({
                                    "dry_run": true,
                                    "retention_days": null,
                                    "would_delete": 0,
                                    "message": "Retention set to 'Never delete'"
                                })
                            );
                        } else {
                            println!("{} Retention is set to 'Never delete'", "ℹ".blue());
                            println!("Change in Settings → Storage to enable auto-cleanup");
                        }
                    }
                }
            } else {
                // Actually run cleanup
                let result = housekeeping::run_cleanup()?;

                if json {
                    println!(
                        "{}",
                        serde_json::json!({
                            "deleted_count": result.deleted_count,
                            "remaining_count": result.remaining_count,
                            "skipped_reason": result.skipped_reason
                        })
                    );
                } else if let Some(reason) = result.skipped_reason {
                    println!("{} Skipped: {}", "ℹ".blue(), reason);
                } else if result.deleted_count > 0 {
                    println!(
                        "{} Deleted {} briefing(s), {} remaining",
                        "✓".green(),
                        result.deleted_count,
                        result.remaining_count
                    );
                } else {
                    println!(
                        "{} No briefings to clean up ({} total)",
                        "✓".green(),
                        result.remaining_count
                    );
                }
            }
        }

        HousekeepingAction::Status => {
            let settings = read_settings()?;
            let conn =
                db::get_connection().map_err(|e| format!("Database connection failed: {}", e))?;
            let total_count = db::count_briefings(&conn)?;

            // Get database file size
            let db_path = get_config_dir().join("claudius.db");
            let db_size = std::fs::metadata(&db_path).map(|m| m.len()).unwrap_or(0);

            // Count candidates if retention is set
            let cleanup_candidates = match settings.retention_days {
                Some(days) => Some(db::count_cleanup_candidates(&conn, days)?),
                None => None,
            };

            if json {
                println!(
                    "{}",
                    serde_json::json!({
                        "total_briefings": total_count,
                        "retention_days": settings.retention_days,
                        "cleanup_candidates": cleanup_candidates,
                        "database_size_bytes": db_size
                    })
                );
            } else {
                println!("{}", "Housekeeping Status".bold());
                println!("─────────────────────");
                println!("Total briefings: {}", total_count.to_string().cyan());

                match settings.retention_days {
                    Some(days) => {
                        println!("Retention: {} days", days.to_string().cyan());
                        if let Some(candidates) = cleanup_candidates {
                            if candidates > 0 {
                                println!(
                                    "Ready for cleanup: {} briefing(s)",
                                    candidates.to_string().yellow()
                                );
                            } else {
                                println!("Ready for cleanup: {}", "none".green());
                            }
                        }
                    }
                    None => {
                        println!("Retention: {}", "Never delete".cyan());
                    }
                }

                // Format database size
                let size_str = if db_size > 1_000_000 {
                    format!("{:.1} MB", db_size as f64 / 1_000_000.0)
                } else if db_size > 1_000 {
                    format!("{:.1} KB", db_size as f64 / 1_000.0)
                } else {
                    format!("{} bytes", db_size)
                };
                println!("Database size: {}", size_str.cyan());
            }
        }

        HousekeepingAction::Optimize => {
            let conn =
                db::get_connection().map_err(|e| format!("Database connection failed: {}", e))?;

            // Get size before
            let db_path = get_config_dir().join("claudius.db");
            let size_before = std::fs::metadata(&db_path).map(|m| m.len()).unwrap_or(0);

            // Run VACUUM
            conn.execute("VACUUM", [])
                .map_err(|e| format!("Failed to optimize database: {}", e))?;

            // Get size after
            let size_after = std::fs::metadata(&db_path).map(|m| m.len()).unwrap_or(0);

            let saved = size_before.saturating_sub(size_after);

            if json {
                println!(
                    "{}",
                    serde_json::json!({
                        "status": "optimized",
                        "size_before": size_before,
                        "size_after": size_after,
                        "bytes_saved": saved
                    })
                );
            } else if saved > 0 {
                let saved_str = if saved > 1_000_000 {
                    format!("{:.1} MB", saved as f64 / 1_000_000.0)
                } else if saved > 1_000 {
                    format!("{:.1} KB", saved as f64 / 1_000.0)
                } else {
                    format!("{} bytes", saved)
                };
                println!("{} Database optimized, {} freed", "✓".green(), saved_str);
            } else {
                println!("{} Database already optimized", "✓".green());
            }
        }
    }

    Ok(())
}
