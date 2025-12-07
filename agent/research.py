"""
Main research orchestrator for Claudius.

Coordinates the research process using Claude AI and MCP servers to gather
context and generate personalized briefings.
"""

import anthropic
from typing import Dict, List, Optional, Any
from datetime import datetime
import json

from .config import load_interests, load_mcp_servers, load_preferences, get_api_key
from .mcp_client import MCPClient
from .briefing import (
    Briefing,
    BriefingCard,
    create_briefing_prompt,
    parse_briefing_response,
)


class ResearchAgent:
    """
    Main agent for conducting research and generating briefings.

    Uses Claude AI with web search capabilities and MCP servers to gather
    context from user's calendar, email, GitHub, and other sources.
    """

    def __init__(self, model: str = "claude-sonnet-4-5-20250929"):
        """
        Initialize the research agent.

        Args:
            model: Claude model ID to use for research
                  (default: claude-sonnet-4-5-20250929)

        Raises:
            ValueError: If ANTHROPIC_API_KEY is not set
        """
        api_key = get_api_key()
        if not api_key:
            raise ValueError(
                "ANTHROPIC_API_KEY not found. Set it in environment or "
                "~/.claudius/.env file."
            )

        self.client = anthropic.Anthropic(api_key=api_key)
        self.model = model
        self.mcp_client = MCPClient()
        self.preferences = load_preferences()

    async def gather_context(self) -> Dict[str, Any]:
        """
        Gather context from enabled MCP servers.

        Connects to configured MCP servers (calendar, gmail, github, etc.)
        and retrieves relevant context for research.

        Returns:
            Dictionary containing context from various sources:
            - calendar: Upcoming meetings and events
            - email: Recent important emails
            - github: Active PRs, issues, notifications
            - slack: Recent messages (if configured)

        Raises:
            Exception: If MCP server connection or tool calls fail
        """
        context = {}

        try:
            servers = load_mcp_servers()

            # Connect to all enabled servers
            for server_config in servers:
                name = server_config.get("name")
                command = server_config.get("command")
                config = server_config.get("config", {})

                await self.mcp_client.connect_server(name, command, config)

            # Gather context from each server type
            for server_config in servers:
                name = server_config.get("name")

                if "calendar" in name.lower() or "gcal" in name.lower():
                    context["calendar"] = await self._get_calendar_context(name)
                elif "gmail" in name.lower() or "email" in name.lower():
                    context["email"] = await self._get_email_context(name)
                elif "github" in name.lower():
                    context["github"] = await self._get_github_context(name)
                elif "slack" in name.lower():
                    context["slack"] = await self._get_slack_context(name)

        except Exception as e:
            # Log error but continue with available context
            print(f"Warning: Error gathering context: {e}")

        return context

    async def _get_calendar_context(self, server_name: str) -> Dict[str, Any]:
        """Get upcoming calendar events from calendar MCP server."""
        try:
            # Request today's and tomorrow's events
            result = await self.mcp_client.call_tool(
                server_name, "get_events", {"days_ahead": 2}
            )
            return result
        except Exception as e:
            print(f"Calendar context error: {e}")
            return {}

    async def _get_email_context(self, server_name: str) -> Dict[str, Any]:
        """Get recent important emails from email MCP server."""
        try:
            # Get unread emails from last 24 hours
            result = await self.mcp_client.call_tool(
                server_name, "get_recent_emails", {"hours": 24, "unread_only": True}
            )
            return result
        except Exception as e:
            print(f"Email context error: {e}")
            return {}

    async def _get_github_context(self, server_name: str) -> Dict[str, Any]:
        """Get GitHub activity from GitHub MCP server."""
        try:
            # Get open PRs, issues, and notifications
            result = await self.mcp_client.call_tool(
                server_name, "get_activity", {"include_notifications": True}
            )
            return result
        except Exception as e:
            print(f"GitHub context error: {e}")
            return {}

    async def _get_slack_context(self, server_name: str) -> Dict[str, Any]:
        """Get recent Slack messages from Slack MCP server."""
        try:
            result = await self.mcp_client.call_tool(
                server_name, "get_recent_messages", {"hours": 24}
            )
            return result
        except Exception as e:
            print(f"Slack context error: {e}")
            return {}

    async def research_topics(
        self, topics: List[str], context: Dict[str, Any]
    ) -> List[Dict[str, Any]]:
        """
        Research each topic using Claude with web search.

        Args:
            topics: List of topics to research
            context: Context gathered from MCP servers

        Returns:
            List of research results, each containing:
            - topic: The researched topic
            - findings: Key findings
            - sources: Source URLs
            - relevance: Relevance score
            - summary: Brief summary

        Raises:
            Exception: If Claude API calls fail
        """
        research_results = []

        for topic in topics:
            try:
                # Create research prompt
                prompt = f"""Research the following topic and provide:
1. Key recent developments (last 24-48 hours)
2. Why this is relevant given the user's context
3. Actionable insights or next steps
4. Credible sources

Topic: {topic}

User Context:
{json.dumps(context, indent=2)}

Provide a concise but informative research summary."""

                # Call Claude with web search
                response = self.client.messages.create(
                    model=self.model,
                    max_tokens=2048,
                    messages=[{"role": "user", "content": prompt}],
                )

                # Extract text from response
                result_text = ""
                for block in response.content:
                    if hasattr(block, "text"):
                        result_text += block.text

                research_results.append(
                    {
                        "topic": topic,
                        "content": result_text,
                        "model": self.model,
                        "tokens": response.usage.input_tokens
                        + response.usage.output_tokens,
                    }
                )

            except Exception as e:
                print(f"Error researching topic '{topic}': {e}")
                research_results.append(
                    {
                        "topic": topic,
                        "content": f"Error researching topic: {str(e)}",
                        "error": True,
                    }
                )

        return research_results

    async def generate_briefing(
        self, research_results: List[Dict[str, Any]]
    ) -> Briefing:
        """
        Synthesize research results into briefing cards.

        Args:
            research_results: List of research results from research_topics()

        Returns:
            Briefing object containing structured briefing cards

        Raises:
            Exception: If synthesis fails
        """
        start_time = datetime.now()

        try:
            # Create synthesis prompt
            prompt = create_briefing_prompt(research_results, self.preferences)

            # Call Claude to synthesize
            response = self.client.messages.create(
                model=self.model,
                max_tokens=4096,
                messages=[{"role": "user", "content": prompt}],
            )

            # Extract response text
            response_text = ""
            for block in response.content:
                if hasattr(block, "text"):
                    response_text += block.text

            # Parse into briefing cards
            cards = parse_briefing_response(response_text)

            # Calculate research time
            end_time = datetime.now()
            research_time_ms = int((end_time - start_time).total_seconds() * 1000)

            # Calculate total tokens
            total_tokens = sum(
                r.get("tokens", 0) for r in research_results if not r.get("error")
            )
            total_tokens += response.usage.input_tokens + response.usage.output_tokens

            # Create briefing
            briefing = Briefing(
                id=None,  # Will be set when saved to database
                date=datetime.now(),
                title=f"Daily Briefing - {datetime.now().strftime('%B %d, %Y')}",
                cards=cards,
                research_time_ms=research_time_ms,
                model_used=self.model,
                total_tokens=total_tokens,
            )

            return briefing

        except Exception as e:
            print(f"Error generating briefing: {e}")
            # Return empty briefing on error
            return Briefing(
                id=None,
                date=datetime.now(),
                title=f"Daily Briefing - {datetime.now().strftime('%B %d, %Y')}",
                cards=[],
                research_time_ms=0,
                model_used=self.model,
                total_tokens=0,
            )

    async def run_daily_research(self) -> Dict[str, Any]:
        """
        Main entry point - runs full research cycle.

        Orchestrates the complete research process:
        1. Load user interests
        2. Gather context from MCP servers
        3. Research topics using Claude
        4. Generate briefing cards
        5. Return structured briefing

        Returns:
            Dictionary representation of the Briefing object

        Raises:
            Exception: If critical steps fail
        """
        try:
            # Load user interests
            interests = load_interests()
            topics = interests.get("topics", [])

            if not topics:
                return {
                    "error": "No topics configured in interests.json",
                    "briefing": None,
                }

            # Gather context from MCP servers
            print("Gathering context from MCP servers...")
            context = await self.gather_context()

            # Research topics
            print(f"Researching {len(topics)} topics...")
            research_results = await self.research_topics(topics, context)

            # Generate briefing
            print("Generating briefing cards...")
            briefing = await self.generate_briefing(research_results)

            # Clean up MCP connections
            await self.mcp_client.disconnect_all()

            # Return as dict for serialization
            return {
                "success": True,
                "briefing": {
                    "date": briefing.date.isoformat(),
                    "title": briefing.title,
                    "cards": [
                        {
                            "title": card.title,
                            "summary": card.summary,
                            "sources": card.sources,
                            "suggested_next": card.suggested_next,
                            "relevance": card.relevance,
                            "topic": card.topic,
                        }
                        for card in briefing.cards
                    ],
                    "research_time_ms": briefing.research_time_ms,
                    "model_used": briefing.model_used,
                    "total_tokens": briefing.total_tokens,
                },
            }

        except Exception as e:
            # Ensure cleanup on error
            await self.mcp_client.disconnect_all()
            return {"success": False, "error": str(e), "briefing": None}
