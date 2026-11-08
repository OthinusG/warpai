# Second review: mapping Superagent workers to independent Warpai Agent processes

Date: 2026-11-08. Status: supplemental source review and required gap audit;
not a claim that every listed failure exists, and not authorization for a new launcher.

Execution procedure: [second-round Agent prompt](../../AGENT-EXECUTION-1.9.0-R2.md).
Per owner direction, assume the first round is provisionally complete and apply
only this review's incremental audit/fixes and necessary dependent regressions.

## Correction to the first review

The first review emphasized task coordination and truthful delivery. It did not
sufficiently specify the execution lifecycle when a Harness worker Session is
replaced by an independently running CLI process. This review adds process/run,
conversation, background-resource, startup and fairness cases to the existing
[ledger](TODO.md). Existing correct behavior should close with evidence, not be rewritten.

External event intake and danger-full-access remediation remain excluded.
No automatic launching, project pause switch, central model routing or new daemon
is introduced by this review. Only demonstrated gaps in current behavior justify
implementation; new product controls need a separate scope decision.

## Correct mapping

| Superagent concept | Warpai equivalent | What cannot be collapsed |
| --- | --- | --- |
| Ticket | Durable task with scope, revision, evidence and review | A PID or terminal tab is not the task |
| Assignment | Execution attempt bound to exact native Agent run and checkout | Retrying an attempt does not by itself authorize a new process or transfer role |
| Worker Session | Independent CLI Agent execution environment, including vendor conversation when available | Process identity and conversational continuity are different facts |
| Worker turn | One period of work/input handling within that process | A persistent process can be idle between turns and alive while a task awaits review |
| Session directory | Confirmed native checkout or non-Git project root | Separate processes can still write the same directory |
| Session jobs/subagents | Owned OS children and explicitly tracked delegated tasks | Internal vendor subagents may be unobservable and are not automatically registered peer tasks |
| Main Session | Selected Worktree Coordinator or a Project-mode supervising peer | Do not introduce a permanent sole Router |
| Session inbox | Durable MCP inbox plus guarded native wake | Native wake is neither next-step steering nor proof of consumption |

Bind the task attempt using existing native run/birth-time/root evidence, not
just a PID, display name or tab. Read a vendor conversation identifier only if
an existing supported mechanism exposes it without transcript/credential access.
Never invent such an identifier or add one merely for symmetry with Harness.

## R2-01: task context versus reusable process context

Upstream `workerSessionId(ticketId)` uses `worker-<ticketId>` and the same Ticket
keeps its directory and Session across Assignments. This gives a task a naturally
scoped conversational environment. Warpai serializes TaskStart by assignee, but
that is not proof that a reused CLI conversation has forgotten an earlier task.

Inspect `TaskStart`, `capture_thread_binding`, bootstrap and task input delivery.
Use the explicit current task, acceptance, exact context pins and stopping point
at each start. Previous conversation content cannot change the new task's scope.
A PID restart may lose vendor conversation state while a surviving PID may keep
unrelated context; either case must be handled honestly through normal bootstrap.

Do not require a fresh process for every task, wipe conversations, inject vendor
reset commands or silently resume an old vendor session. Those change current
user workflow and require supported vendor contracts and explicit product scope.

Acceptance SP01: run task X, then an unrelated Y through the same participating
process; only Y's explicit scope/inputs authorize Y. Replace the process and
reconcile its outstanding attempt; bootstrap can recover retained task inputs
without claiming recovery of unobserved vendor conversation state.

## R2-02: task lifetime, process lifetime and background resources

Upstream `workerIdle`, `handOff` and `background.ts` distinguish a foreground turn
from jobs/subagents that continue afterward. At human handoff it may leave a
preview server alive, release the worker slot and collect its end without waking
new implementation. Ending an Assignment uses a different cleanup path.

Warpai already exposes `NativePty::exit_code`, `is_active` and `stop`; owned Unix
process groups and Windows Job Objects cover children in those native ownership
paths. The test `owned_group_can_stop_background_children_after_observing_leader_exit`
already checks a leader that exited before its child. This does not establish
task-level ownership of every job spawned inside an ordinary user CLI terminal.

Audit each actual ownership path separately: Companion-owned PTY, desktop native
terminal attachment, direct WSL and vendor-internal children. Do not generalize
one backend's kill authority to an attached terminal or a daemon that detached.
Process exit, terminal EOF, prompt idle, task submission and child termination
remain independent facts. Submission must not automatically kill a user-owned
Agent, test server or unrelated child. Cancellation must not claim more than the
observed owner/attempt outcome.

