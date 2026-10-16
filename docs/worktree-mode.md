---
layout: docs
title: "Worktree Mode: Parallel Git Software Engineering"
description: "Comprehensive guide to Warpai's dedicated Worktree Mode for Git repositories: isolated worktrees, parallel feature development, Coordinator oversight, and automated peer review loops."
section: Using Warpai
doc_id: worktree-mode
prev_url: /docs/project-mode.html
prev_title: "Project Mode: Omnidirectional Collaboration & Cross-Domain Guide"
next_url: /docs/using-warpai-agents.html
next_title: Multi-Agent Workflows
---

In rigorous software engineering, allowing multiple agents to concurrently modify a single Git repository directory leads to lock contention, git merge nightmares, and corrupted build caches (such as conflicting `target/` or `node_modules/` artifacts).

**Warpai's Worktree Coding Mode assigns each feature to an isolated physical Git worktree**:
- **Beyond the Coordinator who oversees integration, each worktree develops independent features concurrently in physical isolation**.
- **Complete Decoupling of Files & Build Caches**: Worker A’s backend refactoring on `feature/api` never invalidates Worker B’s frontend build on `feature/ui`.
- **Automated Peer Review & Integration**: Code diffs undergo peer inspection (`accepted` vs. `needs_changes`) before the Coordinator integrates the approved branches into mainline.

---

## Architecture: Coordinator Oversight & Concurrent Feature Worktrees

```
                    ┌────────────────────────┐
                    │  Coordinator Agent     │
                    │   (Oversight & Merge)  │
                    └───────────┬────────────┘
                                │
        ┌───────────────────────┼───────────────────────┐
        │                       │                       │
 ┌──────▼──────┐         ┌──────▼──────┐         ┌──────▼──────┐
 │ Worktree A  │         │ Worktree B  │         │ Worktree C  │
 │ feature/api │         │ feature/ui  │         │feature/tests│
 │ (Agent A)   │         │ (Agent B)   │         │ (Agent C)   │
 └─────────────┘         └─────────────┘         └─────────────┘
```

1. **Coordinator Agent**: Understands high-level requirements, decomposes objectives into subtasks, establishes dependency graphs, oversees peer reviews, and executes clean mainline commits.
2. **Worker Agents**: Operate inside physically isolated checkouts, focusing entirely on writing feature code, collecting test evidence, and submitting clean diffs.

---

## Complete How-to-Use Guide

### Step 1: Create Isolated Physical Worktrees
In your project directory, generate separate worktrees for each feature:
```bash
# Worktree for backend REST API
git worktree add ../my-project-api feature/api

# Worktree for frontend UI components
git worktree add ../my-project-ui feature/ui

# Worktree for integration tests
git worktree add ../my-project-tests feature/tests
```
Each worktree opens in its own vertical tab, locking its local working directory.

### Step 2: Assign & Launch Dedicated Agents
Switch to each tab and launch your chosen CLI agent:
- Tab 1 (`../my-project-api`): launch `claude` or `codex`
- Tab 2 (`../my-project-ui`): launch `cursor-agent` or `trae`
- Tab 3 (`../my-project-tests`): launch `gemini` or `agy`

### Step 3: Designate the Coordinator
1. In the upper-left collaboration panel, switch mode to **Worktree**.
2. Click the **Coordinator** dropdown and select an architecturally capable agent.
3. The Coordinator automatically scans all active worktree branches and agent states.

### Step 4: Requirement Scoping & Task Dispatch
Provide your high-level objective to the Coordinator:
> *"Implement an Excel export feature. Delegate the backend API to Worker A, frontend download modals to Worker B, and regression test suites to Worker C. Have them develop in parallel and peer review each other before final integration."*

The Coordinator invokes native MCP tools (`warp_task_assign`) to dispatch tasks to multiple workers at any time:
```json
{
  "task_id": "task-api-export",
  "target_branch": "feature/api",
  "title": "Implement order export backend API",
  "dependencies": []
}
```
- **Continuous Multi-Agent Dispatch**: The Coordinator can assign work to multiple agents across branches at any time without waiting for prior tasks to finish.
- **Dynamic Supervision List**: Every newly dispatched task automatically enters the Coordinator's dynamic supervision list.
- **Active Exit Upon Completion**: Tasks remain under supervision throughout implementation and peer review, actively leaving the list once accepted and integrated.

### Step 5: Concurrent Development & Local Evidence Collection
Each worker develops inside its isolated physical directory:
- Worker A updates Rust/Go backend code and compiles local `target/` without dirtying Worker B's frontend workspace.
- After running test suites, workers invoke `warp_evidence_add` to attach logs and screenshots.

### Step 6: Automated Peer Review Loop
Once coding is complete, the peer review pipeline activates:
1. **Submission**: Worker calls `warp_task_submit` with git diff details.
2. **Assign Reviewer**: Coordinator invokes `warp_task_review` to designate another available agent as reviewer.
3. **Verdict**:
   - `needs_changes`: Reviewer flags issues or bugs; worker automatically receives actionable feedback and refactors.
   - `accepted`: Reviewer approves the changes.

### Step 7: Mainline Integration & Final Commit
When all parallel subtasks are verified and accepted:
1. Coordinator calls `warp_task_integrate`.
2. Warpai merges the branch into `main` and runs regression checks.
3. A clean, structured Git commit is recorded, concluding the parallel delivery cycle.
