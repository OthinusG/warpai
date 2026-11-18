# Optional remote guardian API contract

Status: R4 SSH/WSL semantic contract. S0 froze the additive wire fields recorded
below; the operation names remain conceptual until their native paths are
implemented and verified. The protobuf remains the eventual single source of
truth; do not create a second JSON terminal protocol or route raw screen data
through the task panel payload.

## Compatibility boundary

Current managed SSH protocol major is 1 and Agent Bus major is 2. Current
`TerminalLaunch`, `TerminalControl` and `TerminalState` remain connection-owned.
Reserved protobuf fields and action numbers remain reserved. Add new operations
and capability negotiation rather than changing legacy disconnect/release meaning.

Proposed capability `guarded_remote_sessions_v1` covers the complete operation
set below for managed Agent and shell/command sessions and snapshot schema 1.
SSH and dedicated WSL guest builds advertise it independently only after their
native launch/attach/input/snapshot/stop paths pass. Native local launches/Broker
and local MCP are unchanged. WSL admission includes exact distro/user and guest
service identity; SSH admission uses the confirmed final execution host/account.

| Client / runtime | Required behavior |
| --- | --- |
| 2.0 client / 6.1.0 SSH or WSL host | Existing features work; persistent launch unavailable with clear upgrade guidance |
| 1.9 client / new host | Legacy operations and connection-owned cleanup stay unchanged |
| 2.0 client / compatible persistent host | Discover, attach and operate negotiated snapshot schema |
| New client / older compatible live runtime | Keep runtime alive; use intersection of capabilities |
| Any client / incompatible live runtime | Reject unsupported operations; never auto-stop active runs |

Legacy terminal control/read/release cannot operate on a persistent session even
with a guessed valid UUID. Return feature/authority error; disconnecting an old
client must not kill an unrelated retained run. Capabilities are functional
contracts, not version-string guesses. Major versions change only if the final
contract actually breaks existing decoding or semantics.

## S0 frozen wire fields (2026-11-18)

Implemented additively in protocol major 1; field/action numbers are now frozen:

| Message | Field | Number |
| --- | --- | --- |
| `TerminalLaunch` | `guarded` | 9 |
| `TerminalLaunch` | `label` | 10 |
| `TerminalLaunch` | `session_kind` | 11 |
| `TerminalAction` | `TERMINAL_ATTACH` | 8 |
| `TerminalAction` | `TERMINAL_DETACH` | 9 |
| `TerminalControl` | `attachment_id` | 11 |
| `TerminalControl` | `attachment_generation` | 12 |
| `TerminalControl` | `takeover` | 13 |
| `TerminalState` | `guarded` | 16 |
| `TerminalState` | `label` | 17 |
| `TerminalState` | `attachment_generation` | 18 |
| `TerminalState` | `attachment_id` | 19 |

`guarded_remote_sessions_v1` is advertised only after the native launch, attach,
input, snapshot and stop paths pass. It is unadvertised at this S0 slice, so
compliant clients must reject guarded fields with clear upgrade guidance.

### S0 decision record: session kind, election and catalog storage (2026-11-18)

`TerminalSessionKind` uses `UNSPECIFIED=0`, `AGENT=1`, `SHELL=2`, `COMMAND=3` in
additive `TerminalLaunch.session_kind = 11`. A guarded launch requires exactly one
of 1..3. `AGENT` requires a valid nonblank `agent_program`; `SHELL` and `COMMAND`
require its absence and create no RunBinding. An unguarded launch must omit the
field and keeps legacy semantics, including an optional `agent_program`. Absent,
zero, unknown or policy-mismatched kind is rejected before any native spawn, and
kind participates in launch replay equivalence. The client transmits it only
when `guarded_remote_sessions_v1` is negotiated.

The account guardian root is
`<passwd-home>/.local/share/warpai/guardian` (Linux) or
`<passwd-home>/Library/Application Support/warpai/guardian` (macOS), resolved from
the effective UID's account, never `HOME`/`XDG`/route variables. It contains
`service.lock`, `service.id` and `catalog.sqlite3`. One private
`/tmp/warpai-guardian-<euid>/` (0700) holds `ssh.sock` and Linux `wsl.sock`
(0600), owned by one process. One lock spans versions and is taken before DB
open, cleanup or bind. A candidate probes an existing owner under the same
deadline; it may reclaim only an owned stale socket inode while holding the lock
and never unlinks a responsive listener. A bounded private IPC preface carries
the guardian IPC major, requested route, stable guardian ID and non-nil boot
UUID; both ends verify kernel peer UID and the handshake before forwarding. No
boot/PID file or lock metadata proves liveness, and there is no second owner or
legacy fallback.

