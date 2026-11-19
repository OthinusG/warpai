# Warpai 2.0.0 remote task guardian

Scope approved on 2026-11-19. Companion owns explicitly launched remote tasks
for the execution account. A task is one independent command, shell or Agent
process and its native process group or Job.

## User workflow

1. Open an SSH or WSL terminal in Warpai. Run
   `warpai run <program> [args...] --name task1` to start a guarded task
   in that terminal's remote working directory. Ordinary commands keep their
   existing behavior.
2. Close the desktop or disconnect. The guardian keeps the original process
   running. The terminal can also detach with Ctrl+B Q.
3. Connect again. The new terminal starts normally. Use `warpai list` and
   `warpai resume task1` to find and attach the same live task explicitly.
4. Use `warpai stop task1` to stop that task's owned process tree. The response
   distinguishes accepted stop from observed exit.
5. Use `--background` to launch without opening a terminal attachment.
   Use the short ID from `warpai list` to distinguish tasks with the same name.
   `warpai logs` reads retained output while a task runs or after it ends.
   `warpai rename` changes the label while preserving the same run.

The command is a session-scoped function in Warpai's participating Bash, Zsh,
or Windows PowerShell terminal. Companion contains the actual command
implementation. Windows Terminal and ordinary SSH consoles can consume native
PTY output when their shell exposes the integration. The bare `warpai` command
is not installed into PATH or profile files.

## Boundaries

Desktop close and reconnect do not take over a remote writer, restore a tab, or
restart tasks. Native local terminals and existing SSH and WSL entries keep
their current behavior. Host reboot or WSL shutdown ends execution and leaves
interrupted catalog evidence. Existing tasks cannot be adopted retroactively.
