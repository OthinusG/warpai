# Companion guardian implementation

## Launch and ownership

The terminal integration forwards explicit argv to the installed Companion.
The CLI resolves a native executable in the invoking remote shell's context,
opens its current working directory through managed control, and asks the
guardian to start a guarded PTY. The invocation attaches directly. Returning
from the CLI releases only the connection's writer.

Companion elects one guardian for each operating-system execution account.
The guardian has a separate state directory and startup lock from the ordinary
Companion. Unix uses a private local socket and independent session. Windows uses a
same-SID named pipe, a launcher verified outside the original Job, and ConPTY
with a guardian-owned process Job. Linux SSH and WSL share an owner only when
the native account is the same. Logical routes, project access and task stores
remain scoped.

## IO and catalog

Reuse the managed PTY, bounded 256 KiB output ring, input receipts, per-run
writer fencing and native process observation. The existing Agent path can
keep its headless task notifications. The CLI presents a bounded screen built
from retained output on attachment and then forwards live terminal IO. It
never evaluates the program's output as shell input.
Companion marks the lifetime of an interactive attachment in its terminal
output. The desktop uses that marker to enable its editable detach action in
the focused terminal. Invoking the action sends a dedicated UTF-8 detach
signal. Companion treats ordinary keystrokes as task input.

The catalog stores only identity, optional display name, route/project,
sequence and observed lifecycle. Review, file transfer, editor, shell model and
rendering state stay outside the guardian. On an unclean guardian restart any
previous live record becomes interrupted. Keep bounded ended output and shut
down the account service after the idle period.

Background launch takes the regular launch path and releases its initial
attachment before returning. Logs copy the existing ring for an exact run
without acquiring its writer. The CLI strips control sequences before printing.
Rename updates the catalog label atomically and never changes the native run.

## Verification

Run platform-native SSH and WSL tests for explicit launch, bridge loss,
same-PID reattach, stop of descendants, and unguarded compatibility. Measure
resident memory, CPU time, threads and context switches separately from the
native programs. Validate the installed helper and shell gates on the same
source as those tests.
