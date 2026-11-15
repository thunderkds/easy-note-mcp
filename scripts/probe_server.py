"""Throwaway T001 probe: a stdio MCP server that logs how a client launched it.

Run: uv run --with 'mcp<2' python scripts/probe_server.py
Log: JSON lines appended to $EASY_NOTE_PROBE_LOG (default: ./probe.log).
Records: "start" (process cwd, env hints, parent command), "initialize" (clientInfo,
capabilities), "roots" (the roots/list answer or the error). No product code.
"""
import asyncio
import json
import os
import sys
import time

from mcp import types
from mcp.server.fastmcp import Context, FastMCP

LOG = os.environ.get("EASY_NOTE_PROBE_LOG", "probe.log")
HINT = ("PWD", "OLDPWD", "CLAUDE", "CODEX", "CURSOR", "PROJECT", "WORKSPACE", "VSCODE")


def log(kind: str, **data) -> None:
    with open(LOG, "a", encoding="utf-8") as fh:
        fh.write(json.dumps({"t": time.time(), "kind": kind, **data}) + "\n")


def parent_cmdline() -> str:
    try:
        with open(f"/proc/{os.getppid()}/cmdline", "rb") as fh:
            return fh.read().replace(b"\0", b" ").decode(errors="replace").strip()
    except OSError:
        return ""


mcp = FastMCP("easy-note-probe")
_probed = False


async def probe(session) -> None:
    global _probed
    if _probed:
        return
    _probed = True
    params = session.client_params
    caps = params.capabilities if params else None
    log(
        "initialize",
        clientInfo=params.clientInfo.model_dump() if params and params.clientInfo else None,
        protocolVersion=params.protocolVersion if params else None,
        roots_capability=caps.roots.model_dump() if caps and caps.roots else None,
    )
    try:
        res = await asyncio.wait_for(session.list_roots(), timeout=5)
        log("roots", ok=True, roots=[str(r.uri) for r in res.roots])
    except Exception as exc:  # the answer "no" is a finding, keep the text
        log("roots", ok=False, error=f"{type(exc).__name__}: {exc}")


@mcp.tool()
async def probe_tool(ctx: Context) -> str:
    """Return the process cwd of this probe server. For T001 only."""
    await probe(ctx.session)
    log("tool_call", cwd=os.getcwd())
    return os.getcwd()


def wrap_list_tools() -> None:
    srv = mcp._mcp_server
    original = srv.request_handlers[types.ListToolsRequest]

    async def handler(req):
        await probe(srv.request_context.session)
        return await original(req)

    srv.request_handlers[types.ListToolsRequest] = handler


if __name__ == "__main__":
    log(
        "start",
        cwd=os.getcwd(),
        argv=sys.argv,
        parent=parent_cmdline(),
        env={k: v for k, v in os.environ.items() if k.startswith(HINT)},
    )
    wrap_list_tools()
    mcp.run("stdio")
