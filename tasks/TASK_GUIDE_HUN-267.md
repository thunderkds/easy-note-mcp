The task tag pattern in `src/easy_note/core/context.py` is `[A-Z]{2,}-\d{1,6}`. PRD FR-005 declares `[A-Z][A-Z0-9]+-\d+`. So a branch like `feat/AB2-12-x` gets no task.

Fix: use the declared pattern and add a test. Follow-up to [HUN-254](https://linear.app/thunderkds/issue/HUN-254/context-capture-cwd-order-main-worktree-repo-name-remote-sanitizing) (P2 from its code review). Minimal track.