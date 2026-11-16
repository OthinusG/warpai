# Superagent source review and proposed Warpai 1.9.0 scope

Date: 2026-11-08. Status: research reference; current execution planning is in
[PRODUCT](1.9.0/PRODUCT.md), [TECH](1.9.0/TECH.md), [TODO](1.9.0/TODO.md) and the
[first-round Agent prompt](1.9.0/AGENT-EXECUTION-1.9.0.md). The research pass did not implement application changes.

## Decision

Make 1.9.0 a reliable supervised-work delivery milestone. Build on the existing
Agent Bus, tasks, questions, immutable context, handoffs and native terminals.
Prioritize actionable supervision, continuous follow-up handling and recovery
over additional agent types or a second orchestration engine.

Owner scope correction (2026-11-08): external event intake, including GitHub and
Feishu listeners, is excluded from 1.9.0 consideration. Source-adapter findings
below are reference evidence only, not planned integrations.

The owner also excluded danger-full-access remediation and requested the adapted
solution be recorded as tasks with detailed execution instructions. Preserve
existing access-mode behavior; task/project/run identity and credential safeguards
remain separate correctness requirements. The linked planning package records
the current solution; the historical candidate wording below is research context.

The original 1.9.0 target list could not be located in this checkout, its remote
branch names, GitHub milestones or matching pull requests. The owner confirmed
that the list is unavailable and requested further research. Consequently, this
document reconstructs a candidate scope; it does not claim to recover past
decisions or prove that an existing target was omitted.

## Evidence and method

Follow-up: [independent Agent process mapping review](1.9.0/PROCESS-REVIEW.md)
adds lifecycle, background-job, startup, fairness and failure-domain cases omitted
or insufficiently specified in the first pass. Its R2 gates are tracked in TODO;
source presence is not equivalent to runtime acceptance.