Route is bound by the accepting listener and resolved server-side; helper strings
and managed frames confer no authority.
A guarded WSL scope uses the distro returned by the guest's native wslpath
translation of the root. A supplied WSL_DISTRO_NAME is a consistency check.
An absent or mismatched native identity refuses guarded WSL scope admission;
the legacy WSL path keeps its existing behavior. Each route keeps its existing logical
service ID and exact fence validation. One account `Terminals` registry and cap
serves all routes with per-run logical-scope checks; Files and Projects/Store stay
route-local, and WSL-only or SSH-only handlers follow the admitted route. Before
opening a Store or cleaning scratch space, the guardian holds that scope's
existing `service.lock` for as long as the scope is in use; live legacy lock
contention refuses that scope with upgrade guidance and never stops or duplicates
the legacy Broker.

Durable catalog metadata lives in `guardian/catalog.sqlite3` with
`application_id=0x57474341` and `user_version=1`, initialized only for a newly
created file and validated read-only before any write. Table
`guardian_catalog_sessions` stores session/run/boot UUIDs, logical service and
project IDs, actual policy and kind, bounded sanitized label, creation and MRU
sequences, state, observation Unix-ms and optional exit status. It stores no
argv, environment, screen, output, attachment token or relaunch plan. On boot,
prior live rows are transactionally marked interrupted/unknown. Pruning is
limited to resolved ended rows; unresolved uncertainty yields capacity errors.
V1 performs no migration or automatic recovery.

## Unified guardian identity and control clients

The guardian process identity is account-global for the execution OS user, not
Agent type or connection profile. Add guardian identity/boot and logical namespace
mapping to the S0 wire freeze rather than silently treating distinct legacy
service IDs as equal. Same-account SSH/WSL helpers elect/discover one compatible
guardian through private IPC; logical ManagedFence/MCP/Store scopes remain distinct.
A new guardian boot invalidates all attachments and catalog watch cursors.

The private WAG1 request mode distinguishes the guarded data bridge, direct
same-UID operator CLI, and connection-owned forwarding from an upgraded ordinary
helper. A data bridge cannot invoke account stop. An ordinary helper does not
start a guardian or acquire guarded capabilities, and verifies that the admitted
logical service ID matches its existing scope before returning the handshake.
The guarded bridge verifies the same scoped service ID and matches the guardian
and boot IDs returned by the private socket handshake before handing the
managed connection to the client. Direct CLI admission checks the same
guardian/boot pair. A mismatch closes the connection before project admission.

Companion-private CLI and GUI use the same operations and authenticated admission.
Account-level catalog metadata is visible to the authenticated owning user, but
contains no project task bodies/files/transcripts and grants no task/terminal
rights. Opening/control still requires the relevant project/scope/run checks.
Catalog watches are bounded, revision/cursor-scoped and contain sanitized IDs/state
only; gaps require bounded relisting. Reconnect restores metadata automatically,
not every writer/snapshot. Detailed CLI/default/name rules live in [CLI.md](CLI.md).

A direct operator handshake may report `ManagedInitialized.current_guarded_session`
(tag 8). The guardian derives this selector from the authenticated local
peer's OS session leader and the currently owned guarded PTY; remote bridges
never receive it. A bare resume in that run succeeds without attaching it to
its own PTY. The selector grants no writer or project authority.

## Optional policy and launch intent

The client stores a future-launch guardian default per confirmed SSH target or
WSL distro/user. Default is off. Each launch explicitly resolves and transmits
its immutable session policy: legacy `connection_owned` or new `guarded`.
The new operation requires `guarded`; legacy operations never silently retain.
A changed profile default does not mutate live sessions, kill work or enable a
service on unrelated targets. Ordinary native WSL Agent discovery remains intact.

Guarded launch includes session kind (`agent`, `shell`, `command`), validated
working directory and original native executable/arguments. Agent kind uses the
existing native MCP/run binding; shell/command kind has no synthetic Agent rights.
An existing unmanaged process cannot become guarded through a catalog ID or PID.
In-place promotion and automatic takeover of old terminals are outside v1.

## Identities and authorization

