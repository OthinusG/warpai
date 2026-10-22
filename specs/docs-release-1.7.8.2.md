# Warpai 1.7.8.2 documentation synchronization

## Scope

Synchronize the English and Chinese README files, current SSH/WSL guides, and
the official website on `gh-pages` with the published `v1.7.8.2` release.
Use the GitHub release body and uploaded asset inventory as the release facts.

## Tasks and acceptance

- Update current version labels, badges, desktop filenames, and download URLs.
- Explain the persisted Agent MCP installation fix and inactive-record reinstall.
- Document local `warpai-local-com`, SSH `warpai-ssh-com`, and direct WSL
  `warpai-wsl-com`; preserve the distinction between Agent environments.
- Keep Companion at 5.5.0 and SSH/Agent Bus protocol majors at 1/2.
- Preserve historical release notes, screenshots, layout, and application code.
- Check every referenced release asset against GitHub's published asset inventory.
- Build Jekyll with `/warpai` as the base URL; inspect English/Chinese home,
  installation, and MCP pages and check their internal links.
- Publish documentation to `main` and `gh-pages` after validation, then verify
  GitHub Pages deployment and live bilingual content.

## Constraints and risks

The main development checkout is on a separate branch; use an isolated worktree
based on current `origin/main`. Avoid unrelated source changes and dependencies.
Do not infer a Companion version bump from the desktop patch version. Version
references in historical release notes and evidence must remain unchanged.

## Validation

- Jekyll 3.9.5 strict front matter build passed with production base URL `/warpai`.
- All 13 unique desktop/Companion download URLs match uploaded v1.7.8.2 assets.
- Internal href/src targets on all 34 rendered home/documentation pages resolve.
- English and Chinese localization keys match; rendered release and MCP scope
  copy is present in both languages with no unresolved Liquid expressions.
- Both Git diffs pass whitespace checks. Browser screenshot validation is
  unavailable in this environment; generated HTML and existing layout were reviewed.
