# Warpai 2.0.0 implementation ledger

## Active scope

On 2026-11-19 the owner made Companion and its in-terminal CLI the sole guardian
workflow. Desktop terminal takeover, automatic catalog/UI restoration, editor
recovery work introduced for that design, and vendored command-block checkpoints
were removed. Earlier design history remains in Git.

## Existing native implementation

- Unix account election keeps independent native PTYs after SSH or WSL bridge
  exit. Source-matched Linux/macOS SSH evidence is in [44344251976](https://github.com/OthinusG/warpai/actions/runs/44344251976).
- Windows account election reuses the ordinary Store lock. The service verifies
  both named-pipe endpoints by account SID and refuses an inherited launcher Job.
  A guarded terminal owns its ConPTY and process Job. Native acceptance is
  pending on the combined source.
- WSL reuses the Unix guardian for an exact guest distribution and account.
  The source-matched bridge-loss and namespace fixture passed in [44333896370](https://github.com/OthinusG/warpai/actions/runs/44333896370).
- Exact live mutations use a writer generation. Existing project/task tools,
  shell bootstrap, and unguarded console behavior remain on their ordinary paths.

## Combined verification

Explicit CLI launch, shell integration, reduced runtime and installed native
verification are in progress. Results here refer to their actual tested source.
