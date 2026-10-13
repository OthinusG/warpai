# Structured task messages and reusable context resources

## Product contract

Local, WSL and SSH use one Agent Bus Operation contract and account/project-owned
Store. All managed agents, custom native MCP clients and panel Send to Agent use
that contract. Vendor adapters configure transport only; they do not define a
second task/context format. Separate hosts/accounts/projects remain independent;
a shared Git remote URL never grants cross-host or cross-project access.

Task messages carry a structured body with schema_version, event, task ID and
revision, objective, acceptance criteria, pinned context references and event data.
Ordinary peer instructions carry a structured instruction body and references.
Keep legacy textual requests readable, but generate structured bodies for new
UI/MCP instructions and task events. PTY wake text remains a fixed notification;
it carries neither full task content nor resource bodies. MCP inbox/wait retain
original IDs and acknowledgement semantics.

## Resource contract

Resource kinds are repository_index, analysis, code_snippet and design_decision.
Each has a typed payload. Index entries contain workspace-relative paths and
symbol summaries; analyses contain summary/findings; snippets contain language,
path/starting line and code; decisions contain decision/rationale/alternatives.
Publish only explicitly supplied context; no background repository scans, shell
execution, filesystem reads, secret-file ingestion or external vector database.

A resource has stable UUID, immutable revision, SHA-256 digest, title, kind,
project, author, created sequence, broker-owned source workspace when available
and optional source revision metadata. Source workspace is provenance, never a
request to open another checkout or a grant of filesystem access. Updating
requires expected_revision, appends a new revision and never rewrites referenced
content. Limit encoded publication to fit the existing 64-KiB managed command;
large indexes are published as scoped module resources. List/search projections
contain summaries and references, never full payloads. Get requires exact revision.

ContextRef is {id, revision, sha256}. TaskAssign, TaskCreatePool and AgentSend
accept references. The server verifies every reference in the caller's project
before queueing; missing revisions, digest mismatches and foreign projects fail
atomically. Resources published to the project can be reused by project agents;
this shares supplied context, never native filesystem or mutation authority.

Resource metadata describes a snapshot, not proof of current repository content.
The consumer compares source revision before edits or correctness-sensitive work;
changed code requires publishing a new snapshot. Explicit pinned revisions make
old results reproducible. A retry/reassignment retains the task's references;
refreshing context requires an explicit new assignment with the new references,
rather than silently following a resource's latest head.

## API and persistence

Add context_publish, context_get and context_list to the shared MCP tools. Advertise
context_resources/structured_task_messages capabilities for remote services and
reject unsupported reference-bearing operations without copying content inline.
Expose bounded immutable resource URIs through the same context_get authority;
no caller-provided URI can trigger arbitrary file/network reads. Tool access
remains available for agents that do not implement native MCP resource browsing.

Reuse Store transactions, idempotency receipts, database budgets and project
sequences. Use existing additive metadata storage for resource versions/heads and
per-task revision references; no second broker, database or daemon. Keep task
ownership, reservations, permission guards and explicit result/review transitions.
Resource reads cannot acknowledge a task or mutate its execution state.

## Panel synchronization

Send to Agent uses the same instruction/task body builders and pinned references.
Select published resources by title/kind in existing native controls, with compact
summaries and optional preview. UI sends IDs/revisions/digests; the receiving agent
loads resource bodies through MCP. Task detail/history display objectives, criteria
and reference titles with readable event details. Show unsupported Companion
capability rather than silently dropping references. Preserve drafts and retry IDs.

## Execution and verification

1. Define shared typed DTOs and bounded validation, then publish/get/list persistence.
2. Add structured message serialization and task/message reference validation.
3. Update task retry/pool/reassignment/history and shared MCP descriptions/resources.
4. Negotiate Companion support on SSH/WSL; update native panel Send to Agent.
5. Verify resource restart/idempotency, immutable old versions, failed publication,
   foreign project/account denial, stale digest and task revision preservation.
6. Exercise real MCP transfer for every managed program, local native IPC on both
   OSes and real SSH/WSL Companion channels. Verify UI reference selection and
   native lifecycle/draft/menu/error/cancel transitions without vendor credentials.

Completion requires source-matched cloud tests/builds/native acceptance and a test
installer. Synthetic agent processes verify transport/contracts, not authenticated
vendor/model availability. The owner WSL repro remains an independent delivery gate.
