"""
CLI bridge for invoking the research agent from the CLI.

This module provides the entry point for the CLI to call agent functionality
via subprocess, enabling seamless integration between the CLI and Python agent.
"""

import asyncio
import json
import sys
from typing import Dict, Any

from .research import ResearchAgent
from .briefing import format_briefing_for_display


async def run_daily_research(args: Dict[str, Any]) -> Dict[str, Any]:
    """
    Run the daily research cycle.

    Args:
        args: Arguments dictionary containing:
            - model: Optional Claude model to use
            - verbose: Whether to print verbose output

    Returns:
        Dictionary containing briefing results
    """
    model = args.get("model", "claude-sonnet-4-5-20250929")
    verbose = args.get("verbose", False)

    try:
        agent = ResearchAgent(model=model)

        if verbose:
            print("Starting daily research cycle...", file=sys.stderr)

        result = await agent.run_daily_research()

        return result

    except Exception as e:
        return {"success": False, "error": str(e), "briefing": None}


async def research_topic(args: Dict[str, Any]) -> Dict[str, Any]:
    """
    Research a specific topic.

    Args:
        args: Arguments dictionary containing:
            - topic: Topic to research
            - model: Optional Claude model to use
            - verbose: Whether to print verbose output

    Returns:
        Dictionary containing research results
    """
    topic = args.get("topic")
    if not topic:
        return {"success": False, "error": "No topic provided", "results": None}

    model = args.get("model", "claude-sonnet-4-5-20250929")
    verbose = args.get("verbose", False)

    try:
        agent = ResearchAgent(model=model)

        if verbose:
            print(f"Researching topic: {topic}", file=sys.stderr)

        # Research the single topic
        results = await agent.research_topics([topic], {})

        return {"success": True, "results": results}

    except Exception as e:
        return {"success": False, "error": str(e), "results": None}


async def test_mcp_connection(args: Dict[str, Any]) -> Dict[str, Any]:
    """
    Test MCP server connections.

    Args:
        args: Arguments dictionary containing:
            - verbose: Whether to print verbose output

    Returns:
        Dictionary containing connection test results
    """
    verbose = args.get("verbose", False)

    try:
        agent = ResearchAgent()

        if verbose:
            print("Testing MCP server connections...", file=sys.stderr)

        context = await agent.gather_context()

        # Cleanup
        await agent.mcp_client.disconnect_all()

        return {
            "success": True,
            "context": {
                "calendar": bool(context.get("calendar")),
                "email": bool(context.get("email")),
                "github": bool(context.get("github")),
                "slack": bool(context.get("slack")),
            },
        }

    except Exception as e:
        return {"success": False, "error": str(e), "context": None}


def main():
    """
    Entry point called from CLI via subprocess.

    Expects a JSON-encoded argument dictionary as sys.argv[1].
    Prints JSON-encoded result to stdout.
    """
    try:
        # Parse arguments from CLI
        if len(sys.argv) > 1:
            args = json.loads(sys.argv[1])
        else:
            args = {}

        # Determine which operation to run
        operation = args.get("operation", "run_daily_research")

        # Execute the appropriate operation
        if operation == "run_daily_research" or args.get("now"):
            result = asyncio.run(run_daily_research(args))
        elif operation == "research_topic" or args.get("topic"):
            result = asyncio.run(research_topic(args))
        elif operation == "test_mcp":
            result = asyncio.run(test_mcp_connection(args))
        else:
            result = {
                "success": False,
                "error": f"Unknown operation: {operation}",
            }

        # Print result as JSON to stdout
        print(json.dumps(result, indent=2))

    except json.JSONDecodeError as e:
        error_result = {
            "success": False,
            "error": f"Invalid JSON arguments: {str(e)}",
        }
        print(json.dumps(error_result, indent=2))
        sys.exit(1)

    except Exception as e:
        error_result = {
            "success": False,
            "error": f"Unexpected error: {str(e)}",
        }
        print(json.dumps(error_result, indent=2))
        sys.exit(1)


if __name__ == "__main__":
    main()
