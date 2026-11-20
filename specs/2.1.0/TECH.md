# Guarded process panel implementation

Use the confirmed SSH/WSL transport and the installed Companion CLI. The
structured list projection is scoped to the focused route and carries
stable task IDs. Discard results after focus changes. Execute rename, logs
and stop only against a selected full ID. Resume uses the same transport
with a PTY and queues the command in the newly returned pane.
Track that pane while its resume is pending or attached, including across
panel or terminal focus changes. Repeated clicks for that task must leave
one new pane. A detached or closed pane must allow a later resume.
Companion remains the single-writer authority across GUI and CLI clients.

Store only the executable basename with new task records. Migrate existing
catalogs in place and display unavailable metadata for older tasks. Verify
CLI serialization, shell argument boundaries, migration, focus fencing,
panel rendering, and new-pane resume on SSH and WSL.

The native desktop capture workflow launches an owned background task through
the confirmed SSH or WSL transport and records its PID. It exercises the
process entry, Back and Resume from the panel and checks that only a new
split attaches to that PID even when Resume is requested twice. The fixture
detaches from the split, returns focus to the source pane and repeats Resume
against the same PID. It then sends input through the resumed split to finish
the owned task and checks that the split returns to its shell.
The Windows capture starts its WSL fixture in a fresh native terminal after
the SSH checkpoints finish.
The native CLI fixture also checks that a second writer is refused and a
subsequent resume after detach accepts input in the same process.
