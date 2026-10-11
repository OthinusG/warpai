# Contributing to Warpai

Warpai is independently maintained. Read [README.md](README.md) and
[AGENTS.md](AGENTS.md) before changing the terminal core, build or release paths.

## Issues and changes

Search [Warpai issues](https://github.com/OthinusG/warpai/issues) before opening
a bug report or feature request. Include the app version, desktop OS, clear
reproduction steps and expected/actual behavior. For SSH Remote or WSL issues,
include the remote OS and Companion version. Remove credentials and private
content from logs and screenshots.

Branch from `main` and keep changes focused. For larger features or deployment
changes, define testable behavior and an implementation/verification plan under
[specs/](specs/) before coding. Existing examples include [GH408](specs/GH408/)
and [GH1063](specs/GH1063/). Bug fixes should reproduce the problem when feasible
and cover the affected behavior with focused validation.

## Scope and verification

Desktop targets are macOS and Windows. SSH Remote and Companion targets are
Linux, macOS and Windows. Preserve terminal/core compatibility, attribution and
licenses; do not restore bundled cloud AI, telemetry, account or billing flows.

Use the pinned lockfiles/toolchain and existing scripts. Run focused tests for
changed logic, relevant application checks (including `warp_platform` when
affected), script syntax checks and workflow validation. For UI changes, inspect
native screenshots and interaction states when the environment permits it.
Report checks that could not run and material compatibility risks in the PR.

Release packaging uses the standalone workflows and scripts linked from the
[README](README.md). Tagged source and installer provenance must agree.
