# Project Memory

## Project

- **Name**: Warpai Website (`warpai` repository, `gh-pages` branch)
- **Path**: `/Users/wqin/workplace/warpai-gh-pages`
- **Remote**: `https://github.com/OthinusG/warpai.git` (branch: `gh-pages`)
- **Hosting**: GitHub Pages at `https://othinusg.github.io/warpai/`
- **Stack**: Jekyll, HTML5, CSS3, Vanilla JS, GitHub Pages native build

## Decisions & Context

- **Repository Structure**: Cloned the dedicated `gh-pages` branch directly into `/Users/wqin/workplace/warpai-gh-pages` for managing documentation and product landing pages.
- **Language Support**: Uses dual-directory structure with root `/` for English and `/zh/` for Simplified Chinese.
- **Jekyll Base URL**: Configured with `baseurl: /warpai`. All internal links and asset references must account for the baseurl prefix.
- **Project Startup Protocol**: Initialized `README.md`, `AGENTS.md`, and `MEMORY.md` on 2026-10-16.
- **Dual Architecture Paradigm (Worktree vs. Project Modes)**:
  - **Worktree Coding Mode**: Tailored for Git repositories. Beyond the Coordinator, each agent writes code concurrently in an isolated physical Git worktree to eliminate build cache and branch collisions, followed by automated peer review and integration.
  - **Project Omnidirectional Mode**: Heavily supported paradigm for non-coding and cross-domain teams (scientific research, creative design, quant finance, patent compliance, media operations, business intelligence). No Git required. All agents communicate as equal peers, dynamically switching between Commander and Worker roles, exchanging messages, questions, shared context, and task pool items.
- **Copy & Terminology Alignments**:
  - Removed "fearless / 无惧中断" (inherited from herdr template) and replaced with "multi-agent / 多智能体协同".
  - Replaced "coral pulse / 珊瑚红光" with status tab indicators (WaitingApproval badge) and native OS desktop notifications.
  - Translated navigation "CLI AGENTS" to "智能体" in Chinese views.
  - Added native Windows WSL/WSL2 integration highlights in Section 05 and documentation.
  - Added dedicated Section 06 on landing pages linking to comprehensive multi-agent guides.

