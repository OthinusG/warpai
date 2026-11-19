# CLI integration

`warpai list --json` returns a bounded array of tasks in the current
route with `id`, `name`, `program`, `state` and `attached`. A successful
empty response indicates an active guardian with no matching tasks.
Noninteractive operations use `rename --id ID NAME`, `stop --id ID`
and `logs --id ID`. Interactive resume uses `resume --id ID`.
