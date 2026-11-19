# Companion terminal commands

| Command | Action |
| --- | --- |
| `warpai run --name task1 -- program args` | Start a guarded task in the current remote directory and attach its terminal. The name is optional. |
| `warpai list` | Show account runs, stable IDs, names, writer and lifecycle. |
| `warpai resume task1` | Attach to a live task by an exact unique name. |
| `warpai resume --id ID` | Select an exact run when names collide. |
| `warpai stop task1` | Request termination of the exact task and report native exit when observed. |

Ctrl+B Q detaches and keeps the task running. EOF also detaches. Bare resume
uses the current task, the sole live task, or the most recently used live task
in the helper's route. Bare stop affects all guarded tasks in the account.
Explicit takeover requires `--takeover`. List, resume and stop query an
existing daemon. Explicit run starts the guardian when needed.

The Bash, Zsh and PowerShell startup functions are scoped to participating
Warpai terminals. Preserve existing aliases, functions and executables. The
installed absolute Companion path and manifest are checked before forwarding
argv. Help and version remain available through the same entry.
