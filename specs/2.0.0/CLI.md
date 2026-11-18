# Remote guardian daemon and private CLI contract

Status: R4 proposed; no commands described here are implemented yet. This refines
[API.md](API.md), [TECH.md](TECH.md) and the owner's requested CLI experience.

## One daemon per execution account

When the guardian option is explicitly enabled, Warpai verifies the target and
ensures a compatible account-owned guardian is running. One daemon owns all
opted-in Agent/shell/command sessions in that OS account, regardless of vendor,
project or originating desktop. No daemon per Agent type, task or tab; no root
machine-wide service and no native-local background runtime.

Canonical scope is the execution OS/kernel plus authenticated user identity.
Different users, WSL distributions and a Windows host versus its Linux guest are
separate execution accounts. Host aliases/IPs are not separate guardian identities.
If SSH and WSL actually reach the same Linux account, both helpers discover the
same guardian through account-private IPC. They retain distinct logical transport/
MCP namespaces and project admission, not distinct guardian processes. Existing
non-guardian services and user state are not forcibly migrated or merged.

Use shared service implementation in Companion with an opt-in guardian mode,
account-global startup election and private endpoint discovery. Preserve legacy
SSH/WSL paths and scope-specific adapters/Stores. If a helper cannot support a
scope on the existing daemon, report incompatibility rather than silently spawning
another guardian or stopping active runs. S0 freezes multi-scope capability rules.

Enabling starts/ensures the daemon immediately. Disabling changes future launches,
not current process policy. With no clients, work, requests or retained output,
the existing idle-exit rule still applies; a later enabled connection ensures it
again. No permanent login service or admin permission is needed by default.

## CLI installation without PATH changes

Ship the control dispatcher in the existing Companion executable, conceptually
`warpai-companion ctl <command>`; the dedicated guest executable exposes the same
contract. Reuse the binary/runtime and protocol client rather than compile another
control daemon or install a general terminal replacement.

Unix SSH helper currently installs at `~/.config/warpai/bin/warpai-companion`;
resolve the actual path from verified installation metadata. WSL uses its existing
private guest installation, Windows its native manifest path. Never assume PATH
lookup, execute a same-named foreign binary or interpolate user labels into shell
command text. Use argument arrays and literal native quoting at launch boundaries.

Warpai's participating interactive remote shell may expose a session-only
`warpai` function forwarding its original argv to the absolute helper path and
`ctl` dispatcher. Use existing native shell bootstrap facilities for Bash/Zsh/
PowerShell; no eval, profile/rc edits, exported function injection, PATH mutation
or global alias. Do not override an existing user command/function/alias; show the
private full-path alternative on collision. Do not inject into noninteractive or
unknown shells, vendor CLI input streams, unrelated SSH sessions or local shells.

In an ordinary independently opened SSH/WSL shell, bare `warpai` is not promised
without PATH/profile changes. The absolute Companion command remains usable.
This limitation is deliberate and must be documented alongside the convenience
syntax. A private packaged launcher can be considered only if it reduces actual
platform complexity; it is not necessary for the first implementation.

## Minimal command surface and selection decisions

R4 withdraws the earlier Herdr-derived command expansion. A command belongs in
this release only when an explicit current SSH/WSL continuity journey needs it
and existing Warpai UI, automatic lifecycle or core commands do not cover it.
Similar tools having a command is not sufficient evidence of need.

| Command | Required user need and behavior |
| --- | --- |
| `warpai list` | Find account guardian sessions and diagnose reconnect: bounded name/ID/project/process/attention/attachment rows plus daemon availability summary |
| `warpai resume` | Continue current/unique/last-used live session, preserving the owner-specified MRU rule and no picker |
| `warpai resume "name"` | Continue one uniquely named managed session |
| `warpai resume --id <session-id>` | Resolve duplicate names using an exact stable session/run |
| `warpai resume "name" --takeover` | Explicitly recover input ownership from a conflicting writer, using generation fencing |
| `warpai stop` | Immediately stop all current-account guardian-managed runs without selector/confirmation/`--yes` |
| `warpai stop "name"` | Stop one uniquely named managed session |
| `warpai stop --id <session-id>` | Stop the exact selected native run |
| `warpai --help` / `warpai <command> --help` / `warpai --version` | Explain installed core syntax/component version; ordinary CLI metadata, not extra functional command groups |

Only three public functional commands are committed: list, resume, stop. Stable
ID and takeover flags solve concrete ambiguity/ownership problems; do not add
aliases or administrative groups merely for symmetry. S0 freezes remaining minimal
parser/output flags required by these paths. GUI clients use the native protocol,
not a new CLI command for every RPC. Do not assume JSON output/completion/new
launch commands are required to implement automatic GUI discovery.

