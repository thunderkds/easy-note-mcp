Port context capture with the same behaviour as [HUN-254](https://linear.app/thunderkds/issue/HUN-254/context-capture-cwd-order-main-worktree-repo-name-remote-sanitizing) and [HUN-267](https://linear.app/thunderkds/issue/HUN-267/align-context-task-pattern-with-prd-fr-005). It runs read-only `git` with a 2 s timeout. It takes the repo name from the main worktree, strips user info from the remote, and finds the task ID in the branch name using the PRD FR-005 pattern.

Replaces [HUN-254](https://linear.app/thunderkds/issue/HUN-254/context-capture-cwd-order-main-worktree-repo-name-remote-sanitizing) and [HUN-267](https://linear.app/thunderkds/issue/HUN-267/align-context-task-pattern-with-prd-fr-005). Depends on R1.

+++ Agent notes

Oracle: Python tests/test_context.py at main 2d497fc.

+++