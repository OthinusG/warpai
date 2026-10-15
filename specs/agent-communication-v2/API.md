# SSH Agent Communication API

## Result delivery (2026-10-12)

Local, SSH and native WSL share the Operation contract. `agent_send` optionally
sets `expects_result` (default false); true queues a `request`. `agent_reply`
requires `reply_to`, `body`, `outcome` (`succeeded`, `failed`, `blocked`) and a UUID
`request_id`. The destination is the original sender, never supplied by the
caller. Only the original recipient can reply. Thread/task associations persist.
Final outcomes atomically queue the result and acknowledge the original request;
blocked leaves it outstanding. A second final outcome is rejected; identical
request retries return their saved receipt. Results have kinds `result_succeeded`,
`result_failed`, `result_blocked` and are acknowledged by their recipient after
processing. Request acknowledgement without a final result is refused.
Task submission separately notifies issuer and reviewer when different. Project
issuers collect their own delegated results; Worktree orchestration remains
Coordinator-only. Busy queues preserve arrival order across all three transports.
Native notification submission is not handling: wake selection skips previously
submitted messages, while inbox/wait retains unacknowledged history for recovery.

## Native state and connection separation (2026-10-15)

Local, native WSL and Companion-owned SSH PTYs share screen classification.
Herdr Blocked maps to existing WaitingApproval/WaitingInput activities. It retains
registration, account/run authority and MCP inbox access; only automatic native
input is gated. Queued messages retain their IDs, task associations and ordering.
Returning to Idle permits guarded delivery; neither transition acknowledges a
message, completes a task, retries an attempt or grants permission.

Native screen captures remain memory-only. Changed evidence is classified off
the desktop UI executor; stable input/output/run stamps reuse prior observations.
Empty queues may be reused against the Broker mutation revision. Pending work and
forwarded Stores retain settling, lease, cancellation and exact-run validation.
No operation fields or protocol majors change.

## Native WSL terminal delivery

Windows and the opt-in Linux WSL service support `GuestNative` on the existing
fenced ProjectTasks channel; default SSH services reject it. `NativeRequest`
contains a confirmed Linux shell PID and observe/claim/validate/finish action.
The service matches canonical checkout, shell/Agent PID start times and exact
run. `NativeState` contains run/program and a serialized Broker wake claim.
Observe carries only edit epochs, draft/approval flags and optional native
readiness. Native readiness false plus a blocked flag projects WaitingApproval;
false without a blocker projects Working. Blocker changes update lifecycle even
when readiness is unchanged, retaining the original MCP run and pending work.
Services advertising `native_screen_activity` additionally accept optional
`screen_run` and `screen_activity`. The exact run must match before any
input/guard mutation; no screen text is transferred. Since Companion 5.3.0, a
stale Observe can return current identity/activity with valid=false and no wake,
without applying old evidence or edit epochs. Claim/Validate/Finish with a stale
terminal or run still fail with ManagedStaleAttachment. A fenced observation returns
optional `activity` and unacknowledged `pending_work` in NativeState. Older hosts omit the fields and receive the
original response shape; newer hosts omit them when the capability is absent.
Known Agent screens follow the owner-approved Herdr policy: after the native
quiet window, no matching rule defaults to Idle. Matching approval/question
rules prevent native submission; recognized viewers retain the prior state.
Unknown custom clients retain existing lifecycle evidence. Submitted/cancelled edit epochs
veto the same sample; host edits fence response projection and wake submission. Native draft presence does not block a queued wake; the owner authorizes
clearing unsent native input at the idle prompt before task submission. Composer
drafts stay in the app. Clearing never grants readiness or bypasses an input epoch,
approval, cancellation or run fence. No draft text is added to the wire.
The Windows host tracks native draft presence locally for clearing and sends
draft=false so an already-running older guest service cannot reintroduce a draft
delivery gate. Approval and edit/cancel epochs continue to cross the wire.
Never transfer draft text or credentials. A settings selection is not authority
to access another guest account. No scoped wrapper or nested command is accepted.
Validate precedes delayed Enter; finish records submission, not acknowledgement.

## Worktree orchestration contract (2026-10-08)

