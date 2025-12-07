"""
Configuration management for Claudius agent.

Handles loading user preferences, interests, and MCP server configurations
from the ~/.claudius directory.
"""

import os
import json
from pathlib import Path
from typing import Dict, List, Optional
from dotenv import load_dotenv


def get_config_dir() -> Path:
    """
    Get the Claudius configuration directory.

    Returns:
        Path to ~/.claudius directory
    """
    config_dir = Path.home() / ".claudius"
    config_dir.mkdir(exist_ok=True)
    return config_dir


def load_interests() -> Dict[str, any]:
    """
    Load user interests from ~/.claudius/interests.json

    Returns:
        Dictionary containing user interests configuration with keys:
        - topics: List of research topics
        - keywords: Important keywords to track
        - sources: Preferred information sources
        - exclusions: Topics/keywords to exclude

    Raises:
        FileNotFoundError: If interests.json doesn't exist
        json.JSONDecodeError: If interests.json is malformed
    """
    config_dir = get_config_dir()
    interests_file = config_dir / "interests.json"

    if not interests_file.exists():
        raise FileNotFoundError(
            f"Interests file not found at {interests_file}. "
            "Run 'claudius init' to create configuration."
        )

    with open(interests_file, 'r') as f:
        interests = json.load(f)

    return interests


def load_mcp_servers() -> List[Dict[str, any]]:
    """
    Load MCP server configurations from ~/.claudius/mcp-servers.json

    Returns:
        List of MCP server configurations, each containing:
        - name: Server identifier
        - command: Command to start the server
        - args: Command-line arguments
        - env: Environment variables
        - enabled: Whether the server is active

    Raises:
        FileNotFoundError: If mcp-servers.json doesn't exist
        json.JSONDecodeError: If mcp-servers.json is malformed
    """
    config_dir = get_config_dir()
    mcp_file = config_dir / "mcp-servers.json"

    if not mcp_file.exists():
        raise FileNotFoundError(
            f"MCP servers file not found at {mcp_file}. "
            "Run 'claudius init' to create configuration."
        )

    with open(mcp_file, 'r') as f:
        servers = json.load(f)

    # Return only enabled servers
    if isinstance(servers, dict) and 'servers' in servers:
        return [
            server for server in servers['servers']
            if server.get('enabled', True)
        ]

    return servers if isinstance(servers, list) else []


def load_preferences() -> Dict[str, any]:
    """
    Load user preferences from ~/.claudius/preferences.json

    Returns:
        Dictionary containing user preferences with keys:
        - model: Claude model to use (default: claude-sonnet-4-5-20250929)
        - schedule: When to run briefings (e.g., "09:00")
        - max_cards: Maximum briefing cards per briefing
        - relevance_threshold: Minimum relevance score ("high", "medium", "low")
        - notification_enabled: Whether to send notifications

    Raises:
        FileNotFoundError: If preferences.json doesn't exist
        json.JSONDecodeError: If preferences.json is malformed
    """
    config_dir = get_config_dir()
    prefs_file = config_dir / "preferences.json"

    # Default preferences
    defaults = {
        "model": "claude-sonnet-4-5-20250929",
        "schedule": "09:00",
        "max_cards": 10,
        "relevance_threshold": "medium",
        "notification_enabled": True,
        "research_depth": "balanced",  # "quick", "balanced", "deep"
    }

    if not prefs_file.exists():
        # Return defaults if file doesn't exist
        return defaults

    with open(prefs_file, 'r') as f:
        preferences = json.load(f)

    # Merge with defaults
    return {**defaults, **preferences}


def get_api_key() -> Optional[str]:
    """
    Get Anthropic API key from environment or .env file.

    Loads from ANTHROPIC_API_KEY environment variable, checking:
    1. Current environment variables
    2. ~/.claudius/.env file
    3. Project root .env file

    Returns:
        API key string or None if not found
    """
    # Load from ~/.claudius/.env if it exists
    config_dir = get_config_dir()
    env_file = config_dir / ".env"
    if env_file.exists():
        load_dotenv(env_file)

    # Also try loading from current directory
    load_dotenv()

    return os.getenv("ANTHROPIC_API_KEY")


def save_preferences(preferences: Dict[str, any]) -> None:
    """
    Save user preferences to ~/.claudius/preferences.json

    Args:
        preferences: Dictionary of preferences to save
    """
    config_dir = get_config_dir()
    prefs_file = config_dir / "preferences.json"

    with open(prefs_file, 'w') as f:
        json.dump(preferences, f, indent=2)


def save_interests(interests: Dict[str, any]) -> None:
    """
    Save user interests to ~/.claudius/interests.json

    Args:
        interests: Dictionary of interests to save
    """
    config_dir = get_config_dir()
    interests_file = config_dir / "interests.json"

    with open(interests_file, 'w') as f:
        json.dump(interests, f, indent=2)
