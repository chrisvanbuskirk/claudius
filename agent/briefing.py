"""
Briefing card generation and management.

Defines data structures for briefings and cards, and provides utilities
for creating prompts and parsing Claude's responses into structured data.
"""

from dataclasses import dataclass, asdict
from typing import Optional, List, Dict, Any
from datetime import datetime
import json
import re


@dataclass
class BriefingCard:
    """
    A single briefing card containing research on a topic.

    Attributes:
        title: Card title (concise, specific)
        summary: Main content and findings
        sources: List of source URLs
        suggested_next: Optional next action or follow-up
        relevance: Relevance level ("high", "medium", "low")
        topic: Original topic that generated this card
    """

    title: str
    summary: str
    sources: List[str]
    suggested_next: Optional[str]
    relevance: str  # "high", "medium", "low"
    topic: str

    def to_dict(self) -> Dict[str, Any]:
        """Convert card to dictionary."""
        return asdict(self)


@dataclass
class Briefing:
    """
    A complete daily briefing containing multiple cards.

    Attributes:
        id: Database ID (None if not yet saved)
        date: Briefing date/time
        title: Briefing title
        cards: List of BriefingCard objects
        research_time_ms: Time taken to generate (milliseconds)
        model_used: Claude model identifier
        total_tokens: Total tokens used in generation
    """

    id: Optional[int]
    date: datetime
    title: str
    cards: List[BriefingCard]
    research_time_ms: int
    model_used: str
    total_tokens: int

    def to_dict(self) -> Dict[str, Any]:
        """Convert briefing to dictionary for serialization."""
        return {
            "id": self.id,
            "date": self.date.isoformat(),
            "title": self.title,
            "cards": [card.to_dict() for card in self.cards],
            "research_time_ms": self.research_time_ms,
            "model_used": self.model_used,
            "total_tokens": self.total_tokens,
        }


def create_briefing_prompt(
    research_results: List[Dict[str, Any]], preferences: Dict[str, Any]
) -> str:
    """
    Create the synthesis prompt for Claude to generate briefing cards.

    Args:
        research_results: List of research results from ResearchAgent.research_topics()
        preferences: User preferences from config

    Returns:
        Formatted prompt string for Claude
    """
    max_cards = preferences.get("max_cards", 10)
    relevance_threshold = preferences.get("relevance_threshold", "medium")
    research_depth = preferences.get("research_depth", "balanced")

    # Format research results
    research_section = ""
    for i, result in enumerate(research_results, 1):
        if result.get("error"):
            continue

        research_section += f"\n## Topic {i}: {result['topic']}\n"
        research_section += f"{result['content']}\n"

    prompt = f"""You are a research assistant creating a personalized daily briefing.
Synthesize the following research results into clear, actionable briefing cards.

{research_section}

Generate briefing cards following these guidelines:

1. **Relevance**: Only include cards with {relevance_threshold} or higher relevance
2. **Limit**: Maximum {max_cards} cards total
3. **Depth**: Use {research_depth} research depth (quick/balanced/deep)
4. **Priority**: Prioritize timely, actionable information

For each card, provide:
- **Title**: Clear, specific title (max 60 chars)
- **Summary**: Key findings and why it matters (2-4 sentences)
- **Sources**: List of source URLs (if available)
- **Suggested Next**: Optional next action or follow-up
- **Relevance**: "high", "medium", or "low"
- **Topic**: The original topic this relates to

Return ONLY valid JSON in this exact format:
{{
  "cards": [
    {{
      "title": "Card title",
      "summary": "Card summary with key findings and relevance.",
      "sources": ["https://example.com/source1", "https://example.com/source2"],
      "suggested_next": "Optional next action",
      "relevance": "high",
      "topic": "Original topic name"
    }}
  ]
}}

Focus on information that is:
- Timely (recent developments)
- Actionable (can inform decisions)
- Relevant (matches user's interests and context)
- Credible (from reliable sources)

Return the JSON response now:"""

    return prompt