`PanelQuery.worktree` selects a read projection and never changes permissions.
Private `worktree_mode { worktree, request_id }` persists the project's active
communication mode in existing Store metadata. Snapshots expose `worktree_mode`
so focus changes restore the project setting. Switching mode preserves selected
Coordinator, Agent IDs, runs, checkout bindings and task ownership. Project mode
permits project peers; Worktree mode permits same-checkout peers and Coordinator
links. Both participant projections filter offline Agents before pagination.
Worktree projection includes bounded active `candidates`, durable
`roles`, registered `worktrees` and `coordinator_online`. Candidates identify the
native Agent, exact run and physical checkout; panel selection never supplies an
arbitrary process identity or changes a process working directory.

Private controller intents `worktree_coordinator` and `worktree_worker` carry
`root`, `agent`, `run`, `request_id`. The owning local Broker or SSH Companion
validates the active run, native repository/root. Multiple Agents may share the same checkout.
Native registration uses repository project identity before any task exists.
Legacy private enrollment retains its unresolved-work safeguard and original
history; it is not the native launch or mode-switch path.
Selection does not spawn or restart a process. Any active Agent in any registered
checkout can become Coordinator; a new selection demotes the old Coordinator
without ending its run. Stale selection must fail. Coordinator state belongs to
the repository and run, independent of the panel's focused terminal. Schema v9
removes the checkout uniqueness constraint after a consistent v8 backup.
Active participants automatically join an enabled repository team. Messages and
Agent discovery are scoped to the same checkout or the selected Coordinator;
other workers in different checkouts are excluded. Project panel pagination
filters offline participants before applying its page limit.

Process exit/revocation or app shutdown clears Coordinator authority and demotes
the role; a same-name restart remains a worker until explicitly selected. A new
Coordinator receives a bounded unfinished-task handoff with paginated retrieval
instructions and subsequent task update notifications. Original assignee,
issuer and designated reviewer provenance remain unchanged.

`worktree_create` carries `root`, a bounded branch `name`, local `base` ref and
`request_id`. It creates a registered checkout through trusted private Git only;
it never starts an Agent or writes through the file protocol. Existing files or
branches are never reset or deleted. Coordinator authority is required before
workers can receive team assignments. Role changes are trusted panel operations;
ordinary Agent tools cannot nominate a Coordinator or claim another checkout.

`task_integrate` is a Coordinator-only Agent operation carrying an accepted
`task_id`, optional final integration `commit`, and `request_id`. It records the
Coordinator's integration choice; it does not execute Git. If a commit is supplied,
verify it in the Coordinator checkout. Existing task assignment, messages, evidence
and designated peer reviewer transitions remain the task/context/review engine.
Project mode does not impose these role restrictions.

> Current 1.1.0 iteration: [remote installation and terminal-driven connection](INSTALLATION.md) supersedes manual SSH alias/root/companion forms and raw companion delivery.

The active scope is PLAN S0–S5. The former manager contract is archived in
https://github.com/OthinusG/warpai/blob/47a2a5a/specs/agent-communication-v2/legacy-ssh-project-manager/API.md and is not an implementation checklist.

## Transport and authority

Use bounded, length-prefixed protobuf over a clean system SSH stdio connection.
No format auto-detection, public listener, device enrollment or federation.
Initialize verifies protocol major, private account/service/boot/connection IDs.
ProjectOpen binds a canonical native root and opaque project UUID. Every later
request/result carries that exact fence; replaced roots and late responses fail.
Removed host-status/list tags and detach/attach/generation fields are reserved.

Capabilities: project_open, managed_agent, project_tasks, project_mcp.

## Existing file tools over SSH

The 2026-10-07 self-contained-tools iteration reuses `project_git_review` and the
existing Git root/status/diff/branches/base-file operations. Explorer and Review
share a session/project attachment. Native installers resolve Git from the private
runtime named by companion-manifest.json; no system Git installation is required.
No wire schema or protocol-major change is introduced.

