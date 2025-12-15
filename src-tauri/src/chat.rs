//! Chat module for briefing conversations.
//!
//! Provides simple chat functionality using the Anthropic API.
//! Users can chat about briefings with Claude, using the briefing content as context.

use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tracing::{info, error};

use crate::db::{self, ChatMessage};

// ============================================================================
// API Structures
// ============================================================================

/// Anthropic API message request (simplified for chat).
#[derive(Debug, Serialize)]
struct ChatRequest {
    model: String,
    max_tokens: u32,
    messages: Vec<Message>,
    system: String,
}

/// A message in the conversation.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct Message {
    role: String,
    content: String,
}

/// Anthropic API response.
#[derive(Debug, Deserialize)]
struct ChatResponse {
    content: Vec<ContentBlock>,
    usage: Usage,
}

/// Content block in API response.
#[derive(Debug, Deserialize)]
struct ContentBlock {
    #[serde(rename = "type")]
    content_type: String,
    #[serde(default)]
    text: Option<String>,
}

#[derive(Debug, Deserialize)]
struct Usage {
    input_tokens: u32,
    output_tokens: u32,
}

// ============================================================================
// Chat Functions
// ============================================================================

/// Build the system prompt for chat, including specific card context.
fn build_system_prompt(briefing_title: &str, briefing_cards: &str, card_index: i32) -> String {
    // Parse the cards JSON and extract the specific card's content
    let card_content = extract_card_content(briefing_cards, card_index);

    format!(
        r#"You are a helpful assistant discussing a research briefing card with the user.

The user is viewing a specific card from a briefing titled "{title}".

Here is the card content:
{content}

Help the user understand this card, answer questions about it, provide additional context, or discuss related topics. Be concise but thorough. If the user asks about something not covered in the card, you can draw on your general knowledge but make it clear when you're going beyond the card content."#,
        title = briefing_title,
        content = card_content
    )
}

/// Extract readable content from a specific card in the briefing cards JSON.
fn extract_card_content(cards_json: &str, card_index: i32) -> String {
    #[derive(Deserialize)]
    struct Card {
        title: Option<String>,
        summary: Option<String>,
        detailed_content: Option<String>,
        relevance: Option<String>,
        topic: Option<String>,
    }

    let cards: Vec<Card> = serde_json::from_str(cards_json).unwrap_or_default();

    if cards.is_empty() {
        return "No briefing cards available.".to_string();
    }

    // Get the specific card by index
    let card_idx = card_index as usize;
    if card_idx >= cards.len() {
        return format!("Card {} not found (briefing has {} cards).", card_index, cards.len());
    }

    let card = &cards[card_idx];
    let title = card.title.as_deref().unwrap_or("Untitled");
    let summary = card.summary.as_deref().unwrap_or("");
    let details = card.detailed_content.as_deref().unwrap_or("");
    let relevance = card.relevance.as_deref().unwrap_or("medium");
    let topic = card.topic.as_deref().unwrap_or("General");

    let mut content = format!(
        "Title: {}\nTopic: {}\nRelevance: {}\nSummary: {}",
        title, topic, relevance, summary
    );

    if !details.is_empty() {
        content.push_str(&format!("\n\nDetails:\n{}", details));
    }

    content
}

/// Build the messages array for the API call.
fn build_messages(history: &[ChatMessage], new_message: &str) -> Vec<Message> {
    let mut messages = Vec::new();

    // Add history (limit to last 20 messages to manage context window)
    let max_history = 20;
    let start_idx = history.len().saturating_sub(max_history);

    for msg in &history[start_idx..] {
        messages.push(Message {
            role: msg.role.clone(),
            content: msg.content.clone(),
        });
    }

    // Add the new user message
    messages.push(Message {
        role: "user".to_string(),
        content: new_message.to_string(),
    });

    messages
}

