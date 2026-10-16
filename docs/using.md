---
layout: docs
title: "User Guide Overview"
description: "Comprehensive guide to Warpai's core engineering and cross-domain workflows: Git Worktree parallel coding, Project omnidirectional collaboration, coordinator task pools, and automated peer review."
section: Using Warpai
doc_id: using
prev_url: /docs/features.html
prev_title: Features Matrix & Agent Systems Comparison
next_url: /docs/using-terminal.html
next_title: Terminal & Command Blocks
---

This guide covers Warpai's primary workflows across software engineering and multi-agent cross-domain teams: from concurrent feature development in isolated Git worktrees and omnidirectional Project mode to integrated terminal editing, peer review pipelines, and remote SSH/WSL setups.

---

## Workflow 1: Parallel Development with Git Worktrees (For Software Coding)

Warpai's flagship workflow for software repositories is **Worktree Mode**. By allocating each feature to an isolated physical Git worktree, multiple CLI agents write code concurrently without dirtying branches or invalidating build caches.

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                                                                             │
│   1. Switch to "Worktree" mode at the top of the collaboration panel        │
│   2. Create isolated physical worktrees for each feature (api, ui, tests)   │
│   3. Launch your preferred CLI agent inside each corresponding tab          │
│   4. Select a Coordinator agent from the dropdown                           │
│   5. Coordinator scopes requirements and dispatches tasks to worktrees      │
│   6. Workers develop concurrently in isolation and submit diffs for review  │
│   7. Coordinator verifies results and integrates changes into the main repo │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

### Step-by-Step Instructions:

#### 1. Create Physical Worktrees for Features
Run standard Git commands or use Warpai's worktree wizard:
```bash
git worktree add ../my-project-api feature/rest-api
git worktree add ../my-project-ui feature/frontend-components
git worktree add ../my-project-tests feature/integration-tests
```
Each worktree opens in its own vertical tab, locking its working directory.

#### 2. Launch Agents in Each Worktree
Switch to each tab and launch your chosen agent:
- In `../my-project-api`: launch `claude` or `codex`
- In `../my-project-ui`: launch `cursor-agent` or `trae`
- In `../my-project-tests`: launch `gemini` or `agy`

#### 3. Designate the Coordinator
In the left collaboration panel:
1. Switch the mode toggle from **Project** to **Worktree**.
2. Click the **Coordinator** dropdown and select an agent with strong architectural oversight.
3. The Coordinator receives a unified view of all active worktree branches and agent states.

#### 4. Dispatch Requirements and Supervise Execution
Provide your high-level objective to the Coordinator:
> *"Implement a data export feature. Delegate the REST API to Worker A, frontend download components to Worker B, and integration tests to Worker C. Have them peer-review each other before final integration."*

The Coordinator dispatches tasks via Warpai's native MCP tool (`warp_task_assign`). Workers automatically pull tasks when idle.

---

## Workflow 2: Omnidirectional Project Mode (For Cross-Disciplinary & Non-Coding Teams)

For non-coding and multi-agent research domains (scientific paper drafting, creative design, quantitative finance, patent drafting, content operations, and business intelligence), rigid Git branching is unnecessary. Warpai delivers full first-class support for **Project Mode**:

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                                                                             │
│   1. Switch to "Project" mode at the top of the collaboration panel         │
│   2. No Git repo required: launch agents inside any folder or cloud drive   │
│   3. Break rigid hierarchies: all agents act as Commanders or Workers       │
│   4. Direct two-way messaging across any agent (warp_agent_send / reply)    │
│   5. Structured Q&A channels for peer inquiries (warp_agent_ask / answer)   │
│   6. Autonomous task claiming from public pools (warp_task_create_pool)     │
│   7. Expose verified milestones via shared memory (warp_context_publish)    │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

- **Omnidirectional Messaging & Tasks**: Every agent operates as an equal peer node. In scientific research, a literature retrieval agent supplies citations directly to the manuscript author and recommends plot aesthetics to the data visualization agent, while a mock peer reviewer dispatches revision notices.
- **Dynamic Role Reversals**: An agent can coordinate a roadmap as Commander in stage 1, then transition to Worker in stage 2 under a specialized QA lead.
- **Dedicated Implementation Manuals**: For step-by-step procedures and templates, see our dedicated guides on [Project Mode: Omnidirectional Collaboration & Cross-Domain Guide]({{ '/docs/project-mode.html' | relative_url }}) and [Worktree Mode: Parallel Git Software Engineering]({{ '/docs/worktree-mode.html' | relative_url }}).

---

## Workflow 3: Automated Peer Review & Integration Pipeline

Inside Worktree Mode, completed tasks are never blindly merged into `main`. Instead, Warpai enforces an automated peer review loop:

1. **Submission**: When a worker finishes coding, it calls `warp_task_submit` with test logs (`warp_evidence_add`) and git diffs.
2. **Review Assignment**: Coordinator assigns another available agent as reviewer (`warp_task_review`).
3. **Review Verdict**:
   - `needs_changes`: Reviewer flags issues or anti-patterns; worker receives actionable feedback and refactors.
   - `accepted`: Reviewer approves the changes.
4. **Integration**: Coordinator calls `warp_task_integrate` to merge the branch into `main`, execute regression tests, and create a clean Git commit.

---

## Workflow 4: The Terminal-Integrated Workbench

Keep your hands on the keyboard without switching windows:
- **Project Explorer**: Browse, create, and manage files alongside your terminal panes.
- **Integrated Code Editor & Vim Modes**: Edit configuration files and scripts directly with `:w` and `:q` keybindings.
- **Live Markdown Preview**: Render technical documentation, tables, and diagrams instantly.
- **Native Git Diff Inspector**: Review modifications side-by-side before committing.