`project_files` admits only typed `ProjectFilesRequest` operations under the
current project fence: directory metadata, staged reads, staged saves, explicit
create/rename/delete and transfer release. Requests and results echo selection
generation. Metadata reuses the existing repository snapshot schema, including
non-Git directories. File bytes use SFTP, never a managed file-content payload.
Staged transfers are bounded and connection-owned; transfer IDs are not reusable
across attachments. Saves require the observed SHA-256 before replacement.
Root/path replacement, symlink escape and stale replies must fail before access.
An old Companion without this capability keeps existing Agent operations.

Managed file errors distinguish invalid input, missing path (`MANAGED_NOT_FOUND`,
appended as code 9), permission denial, conflict, capacity and stale attachment.
Stale/incompatible control and failed SFTP routes close the retained attachment;
they cannot silently retry against a desktop path or a newly connected host.
Service crash-transfer cleanup requires exclusive service ownership.

## Agent lifecycle and IO

TerminalLaunch carries a launch-intent UUID, absolute executable, bounded argv,
terminal dimensions and optional Agent program. A managed Agent uses the existing
session::launch adapter inside the service-owned native PTY. Fresh MCP endpoint,
capability and terminal binding are injected into its environment only; credentials
never enter control responses, arguments or preferences. The Broker run UUID equals
the owned native run. Ordinary commands receive no Agent authority.

Only the launch connection may read/input/resize/stop/release its run. Input is
bounded and sequenced; output is bounded and paged with explicit truncation.
Release requires observed full group/job exit and output EOF. Disconnect revokes
MCP and requests owned stop; no listing, detach/reattach or takeover. Failed native
activity observation retains ownership. Launch-intent replay reconciles only the
same connection and immutable payload.

## Existing communication domain

ProjectTasks carries a typed command (panel/operator/controller), query generation
and bounded JSON domain payload. The private remote Broker/Store is authoritative.
The GUI receives value/error envelopes, scoped projections and fixed failure
metadata. It cannot register Agents, claim readiness, start/submit Agent execution,
or access retired device operations. Existing messages/tasks/reviews/versions/
original request receipts remain unchanged. Changed-payload retries conflict;
unknown writes retain their original intent and are never automatically replayed.

Panel Agent rows include optional `run` from the currently observed Broker run.
It is projection metadata, separate from the persisted Agent identity, and is
absent when no active run is observed. Offline cached rows are explicitly stale.

## Explicit SSH terminal launch

`warpai-companion agent <absolute-root> <program> <absolute-vendor-executable>
[vendor-arguments...]` joins the same private service, forwards native terminal IO
and tracks resize. The companion supports mcp and the existing forward relay.
System SSH and vendor authentication remain user-managed on the remote account.

## Owned local Codex bridge invocation

`warpai-agent mcp --native-directory` captures its OS working directory and
supplies it through the existing authenticated request directory field before
MCP discovery. The flag takes no user-supplied path and does not change cwd.
The Codex per-invocation server uses this mode; other native adapters retain
`mcp`. Initial local scope follows the verified native directory, while already
registered or explicitly bound workspace identities cannot change scope.

### Existing Review over SSH

`project_git_review`, alongside `project_files`, admits typed, read-only `PROJECT_GIT_ROOT`,
`PROJECT_GIT_BRANCHES`, `PROJECT_GIT_STATUS`, `PROJECT_GIT_DIFF`, and
`PROJECT_GIT_PREPARE_BASE` actions. There is no caller-supplied command or argument
vector. `git_previous_path` admits a validated project-relative old path for
rename/copy patch selection; it is never a command argument override. Status carries porcelain-v2 records, the resolved base commit, and
name-status records for merge-base comparisons. Git metadata and patches are
bounded at 512 KiB and each subprocess has a 10-second deadline. Base file bytes
use the same connection-owned, hashed SFTP staging and release lifecycle.

Head mode includes staged, unstaged and untracked files. Branch comparisons use
local remote-host refs and merge bases; no implicit network fetch occurs. Git
write operations remain available in the remote terminal and are visibly disabled
in Review. Failed reads never fall through to desktop Git or desktop file paths.

Task list projections include a bounded 512-character `description` preview.
Task detail continues to carry the full description; mutation revisions remain
unchanged and are no longer displayed in the daily Collaboration view.

