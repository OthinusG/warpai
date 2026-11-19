# Warpai 2.0.0 implementation ledger

## Active scope

On 2026-11-19 the owner made Companion and its in-terminal CLI the sole guardian
workflow. Desktop terminal takeover, automatic catalog/UI restoration, editor
recovery work introduced for that design, and vendored command-block checkpoints
were removed. Earlier design history remains in Git.

## Existing native implementation

- Unix account election keeps independent native PTYs after SSH or WSL bridge
  exit. Source-matched Linux/macOS SSH evidence is in [44344251976](https://github.com/OthinusG/warpai/actions/runs/44344251976).
- Windows account election uses a separate guardian startup lock. The service verifies
  both named-pipe endpoints by account SID and refuses an inherited launcher Job.
  A guarded terminal owns its ConPTY and process Job.
- WSL reuses the Unix guardian for an exact guest distribution and account.
  The source-matched bridge-loss and namespace fixture passed in [44333896370](https://github.com/OthinusG/warpai/actions/runs/44333896370).
- Exact live mutations use a writer generation. Existing project/task tools,
  shell bootstrap, and unguarded console behavior remain on their ordinary paths.

## Combined verification

The public terminal commands now include explicit guarded launch and same-run
attachment. Each command keeps its original argv. Account lists read the
catalog directly. The unused desktop watch stream, future-launch settings
and terminal snapshot wire code were removed.

Existing ordinary Companion services and the guardian keep independent
state directories. The guardian starts while the ordinary startup lock is held.
A native mixed-load fixture verifies that a noisy run and a blocked input pipe
leave a quiet task in a second project usable.

On macOS ARM64, one-minute samples of the isolated native terminal registry
measured 5.9, 8.3, 9.5 and 10.8 MiB RSS at 0, 1, 8 and 32 idle guarded runs.
The 32-run fixture used 9.8 ms of CPU time and 66 threads. These figures exclude
child processes. In the separate mixed-load test, the quiet task answered in
2 ms and the busy task's exit was observed 3 ms after stop.

Both native acceptance workflows passed at
`6e27d0e759daf911b001188f9f4051933f04fcfb`:

- Linux, macOS and Windows passed native SSH, installer and legacy connection
  tests. The fixture verified CLI launch, bridge loss, same-PID resume and
  observed stop. Windows exercised run and resume through the installed
  PowerShell function. [Remote Companion CI](https://github.com/OthinusG/warpai/actions/runs/44482504584)
- Windows-to-WSL passed all three workspace fixtures, including CLI launch and
  observed stop, all-bridge loss, same-PID resume and both SSH/WSL startup
  orders. The WSL runner also tested the bundled shell function in the guest.
  [Windows-to-WSL CI](https://github.com/OthinusG/warpai/actions/runs/44482504436)

On the same application source, the macOS and Windows desktop compile checks
and platform protocol jobs passed. The WSL fixture was rerun with isolated
dependency installation after an Ubuntu index download exceeded the previous
provisioning timeout. [Desktop and protocol CI](https://github.com/OthinusG/warpai/actions/runs/44478098306)

## Task controls and collaboration headings

The CLI accepts program-first launch, optional background launch, short task
IDs, retained plain-text logs and catalog-only rename. Warpai's Keys settings
own the editable detach shortcut. Companion marks active attachments, forwards
ordinary keys and detaches when it receives the dedicated signal. The Agents
heading now shares the Tasks heading's left origin while agent rows retain
their inset.

The native SSH matrix passed at `76c656be`, covering Linux, macOS and Windows
PowerShell. It exercised same-PID reattach, live and ended output, rename by
short ID, the dedicated detach signal, and background-task termination.
[Native SSH CI](https://github.com/OthinusG/warpai/actions/runs/44494986053)

The Windows-to-WSL matrix passed on the same revision, including the shared
management fixture and custody after both bridges closed.
[Native WSL CI](https://github.com/OthinusG/warpai/actions/runs/44494985483)

The macOS and Windows desktop builds, tests and native captures passed at
`667d12a1`. The English `live-ssh-task.png` on both platforms shows the
Agents and Tasks heading icons and labels aligned, with agent rows still
inset. The later SSH and WSL revision changed only the shutdown test fixture.
[Desktop CI and visual artifacts](https://github.com/OthinusG/warpai/actions/runs/44492631197)
