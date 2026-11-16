# Task archive and home visibility

Owner request: 2026-11-16. Work on `r2-agent-lifecycle-ci`, then merge it and
`fix/1.9.0` into `main` and start GitHub validation.

## Acceptance and implementation

- Archive accepts no reason. The current native form and controller already
  require no reason; protect this behavior with a focused regression.
- Hide cancelled tasks from the default native task list before pagination.
  Explicit cancelled-state/history browsing and Agent task-list APIs retain access.
- Returning from History/detail to the home list resets archive/state filters
  and pagination so archived tasks do not leak into the home list.
- Preserve task history, handoffs, unresolved attention/recovery facts, execution
  uncertainty, archive eligibility and deletion guards. Add no protocol fields.

## Verification

Check Rust parsing, diff whitespace and strict locale coverage locally. Run the
existing communication CI on merged main, including protocol regressions,
default/platform desktop builds, application tests and native UI captures.
Add focused regressions for reason-free archive and filtered pagination/history.
Keep unrelated uncommitted owner documents in the original worktree.