Common scope may be narrowed with an explicit project filter where the shared
API already supports it; bare stop remains account-wide. Named selection must be
unique, otherwise return candidate IDs. Labels denote managed sessions/jobs, not
Agent vendors or business tasks. One task may involve several Agents and one
session several queued tasks; task-title matches never expand a named stop.
Use exact case-sensitive bounded/control-character-free Unicode, no hidden
normalization or fuzzy destructive match. `--` ends option parsing for names
beginning with a dash. Preserve argument arrays without shell evaluation.

## Why other commands are not included

| Candidate | Current workflow coverage | Decision |
| --- | --- | --- |
| status / daemon status | list includes bounded availability and session states; Warpai panel shows detailed facts | No separate command |
| new / start | Existing Warpai remote launch plus guard option creates sessions | No duplicate CLI launch workflow |
| detach | Closing the client/view or the CLI-only detach escape releases the attachment | No separate command |
| rename | Existing launch labels/UI naming and stable IDs cover discovery | No remote naming API solely for a CLI |
| read | Resume restores the terminal; GUI shows current terminal/task state | No separate read-only projection API/reader handles |
| wait | Existing task/supervision and native event handling already drive execution | No user-facing automation waiter service |
| remove | Existing retention/eligible catalog cleanup and current UI/history rules | No new deletion command or task-purge path |
| daemon start / stop | Guard option ensures a runtime; bare stop ends work, then genuine idle exit | No independent admin group |
| completion | Three commands plus installed help; no demonstrated shell-completion need | No generated scripts/loader or profile work |
| stop --all | Bare stop already means all guarded sessions | No redundant alias required |
| plugins / machine / worktree / layout / Agent-specific groups | Existing Warpai product boundaries and interfaces | No Herdr ecosystem duplication |

Reconsider a candidate only after documenting the missing user journey, why current
coverage is inadequate, smallest reusable method and acceptance/cost. Do not keep
withdrawn commands as optional implementation tasks or hidden compatibility aliases;
they were proposals, never shipped. Herdr informs ownership/fairness/attach design,
not a target checklist of tools to reproduce.

### Bare resume and most recently used selection

1. Apply an explicit project filter if supplied. Validate current-session context
   against account/project/runtime/run, not inherited environment values alone.
   A valid current live session wins. If this caller already owns it, report that
   fact rather than recursively attaching the PTY to itself.
2. With no valid current context, a single live resumable session wins.
3. With several live sessions, pick the highest `last_active_sequence` recorded
   by the daemon on successful foreground attachment or explicit foreground view
   selection. This is the last used live session, not the last output, last task,
   most recently polled row or last Agent vendor. Catalog queries and background
   refresh never change it. Detach does not reorder it.
4. If the latest used session exited, skip it and choose the next live session.
   If no surviving session has attachment history, use the newest created live
   session with stable ID as deterministic tie-break. No interactive picker.
5. A selected session occupied by another writer returns a writer conflict;
   do not silently switch to another target. Explicit takeover is required.
   Zero live sessions reports none; never auto-launch a replacement command.

Persist only the nonsecret sequence/identity metadata; native runtime boot/run
validation still excludes interrupted/unknown or rebooted records. Concurrent
foreground choices receive a daemon-serialized monotonic sequence. Sequence
updates do not acknowledge tasks or grant Coordinator rights. Existing GUI
reconnect uses the same most-recent selection when its saved view is unavailable.

### Bare stop and bulk targeting

Bare stop stops all Warpai guardian-managed runs in the current
execution account, regardless of current Agent, shell, project focus or originating
client. They execute immediately in interactive and noninteractive mode without
an extra confirmation or `--yes`. An explicit `--project` narrows the target set;
there is no hidden current-project restriction on bare stop. Other accounts,
distros/kernels, native local terminals, unguarded legacy sessions and unrelated
system processes are excluded.

At admission, atomically capture eligible session/run IDs and catalog revision,
then issue bounded concurrent native stops with individual receipts. Sessions
created after that snapshot are not included. Prevent a retry from targeting a
replacement run; preserve requested/observed/unknown outcomes and partial failure.
An empty set is successful with stopped count zero. Bare stop does not kill the
daemon immediately; it can continue responding and follow its normal idle exit.
After the final run/output retention/request ends, normal idle shutdown reclaims
the daemon. No public daemon-stop command is needed.

The daemon owns an admitted stop batch until completion, not the requesting CLI
socket. Stopping the caller's own guarded shell can disconnect that CLI; this must
not cancel the remaining targets. Retain bounded operation-ID/per-run receipt
metadata so later status can reconcile a lost reply. Deduplicate retries with the
same operation ID; a new explicit command is a new batch over its own fresh target
set. A runtime crash still leaves unconfirmed outcomes, never fabricated success.

## Minimal implementation methods

