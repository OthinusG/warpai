# Private guardian protocol

This uses the existing versioned managed Companion channel. Local transports
verify the execution account and negotiate a guardian and boot fence. Keep
the established ordinary managed channel compatible with older helpers.

- `TerminalLaunch` with guarded policy starts a new exact run beneath a native
  PTY in an admitted project. The response binds session, run and writer.
- `AccountCatalogList` and `AccountCatalogInspect` describe bounded known runs
  without acquiring input or restarting work.
- `GuardedProjectOpen` resolves the original project only for an authenticated
  local operator holding the exact run selector.
- `TerminalControl` handles attach, detach, input, resize, read and terminal
  exit observation. An explicit takeover must present the last observed writer
  generation. A stale generation cannot act on its successor.
- `AccountStop` captures an exact run or the bounded current set for the
  requested account/project. A retry within its boot uses the same run set.
  Native exit is distinct from delivery and request acceptance.

Managed framing, bounded reads and input dispositions come from [the protocol
definition](../../crates/remote_protocol/proto/remote_server.proto). Persist
opaque run metadata only. Neither the desktop nor the protocol automatically
starts or reattaches previously running commands.
