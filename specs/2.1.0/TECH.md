# Guarded process panel implementation

Use the confirmed SSH/WSL transport and the installed Companion CLI. The
structured list projection is scoped to the focused route and carries
stable task IDs. Discard results after focus changes. Execute rename, logs
and stop only against a selected full ID. Resume uses the same transport
with a PTY and queues the command in the newly returned pane.

Store only the executable basename with new task records. Migrate existing
catalogs in place and display unavailable metadata for older tasks. Verify
CLI serialization, shell argument boundaries, migration, focus fencing,
panel rendering, and new-pane resume on SSH and WSL.
