// Deduplication module for briefing cards
//
// Uses string similarity to detect and filter duplicate cards
// across research sessions.

use serde::{Deserialize, Serialize};
use strsim::normalized_levenshtein;
use tracing::info;

use crate::research::BriefingCard;

/// Fingerprint of a past card for comparison
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CardFingerprint {
    pub title: String,
    pub topic: String,
    pub summary: String,
}

impl CardFingerprint {
    pub fn from_card(card: &BriefingCard) -> Self {
        Self {
            title: card.title.clone(),
            topic: card.topic.clone(),
            summary: card.summary.clone(),
        }
    }
}

/// Normalize text for comparison (lowercase, strip extra whitespace)
fn normalize(s: &str) -> String {
    s.to_lowercase()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

/// Calculate similarity ratio between two strings (0.0 - 1.0)
/// Uses normalized Levenshtein distance
pub fn similarity(a: &str, b: &str) -> f64 {
    let a = normalize(a);
    let b = normalize(b);

    if a.is_empty() && b.is_empty() {
        return 1.0;
    }
    if a.is_empty() || b.is_empty() {
        return 0.0;
    }

    normalized_levenshtein(&a, &b)
}

/// Check if a card is a duplicate of any past card
/// Only compares cards with the same topic
pub fn is_duplicate(card: &BriefingCard, past: &[CardFingerprint], threshold: f64) -> bool {
    for past_card in past {
        // Only compare cards from the same topic
        if normalize(&card.topic) != normalize(&past_card.topic) {
            continue;
        }

        // Check title similarity
        let title_sim = similarity(&card.title, &past_card.title);
        if title_sim >= threshold {
            info!(
                "Duplicate detected: '{}' similar to '{}' (similarity: {:.2})",
                card.title, past_card.title, title_sim
            );
            return true;
        }

        // Also check summary similarity for same-topic cards
        let summary_sim = similarity(&card.summary, &past_card.summary);
        if summary_sim >= threshold {
            info!(
                "Duplicate detected via summary: '{}' (similarity: {:.2})",
                card.title, summary_sim
            );
            return true;
        }
    }

    false
}

/// Filter out duplicate cards from new cards
/// Returns only cards that are sufficiently different from past cards
pub fn filter_duplicates(
    new_cards: Vec<BriefingCard>,
    past: &[CardFingerprint],
    threshold: f64,
) -> Vec<BriefingCard> {
    if past.is_empty() {
        return new_cards;
    }

    let original_count = new_cards.len();
    let filtered: Vec<BriefingCard> = new_cards
        .into_iter()
        .filter(|card| !is_duplicate(card, past, threshold))
        .collect();

    let removed = original_count - filtered.len();
    if removed > 0 {
        info!(
            "Deduplication: removed {} duplicate cards (threshold: {:.2})",
            removed, threshold
        );
    }

    filtered
}

/// Format past cards for inclusion in synthesis prompt
pub fn format_past_cards_for_prompt(past: &[CardFingerprint]) -> String {
    if past.is_empty() {
        return String::new();
    }

    let formatted: Vec<String> = past
        .iter()
        .take(30) // Limit to avoid huge prompts
        .map(|c| format!("- [{}]: \"{}\"", c.topic, c.title))
        .collect();

    format!(
        "RECENTLY COVERED TOPICS (avoid duplicating unless there's significant NEW information):\n{}",
        formatted.join("\n")
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_similarity_exact_match() {
        assert!((similarity("Hello World", "Hello World") - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_similarity_case_insensitive() {
        assert!((similarity("Hello World", "hello world") - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_similarity_different_strings() {
        let sim = similarity("OpenAI releases GPT-5", "Anthropic announces Claude 4");
        assert!(sim < 0.5);
    }

    #[test]
    fn test_similarity_similar_strings() {
        let sim = similarity(
            "OpenAI releases GPT-5 with new features",
            "OpenAI releases GPT-5 with improved capabilities",
        );
        assert!(sim > 0.6);
    }

    #[test]
    fn test_is_duplicate_same_topic() {
        let card = BriefingCard {
            title: "OpenAI releases GPT-5".to_string(),
            summary: "Major AI announcement".to_string(),
            detailed_content: String::new(),
            sources: vec![],
            suggested_next: None,
            relevance: "high".to_string(),
            topic: "AI News".to_string(),
        };

        let past = vec![CardFingerprint {
            title: "OpenAI releases GPT-5 model".to_string(),
            topic: "AI News".to_string(),
            summary: "Big AI news today".to_string(),
        }];

        assert!(is_duplicate(&card, &past, 0.75));
    }

    #[test]
    fn test_is_duplicate_different_topic() {
        let card = BriefingCard {
            title: "OpenAI releases GPT-5".to_string(),
            summary: "Major AI announcement".to_string(),
            detailed_content: String::new(),
            sources: vec![],
            suggested_next: None,
            relevance: "high".to_string(),
            topic: "AI News".to_string(),
        };

        let past = vec![CardFingerprint {
            title: "OpenAI releases GPT-5 model".to_string(),
            topic: "Tech News".to_string(), // Different topic
            summary: "Big AI news today".to_string(),
        }];

        // Should NOT be duplicate because topics differ
        assert!(!is_duplicate(&card, &past, 0.75));
    }

    #[test]
    fn test_filter_duplicates() {
        let cards = vec![
            BriefingCard {
                title: "New development in AI".to_string(),
                summary: "Fresh news".to_string(),
                detailed_content: String::new(),
                sources: vec![],
                suggested_next: None,
                relevance: "high".to_string(),
                topic: "AI".to_string(),
            },
            BriefingCard {
                title: "OpenAI releases GPT-5".to_string(),
                summary: "Major AI announcement".to_string(),
                detailed_content: String::new(),
                sources: vec![],
                suggested_next: None,
                relevance: "high".to_string(),
                topic: "AI".to_string(),
            },
        ];

        let past = vec![CardFingerprint {
            title: "OpenAI releases GPT-5 today".to_string(),
            topic: "AI".to_string(),
            summary: "AI company news".to_string(),
        }];

        let filtered = filter_duplicates(cards, &past, 0.75);
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].title, "New development in AI");
    }
}
