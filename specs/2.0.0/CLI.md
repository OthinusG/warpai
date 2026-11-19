# Companion terminal commands

| Command | Action |
| --- | --- |
| `warpai run program args --name task1` | Start a guarded task in the current remote directory and attach its terminal. Omit `--name` to use the executable's filename. |
| `warpai run program args --name task1 --background` | Start a guarded task and return to your shell with its ID. |
| `warpai list` | Show short IDs, names, lifecycle, writer, project ID and last observation time in UTC. |
| `warpai resume task1` | Attach to a live task by an exact unique name. |
| `warpai resume --id ID` | Select an exact run when names collide. |
| `warpai stop task1` | Request termination of the exact task and report native exit when observed. |
| `warpai logs task1` | Print retained terminal output as plain text, including recently ended tasks with output still retained. |
| `warpai rename task1 task2` | Set the display name without changing the task identity or attachment. |

Use a full ID or its unique prefix of at least eight hexadecimal digits with
`--id`. Logs are bounded to the native output ring and report when earlier
bytes have been discarded. The catalog retains task metadata independently.

In Warpai Settings > Keys, edit **Detach guarded task**. Its default is Ctrl+B
followed by lowercase q. The shortcut applies to the focused guarded task and
keeps that task running. Rebinding it returns the original keys to the task.
EOF also detaches.
Bare resume
uses the current task, the sole live task, or the most recently used live task
in the helper's route. Bare stop affects all guarded tasks in the account.
Explicit takeover requires `--takeover`. List, resume and stop query an
existing daemon. Explicit run starts the guardian when needed.

Run options `--name NAME` and `--background` go after the program's arguments. Use
`warpai run -- program args` to pass every argument literally, including a
trailing `--name NAME` intended for the program. The earlier
`warpai run --name NAME -- program args` form still works.

The Bash, Zsh and PowerShell startup functions are scoped to participating
Warpai terminals. Preserve existing aliases, functions and executables. The
installed absolute Companion path and manifest are checked before forwarding
argv. Help and version remain available through the same entry.