Remote Review verifies working-content hashes against the patch observation and
checks status/HEAD again before publishing. Rapid branch changes coalesce after
the bounded transfer; obsolete observations never replace the selected mode.
Remote cache sources stay referenced while Review or an editor uses them.
Explicit reconnect verifies the original account, canonical root and root identity;
it cannot replay an uncertain write or silently adopt a replaced project.
# Cross-worktree extension — 2026-10-08

Optional Companion capability `worktree_collaboration` adds private controller
operations `worktree_join` and `worktree_leave`, each with a canonical checkout
`root` and UUID `request_id`. Remote roots are derived from the current native
attachment; a payload cannot select another checkout. Agent MCP tools are unchanged.

Task commands may use `{ "kind": "scoped", "operation": { "scope": "...",
"command": { "kind": "operator", "operation": { "op": "..." } } } }`.
Only one wrapper is accepted. Team task writes require the scope returned by the
last panel projection; mismatches fail with `scope_denied`. Membership operations
pin the physical project and reconcile their original receipt in the repository
Store even after joining/leaving changes task routing. Private old clients retain
their unwrapped command contract. Desktop wraps commands only when the Companion
advertises this capability.

Panel responses keep `project` equal to the physical attachment's project ID for
existing client fence checks. Additive fields `collaboration_scope`,
`worktree_available`, `worktree_joined`, `worktree_root`, and `worktree_branch`
describe the selected team/checkout. Each participant row may carry its own
`worktree_branch` alongside existing `workspace`. Scope changes discard task
selection and pagination cursors. File requests, SFTP roots and ManagedFence are
unchanged. See [the acceptance contract](WORKTREE-COLLABORATION.md).

## Direct local WSL extension (1.6.0 / Companion 4.0.0)

The additive file-chunk fields/actions and capability are specified in
[the WSL contract](../wsl-1.6.0/TECH.md). Protocol major remains 1.
SSH clients retain SFTP; WSL requires project_file_chunks before file access.

## Peer supervision and decisions (2026-10-12)

Additive MCP operations shared by local, SSH and WSL:
`agent_ask {task_id?, reply_to?, question, request_id}` requires a task or original
instruction; `agent_answer {question_id, answer, request_id}` routes exclusively
to the asker and consumes the question atomically; `agent_questions
{task_id?, cursor?, limit?}` pages unresolved questions in arrival order.
Mutation UUID replay preserves the original response and never duplicates a
question or answer. Project participants can inspect and answer shared questions
and supervise all project tasks. Worktree participation restrictions remain.
Question/answer events preserve task-linked decision history; revision changes
retire obsolete questions. Pending decisions prevent submission and review.
Task detail adds `supervision` with observation age, pending-answer count,
deadline flags and execution uncertainty. Agent discovery adds last-observed and
last-output timestamps and whether the observation is confirmed. None of these
observations authorizes automatic re-execution. Decisions retain version/run
fences and history. Capabilities: `correlated_results`, `decision_questions`,
`project_supervision`; SSH control major remains 1 and local Agent Bus major
remains 2. Informational sends do not promise a processing result; work requests
and their correlated results have separate response instructions.

Cooperative `agent_wait` uses the same per-run notification receipt as native
wake: delivering an unacknowledged head does not hide later mail. It returns
each notification once per run; `agent_inbox` retains the full pending history.
Asking about an instruction records that instruction's notification as delivered,
so waiting for its answer does not repeatedly return the original request.

Native lifecycle and permission-guard changes append `task_observed` events to
the current task/attempt run, without storing draft or output text. Disconnect
records loss of observation, not a stopped execution. Supervision combines these
durable observations with progress and keeps unknown/interrupted attempts visible
even outside the bounded detail history. These events never grant execution.

Successful ordinary replies require all associated questions to be resolved.
Failed replies atomically retire those questions with the final result; blocked
replies retain them. Native cancellation records an observation, not proof that
the execution stopped. Attention notices wake the issuer through the existing
queue; full inboxes retain the observation in task history for shared polling.

## Shared native supervision and task delivery (2026-10-13)

Native approval/input/error/cancellation observations, and returning to an idle
prompt with an unfinished executing revision, queue an `observation` message to
the task's issuer. These are ordinary durable MCP inbox/wait messages with the
original task/revision, not UI-only notices. Busy receivers preserve arrival
order; idle delivery uses the existing fenced native wake, after which the Agent
reads `warp_agent_inbox` and `warp_task_get`, handles the decision and explicitly
acknowledges the observation. Screen observations never submit/fail/retry a task.

