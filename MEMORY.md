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
  - Split multi-agent documentation into dedicated standalone guides: `project-mode.md` (omnidirectional, non-coding, 6+ domain tutorials) and `worktree-mode.md` (Git parallel coding, Coordinator peer review) alongside `using-warpai-agents.md`. Updated docs sidebar navigation in `_layouts/docs.html` with direct entries.
  - Streamlined landing page Hero lede and Section 06 copy across both English and Chinese to punchy, &lt;=4 line presentations.
- **Product Showcase Mockup Overhaul (2026-10-16)**:
  - Redesigned landing page mockup widget in `index.html` and `zh/index.html` to faithfully replicate the authentic 3-column Warpai application layout from `/Users/wqin/Desktop/截屏2026-10-15 17.46.51.png`.
  - Left column: Window controls, tab search filter, session list with active indicators, badges, and bottom action links (File Explorer, History).
  - Center column: Terminal viewport running interactive multi-agent session (`claude`, `agy`, `warpai`, `dsh`), ASCII mascot, dark-styled peer notification callout box with interactive focus tags, and command bar.
  - Right column: Agents Collaboration Panel featuring interactive Mode Switcher (`Worktree` vs `Project`), dynamic Coordinator selector, branch/channel lists, dynamic task pool, and data usage counters.
  - Full interaction in vanilla JS: tab switching, mode switching, coordinator changing, branch creation, tag focusing, drawer toggling, sidebars toggle, and in-window theme switching.
  - Bilingual parity across English and Chinese pages with complete styling support for both Light (`paper`) and Dark (`ink`) themes.
