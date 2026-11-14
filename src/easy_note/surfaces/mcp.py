"""MCP surface. Zero tools for now: the five tools arrive in T008."""

from __future__ import annotations

from mcp.server.fastmcp import FastMCP


def build_server() -> FastMCP:
    return FastMCP("easy-note")


def run() -> None:
    build_server().run(transport="stdio")
