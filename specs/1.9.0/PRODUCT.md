# Warpai 1.9.0 supervised-work delivery

Date: 2026-11-08.
Status: planning baseline requested by the owner; implementation has not started.

## Purpose

Make existing CLI Agent collaboration reliable from dispatch through a usable,
reviewed deliverable. Reuse Agent Bus, native terminals, task attempts, structured
questions, review, immutable context and handoffs. Existing working behavior
needs evidence, not a replacement implementation.

The original historical 1.9.0 list is unavailable. This baseline records the
current conversation's solution and exclusions; it does not recover that list.
The owner requested an execution package, not immediate implementation or release.
The first-round [execution prompt](AGENT-EXECUTION-1.9.0.md) is intended for explicit
invocation by an implementation Agent.

## Scope

P0: actionable supervision in the existing panel, continuous supervision,
task-linked follow-ups, truthful delivery/recovery and evidence-based delivery.
P1: continuation using existing handoff/context and protected history retention.
Capacity work is conditional: only include a missing control if an in-scope
automatic dispatch path actually needs it. Do not introduce automatic launching
to justify capacity work.

## Behavior invariants

1. **BI-01 — Truthful delivery.** Queue admission, native notification, explicit
   task start, result submission and review are distinct facts. PTY input,
   terminal output, idle status or elapsed time never proves model consumption.
   `warp_task_start` proves a task transition, not that every related message was
   read. Explicit message acknowledgement is not proof of implementation success.
2. **BI-02 — Task-linked continuation.** An in-scope follow-up uses the existing
   task/thread association and durable inbox. Busy delivery preserves existing
   input/run fences. Review rejection uses the current revision workflow. An
   overtaken follow-up stays visible with its original text and revision and is
   ended explicitly before a current-revision replacement is sent; ending it
   changes no ownership or execution and notifies a waiting sender. Additional
   work after accepted completion uses a new authorized task with an exact source
   handoff reference. Failed/cancelled work uses existing retry rules, with
   unresolved execution reconciled first. Topic similarity grants nothing. The
   Operator can page through every unresolved follow-up of one task, including
   old entries that later work has overtaken; reading changes nothing and ending
   one entry never hides or ends the others. Reassigning a task without naming a
   reviewer keeps the reviewer it already has, including the panel Operator.
3. **BI-03 — Continuous supervision.** Outstanding task/request IDs remain tracked
   across waits, yields and focus changes. New work can be dispatched while other
   tasks run. Pending questions, submitted reviews, cancellation requests and
   execution uncertainty cannot disappear merely because the CLI turn ends. After
   a restart or handover, a supervisor can rebuild the authorized unfinished set
   from durable facts (including delegated tasks assigned elsewhere, requests
   awaiting results and received results not yet handled) through a bounded,
   paginated read; the read itself acknowledges and advances nothing.
4. **BI-04 — Questions and results.** Structured questions require the existing
   answer operation. A plain message cannot resolve them. Final processing results
   use existing atomic result/acknowledgement behavior; blocked stays unresolved.
   Result notifications must not create reply loops.
5. **BI-05 — Guarded recovery.** Preserve request UUID/content, task version,
   revision, attempt, owner run, checkout and remote-generation fences. A timeout,
   disconnect or restart never proves an old process stopped. Cancel-requested is
   not cancelled. Unknown external writes are never automatically replayed.
6. **BI-06 — Existing collaboration topology.** Worktree uses the explicitly
   selected Coordinator; Project retains peer delegation. Selection, focus,
   display names and restarts cannot grant or transfer authority. Worktrees isolate
   checkouts and branches; they are not a new security sandbox.
7. **BI-07 — Honest delivery evidence.** Submitted, accepted, integration-selected
   and actual Git integration/deliverable verification remain separate. Integration
   selection does not execute Git. Changed content cannot silently inherit approval
   of old content. Non-coding work does not require Git.
8. **BI-08 — Actionable panel.** Reuse the collaboration panel. Show authorized
   tasks once with all relevant pending actions. Distinguish questions, review,
   permission/input observations, delivery trouble, dependency waits and unknown
   execution. Counts and lists use the same scoped source; totals describe the
   whole filtered set while rows are paged, so later tasks stay reachable and a
   page length is never a total; transient observations show freshness and a
   missing observation is neither healthy nor failed. No new dashboard or
   replacement persisted lifecycle enum. A task with unresolved follow-ups offers
   a "View all" entry that pages the full list independently of the newest
   previews, and the panel trims that page within its response budget while
   keeping a resumable position.
9. **BI-09 — Quiet, safe interaction.** Routine progress stays in history; repeated
   polling does not repeat notifications. New actionable changes remain visible.
   Mutating actions revalidate current state/version and retain drafts on failure.
10. **BI-10 — Bounded continuation and retention.** Read compact indexes and exact
    handoff/context pins lazily. Current review/source status is separate from an
    immutable brief. Sharing a brief grants no execution/filesystem authority.
    Existing capacity, archival, export and referenced-record protection remain.
11. **BI-11 — Transport/platform coverage.** Shared domain behavior applies to
    local, SSH and direct WSL. Preserve host/account/project isolation, SSH and WSL
    differences and all desktop targets. Missing capabilities remain explicit.
12. **BI-12 — Conditional capacity.** If existing automatic dispatch requires a
    fix, reserve admitted startup capacity without starving control messages.
    Answers and cancellation cannot wait for a worker slot. Do not claim a global
    process, CPU or token cap from partially observable vendor sessions.

## Explicit exclusions

- External event intake, GitHub/Feishu listeners, webhooks and connector settings.
- Remediation of `danger-full-access`, new permission prompts or sandbox work.
  Preserve existing credential protections and task/project/run authority checks;
  the permission exclusion does not remove those independent correctness rules.
- Bundled model calls, cloud AI/accounts/billing, telemetry, vector databases,
  terminal transcript ingestion, new daemons and cross-host federation.
- A permanent sole Router, a second task store or orchestration engine, automatic
  Agent launching, prompt-only auto-merge and automatic force-push/history rewriting.
- Unrelated UI redesign, terminal-core replacement, personal installation/config
  changes, release publication or assuming a Companion version bump is necessary.

Validation procedures and scenario mappings belong in [TECH.md](TECH.md).
