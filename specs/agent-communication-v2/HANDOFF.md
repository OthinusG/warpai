# Task handoff briefs

Date: 2026-11-03. Status: implementation in progress; runtime gates pending.

## Scope and acceptance

Local, SSH and direct WSL use the shared Agent Bus operation and account-owned
Store. A tracked task is the only generation anchor. Reuse immutable context
versions, existing SQLite transactions, task notifications and native panel.
No model, repository scan, command execution, transcript ingestion or new daemon.
Implementation must pass DTO, authorization, transaction/replay, lifecycle,
restart and transport tests, followed by source-matched three-platform builds,
SSH/WSL acceptance and native multilingual screenshots. Pending gates are not
completion. The implementation guide remains the acceptance checklist.

## Data and sources

`ContextContent::Handoff { snapshot }` is reserved to the Broker. Public publish
cannot create or update it. `HandoffSubmission` contains decisions (decision,
rationale), unresolved issues and ordered next steps. Each section uses tagged
`provided`, `not_provided` or `not_applicable`; provided empty lists are explicit.
The encoded submission budget is 8 KiB including JSON escaping and UTF-8.
Unknown fields are rejected. These statements are Agent reports, never commands.

`HandoffSnapshot` schema 1 records task/thread ID, task revision, exact attempt,
producer and immutable native binding captured at TaskStart, request UUID,
generation reason, source event sequence and ID, evidence IDs and Agent sections.
Task objective, result/reason and evidence remain in the source event/task instead
of being copied. A bounded result preview is explicitly marked when shortened.
No final commit or clean-workspace inference. Missing binding is explicit null.
Test evidence is Agent-reported; content verification and task acceptance are
separate facts. Resources retain the 32-KiB content limit and 64-KiB wire bound.

## API and lifecycle

TaskSubmit always creates a baseline brief, including for legacy input.
TaskProgress creates a checkpoint only with explicit `handoff` input; TaskFail
and TaskFinishCancel create partial/stopped briefs with optional Agent sections.
Each accepts optional `handoff` (omitted on the wire when absent).
One stable resource ID per task, append-only revisions across all attempts.
Input context_refs never change. Task detail exposes current `handoff_ref` only
when the resource matches the current task revision and attempt; historical
versions remain accessible through paged task events and exact context_get.
Execution thread/bootstrap expose compact references only. Review never mutates
a brief or grants execution; retry/reassignment preserve original source bindings.
Consumers refresh bootstrap/task, read the exact pin, check current source review
and uncertainty, inspect necessary evidence and use normal TaskStart/recovery.

Resource generation, association, task transition, events, notifications and
idempotency receipt share the existing transaction. Any error rolls all back.
Same UUID/content returns the original reference; changed content is rejected.
No new acknowledgement, read receipt, retry or auto-accept semantics.

## Authority, sharing and retention

Handoff get/list/browsing/bootstrap inherit source task authorization. Filter
before pagination/preview. Source producer liveness is irrelevant. The trusted
panel operator uses existing project authority. Fixed handoff pins explicitly
passed to a newly assigned/pool task grant only that version to participants
authorized to read the target task; sender must first be authorized to read it
and create the target task. Ordinary instructions/replies require explicit
version-scoped recipient grants. These grants do not grant source task access,
recursive resource access, filesystem or Coordinator authority. Direct source
reads remain subject to current task permissions.

Keep source task/evidence/events while a live target task or message pins any
handoff version so provenance cannot dangle. Unreferenced archived source tasks
use the existing purge policy, deleting their briefs in the same transaction. Cleanup preview reports protected tasks. Resources already follow Store
capacity limits; no independent retention service or automatic memory promotion.
Retiring memory never deletes immutable versions. Source review changes are
visible through source task refresh; consumers must revalidate before use.

## Compatibility and UI

Use capability `handoff_briefs_v1`, Agent Bus 2 / SSH major 1 unchanged. New optional
fields serialize only when supplied. Reject handoff-bearing remote requests
before transmission when capability is absent. Do not flatten into inline prose.
Legacy records have no brief. A companion may advertise the capability only after
the full contract and transport gates pass; packaging versions follow release gates.

The visual source is the existing Collaboration task detail/history, native
section/row controls, shared Appearance tokens and spacing. Add a compact handoff
section and explicit load control; fetch full content only on selection. Fence
responses by context, generation, task and exact pin. Loading, absent, denied,
unsupported and failed reads remain distinct. Include current review separately
from snapshot reason, decisions, unresolved issues, next steps, binding/evidence
and missing-field states. Fixed fixtures precede live wiring and screenshots;
English/zh-Hans/zh-Hant must have equivalent coverage.

## Execution plan and verification

1. DTO/schema, bounded validation and reserved publication tests.
2. Shared Store generation, pinned authority/sharing, retention, rollback/replay
   and restart tests, including stale runs and review/retry transitions.
3. MCP discovery/guidance, bootstrap and capability compatibility tests.
4. Native detail fixtures, lazy reads, multilingual strings and screenshots.
5. Existing GitHub protocol/remote/desktop workflows and source-linked receipts;
   no personal configuration or installation changes. Create a reviewable PR
   identifying every pending acceptance gate.
