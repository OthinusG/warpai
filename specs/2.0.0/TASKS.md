# Warpai 2.0.0 implementation tasks

This is the active CLI-only guardian scope approved on 2026-11-19.

- [ ] Launch a native command or shell in the current guarded SSH/WSL
  project, attach its terminal and release the writer on disconnect.
- [ ] Verify exact task selectors, explicit takeover, sibling isolation and
  truthfully observed stop for native process trees.
- [ ] Verify session-local Bash, Zsh and Windows PowerShell integration,
  literal argv, installation checks, and command collisions.
- [ ] Remove automatic desktop ownership, desktop restoration and unused
  runtime/protocol machinery for those paths.
- [ ] Verify low overhead, legacy compatibility, and packaged helpers on
  Linux, macOS and Windows SSH and real Windows-to-WSL.
- [ ] Push the completed source and run the native verification workflows
  together on the same revision. Record actual results in IMPLEMENTATION.md.