def parse_briefing_response(response: str) -> List[BriefingCard]:
    """
    Parse Claude's response into structured BriefingCard objects.

    Args:
        response: Raw text response from Claude

    Returns:
        List of BriefingCard objects

    Raises:
        ValueError: If response cannot be parsed
    """
    try:
        # Try to extract JSON from response
        # Claude might wrap it in markdown code blocks
        json_match = re.search(r"```(?:json)?\s*(\{.*\})\s*```", response, re.DOTALL)
        if json_match:
            json_str = json_match.group(1)
        else:
            # Try to find JSON object directly
            json_match = re.search(r"\{.*\}", response, re.DOTALL)
            if json_match:
                json_str = json_match.group(0)
            else:
                raise ValueError("No JSON found in response")

        # Parse JSON
        data = json.loads(json_str)

        # Extract cards
        if "cards" not in data:
            raise ValueError("Response missing 'cards' field")

        cards = []
        for card_data in data["cards"]:
            card = BriefingCard(
                title=card_data.get("title", "Untitled"),
                summary=card_data.get("summary", ""),
                sources=card_data.get("sources", []),
                suggested_next=card_data.get("suggested_next"),
                relevance=card_data.get("relevance", "medium"),
                topic=card_data.get("topic", "Unknown"),
            )
            cards.append(card)

        return cards

    except json.JSONDecodeError as e:
        raise ValueError(f"Invalid JSON in response: {str(e)}")
    except Exception as e:
        raise ValueError(f"Failed to parse briefing response: {str(e)}")


def format_card_for_display(card: BriefingCard) -> str:
    """
    Format a briefing card for CLI display.

    Args:
        card: BriefingCard to format

    Returns:
        Formatted string for terminal output
    """
    # Relevance emoji
    relevance_indicators = {
        "high": "🔴",
        "medium": "🟡",
        "low": "🟢",
    }
    indicator = relevance_indicators.get(card.relevance.lower(), "⚪")

    output = f"\n{indicator} **{card.title}**\n"
    output += f"Topic: {card.topic} | Relevance: {card.relevance}\n"
    output += f"\n{card.summary}\n"

    if card.sources:
        output += "\nSources:\n"
        for source in card.sources:
            output += f"  - {source}\n"

    if card.suggested_next:
        output += f"\nSuggested Next: {card.suggested_next}\n"

    output += "-" * 80

    return output


def format_briefing_for_display(briefing: Briefing) -> str:
    """
    Format a complete briefing for CLI display.

    Args:
        briefing: Briefing to format

    Returns:
        Formatted string for terminal output
    """
    output = f"\n{'=' * 80}\n"
    output += f"{briefing.title}\n"
    output += f"{briefing.date.strftime('%A, %B %d, %Y at %I:%M %p')}\n"
    output += f"{'=' * 80}\n"

    if not briefing.cards:
        output += "\nNo briefing cards generated.\n"
    else:
        output += f"\n{len(briefing.cards)} cards | "
        output += f"{briefing.research_time_ms}ms | "
        output += f"{briefing.total_tokens} tokens\n"

        for i, card in enumerate(briefing.cards, 1):
            output += f"\n--- Card {i}/{len(briefing.cards)} ---"
            output += format_card_for_display(card)

    output += f"\n{'=' * 80}\n"

    return output


def filter_cards_by_relevance(
    cards: List[BriefingCard], min_relevance: str = "medium"
) -> List[BriefingCard]:
    """
    Filter cards by minimum relevance level.

    Args:
        cards: List of BriefingCard objects
        min_relevance: Minimum relevance level ("high", "medium", "low")

    Returns:
        Filtered list of cards
    """
    relevance_order = {"low": 0, "medium": 1, "high": 2}
    min_level = relevance_order.get(min_relevance.lower(), 1)

    return [
        card
        for card in cards
        if relevance_order.get(card.relevance.lower(), 0) >= min_level
    ]


def sort_cards_by_relevance(cards: List[BriefingCard]) -> List[BriefingCard]:
    """
    Sort cards by relevance (high to low).

    Args:
        cards: List of BriefingCard objects

    Returns:
        Sorted list of cards
    """
    relevance_order = {"high": 0, "medium": 1, "low": 2}

    return sorted(cards, key=lambda c: relevance_order.get(c.relevance.lower(), 2))