- Warpai baseline: `a70280b4c`, with a pre-existing local `MEMORY.md` change preserved.
- Official reference: [deepseek-ai/dsh-superagent-prototype](https://github.com/deepseek-ai/dsh-superagent-prototype/tree/8c32a9dc5d72c033816c6a8110b37f40cf94fba4), pinned at `8c32a9dc5d72c033816c6a8110b37f40cf94fba4`.
- Read the README and numbered documentation, then traced intake, admission,
  evaluation, ticket creation, assignment, delivery, reporting, retry/restart,
  channel replies, prompt assembly, tool restrictions and overview projection.
- Inspected implementation in `src/core/{engine,model,intake,evals,tickets,assignments,replies,stage}.ts`,
  `src/policy.ts`, `src/executor/dsh.ts`, `src/executor/background.ts`,
  `src/app/{runtime,effects,outbox,delivery,assessment,sessions,prompts}.ts`,
  `src/app/views/overview.ts` and `src/sources/github-channel.ts`.
- Read selected test bodies for delivery persistence, paused/outgoing messages,
  concurrent preparation, stop-versus-delivery races and slot reservation; also
  inspected core/policy test coverage. Test presence is not a test-pass claim.
- Compared with Warpai's current implementation and active contracts, not its
  archived enrolled-device collaboration designs. No historical Codex sessions,
  personal credentials or user configuration were accessed.

The upstream README explicitly distinguishes this public research prototype
from its internal deployment and design article. No article-only capability is
treated here as implemented. Neither runtime was started for this review.

## What the reference actually implements

| Mechanism | Source evidence | Consequence for Warpai |
| --- | --- | --- |
| Explicit authority before execution | [`settingsPolicy`, `assessmentOutcome`](https://github.com/deepseek-ai/dsh-superagent-prototype/blob/8c32a9dc5d72c033816c6a8110b37f40cf94fba4/src/policy.ts); `Evals.decide`, `Tickets.create` | An incoming event, model interpretation and authorized task are different facts. Preserve this distinction if adding automation. |
| Per-thread batching and duplicate prevention | [`Intake.receive`](https://github.com/deepseek-ai/dsh-superagent-prototype/blob/8c32a9dc5d72c033816c6a8110b37f40cf94fba4/src/core/intake.ts); `Evals.start`, `accepted` | Repeated event delivery must not create duplicate work; a later message must remain eligible while an earlier batch is evaluated. |
| Separate work and execution identities | `Ticket`, `Assignment`, `Tickets.retry`, `Assignments.interrupted` | Session idle, an execution attempt ending and the user's objective being delivered must remain distinct. |
| Durable effects with generation fencing | [`Effects.enqueue`, `attempt`](https://github.com/deepseek-ai/dsh-superagent-prototype/blob/8c32a9dc5d72c033816c6a8110b37f40cf94fba4/src/app/effects.ts) | Commit domain changes and pending effects together; a slow old attempt cannot remove a replacement effect. This is not a guarantee that arbitrary external side effects execute exactly once. |
| Follow-up ownership and hand-back | `followupOwners`, `Tickets.attachable`, `Evals.handback`, `Tickets.letGo` | A follow-up already owned by a worker is not simultaneously a new router task. Out-of-scope work returns for a new decision. |
| Delivery distinct from model consumption | [`AgentDelivery.send`, `holds`](https://github.com/deepseek-ai/dsh-superagent-prototype/blob/8c32a9dc5d72c033816c6a8110b37f40cf94fba4/src/app/delivery.ts); `Assignments.read`, `Tickets.readBefore` | In-process Harness history supports evidence unavailable to a generic external TUI. Warpai must not infer equivalent consumption from a PTY paste. |
| Work slots include startup and background work | [`WorkerSlots`, `DshExecutor.workerIdle`](https://github.com/deepseek-ai/dsh-superagent-prototype/blob/8c32a9dc5d72c033816c6a8110b37f40cf94fba4/src/executor/dsh.ts) | A preparing worker or background operation may still consume capacity after the foreground turn ends. Human follow-ups receive preferential capacity. |
| Structured human attention | `Assignments.report`, `Replies.answer`, `ticketStage`, `overview` | A question has identity and an explicit clear/replace lifecycle. Ordinary progress is not a human decision request. |
| Durable outbound replies | [`Outbox`](https://github.com/deepseek-ai/dsh-superagent-prototype/blob/8c32a9dc5d72c033816c6a8110b37f40cf94fba4/src/app/outbox.ts); [`sendGithub`](https://github.com/deepseek-ai/dsh-superagent-prototype/blob/8c32a9dc5d72c033816c6a8110b37f40cf94fba4/src/sources/github-channel.ts) | Retry a known reply using its stable identity. GitHub uses an account-scoped hidden marker; Feishu uses an idempotency key. Destination loss is recorded separately from success. |
| Versioned instructions | `Prompts`, `Versions`, `Sessions.promptFor` | A behavior change should be explainable by the configuration/instructions actually used. A new settings UI is not required to obtain that property. |

The router is a Harness model session, not a deterministic scheduler. Its prompt
discourages technical implementation planning, but it still has shell and read
tools. Worktree creation is an operator-provided prepare command, not a universal
automatic isolation guarantee. The host is designed around one configured GitHub
repository and optional Feishu; it is not Warpai's heterogeneous CLI fabric.

## Existing Warpai capabilities to retain and reuse

| Existing capability | Current evidence | Planning treatment |
| --- | --- | --- |
| Durable task queue, dependencies, eligible pool, revisions and attempts | [`Operation`, `Task`, `Attempt`](../crates/agent_bus/src/lib.rs); [`Store`](../crates/agent_bus/src/storage.rs) | Existing foundation; improve end-to-end behavior, not a new ticket database. |
| Structured questions and correlated replies | `AgentAsk`, `AgentAnswer`, `AgentReply`, `Store.questions`; [`MCP guidance`](../crates/agent_bus/src/mcp.rs) | Reuse question IDs and atomic result/acknowledgement rules. |
| Supervision facts | `Store.task_supervision`: waiting for permission/input/answer, deadlines, disconnection, uncertainty | Derive a clearer presentation from these facts before adding persisted status fields. |
| Explicit recovery safeguards | `Store.recover_in_transaction`, `TaskRetry`, `TaskReassign`, exact-run attempt checks | Preserve unknown execution and guarded recovery. A restart cannot itself prove old processes stopped. |
| Review and separate integration selection | `TaskReview`; [`integrate_task`](../crates/agent_bus/src/storage_orchestration.rs) | Accepted work and selected integration already differ; integration selection explicitly does not execute Git. |
| Immutable handoff and reviewed project memory | [Handoff contract](agent-communication-v2/HANDOFF.md); [`context`](../crates/agent_bus/src/context.rs), [`project memory`](../crates/agent_bus/src/project_memory.rs) | Existing foundation; strengthen continuation tests and stale-source UX instead of building another memory service. |
| Local, SSH and WSL collaboration and native worktrees | [Active plan](agent-communication-v2/PLAN.md); [Worktree contract](agent-communication-v2/WORKTREE-COLLABORATION.md) | Keep existing authority/transport boundaries; no cross-host federation expansion. |

These findings establish implementation presence. They do not establish that
every current vendor, platform and interactive scenario has passed live tests.

## Proposed 1.9.0 targets

### P0: one actionable supervision view

Make the existing collaboration panel answer: what needs a decision, what needs
review, what cannot continue, and what requires recovery? Prefer a small existing
panel section/filter over a separate dashboard.

- Derive actions from existing question, task, review and observation facts.
- A task may need both an answer and review; show one task with its pending
  actions, without hiding facts behind a single replacement state enum.
- Distinguish agent permission prompts, task questions, delivery trouble,
  dependency waits and execution uncertainty. Do not convert silence to failure.
- Counts, permissions and visible task membership must use the same projection.
- Keep routine progress in the task timeline; notify for a new actionable change,
  not each poll. Never suppress a newly necessary recovery decision as noise.

Acceptance: multiple simultaneous questions, new input during review, offline
workers and stale observations remain actionable without duplicate or misleading
counts; native English, simplified Chinese and traditional Chinese captures pass.

### P0: continuous supervision and exact follow-up ownership

Treat dynamic supervision as a durable workflow to verify, rather than adding a
second orchestration role. Retain Worktree Coordinator and Project mesh semantics.

- Dispatch new tasks while previous tasks are outstanding; handle results across
  the entire supervision set, not only the most recently contacted worker.
- Keep follow-ups pinned to the intended task/revision/attempt or message thread.
  Reuse explicit task-linked messages; do not attach by topic similarity.
- For an out-of-scope request, preserve the original task and create a separately
  authorized task or return a concrete clarification. Context does not widen authority.
- Yielding a CLI turn does not retire supervision; blocked and uncertain work
  remains outstanding. Submitted work remains pending until its review is handled.
- Keep native vendor behavior: a generic PTY session cannot promise Harness-style
  next-step steering. Queue safely and report the strongest actual delivery fact.

Acceptance: A and B work concurrently; B asks a question while A submits; another
task C can be assigned; the supervisor handles all three without new user nudges,
lost questions or treating a result as a fresh instruction.

### P0: recovery and delivery correctness

Make restart, disconnect and cancellation behavior an explicit release gate.
Retain existing idempotency, task versions, run bindings and receipts.

- Distinguish queued, notified, explicitly consumed/started, submitted and reviewed
  facts. Only expose consumption when supported by a real protocol acknowledgement.
- On reconnect, reconcile retained task and attempt facts before permitting a
  new owner. Do not automatically replay uncertain shell, file or remote writes.
- Late callbacks and replies from a replaced revision, run, checkout or remote
  generation cannot advance new work.
- A cancellation request is not confirmed termination. Reassignment requires the
  existing certainty checks or an explicit audited uncertainty override.
- Preserve original request identity on safe retry. Do not add a generic retry
  daemon or promise exactly-once external execution.

Acceptance: fault tests at dispatch, notification, start, submit, review and
cancel boundaries preserve the original work and reject stale mutations. Cover
local, SSH and WSL through their existing workflows.

### P0: delivery evidence and integration remain separate

Represent submitted, accepted and integration-selected facts honestly using the
existing task and integration records. Do not equate a final message, created PR,
accepted task or integration selection with verified mainline delivery.

Coding work should carry the relevant base/head/commit and checks; non-coding work
should carry its actual deliverable and validation without requiring Git. A new
revision must not silently reuse approval of old content. UI work requires real
rendered evidence, with a comparable baseline when evaluating a visual change.

Acceptance: rejection returns work for revision; delayed old evidence cannot
authorize a new revision; integration selection alone never reports a Git merge.
Combined verification occurs before the supervisor declares overall completion.

### P1: cheap continuation with existing context and handoffs

Use compact bootstrap indexes, exact context pins and lazy detail reads. Carry
decisions, unresolved issues and next steps through current handoff records;
check current source status and review separately.

Do not ingest full terminal transcripts or add a vector database. The upstream
`src/core/memory.ts` is a test repository implementation, not semantic memory.
Warpai already has reviewed project memory and immutable context resources.

Acceptance: a replacement worker can locate its inputs and stopping point;
changed, withdrawn, inaccessible and unsupported sources are distinct, and no
handoff transfers execution authority. Measure bootstrap/detail payload size on
representative task sets before adding further context machinery.

### P1, only if the candidate scope needs automatic dispatch: bounded capacity

Keep per-agent serial execution. If 1.9.0 adds automatic admission or launching,
define a project dispatch limit, startup reservations, bounded queues and
priority for human control/answers. Stop/cancel delivery must not require a free
worker slot. Never interrupt existing work merely to satisfy queue latency.

Do not claim a strict CPU/process/token cap from agent status: unrelated vendor
jobs and subagents are not all observable. Do not add a billing subsystem.

Acceptance: a saturated queue cannot block answers or cancellation; concurrent
starts cannot overbook the admitted capacity. Without automatic dispatch, retain
the current model and postpone this feature.

## Designs to modify or exclude

1. Do not replace Worktree Coordinator/Project mesh with a permanent sole router.
   Reference role separation is useful, but its topology differs from Warpai's.
2. Leave danger-full-access outside remediation scope, per the owner's decision.
   Do not introduce process-wide PATH replacement or Git credential rewriting.
   Keep vendor-owned authentication and existing access-mode behavior.
3. Do not call worktrees a security sandbox. Upstream documentation explicitly
   says command shims and directories are defaults, and absolute paths bypass them.
4. Do not copy automatic resume from any human thread comment: upstream permits
   that within an old mandate. Warpai must keep current project/run authority and
   distinguish context, continuation and new scope.
5. Do not adopt prompt-only auto-merge as an enforced gate. Upstream merging is
   governed by `prompts/team/delivery.md`, not host merge logic. Its default also
   rewrites branch history to credit reviewers; that is deployment-specific and
   inappropriate as Warpai's general default. Keep explicit integration behavior.
6. Do not copy unbounded retention: upstream architecture documents that tickets,
   triggers, decisions, outbox receipts and configuration history have no cleanup.
   Preserve Warpai's capacity, protected references and existing purge guards.
7. Do not prioritize more dashboards, connector breadth, autonomous self-improvement,
   cross-host federation, a new model runtime or blanket prompt customization.
   None is necessary to close the supervised-work loop above.

## Execution order and release gate

1. Use the linked 1.9.0 planning package as the current execution direction when
   explicitly invoked. Align any necessary API changes before implementation.
   Keep this review as provenance, not the execution contract.
2. Add representative lifecycle/fault scenarios to the existing Agent Bus fixtures;
   distinguish already-covered behavior from actual implementation gaps.
3. Implement only uncovered orchestration/projection behavior. Reuse existing
   operations, storage, panel controls, transport and localization.
4. For UI changes, establish native static fixtures and screenshot acceptance
   before live wiring; preserve terminal core guardrails.
5. Run source-matched CI on desktop targets and Companion paths, followed by the
   local/SSH/WSL workflow. Fixture passes and real vendor acceptance are separate.

The release must demonstrate a full dispatch → execution → question/answer →
submission → review → authorized integration/deliverable cycle, plus restart,
stale-message and disconnect cases. No numerical reliability, latency or cost
claim should be published without measured evidence.

## Verification limits

This is a source-and-contract review. No application code, dependencies, user
configuration, installed apps, external messages or release state were changed.
Upstream test bodies were inspected; upstream tests, Warpai builds and live vendor
sessions were not run. The original proposal was a research recommendation;
subsequent owner scope decisions and the adapted solution are recorded in the
linked planning package. Planning completion does not imply runtime acceptance.