| Field | Meaning and lifetime | Validation |
| --- | --- | --- |
| `service_id` | Per-account runtime installation identity | Existing verified identity, not UI hostname |
| `service_boot_id` | Current runtime instance | Must match live handshake |
| `connection_id` | Current transport connection | Existing admission fence, replaced after reconnect |
| `project_id` | Admitted project/root identity | Confirmed canonical identity, existing project rules |
| `route_identity` | Final SSH host/account or exact WSL distro/user | Verified transport/guest identity, not a profile label |
| `session_id` | Host-account session catalog ID | Non-nil UUID, never a bearer credential |
| `run_id` | Exact native execution attempt | Stable across detach; new for a new process |
| `attachment_id` | Current writable client binding | Runtime-issued non-nil UUID |
| `attachment_generation` | Monotonic writer epoch within live session | Exact match, checked at native write/resize |
| `request_id` | One API operation's correlation/dedup key | Bounded, unique within runtime/request scope |

Reuse the current ManagedFence for connection admission. New attachment identity
is additional; do not overload `connection_id` with native run lifetime. Reconnect
reopens the project, validates host/account/runtime/run, then requests attachment.
Project/root replacement cannot inherit the old admission. Git common-directory,
non-Git root and Coordinator rules remain those of the existing task engine.

IDs are not secrets and grant no rights. Check OS-user IPC peers or authenticated
SSH account, project admission, operation authority and exact run before catalog
access or mutation. Apply current Windows pipe and Unix private-directory rules.
List results contain sanitized metadata only; omit raw argv/environment/credentials.
An exact attachment tuple reattaches only from the owning connection or when no
writer is attached; a different live connection receives `writer_conflict` and must
use explicit takeover.

Account catalog discovery uses a new explicit account-authenticated operation with
an account fence; it does not fabricate a project ID to satisfy project-required
validation. Existing project list/read/control APIs remain project-admitted. S0
freezes the account-catalog/change-watch messages separately from task operations.

List and watch cursors bind the boot, revision and normalized filter values.
Watch applies the filter before selecting a page, and removals carry the prior
sanitized row identity so a reader can discard the correct session.

## Proposed operations

| Operation | Request essentials | Response / effect |
| --- | --- | --- |
| `PersistentLaunch` | Fence, request ID, session UUID, original validated native launch specification | Creates one retained run; returns run and writer attachment |
| `PersistentList` | Fence, admitted project filter, bounded cursor/limit | Sanitized live/ended records and real pagination cursor |
| `PersistentInspect` | Fence, session/run IDs | Authoritative process, attachment and attention observations with age |
| `PersistentAttach` | Fence, session/run IDs, viewport, client snapshot support | Acquires writer if free; returns attachment and initial snapshot handle |
| `PersistentTakeover` | Attach fields plus exact expected current writer generation | Atomically fences previous writer and acquires a new attachment |
| `PersistentDetach` | Fence, session/run IDs, attachment tuple | Revokes that attachment; leaves native process and run binding alive |
| `PersistentInput` | Full attachment fence, sequence, bounded bytes | Validates once and queues bounded native input; returns disposition |
| `PersistentResize` | Full attachment fence, resize sequence, columns/rows | Ordered resize acknowledgement and terminal revision |
| `PersistentSnapshot` | Attachment fence, snapshot ID/page cursor or new snapshot request | Immutable bounded snapshot pages and cut offset |
| `PersistentRead` | Attachment fence, snapshot ID, output offset, limit | Ordered output tail or explicit gap/resnapshot result |
| `PersistentStop` | Fence, session/run IDs, authorized operator identity, request ID | Idempotent stop request; later status proves exit or reports uncertainty |

Guarded launch reserves durable run identity before any native spawn. Once a
reserved run starts or a writer is acquired, the reply returns the native
writer tuple. If SQLite is temporarily unwritable, the registry retains the latest
unpublished generation for that run and the existing maintenance pass retries
the metadata projection. The catalog cannot roll back a newer writer, repeated
projection does not reorder MRU, and a stale catalog observation grants no
authority.

Only one writable attachment exists. Multiple GUI views can display metadata but
are not independent terminal observers in v1. Takeover is explicit and compare-
and-swap guarded; two competing requests cannot both win. A delayed detach for
the previous attachment cannot detach the new writer. Stop is available to the
authenticated same-account/project operator independently of a writer lease; an
arbitrary Agent MCP caller gains no new ability to stop peers. The CLI attach path
is an explicit authenticated SSH/WSL client, not a new public listener. A writable
CLI attachment has the same
lease/generation rules as a GUI; an ANSI projection does not erase the native
terminal/shell-block state needed by a later Warpai attachment.

## Ordering, retries and uncertain delivery

