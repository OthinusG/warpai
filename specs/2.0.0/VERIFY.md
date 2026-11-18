# Optional SSH/WSL guardian verification

Status: The complete V01–V30 release matrix remains open. The implementation
ledger records native Linux/macOS OpenSSH reconnect, allocated-TTY CLI and
caller-owned stop, Windows-to-WSL bridge survival, and Linux task submission
with both SSH bridges closed. Use exact source, runner OS and architecture,
helper/app version, workflow job/artifact, expected/actual result and redacted
receipts. Submitted tests are not passed tests.

## Traceability

| Requirements | Tasks | Acceptance cases |
| --- | --- | --- |
| FR-01, FR-10 | S0, S4, S5 | V01, V08, V20 |
| FR-02, FR-03 | S1, S3, S4 | V02, V04, V16 |
| FR-04 | S5 | V03, V12, V18 |
| FR-05 | S1, S6, S7 | V05, V09, V21 |
| FR-06 | S2, S3 | V06, V07, V17 |
| FR-07 | S3 | V04, V10 |
| FR-08 | S1, S6 | V11, V12, V13 |
| FR-09 | S4-S6 | V09, V12, V15 |
| FR-11 | S1, S7 | V21, V22, V23 |
| FR-12 | S2, S7, S8 | V13, V19, V23, V24 |
| FR-13 | S5, S9 | V08, V16, V18 |
| FR-14 | S4, S6 | V14, V17 |
| FR-15 | S0, S1, S5 | V19, V27 |
| FR-16 | S0, S4, S9 | V25, V26 |
| FR-17 | S3, S4 | V16, V28 |
| FR-18 | S0, S4, S6 | V25, V29 |
| FR-19 | S0, S4 | V26, V30 |

## Fixtures and execution boundaries

Use disposable isolated accounts/projects and deterministic child fixtures with
native process birth identity, counters, permissions/drafts, exit/descendant and
partial-write controls. Test real PTYs, Broker/Store, SSH and wsl.exe, not only
protocol mocks. A permitted supported-CLI smoke test does not justify reading
personal credentials or asserting synthetic fixtures prove vendor auth.

Separate client disconnect, Bridge exit, observer loss, service crash and host
shutdown injections. Agent PIDs alone cannot prove same execution. Include mixed
guarded/unguarded sessions and Agent/shell/test/server session kinds.

## Acceptance matrix

| ID | Scenario and method | Required result |
| --- | --- | --- |
| V01 | Fresh/default-off profile and existing launch/close flow; local terminals/Broker regression | No guardian opt-in, local daemon, global migration or behavior change |
| V02 | Guarded SSH Agent/command; drop transport and exit desktop; reconnect | Same native birth/run identity, work/output continues, no duplicate launch |
| V03 | Guarded guest PTYs; exit GUI and every wsl.exe bridge; reopen exact distro/user | Linux guardian/work remains, no Windows-held lifetime dependency |
| V04 | Race GUI/CLI attaches/takeovers, late detach/input/resize | One writer per session, old generation refused, siblings unaffected |
| V05 | No desktop: Coordinator and workers exchange questions/results and queued tasks | Existing admission/revision/readiness/review rules continue headlessly |
| V06 | Prompt/alt screen/Unicode/blocks and CLI-to-GUI model export/import | Correct interactive state and block boundaries; no repeated OSC/clipboard actions |
| V07 | Snapshot/output/resize races, split escape/UTF-8, expired handle/ring gap | Coherent cut/tail or explicit resnapshot; bounded allocation and no false complete history |
| V08 | Old/new SSH and WSL clients/hosts; legacy controls against guarded IDs | Existing paths work, unsupported guarding explicit, legacy release cannot kill retained runs |
| V09 | Detach with permission prompt or protected draft; deliver peer work | No auto-approval, draft overwrite, false readiness or task acknowledgement |
| V10 | Lost input responses, duplicate/conflicting sequence, partial native write | No duplicate input/remainder replay; unknown stays explicit |
| V11 | Stop one, descendants, failed stop and snapshot-based stop-all with partial failure | Native exit proof required; single-target stop leaves siblings, bulk stop reports each captured run; no unsafe replacement |
| V12 | Runtime crash, SSH host reboot, distro terminate/shutdown or Windows reboot | Interruption/unknown shown, no automatic resume or inherited authority |
| V13 | Ended TTL, pressure eviction, retained unresolved task and idle service exit | Bounded output memory; task evidence retained; expired history explicit; no permanent idle daemon |
| V14 | Account/root/distro/user/boot replacement and nested SSH | Cached authority refused, final host correct, no local fallback or namespace crossover |
| V15 | EN/ZH native static then live UI at small windows/scaling/themes | Optional mode clear, no clipping; transport/process/attention distinct, drafts/focus preserved |
| V16 | Desktop update/restart/language change, plain SSH/WSL CLI detach then Warpai attach | Remote run survives client lifetime; compatible runtime reused, incompatible one not killed |
| V17 | File transfer/write disconnect, reopen Explorer/editor/Review and task panel | Lazy confirmed-project restore; no uncertain write replay, hidden UI/watchers or automatic result acceptance |
| V18 | Guest/SSH install/reinstall, Unicode paths, corrupt/missing helpers, live upgrade/uninstall | Provenance correct, no unsafe fallback, no removal of dependencies still used by live runs |
| V19 | Foreign-user IPC, malformed/oversized snapshot, lost wake and startup races | Auth/bounds enforced, one service owner, no repeated side effects or permanently lost work |
| V20 | Change future-launch preference with existing mixed-policy sessions | Existing policy unchanged; disabling preference does not stop current guarded runs |
| V21 | Several Agents and commands in two projects; one Agent receives several tasks | Parallel sessions progress, same-Agent tracked execution serial, projects/worktrees isolated |
| V22 | Flood one PTY, block one writer/snapshot/client, issue stop and quiet-session wake | Fair bounded IO/control progress; no shared-lock starvation or hidden auto-worker creation |
| V23 | Fill session/input/snapshot/project capacity; cancel or fail launches/attach | Backpressure explicit, all reservations freed, no unrelated run killed and no unbounded queue |
| V24 | Repeat idle/flood/reconnect resource benchmark with 0/1/8/32 sessions | CPU/RSS/thread/latency receipts meet frozen budgets or keep release gate open |
| V25 | CLI labels/IDs, duplicates/Unicode, current/unique/MRU resume and immediate bare stop-all in TTY/scripts | No resume picker; bare stop captures all account guarded runs without extra confirmation, survives caller-PTY/socket loss, retains exact receipts, no fuzzy/task-title or unrelated-process kill |
| V26 | Session-only function, external absolute helper, command collisions/quoting and split/literal CLI detach escape | No PATH/profile mutation or override; literal args/input round-trip; detach restores TTY without stopping run |
| V27 | Concurrent enable/reconnect through aliases/clients/Agent kinds and proven same-account SSH/WSL routes | One guardian per execution account; logical scopes isolated; no second daemon on incompatibility |
| V28 | Reconnect after CLI writer takeover, catalog watch gap or runtime replacement | Automatic bounded metadata/task restoration, no writer theft/relaunch/ack, selected snapshots only |
| V29 | Multi-client foreground MRU, last run exit, no-history fallback, read-only/background calls and writer conflict | Monotonic deterministic current/unique/MRU target, no read-induced reorder/ack, no writer theft or silent conflict fallback |
| V30 | Minimal list/resume/stop parser/help/version, list availability/state and unknown extra command rejection | Correct core behavior with bounded list, no MRU/writer/daemon-start side effects, no withdrawn reader/waiter/admin/completion services |