Acceptance SP02: owned leader exits with child running; retained ownership stays
truthful. SP03: a task reports ready with a retained preview process; review and
later unrelated work do not trigger an unauthorized kill or stale automatic wake.
Unknown task-to-job ownership stays unknown; do not add a job registry to hide it.

## R2-03: startup and workspace preparation are not a single success bit

Upstream `prepareDirectory`, `launch`, `Sessions.ensure` and `assignmentStarted`
cover workspace preparation, Session creation and first delivery. It deliberately
does not auto-retry a failed prepare command, because leftover files can otherwise
be mistaken for a prepared directory; its `launch` currently treats existence as
prepared. This is a reason to improve recovery design, not copy its shortcut.

For Warpai's existing launch/registration paths distinguish: confirmed project
root, actual child run, communication binding/handshake, safe native readiness
and explicit TaskStart. These are observable stages, not a demand for new stored
states or an automatic process factory. Creating a worktree still launches no Agent.

Acceptance SP04: checkout exists but Agent launch/handshake failed; retry does not
overwrite the checkout, assume readiness, duplicate a live process or adopt the
wrong root. Also cover a process that exits before MCP registration and a late
registration from a replaced run. Reuse existing native fixtures and receipts.

## R2-04: backpressure and fairness also matter without automatic launching

Upstream `WorkerSlots`, effect limits and `Later` separate temporary capacity wait
from a real failure. Human follow-ups get preference. This is useful even when
Warpai's workers were all launched manually: enough queued work or notifications
can still delay control or starve another task.

Correct the earlier shorthand: absence of automatic launching justifies skipping
a new project worker-slot scheduler. It does not justify skipping saturation,
fairness and control-delivery tests of existing queues.

Reuse existing Store bounds, `CONTROL_MESSAGE_RESERVE`, per-assignee execution and
native wake fences. Test control admission and actual eventual handling separately.
A reserved queue slot alone does not prove bounded end-to-end control latency.
Preserve ordering within a request/task; do not arbitrarily reorder the inbox.

Acceptance SP05: a saturated/noisy worker cannot prevent another worker's result
or a supervisor's question from being retained and eventually handled. Cancel and
answer requests remain admissible, without unsafe Enter injection into busy or
approval prompts. Check offline recipients and replay storms. No fixed latency
claim is made without measurement.

## R2-05: failure classes, retry ownership and independent fault domains

Upstream distinguishes transient provider/network failure, failed preparation,
delivery failure, capacity wait and an actual blocked/failed report. `RetryHealth`
observes structured native retry events rather than treating assistant text as
proof of provider state; model retrying is not itself successful recovery.

Warpai must not add a second retry loop around an Agent CLI that already retries.
Do not parse a phrase such as "429" from the terminal and repeat the entire task,
switch model/account, or restart its process. When no supported structured event
exists, show the available observation and uncertainty rather than an invented
precise provider diagnosis. Account quota data is not per-task execution evidence.

Inspect health/error owners so a healthy peer cannot clear another run's error,
one broken peer cannot mark the whole project failed, and a stale result cannot
clear a replacement run's blocker. Keep all still-actionable failures visible
without transient-error notification storms.

Acceptance SP06: one Agent's disconnect/provider wait or unavailable process does
not stop healthy peers; no duplicate task retry is layered over CLI retry.
Old-run recovery events cannot clear new-run issues. Use fixtures, not paid model calls.

## R2-06: one process does not isolate a writable checkout

The prototype gives each Ticket its own directory and can prepare a worktree.
Warpai supports multiple Agents in a checkout and has advisory reservations;
per-Agent task serialization alone cannot prevent cross-process file collisions.

Preserve native worktree allocation for parallel independent code changes. When
Agents intentionally share a checkout/non-Git project, make ownership and overlapping
write coordination explicit through existing bounded task instructions and advisory
reservations. A reservation is not an OS file lock, and independent processes
are not independent Git indexes. Do not add a global workspace lock or relocate
an existing user process without scope and evidence.

Acceptance SP07: two independent workers with overlapping target paths cannot be
represented as safely isolated merely because their runs differ. Exercise the
existing reservation conflict/lifetime behavior and distinct-worktree success;
failed/uncertain work must not silently free ownership for conflicting new work.

## R2-07: supervisor responsiveness, duplicates and durable loop termination