The sidebar projects the same native activity and unacknowledged MCP work as the
collaboration panel. Its unread indicator does not acknowledge MCP messages when
a person focuses the tab or marks UI notices read. Native-grid events are typed
separately from plugin completion events, avoiding broker feedback and legacy
cloud task reporting. Keep awake consumes the same observed activity: working
holds the guard; idle, queued work and approval waits do not. Unknown chrome
retains existing native evidence rather than manufacturing completion.

### WSL attachment recovery

The WSL configuration generation is a desktop selection fence, not account or
project authority. A successful setup verifies Initialize and ProjectOpen for the
confirmed guest account and advances that fence. Explorer, Review and collaboration
invalidate cached pre-install connection failures through their shared selection
key. Scanning does not remove a confirmed WSL project. Failed verification does
not schedule recurring reconnect attempts or repeat uncertain file mutations.
Agent discovery resolves symlinks and excludes Windows drive mappings through
native WSL path translation; imported Windows PATH entries do not authorize
Windows Agent configuration changes from the guest.

## Structured bodies and reusable context (2026-10-14)

All transports share `Operation` and `context::{Instruction,ContextRef,ContextContent}`
from `warp-agent-bus`. Native panel Send, assignment and pool creation use these
same DTOs. Existing textual input remains accepted; typed sends use
`content: {objective, context_refs}` and may omit `body`. Supplying both requires
an identical objective. Do not parse task instructions from notification prose.

`warp_context_publish` accepts `title`, typed `content`, optional `source_revision`
and a UUID `request_id`. New resources omit `resource_id`/`expected_revision`;
updates require both and retain the same kind. Content kinds:

| kind | content fields |
| --- | --- |
| repository_index | entries: [{path, symbols, summary?}] |
| analysis | summary, findings: [string] |
| code_snippet | path, language, start_line, code |
| design_decision | decision, rationale, alternatives: [string] |

Publication returns a compact summary with `reference: {id, revision, sha256}`,
author/project and broker-owned source_workspace provenance when available.
Content is limited to 32 KiB; paths must be workspace-relative. Publish explicitly
supplied snapshots only; the server does not read source files or credentials.
`warp_context_get({reference})` returns that exact immutable version and verifies
project authority/digest. `warp_context_list({cursor?,limit?})` returns summaries
and a continuation cursor; it never copies the resource payloads.
MCP resources/list and resources/read expose the same authority through
`warpai-context://<uuid>/<revision>/<sha256>`; external/file URIs are rejected.

Task assignment/pool creation accept `context_refs`, limited to 16 unique
references. Retry/reassignment retains pins; updating the resource head never
changes existing work. A refreshed snapshot requires an explicit new assignment.
Ordinary typed messages expose a body object:
`{type:"instruction",schema_version:1,instruction:{objective,context_refs}}`.
Task-linked notifications expose
`{type:"task",schema_version:1,event,task_id,revision,objective,acceptance,context_refs,
details:{instruction,state,result,evidence_ids}}`. Coordinator forwards retain
the typed task snapshot without transferring task ownership. Legacy string bodies
stay readable in inbox/history. PTY wake messages contain only notification IDs,
not full resource content. Reading context never acknowledges or starts a task.

SSH/WSL advertise `context_resources` and `structured_task_messages`; unsupported
Companions reject typed/reference-bearing sends before transmission. No fallback
copies resources into text. Panel projections carry compact resource summaries
and `context_cursor`; native selectors show titles/kinds/revisions and pin the
original selection while retrying the same request UUID. Local/remote stores
remain independent per host/account/project, including matching repository URLs.
# Native threads and project memory

Additive Agent Bus operations and native MCP discovery behavior are specified in
[PROJECT-MEMORY.md](PROJECT-MEMORY.md). `execution_threads` and `project_memory`
capabilities cover execution_thread_get, project_bootstrap and project_memory_update.
These operations share the existing authenticated project/account scope on local,
WSL and SSH; memory references confer no execution or filesystem authority.
