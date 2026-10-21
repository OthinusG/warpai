# MCP server-name migration bugfix (1.7.8.2)

## Problem

1.7.8.1 changed the persisted `Installed.server_name`, while existing native CLI
adapters kept the previous server name in their serialized `add`, `remove` and
`get` argv vectors. A QoderCN record can therefore claim `warpai-local-com` while
querying `warp-lite-communication`. Its successful “old server not found” output
is misread as an existing user-owned server; an enable retry may also add the old
name again. Existing machine evidence: 1.7.8.1 QoderCN metadata has this mismatch,
and its current CLI reports exact not-found results for both names.

## Scope and acceptance

- Treat the persisted scope name as canonical and migrate only recognized Warpai
  names in native CLI argv; preserve argument order, scope flags and other args.
- Migrate loaded app preferences and save them through the existing atomic
  preferences writer. Normalize at the setup/uninstall boundary too, covering
  remote and direct callers that deserialize old preferences independently.
- Old local/SSH shared registration is removed before local install/retry;
  WSL legacy registration remains isolated to WSL.
- Exact absence is idempotent success for cleanup; successful enable adds the new
  name, and repeat cleanup removes it without an error.
- Preserve unrelated CLI arguments, user MCP entries, vendor-owned commands and
  credentials. No extra process, install path or external state changes.

## Tasks and validation

1. Add small argv migration helper plus `Preferences` traversal.
2. Apply migration at app load and configuration/uninstall entry points.
3. Test old local, SSH and WSL names; exact QoderCN output; install, cleanup and
   repeated cleanup against an isolated fake CLI; preserve unrelated arguments.
4. Run focused Agent Bus tests and all existing cloud validation/release gates
   against the immutable `v1.7.8.2` source before publishing the release.
