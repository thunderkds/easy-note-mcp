"""Where the notes dir is (FR-001): `EASY_NOTE_DIR`, else `dir` in config.toml, else `~/easy-notes`."""

from __future__ import annotations

import json
import os
import tempfile
import tomllib
from pathlib import Path


class ConfigError(ValueError):
    """config.toml exists but cannot be read."""


def config_path() -> Path:
    return Path.home() / ".config" / "easy-note" / "config.toml"


def notes_dir() -> Path:
    env = os.environ.get("EASY_NOTE_DIR")
    if env:
        return Path(env).expanduser()
    path = config_path()
    try:
        data = tomllib.loads(path.read_text(encoding="utf-8"))
    except FileNotFoundError:
        return Path.home() / "easy-notes"
    except (OSError, UnicodeDecodeError, tomllib.TOMLDecodeError) as exc:
        raise ConfigError(f"cannot read {path}: {exc}") from None
    value = data.get("dir")
    if value is None:
        return Path.home() / "easy-notes"
    if not isinstance(value, str) or not value:
        raise ConfigError(f"{path}: 'dir' must be a non-empty string")
    return Path(value).expanduser()


def save_dir(path: Path) -> None:
    """Write `dir = "<path>"` as the whole config.toml, atomically (used by `easy-note init`)."""
    target = config_path()
    target.parent.mkdir(parents=True, exist_ok=True)
    # A JSON string is a valid TOML basic string once DEL (raw-forbidden in TOML) is escaped.
    value = json.dumps(str(path), ensure_ascii=False).replace("\x7f", "\\u007f")
    data = f"dir = {value}\n".encode()
    fd, tmp = tempfile.mkstemp(prefix=".config.", suffix=".tmp", dir=target.parent)
    try:
        with os.fdopen(fd, "wb") as fh:
            fh.write(data)
            fh.flush()
            os.fsync(fh.fileno())
        os.replace(tmp, target)
    finally:
        if os.path.exists(tmp):
            os.unlink(tmp)
