"""The `easy-note` and `n` commands.

This module must never import the MCP SDK at import time (NFR-010): `serve`
imports `easy_note.surfaces.mcp` lazily inside its handler.
"""

from __future__ import annotations

import argparse
import sys
from collections.abc import Sequence

from easy_note import __version__


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(prog="easy-note", description="Plain-Markdown notes.")
    parser.add_argument("--version", action="version", version=__version__)
    sub = parser.add_subparsers(dest="command", metavar="<command>")

    add = sub.add_parser("add", help="add a note (not implemented yet)")
    add.add_argument("text", nargs="*", help="note text; arguments are joined with spaces")
    add.set_defaults(handler=_cmd_add)

    serve = sub.add_parser("serve", help="run the MCP server over stdio")
    serve.set_defaults(handler=_cmd_serve)

    return parser


def _cmd_add(args: argparse.Namespace) -> int:
    print("easy-note: 'add' is not implemented yet", file=sys.stderr)
    return 2


def _cmd_serve(args: argparse.Namespace) -> int:
    from easy_note.surfaces.mcp import run

    run()
    return 0


def main(argv: Sequence[str] | None = None) -> int:
    parser = build_parser()
    args = parser.parse_args(argv)
    handler = getattr(args, "handler", None)
    if handler is None:
        parser.print_help()
        return 0
    return handler(args)


def main_n(argv: Sequence[str] | None = None) -> int:
    """Entry point of `n`: the same as `easy-note add`, with `add` pre-selected."""
    args = list(sys.argv[1:] if argv is None else argv)
    if args[:1] in (["--version"], ["-h"], ["--help"]):
        return main(args)
    return main(["add", *args])


if __name__ == "__main__":
    sys.exit(main())
