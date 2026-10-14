# Native execution threads and project memory

## Product scope

Implement audit items 18, 21 and 22. A tracked task is the stable anchor of a
logical execution thread; retries, review changes and reassignment preserve it.
Starting an attempt binds that thread to the exact existing native Agent, terminal,
run and checkout. Nested delegation records the issuer's running parent thread.
Message threads remain conversation history, not execution ownership. No second
session manager, worker spawning, transcript capture or vendor conversation-ID
emulation is introduced. Ordinary Project has no permanent Coordinator.

Every native MCP participant receives a compact project bootstrap index during
tool discovery and can refresh/page it through a tool. It includes current project
memory references and its assigned execution threads, never resource bodies.
Agents fetch only needed immutable resources. A bootstrap does not start tasks,
acknowledge messages, grant file access or restore an old run's authority.

Project memory is a bounded index of named slots over existing typed resources.
Participants deliberately extract reusable facts/decisions with their own model;
Warpai never runs a summarizer, scans repositories or reads vendor transcripts.
An explicit standalone memory update takes effect immediately. Task-associated
proposals are promoted automatically only when that exact revision is accepted.
Rejected proposals leave previous memory intact; withdrawing acceptance retires
memory derived from that result. Updates and retirement use optimistic versions;
immutable history and already-pinned task context remain readable.

## Shared API and implementation

- `execution_thread_get(task_id)` reuses task authorization and returns the stable
  task/thread ID, optional parent ID, state, attempt bindings and context refs.
  Bindings are captured at TaskStart, never inferred from a terminal's current cwd.
  Attempt certainty is reported, not manufactured; legacy attempts have no binding.
- `project_bootstrap(cursor?, limit?)` pages indexed memory slots and compact
  assigned thread summaries, plus published resource summaries with a separate
  context_list cursor. Tool discovery injects its first page in the bootstrap
  tool description; subsequent reads refresh it. The caller's native identity is
  authenticated by the existing Bridge/Broker. No early handshake registration.
- `project_memory_update(key, reference?, source_task?, task_revision?, expected_version,
  request_id)` creates/replaces/retires a slot. Version 0 creates it; all later
  writes require the current version. Keys are bounded ASCII identifiers. A null
  reference retires the slot. Task proposals require the active assignee and exact
  running/submitted revision, or an accepted result; they never bypass review.
- Shared Store metadata holds memory heads/history, task parent links and immutable
  attempt bindings. Existing transactions, budgets, events and idempotency receipts
  cover writes; no schema migration or added dependency.
  The index allows 128 named slots per project, including retired slots; resource
  payloads retain the existing 32-KiB bound. A slot holds current and proposed
  snapshots so rejection cannot erase the previous effective memory.
- Local, WSL and SSH share Operation/MCP implementations and negotiate capabilities.
  Old companions reject unsupported operations. Panel assignments use the same
  TaskStart/TaskReview transitions; no new overview or memory editor is required.

## Acceptance and tasks

1. Verify immutable native binding across terminal reuse/restart/reassignment,
   stable thread identity through retries, nested parents and denied foreign reads.
2. Verify memory CAS, idempotency, page bounds, immutable versions, retirement,
   project isolation and persistence after Store restart.
3. Verify proposals remain absent from effective memory before review, promote
   on acceptance, preserve prior memory on rejection and retire on changed review.
4. Verify bootstrap reaches managed/custom native MCP tool discovery and refresh,
   contains references rather than payloads, and preserves readiness/ack fences.
5. Verify shared local/SSH/WSL transport contracts on GitHub. Rust compilation and
   tests run in cloud; local syntax, diff and script checks precede dispatch.

Native CLI compliance with instructions remains model-dependent. No resource is
proof of current source: consumers compare source_revision before edits. Items
13–17 remain pending runtime acceptance; 19–20 and 23–30 are deferred.
