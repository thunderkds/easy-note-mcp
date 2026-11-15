"""Throwaway T001 probe for the Claude Code UserPromptSubmit hook (stdlib only).

Dumps the raw stdin event to $EASY_NOTE_HOOK_DIR/event-<time>.json. When the prompt
starts with $EASY_NOTE_HOOK_PREFIX (default "/note ") it prints the block JSON.
"""
import json
import os
import sys
import time

raw = sys.stdin.read()
out = os.environ.get("EASY_NOTE_HOOK_DIR", ".")
os.makedirs(out, exist_ok=True)
with open(os.path.join(out, f"event-{time.time():.3f}.json"), "w", encoding="utf-8") as fh:
    fh.write(raw)
try:
    prompt = json.loads(raw).get("prompt", "")
except ValueError:
    prompt = ""
if prompt.startswith(os.environ.get("EASY_NOTE_HOOK_PREFIX", "/note ")):
    print(json.dumps({"decision": "block", "reason": "✓ noted PROBE-ID"}))
