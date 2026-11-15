"""T001: the two throwaway probe scripts behave as documented (subprocess, offline, bounded)."""
import json
import os
import queue
import shutil
import subprocess
import sys
import threading
from pathlib import Path

import pytest

SCRIPTS = Path(__file__).parent.parent / "scripts"
HOOK = SCRIPTS / "probe_hook.py"
SERVER = SCRIPTS / "probe_server.py"
TIMEOUT = 20


def run_hook(tmp_path, stdin, prefix=None):
    env = {**os.environ, "EASY_NOTE_HOOK_DIR": str(tmp_path)}
    env.pop("EASY_NOTE_HOOK_PREFIX", None)
    if prefix is not None:
        env["EASY_NOTE_HOOK_PREFIX"] = prefix
    return subprocess.run(
        [sys.executable, str(HOOK)], input=stdin, capture_output=True, text=True,
        env=env, timeout=TIMEOUT,
    )


def events(tmp_path):
    return sorted(tmp_path.glob("event-*.json"))


def test_hook_blocks_note_prefix_and_records_event(tmp_path):
    raw = json.dumps({"prompt": "/note retry bug", "cwd": "/x", "session_id": "s"})
    res = run_hook(tmp_path, raw)
    assert res.returncode == 0
    out = json.loads(res.stdout)
    assert out["decision"] == "block"
    assert out["reason"]
    files = events(tmp_path)
    assert len(files) == 1
    assert files[0].read_text(encoding="utf-8") == raw


def test_hook_ignores_other_prompt_but_still_records_event(tmp_path):
    raw = json.dumps({"prompt": "hello there", "cwd": "/x", "session_id": "s"})
    res = run_hook(tmp_path, raw)
    assert res.returncode == 0
    assert res.stdout == ""
    assert len(events(tmp_path)) == 1


def test_hook_prefix_env_var_changes_match(tmp_path):
    raw = json.dumps({"prompt": ";n retry bug"})
    assert run_hook(tmp_path, raw).stdout == ""
    res = run_hook(tmp_path, raw, prefix=";n ")
    assert json.loads(res.stdout)["decision"] == "block"


def test_hook_invalid_json_prints_nothing_and_does_not_crash(tmp_path):
    res = run_hook(tmp_path, "not json {")
    assert res.returncode == 0
    assert res.stdout == ""
    assert "Traceback" not in res.stderr
    files = events(tmp_path)
    assert len(files) == 1
    assert files[0].read_text(encoding="utf-8") == "not json {"


def _server_cmd():
    """Interpreter that can import `mcp`: this one, else uv from its offline cache, else None."""
    probe = subprocess.run([sys.executable, "-c", "import mcp"], capture_output=True)
    if probe.returncode == 0:
        return [sys.executable, str(SERVER)]
    if shutil.which("uv"):
        cmd = ["uv", "run", "--offline", "--with", "mcp<2", "python", str(SERVER)]
        check = subprocess.run(
            cmd[:-1] + ["-c", "import mcp"], capture_output=True, timeout=TIMEOUT
        )
        if check.returncode == 0:
            return cmd
    return None


def _send(proc, msg):
    proc.stdin.write(json.dumps(msg) + "\n")
    proc.stdin.flush()


def _recv(lines):
    return json.loads(lines.get(timeout=TIMEOUT))


def test_server_logs_start_initialize_and_roots(tmp_path):
    cmd = _server_cmd()
    if cmd is None:
        pytest.skip("mcp SDK not importable and not in the uv offline cache")
    log = tmp_path / "probe.log"
    env = {**os.environ, "EASY_NOTE_PROBE_LOG": str(log)}
    proc = subprocess.Popen(
        cmd, stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=subprocess.DEVNULL,
        text=True, env=env, cwd=tmp_path,
    )
    lines = queue.Queue()
    threading.Thread(
        target=lambda: [lines.put(ln) for ln in proc.stdout], daemon=True
    ).start()
    try:
        _send(proc, {
            "jsonrpc": "2.0", "id": 1, "method": "initialize",
            "params": {
                "protocolVersion": "2025-03-26",
                "capabilities": {"roots": {"listChanged": True}},
                "clientInfo": {"name": "pytest-client", "version": "9.9"},
            },
        })
        assert _recv(lines)["id"] == 1
        _send(proc, {"jsonrpc": "2.0", "method": "notifications/initialized"})
        _send(proc, {"jsonrpc": "2.0", "id": 2, "method": "tools/list"})
        # tools/list makes the probe ask the client for roots; answer it.
        msg = _recv(lines)
        while msg.get("method") != "roots/list":
            assert msg.get("id") != 2, "tools/list answered before roots/list was asked"
            msg = _recv(lines)
        _send(proc, {"jsonrpc": "2.0", "id": msg["id"],
                     "result": {"roots": [{"uri": "file:///proj", "name": "proj"}]}})
        msg = _recv(lines)
        while msg.get("id") != 2:
            msg = _recv(lines)
        assert any(t["name"] == "probe_tool" for t in msg["result"]["tools"])
    finally:
        proc.stdin.close()
        try:
            proc.wait(timeout=TIMEOUT)
        except subprocess.TimeoutExpired:
            proc.kill()
            proc.wait()

    records = [json.loads(ln) for ln in log.read_text(encoding="utf-8").splitlines()]
    by_kind = {r["kind"]: r for r in records}
    assert by_kind["start"]["cwd"] == str(tmp_path.resolve())
    assert by_kind["initialize"]["clientInfo"]["name"] == "pytest-client"
    assert by_kind["initialize"]["roots_capability"] == {"listChanged": True}
    assert by_kind["roots"]["ok"] is True
    assert by_kind["roots"]["roots"] == ["file:///proj"]
