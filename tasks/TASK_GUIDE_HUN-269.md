Port the note format (DDR 0002, `docs/NOTE_FORMAT.md`) and the store to Rust with the same behaviour as the Python [HUN-253](https://linear.app/thunderkds/issue/HUN-253/note-format-and-store-parserender-id-atomic-write-clash-suffix). New IDs. Never-overwrite atomic writes, using a temp file, fsync and a hard link. Clash suffix. Trash and restore. Unreadable files are reported as skipped.

Replaces [HUN-253](https://linear.app/thunderkds/issue/HUN-253/note-format-and-store-parserender-id-atomic-write-clash-suffix). Depends on R1.

+++ Agent notes

Oracle: the Python `tests/test_format.py` and `tests/test_store.py` in git history (main 2d497fc), ported to Rust tests first (red).

+++