/// Send a chat message and get a response from Claude.
///
/// This function:
/// 1. Loads the briefing for context
/// 2. Loads existing chat history for this specific card
/// 3. Calls the Anthropic API
/// 4. Saves both user message and assistant response to the database
/// 5. Returns the assistant's message
pub async fn send_chat_message(
    api_key: &str,
    model: &str,
    briefing_id: i64,
    card_index: i32,
    user_message: &str,
) -> Result<(ChatMessage, i32), String> {
    // Get database connection
    let conn = db::get_connection()
        .map_err(|e| format!("Database connection failed: {}", e))?;

    // Load briefing for context
    let briefing = load_briefing(&conn, briefing_id)?;

    // Load existing chat history for this specific card
    let history = db::get_chat_messages(&conn, briefing_id, card_index)?;

    // Build system prompt with specific card context
    let system_prompt = build_system_prompt(&briefing.title, &briefing.cards, card_index);

    // Build messages array
    let messages = build_messages(&history, user_message);

    // Create HTTP client
    let client = Client::builder()
        .timeout(Duration::from_secs(60)) // 60s timeout for chat
        .connect_timeout(Duration::from_secs(10))
        .build()
        .map_err(|e| format!("Failed to build HTTP client: {}", e))?;

    // Create API request
    let request = ChatRequest {
        model: model.to_string(),
        max_tokens: 1024, // Shorter responses for chat
        messages,
        system: system_prompt,
    };

    info!("Sending chat message for briefing {} card {}", briefing_id, card_index);

    // Send request to Anthropic API
    let response = client
        .post("https://api.anthropic.com/v1/messages")
        .header("x-api-key", api_key)
        .header("anthropic-version", "2023-06-01")
        .header("content-type", "application/json")
        .json(&request)
        .send()
        .await
        .map_err(|e| format!("HTTP request failed: {}", e))?;

    // Check for errors
    if !response.status().is_success() {
        let status = response.status().as_u16();
        let body = response.text().await.unwrap_or_default();
        error!("Chat API error {}: {}", status, body);
        return Err(format!("API error {}: {}", status, body));
    }

    // Parse response
    let chat_response: ChatResponse = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse response: {}", e))?;

    // Extract text from response
    let assistant_text = chat_response
        .content
        .iter()
        .filter_map(|block| {
            if block.content_type == "text" {
                block.text.clone()
            } else {
                None
            }
        })
        .collect::<Vec<_>>()
        .join("\n");

    let total_tokens = (chat_response.usage.input_tokens + chat_response.usage.output_tokens) as i32;

    info!("Chat response received: {} tokens", total_tokens);

    // Save user message to database
    let _user_id = db::insert_chat_message(&conn, briefing_id, card_index, "user", user_message, None)?;

    // Save assistant response to database
    let assistant_id = db::insert_chat_message(
        &conn,
        briefing_id,
        card_index,
        "assistant",
        &assistant_text,
        Some(total_tokens),
    )?;

    // Get the saved assistant message
    let assistant_message = db::get_chat_message_by_id(&conn, assistant_id)?
        .ok_or("Failed to retrieve saved message")?;

    Ok((assistant_message, total_tokens))
}

/// Load a briefing from the database.
fn load_briefing(conn: &rusqlite::Connection, briefing_id: i64) -> Result<BriefingData, String> {
    let mut stmt = conn.prepare(
        "SELECT id, title, cards FROM briefings WHERE id = ?1"
    ).map_err(|e| format!("Failed to prepare statement: {}", e))?;

    stmt.query_row([briefing_id], |row| {
        Ok(BriefingData {
            id: row.get(0)?,
            title: row.get(1)?,
            cards: row.get(2)?,
        })
    }).map_err(|e| format!("Failed to load briefing: {}", e))
}

/// Minimal briefing data for chat context.
struct BriefingData {
    #[allow(dead_code)]
    id: i64,
    title: String,
    cards: String,
}

/// Get chat history for a specific card in a briefing.
pub fn get_chat_history(briefing_id: i64, card_index: i32) -> Result<Vec<ChatMessage>, String> {
    let conn = db::get_connection()
        .map_err(|e| format!("Database connection failed: {}", e))?;

    db::get_chat_messages(&conn, briefing_id, card_index)
}

/// Clear chat history for a specific card in a briefing.
pub fn clear_chat_history(briefing_id: i64, card_index: i32) -> Result<usize, String> {
    let conn = db::get_connection()
        .map_err(|e| format!("Database connection failed: {}", e))?;

    db::delete_chat_messages(&conn, briefing_id, card_index)
}
