# Herdr screen detection attribution

Warpai adapts Agent screen detection manifests and prompt-region semantics from
[herdrdev/herdr](https://github.com/herdrdev/herdr), commit
`2f3432c6889d7e7cd8a0156a7daefa69d63891b2`, under Apache-2.0.
See [HERDR-LICENSE](HERDR-LICENSE) for the complete license.

Original source: `src/detect/manifests/` and `src/detect/manifest.rs`.
Modifications rename canonical Agent IDs and translate working/blocked to the
existing Warpai running/waiting manifest states and visible evidence field.
Warpai preserves its native run, account, task, permission and delivery fences.

The standalone Companion uses vt100 0.16.2 (MIT) for bounded native terminal
snapshots. See [VT100-LICENSE](VT100-LICENSE). Its parser is separate from the desktop terminal rendering core.
