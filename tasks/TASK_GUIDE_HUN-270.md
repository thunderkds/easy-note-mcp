Port the secret guard. The ten declared kinds are replaced by `[masked:<kind>]`. Masking runs on every store write and read path, and covers the body, tags, context fields and front matter. `add` and `update` report the kinds they masked. A leak test suite checks that no planted fake secret reaches a file, a response, a log or printed output. Behaviour matches the verified Python [HUN-255](https://linear.app/thunderkds/issue/HUN-255/secret-guard-declared-patterns-mask-on-write-and-read-leak-test-suite) branch.

Replaces [HUN-255](https://linear.app/thunderkds/issue/HUN-255/secret-guard-declared-patterns-mask-on-write-and-read-leak-test-suite) (never merged). Part 1, the guard module, depends on R1. Part 2, the store wiring, depends on R2.

+++ Agent notes

Oracle: `feat/HUN-255-secret-guard` (3c5e40f) tests/test_guard.py, tests/test_leak.py, tests/test_store.py parse-back test, fixtures. Look-ahead patterns: restate without look-around or use fancy-regex. Do not commit token literals that GitHub push protection flags (build fake tokens at runtime, e.g. "xox" + "b-...").

+++