Reuse the existing account catalog for list, normal guarded attachment/input/resize
for resume, and exact-run daemon-owned stop batches for stop. List aggregates cached
bounded metadata including daemon availability; it creates no screen snapshots,
new observer service, polling thread or writer lease. Existing internal terminal
snapshot/tail and task event mechanisms remain required; they do not imply new
public read/wait commands or separate APIs for them.

The CLI resume client needs a reliable detach path because forwarding Ctrl-D to
an Agent/shell can terminate the owned process. In the direct CLI attachment only,
use a documented local escape such as `Ctrl+B Q`; doubled `Ctrl+B` forwards a literal
prefix. Freeze the exact encoding in S0/S3 and test split input and literal-prefix
handling. It releases only this attachment, restores native TTY state and leaves
the PTY running. Do not install global desktop keybindings, change normal shell
input or add a second multiplexer UI. Transport/window closure also detaches.

Proposed core exit codes: 0 success/empty list/empty stop, 1 domain/partial or
unconfirmed stop/no resumable session, 2 invalid syntax or ambiguous named target.
No wait-timeout/completion/admin code paths are needed. Freeze actual native
parser/TTY behavior in S0 fixtures. Help/version do not start a missing daemon.

## Operation semantics

Resume means reattach an existing live PTY; it never launches a vendor CLI,
restarts a completed command, resumes a rebooted process or reassigns a task.
A writer conflict needs explicit takeover with the observed writer generation.
TTY mode, signals and resize are restored on detach/error. Client exit detaches
without stopping the guardian-owned process. GUI and CLI use the same run/lease,
input receipt and output-gap rules; no second IO protocol/task engine.

Stop is a native process stop request, not task completion/cancellation acceptance.
Return target ID/run, requested outcome and proven/unknown exit status. Existing
task guards retain unresolved execution/review facts. A bulk operation snapshots
exact IDs/run/revisions and reports each result; new sessions created afterward
are not included. A partial failure never becomes overall success or an account
wildcard kill. Native signal behavior and grace/escalation follow the existing
explicit-stop policy, not global name-based termination.

List and help/version do not start a stopped guardian, acquire terminal writers, acknowledge
work or require terminal snapshots. They may report daemon unavailable plus safe
catalog interruption metadata. Resume cannot resurrect a missing runtime; use
explicit guarded recovery, not automatic command replay. Error codes and structured
results map to existing managed semantics; freeze CLI exit codes in S0 fixtures.

## Automatic Warpai reconnect

1. Reauthenticate the selected SSH/WSL execution account. If guardian remains
   enabled, discover/ensure the compatible daemon without replacing active work.
2. Fetch bounded daemon identity/capabilities and session catalog metadata, then
   restore admitted projects and existing task/supervision/attention projections.
3. Rebuild sidebar/status automatically. Refresh via a bounded generation/cursor
   watch or change notification with low-frequency fallback, not per-tab polling.
4. Reattach the saved live guarded view or the daemon-selected most recently
   used live session under the owner-specified resume rule, only if the writer is free.
   Otherwise show conflict/interrupted state; never steal an external writer.
5. Restore other terminal views on selection, generating snapshots only on demand.
   Do not allocate one screen import or worker for every listed session.
6. Reopen file/editor/Review facilities lazily; do not replay uncertain writes,
   acknowledge work, change Coordinator or auto-launch replacement Agents.

Catalog/change cursors are scoped to boot, account and projection revision. A gap
requires a fresh bounded metadata scan, not a guessed lifecycle event. Notifications
carry sanitized IDs/revisions and contain no transcripts or credentials. Listing
own-account metadata grants no task/project read or mutation beyond current scope;
terminal/control operations still revalidate project admission and exact native run.

## Implementation and verification additions

S0 freezes account-global guardian identity, names/ID targeting, CLI defaults,
minimal output/exit codes and same-account cross-scope discovery. S1 implements one-owner
election and routes logical namespaces without Store or capability merging. S4
adds the private CLI plus session-only command and automatic reconnect projection.
S5 verifies real WSL and the same-account SSH/WSL overlap case. S9 packages the
existing executables without PATH/rc changes. Detailed records include actual
symbols, argv/quoting, collisions, OS/namespace election, failure and native proofs.

V25 covers owner-specified MRU/default/all-stop/name/current-context/bulk-stop
semantics. V26 covers no PATH/profile writes, shell collisions/quoting and local
CLI detach escape. V27 covers a single guardian for concurrent clients/aliases and
verified same-account SSH/WSL routes. V28 covers automatic metadata/state restore,
no writer theft, cursor gaps, lazy snapshots and missing-runtime behavior. V29
covers MRU sequencing and fallback. V30 proves the minimal parser/help/list path
and absence of withdrawn CLI services/side effects. All runtime cases remain NOT RUN.
