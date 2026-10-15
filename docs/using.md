---
layout: docs
title: User Guide Overview
description: "Comprehensive guide to engineering workflows in Warpai: Git worktree parallel development, coordinator delegation, and in-terminal tooling."
section: Using Warpai
doc_id: using
prev_url: /docs/features.html
prev_title: Features Matrix & Comparison
next_url: /docs/using-terminal.html
next_title: Terminal & Command Blocks
---

This guide covers real-world engineering workflows with Warpai, from orchestrating parallel agent squads across Git worktrees to in-terminal code editing and remote SSH development.

---

## Workflow 1: Parallel Development with Git Worktrees

The ultimate multi-agent workflow in Warpai is **Worktree Mode**. By isolating each feature into a dedicated Git worktree, multiple CLI agents work concurrently without corrupting each other's files or compiler caches.

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                                                                             │
│   1. Switch to Worktree Mode (Collaboration Panel top bar)                  │
│   2. Create separate worktrees for each component (e.g. api, ui, db)        │
│   3. Launch your CLI agents in their respective worktree tabs               │
│   4. Select your Coordinator Agent from the dropdown                        │
│   5. The Coordinator assigns tasks and context to worker checkouts          │
│   6. Workers build in parallel, submit diffs, and conduct peer review       │
│   7. The Coordinator reviews, integrates, and makes the final Git commit    │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

### Step-by-Step Instructions:

#### 1. Create Worktrees for Your Features
Run standard Git commands or use the worktree helper:
```bash
git worktree add ../my-project-api feature/rest-api
git worktree add ../my-project-ui feature/frontend-components
git worktree add ../my-project-tests feature/integration-tests
```
Each worktree opens in a new tab, pinned to its physical directory.

#### 2. Launch Agents in Each Checkout
Switch to the respective tabs and start your agents:
- In `../my-project-api`: launch `claude` or `codex`
- In `../my-project-ui`: launch `cursor-agent` or `trae`
- In `../my-project-tests`: launch `gemini` or `agy`

#### 3. Select the Coordinator
In the left collaboration panel:
1. Switch the mode selector from **Project** to **Worktree**.
2. Click the **Coordinator** dropdown and select the agent responsible for architectural leadership.
3. The selected Coordinator receives a unified view of all active checkouts and workers.

#### 4. Delegate & Monitor
Prompt your Coordinator:
> *"We need to add a user export feature. Please break down the work: delegate the backend endpoints to Worker A, frontend download UI to Worker B, and integration tests to Worker C. Oversee peer review and integrate when ready."*

The Coordinator uses Warpai's built-in MCP tools (`warp_task_assign`) to dispatch tasks. Workers receive their tasks automatically when idle.

---

## Workflow 2: Automated Peer Review & Integration Pipeline

Warpai provides an automated, structured peer review cycle directly between agents:

1. **Submission**: When a worker finishes, it submits evidence and a diff summary via `warp_task_submit`.
2. **Review Assignment**: The Coordinator assigns a peer agent as reviewer via `warp_task_review`.
3. **Review Decision**:
   - If issues are detected, the reviewer flags the task as `needs_changes` with explicit comments. The worker automatically receives the revision request.
   - Once satisfied, the reviewer marks the task as `accepted`.
4. **Integration**: The Coordinator runs `warp_task_integrate`, merges the branch into main, executes the test suite, and creates the final commit.

---

## Workflow 3: Flat Collaboration for Research & Writing

When working on non-code projects (literature reviews, competitive analysis, business proposals, documentation), use **Project Mode**:

- **No Git Required**: Operates in any directory or shared folder.
- **Shared Context**: Agents publish shared research notes and references using `warp_context_publish`.
- **Collaborative Problem Solving**: An agent researching a topic can ask clarifying questions to a fact-checker agent using `warp_agent_ask` and receive structured callbacks via `warp_agent_reply`.

---

## Workflow 4: The In-Terminal Developer Workbench

Keep your hands on the keyboard and reduce mental overhead:
- **Project Explorer**: Navigate files and create directories directly alongside your terminal.
- **Lightweight Editor with Vim Bindings**: Edit configs and scripts in-app with `:w`, `:q`, and modal navigation.
- **Live Markdown Preview**: Instant visual verification of documentation, tables, and images.
- **Native Git Diff Review**: Review additions and deletions before committing without opening a heavyweight IDE.