- Launch is idempotent for the same request/session identity and identical launch
  intent in the same runtime. Store a bounded in-memory intent digest, not a new
  durable copy of sensitive argv. A mismatch returns conflict. An uncertain launch
  is inspected by session/request identity, never retried with a fresh session ID.
- Each attachment has independent monotonically increasing input sequence. Same
  sequence with identical bytes returns its known disposition without enqueueing
  twice; different bytes or skipped/reversed sequence returns conflict. Sequence
  overflow invalidates the attachment rather than wrapping.
- Track `accepted`, `written`, `failed` and `unknown` distinctly. Queue acceptance
  is not native write completion, Agent acknowledgement or task execution. A
  partial native write is an unknown/failed delivery with a byte count when known;
  do not resend the remainder automatically after reconnect.
- A disconnect cancels queued human input not yet written and records its known
  disposition. Serialize generation invalidation with the writer: bytes already
  written cannot be recalled; not-yet-written bytes from the old writer cannot
  cross the takeover boundary. Record any in-flight uncertainty truthfully.
- Peer wakes keep existing task/message dedup and readiness checks. Do not turn
  the terminal input receipt into a task acknowledgement. Do not replay queued
  human drafts through the peer-delivery mechanism.
- Resize has its own monotonic sequence and is ordered against output/model
  revision. Repeat requests with the same dimensions are idempotent; old writers
  and conflicting same-sequence dimensions are rejected.
  Guarded resize uses `TerminalControl.resize_sequence` (tag 14) and echoes
  `TerminalState.accepted_resize_sequence` (tag 21). It starts at one for
  each new writer. The v1 CLI attaches first and sets the viewport using the
  first resize for that writer.
- Stop/detach retries are scoped to the exact old run/attachment. An old stop
  receipt never authorizes stopping a newly launched run with a similar label.

## Snapshot schema and replay

Snapshot header requires schema/parser revision, snapshot UUID, session/run IDs,
runtime boot ID, dimensions, terminal revision, byte cut offset, total size,
content digest and trimming flags. Content must represent active/alternate screen,
cursor/modes/attributes, bounded scrollback and necessary Warpai shell/block state.
S2 records exact native model fields and encoding; schema 1 is not implemented
until native import/export fixtures prove the representation complete.

Create snapshots under a terminal ordering barrier. Retain an immutable paged
image while parsing live output; avoid holding a parser mutex across network IO.
The tail starts exactly after the snapshot cut. Import must preserve incomplete
decoder/escape state or prove the cut is parser-safe. Validate schema, total bytes,
page index, digest, dimensions and offset arithmetic before allocation/import.
Do not execute imported terminal side effects during replay.

Proposed bounds: 32 KiB raw page payload, 2 MiB maximum serialized snapshot,
one on-demand immutable snapshot per session, 30-second snapshot-handle expiration, at most
64 KiB pending human input per session and existing 1 MiB managed message ceiling.
Reject oversize requests before copying bytes. The task panel's 512 KiB result
boundary is separate and must not be increased for terminal transport. S8 may
adjust proposed limits with measurements and matching tests/document updates.

A stale/expired snapshot handle returns a structured resnapshot requirement.
Output older than the bounded ring returns `output_gap` with the available start
and current end; the client obtains a newer snapshot. Do not claim that discarded
history is complete or concatenate a gap as contiguous terminal bytes.

## Failure vocabulary and state projection

Use existing managed error codes where semantically correct. S0 maps any required
new semantic errors to additive wire codes without changing legacy meanings:
`unsupported_capability`, `incompatible_snapshot`, `stale_runtime`,
`stale_attachment`, `writer_conflict`, `project_identity_changed`,
`input_conflict`, `delivery_unknown`, `output_gap`, `snapshot_expired`,
`capacity_exceeded`, `stop_unconfirmed`, `session_not_found`.

Session status includes independently: native process state and observation time,
transport/attachment state, stop request/result, bounded output availability,
permission/input attention and evidence freshness. Unknown does not collapse into
exited. Eligible catalog cleanup cannot remove live runs or unresolved execution
evidence, delete task history or resolve outstanding supervision. No new CLI
forget/remove operation is required.

## Storage and downgrade

Persistent metadata records actual guard policy and session kind, but contains
no launch command, environment, transcript or bearer
token. Use new namespaced catalog tables; preserve existing task schema semantics.
An older runtime must fail safely on an unsupported catalog version rather than
overwrite it. Metadata migration occurs under exclusive host ownership, with a
validated backup/recovery strategy defined in S0. Do not allow two app versions
to start competing Brokers for the same project Store. Native local Store ownership is unchanged. No default auto-resume
launch plan is written, so downgrade cannot accidentally restart an old command.

