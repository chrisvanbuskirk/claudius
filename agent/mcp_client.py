"""
MCP (Model Context Protocol) client for communicating with MCP servers.

Handles stdio-based communication with MCP servers to retrieve context
from various sources like calendar, email, GitHub, etc.
"""

import asyncio
import subprocess
import json
from typing import Dict, List, Optional, Any
import os


class MCPClient:
    """
    Client for managing connections to MCP servers.

    Communicates with MCP servers via stdio using the JSON-RPC protocol
    to retrieve context and call tools.
    """

    def __init__(self):
        """Initialize the MCP client with empty server connections."""
        self.servers: Dict[str, Dict[str, Any]] = {}
        self.request_id = 0

    async def connect_server(
        self, name: str, command: str, config: Dict[str, Any]
    ) -> None:
        """
        Start and connect to an MCP server via stdio.

        Args:
            name: Unique identifier for the server
            command: Command to start the MCP server (e.g., "npx", "python")
            config: Configuration dictionary containing:
                - args: List of command-line arguments
                - env: Dictionary of environment variables

        Raises:
            Exception: If server fails to start or initialize
        """
        try:
            # Extract args and env from config
            args = config.get("args", [])
            env = config.get("env", {})

            # Prepare environment
            server_env = os.environ.copy()
            server_env.update(env)

            # Start the server process
            process = await asyncio.create_subprocess_exec(
                command,
                *args,
                stdin=subprocess.PIPE,
                stdout=subprocess.PIPE,
                stderr=subprocess.PIPE,
                env=server_env,
            )

            # Store server info
            self.servers[name] = {
                "process": process,
                "command": command,
                "args": args,
                "available_tools": [],
            }

            # Initialize connection and discover tools
            await self._initialize_server(name)

        except Exception as e:
            raise Exception(f"Failed to connect to MCP server '{name}': {str(e)}")

    async def _initialize_server(self, name: str) -> None:
        """
        Initialize MCP server connection and discover available tools.

        Sends initialization request and queries for available tools.

        Args:
            name: Server identifier

        Raises:
            Exception: If initialization fails
        """
        server = self.servers.get(name)
        if not server:
            raise Exception(f"Server '{name}' not found")

        try:
            # Send initialize request
            init_request = {
                "jsonrpc": "2.0",
                "id": self._next_request_id(),
                "method": "initialize",
                "params": {
                    "protocolVersion": "2024-11-05",
                    "capabilities": {},
                    "clientInfo": {
                        "name": "claudius",
                        "version": "0.1.0",
                    },
                },
            }

            response = await self._send_request(name, init_request)

            # List available tools
            tools_request = {
                "jsonrpc": "2.0",
                "id": self._next_request_id(),
                "method": "tools/list",
                "params": {},
            }

            tools_response = await self._send_request(name, tools_request)

            if "result" in tools_response and "tools" in tools_response["result"]:
                server["available_tools"] = tools_response["result"]["tools"]

        except Exception as e:
            raise Exception(f"Failed to initialize server '{name}': {str(e)}")

    async def call_tool(
        self, server_name: str, tool_name: str, args: Dict[str, Any]
    ) -> Dict[str, Any]:
        """
        Call a tool on an MCP server.

        Args:
            server_name: Identifier of the server to call
            tool_name: Name of the tool to invoke
            args: Dictionary of arguments for the tool

        Returns:
            Dictionary containing the tool's response

        Raises:
            Exception: If server not found, tool not available, or call fails
        """
        server = self.servers.get(server_name)
        if not server:
            raise Exception(f"Server '{server_name}' not connected")

        # Check if tool is available
        available_tool_names = [t.get("name") for t in server["available_tools"]]
        if tool_name not in available_tool_names:
            raise Exception(
                f"Tool '{tool_name}' not available on server '{server_name}'. "
                f"Available tools: {available_tool_names}"
            )

        try:
            # Send tool call request
            tool_request = {
                "jsonrpc": "2.0",
                "id": self._next_request_id(),
                "method": "tools/call",
                "params": {"name": tool_name, "arguments": args},
            }

            response = await self._send_request(server_name, tool_request)

            if "result" in response:
                return response["result"]
            elif "error" in response:
                raise Exception(
                    f"Tool call error: {response['error'].get('message', 'Unknown error')}"
                )
            else:
                return response

        except Exception as e:
            raise Exception(
                f"Failed to call tool '{tool_name}' on server '{server_name}': {str(e)}"
            )

    async def _send_request(
        self, server_name: str, request: Dict[str, Any]
    ) -> Dict[str, Any]:
        """
        Send a JSON-RPC request to an MCP server and receive response.

        Args:
            server_name: Server identifier
            request: JSON-RPC request dictionary

        Returns:
            JSON-RPC response dictionary

        Raises:
            Exception: If communication fails
        """
        server = self.servers.get(server_name)
        if not server:
            raise Exception(f"Server '{server_name}' not found")

        process = server["process"]

        try:
            # Serialize request as JSON with newline delimiter
            request_json = json.dumps(request) + "\n"
            request_bytes = request_json.encode("utf-8")

            # Write to server's stdin
            process.stdin.write(request_bytes)
            await process.stdin.drain()

            # Read response from stdout
            response_line = await process.stdout.readline()
            if not response_line:
                raise Exception("Server closed connection")

            # Parse JSON response
            response = json.loads(response_line.decode("utf-8"))
            return response

        except json.JSONDecodeError as e:
            raise Exception(f"Invalid JSON response from server: {str(e)}")
        except Exception as e:
            raise Exception(f"Communication error with server: {str(e)}")

    async def disconnect_all(self) -> None:
        """
        Disconnect from all MCP servers.

        Gracefully terminates all server processes and clears connections.
        """
        for name, server in self.servers.items():
            try:
                process = server["process"]

                # Try graceful shutdown first
                if process.stdin:
                    process.stdin.close()

                # Wait briefly for process to exit
                try:
                    await asyncio.wait_for(process.wait(), timeout=2.0)
                except asyncio.TimeoutError:
                    # Force terminate if graceful shutdown fails
                    process.terminate()
                    await process.wait()

            except Exception as e:
                print(f"Error disconnecting from server '{name}': {e}")

        # Clear all servers
        self.servers.clear()

    def _next_request_id(self) -> int:
        """
        Get next JSON-RPC request ID.

        Returns:
            Monotonically increasing request ID
        """
        self.request_id += 1
        return self.request_id

    def get_available_tools(self, server_name: str) -> List[Dict[str, Any]]:
        """
        Get list of available tools for a server.

        Args:
            server_name: Server identifier

        Returns:
            List of tool descriptions

        Raises:
            Exception: If server not found
        """
        server = self.servers.get(server_name)
        if not server:
            raise Exception(f"Server '{server_name}' not found")

        return server.get("available_tools", [])

    def is_connected(self, server_name: str) -> bool:
        """
        Check if a server is connected.

        Args:
            server_name: Server identifier

        Returns:
            True if server is connected and running
        """
        server = self.servers.get(server_name)
        if not server:
            return False

        process = server["process"]
        return process.returncode is None
