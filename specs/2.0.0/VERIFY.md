# CLI guardian acceptance

| Case | Required evidence |
| --- | --- |
| G01 Explicit launch | Run from Bash, Zsh and Windows PowerShell with literal argv, Unicode cwd and an optional trailing task name. Verify executable-name defaults and literal forwarding with the leading `--`. Ordinary commands remain connection-owned. |
| G02 Bridge loss | Start a native task, close its terminal and every bridge, reconnect into a new shell, and verify the original PID with CLI resume. |
| G03 Scope | Verify Windows native SSH and exact WSL distribution/user. Use one owner only for the same native account while keeping SSH and WSL logical Stores separate. |
| G04 Writer and detach | Reject a second writer unless explicitly taking over. Check the split detach signal, ordinary keystroke forwarding, EOF and restoration of local TTY settings. The desktop keybinding must apply only during an interactive guarded attachment. |
| G05 Stop | Prove process-tree exit, observe a stop initiated from a surviving terminal, and leave unrelated tasks alone. |
| G06 CLI integration | Verify session-only exposure, installation manifest checksum, command collisions, exact and ambiguous labels. |
| G07 Load | Measure resident memory, CPU, threads and context switches at 0, 1, 8 and 32 guarded sessions. Test an active flood, a blocked writer and a quiet sibling. |
| G08 Compatibility | Verify unguarded, existing project tools, installed helper version, old clients, and reboot/lost daemon interruption. |
| G09 CLI management | Launch in the background and verify a free writer, renamed task and unchanged ID. Read bounded plain text while a writer is attached and after observed exit. Resolve short IDs and reject ambiguous names and prefixes. |
| P01 Collaboration panel | The Tasks and Agents heading icons and labels share the same horizontal origin in Project mode. Agent list rows retain their nested inset. |

Platform matrix is Linux/macOS/Windows native SSH plus Windows-to-WSL.
Evidence must identify the actual source and platform. Local component tests
complement native end-to-end runs.
