# Companion terminal commands

| Command | Action |
| --- | --- |
| `warpai run program args --name task1` | Start a guarded task in the current remote directory and attach its terminal. Omit `--name` to use the executable's filename. |
| `warpai list` | Show account runs, stable IDs, names, writer and lifecycle. |
| `warpai resume task1` | Attach to a live task by an exact unique name. |
| `warpai resume --id ID` | Select an exact run when names collide. |
| `warpai stop task1` | Request termination of the exact task and report native exit when observed. |

In Warpai Settings > Keys, edit **Detach guarded task**. Its default is Ctrl+B
followed by lowercase q. The shortcut applies to the focused guarded task and
keeps that task running. Rebinding it returns the original keys to the task.
EOF also detaches.
Bare resume
uses the current task, the sole live task, or the most recently used live task
in the helper's route. Bare stop affects all guarded tasks in the account.
Explicit takeover requires `--takeover`. List, resume and stop query an
existing daemon. Explicit run starts the guardian when needed.

Only a trailing `--name NAME` is consumed as a task name. Use
`warpai run -- program args` to pass every argument literally, including a
trailing `--name NAME` intended for the program. The earlier
`warpai run --name NAME -- program args` form still works.

The Bash, Zsh and PowerShell startup functions are scoped to participating
Warpai terminals. Preserve existing aliases, functions and executables. The
installed absolute Companion path and manifest are checked before forwarding
argv. Help and version remain available through the same entry.
