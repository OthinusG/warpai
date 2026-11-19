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
  A guarded terminal owns its ConPTY and process Job. Native acceptance is
  pending on the combined source.
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

On macOS ARM64, a five-second sample of the isolated native terminal registry
measured 5.8, 7.9, 9.3 and 10.7 MiB RSS at 0, 1, 8 and 32 idle guarded runs.
The 32-run fixture used 0.79 ms of CPU time during that window. These figures
exclude child processes. In the separate mixed-load test, the quiet task
answered in 2 ms and the busy task's exit was observed 3 ms after stop.

Native CLI launch, bridge loss, explicit resume and stop have been wired into
the SSH and WSL verification fixtures. The Windows SSH fixture uses the
installed Companion through the actual PowerShell function for run and resume.
Same-source cloud results are pending.