Upstream restricts long router waits, coalesces concurrent passes and distinguishes
notice injection from waking new work. Worker reports, job completions and human
questions have different notification rules. Its communication prompt tracks
automated review rounds across restarts; blindly adopting its numeric limit would
be inappropriate, but persistent duplicate/no-progress detection is useful.

For Warpai, use bounded existing MCP wait/ready and task/inbox refresh; do not try
to control an external CLI's internal shell waits by copying Harness tool guards.
Coalesce only replaceable observations; retain distinct questions, result IDs,
revisions, source references and unresolved control requests. A stale generation
cannot retire a new pending wake or result. A result is not another instruction.

Acceptance SP08: duplicate notification/report bursts do not multiply work or
erase a new question. SP09: after supervisor context compaction/restart or
Coordinator handover, reconstruct all outstanding tasks/requests from existing
durable records and pagination; do not depend on a memory-only watched-peer set.
Record automated no-progress cycles using existing task evidence when available;
do not impose a blanket implementation-round limit on authorized human work.

## R2-08: configuration provenance and capability-aware continuation

Upstream records prompt versions used by Sessions and seeds role models only at
Session creation, preserving later explicit selections and global defaults.
The transferable idea is stable, inspectable execution context, not a centralized
Warpai model router or permission manager.

Audit current native launch/binding evidence for executable/args/cwd/run/checkout
and context pins. Preserve vendor-managed model/auth/global config. Record only
available nonsecret metadata through existing bounded descriptors; do not assert
the exact prompt or model if the CLI does not expose it. Late settings changes
must not silently mutate an already-bound attempt.

Acceptance SP10: changing focus, defaults or selected Agent cannot retarget an
existing attempt; a replacement run needs its normal admission. Unsupported
vendor capabilities produce explicit limitations, not a generic terminal-text
fallback pretending to be an acknowledgement or conversation restore.

## Optional mechanisms that require a separate product decision

- A global dispatch pause/drain UI distinct from task cancellation. Upstream has
  processing pause, but Warpai's per-run cancelled/paused delivery flags are not
  automatically equivalent. No new pause switch is authorized here.
- Automatic worker provisioning, pooling or replacement and a global slot limit.
  Process-based execution does not require these product features.
- Per-role CLI/model presets and a prompt-version editor. Preserve current vendor
  configuration ownership; no new global configuration writes.

## Source evidence and validation limits

Reference commit: `8c32a9dc5d72c033816c6a8110b37f40cf94fba4` of the official prototype.

- [Executor and slots](https://github.com/deepseek-ai/dsh-superagent-prototype/blob/8c32a9dc5d72c033816c6a8110b37f40cf94fba4/src/executor/dsh.ts)
- [Background jobs and subagents](https://github.com/deepseek-ai/dsh-superagent-prototype/blob/8c32a9dc5d72c033816c6a8110b37f40cf94fba4/src/executor/background.ts)
- [Effect queue](https://github.com/deepseek-ai/dsh-superagent-prototype/blob/8c32a9dc5d72c033816c6a8110b37f40cf94fba4/src/app/effects.ts)
- [Pass coalescing](https://github.com/deepseek-ai/dsh-superagent-prototype/blob/8c32a9dc5d72c033816c6a8110b37f40cf94fba4/src/app/passes.ts)
- [Structured retry health](https://github.com/deepseek-ai/dsh-superagent-prototype/blob/8c32a9dc5d72c033816c6a8110b37f40cf94fba4/src/app/retry-health.ts)
- [Role model initialization](https://github.com/deepseek-ai/dsh-superagent-prototype/blob/8c32a9dc5d72c033816c6a8110b37f40cf94fba4/src/app/role-models.ts)
- Warpai: [`NativePty`](../../crates/agent_bus/src/native_pty.rs),
  [Unix ownership](../../crates/agent_bus/src/native_pty_unix.rs),
  [Windows ownership](../../crates/agent_bus/src/native_pty_windows.rs),
  [owned-child regression](../../crates/agent_bus/tests/native_pty.rs),
  [per-launch binding](../../crates/agent_bus/src/session.rs),
  [task operations](../../crates/agent_bus/src/storage.rs).

This pass read implementation and selected test source. No Rust builds, process
experiments, vendor sessions or CI acceptance ran. Process backend coverage is
not universal task-job ownership. These additions are review/verification gaps;
each needs current-source classification before code is changed. Completeness
means mapped responsibilities and evidence, not a guarantee that no future
failure mode can exist.
