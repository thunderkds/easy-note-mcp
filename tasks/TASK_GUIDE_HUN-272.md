Port the query engine with the same behaviour as [HUN-256](https://linear.app/thunderkds/issue/HUN-256/query-engine-list-search-stats-bounded-results): list, search and stats over the notes. Results are bounded and in a fixed order. It has no index and no model.

Replaces [HUN-256](https://linear.app/thunderkds/issue/HUN-256/query-engine-list-search-stats-bounded-results). Depends on R2.

+++ Agent notes

Oracle: Python tests/test_query.py and its fixtures at main 2d497fc.

+++