## Multi-session/task and bounded discovery contract

Catalog queries are scoped to admitted project and execution account, defaulting
to 16 records with a maximum of 32 per page. Return stable cursor/revision and
whole-filter totals independent of page length; listing acquires no writer,
performs no resize and creates no terminal snapshots. Reconnect does not attach
all sessions automatically. Task/supervision data use existing paged APIs, with
existing authorization, acknowledgement and revision rules.

One account guardian across Agent types/clients manages multiple independent sessions; one Agent still owns one
tracked execution slot and serial recipient queue. Session IDs are not task IDs.
No new task scheduler, auto-worker spawn or implicit task reassignment is added.
The host routes dirty-screen/wake notifications by session/run and uses fair
ready queues; a blocked input queue cannot hold a global service lock.

Proposed shared limits: existing 32 sessions, lazy 256 KiB output ring/session,
64 MiB account terminal-data ceiling including parsers/history/queues/snapshots,
and at most two snapshot serializers per account. Concurrency slots are reserved
before allocation, released on every success/failure/cancel path, and observable
through bounded nonsecret diagnostics. A proposed 64 KiB per-session output turn
prevents flood starvation. S8 measures and freezes limits with native receipts.

Bare stop immediately stops all guardian-managed runs in the
current execution account, with no selector/confirmation/`--yes`. Capture exact
IDs/run/revisions before bounded dispatch and return individual native receipts;
never use a process-name wildcard or include sessions created afterward. An explicit
project filter narrows the set; implicit focus does not. Retried calls never address
replacement runs. Turning the guardian preference off does not imply stop-many. Ended output expires after a proposed
5-minute TTL and may be evicted earlier under pressure; task results/unknown
attempts keep existing durable retention. API status reports history expiry.

CLI names are resolved to exact session/run IDs before API mutation. Environment
current-session hints and user labels are untrusted selectors, not capabilities.
No task-title match can implicitly stop multiple Agent runs. Explicit bulk stop
captures an exact target set and per-run receipts. List/resume do not launch a
replacement process when runtime identity has changed. Automatic previous-view
reattach uses normal lease acquisition and refuses writer conflicts.

## MRU, batch stop and minimal public API exposure

Store `last_active_sequence` and creation sequence as nonsecret catalog metadata.
Assign an account-global monotonic sequence on successful foreground attach or
explicit foreground selection; do not change it from list/status/read/wait,
background refresh or detach. Bare resume validates current context,
then a unique live run, then the highest MRU live run; without activity history,
use newest creation/stable-ID tie-break. Exclude exited/interrupted/unknown records.
A writer conflict never selects another run silently or steals the writer.

Bare stop snapshots all guarded runs owned by the authenticated execution account,
not just current project/Agent/client. Use explicit account-operator authorization,
existing native ownership checks and per-run receipts; Agent MCP gets no new
bulk-stop rights. `--project` is an explicit restriction. Native task lifecycle
remains separate, empty stop succeeds, and partial/unknown outcomes return failure.

The daemon retains up to 1,024 admitted stop receipts per boot. An identical
request ID and intent replay the captured receipts. A different intent under an
existing ID conflicts. At capacity, new IDs are refused while existing receipts
remain readable. Requested-only receipts report an unconfirmed batch.

S0 freezes only activity sequencing, necessary account catalog/status projection,
normal terminal attach/detach and daemon-owned batch-stop additions beyond the
existing internal transport. List returns bounded cached availability/session
facts without writer/resize authority or starting a missing daemon. Internal
snapshot/tail and Broker notifications remain core runtime implementation, not
reasons to expose extra tools. Core help/version are static installed metadata.

R4 withdraws standalone read projection/reader handles, state-waiter APIs,
rename/remove transactions solely for CLI, completion scripts and public daemon
administration. Existing GUI/native task/retention behaviors remain intact; do not
remove existing project APIs. Any future public operation needs a documented gap,
reuse/cost analysis and acceptance before the contract expands.

Account-wide stop is a daemon-owned admitted batch, not client-side iteration.
Persist only existing necessary execution evidence and keep bounded batch receipt
metadata; request-socket loss or stopping the caller's PTY does not cancel other
targets. Same operation ID deduplicates the exact captured target set. Status can
reconcile per-run results after a lost reply. Runtime crash preserves uncertainty.