## Platform and workflow coverage

SSH execution hosts: Linux/macOS/Windows. Desktop clients: all advertised platforms
with existing packaging coverage. Include macOS-to-Linux, Windows-to-Linux and
Windows SSH host native lifetime tests. WSL requires real Windows plus selected
Linux distro/user, multiple accounts and distro shutdown/bridge-exit distinction.
Do not substitute WSL tests with Linux SSH. Native local is regression-only.

Rust builds/native acceptance run in GitHub/cloud, not on the personal machine.
Extend existing workflows and closest fixtures:

- `.github/workflows/validate-agent-communication.yml`
- `.github/workflows/validate-remote-companion.yml`
- Existing platform artifact workflows and real WSL installation/native acceptance.

```sh
cargo test -p remote_protocol --locked
cargo test -p warp-agent-bus --locked
cargo check -p warpai --bin warpai --locked
cargo check -p warpai --bin warpai --features warp_platform --locked
```

Verify actual manifest features and source-matched helper provenance before CI.
Preserve task/handoff/review, native local, unguarded SSH/WSL, file/installation,
locale/network and Agent adapter regression checks. No local build is required
for these documentation-only changes.

## Low-overhead benchmark protocol

Record guardian PID/process tree, RSS and where available private/PSS memory,
CPU-time delta divided by wall time and one core, thread/context-switch counts,
account/project/session/parser/snapshot/queue usage, stop/wake latency and output
throughput. Report guardian overhead separately from native Agents/commands, while
including every guardian worker and Broker cache. Record target CPU/OS/VM noise.

Compare the current 300 ms worker baseline to the proposed event-driven guarded
implementation with equivalent 120x40 idle sessions, 0/1/8/32 sessions, same and
multiple projects, attached/detached modes, a flooded PTY and snapshot storms.
Warm up and repeat at least three 60-second idle samples; capture reconnect/flood
peaks separately. Inspect loss-of-notification and timer-boundary correctness.

TECH proposes base RSS <=32 MiB, incremental idle session <=2 MiB and 8-session
idle CPU <=0.2% of one core on a documented quiet 2-vCPU VM. These are unverified
design targets. Freeze platform-appropriate, reproducible final thresholds in S8;
any change needs measured rationale and a ledger update. Do not claim compliance
from serialized output size or mocks. With no live work/retention/requests, prove
the service exits after its existing idle window.

## Evidence rules

Write redacted receipts to `specs/2.0.0/evidence/` as implementation proceeds;
reference source-matched CI artifacts instead of storing raw private transcripts.
Every case is not_run/running/passed/failed/blocked, with concrete blockers.
Only passed relevant source closes gates; code changes invalidate affected prior
receipts. Screenshots use deterministic fixture data. No credentials, raw drafts,
environments or bearer values may appear in records.
