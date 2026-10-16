# AGENTS.md

Warpai official website and documentation repository (`gh-pages` branch).

## Project Overview

- **Project Name**: Warpai Website
- **Date**: 2026-10-16
- **Branch**: `gh-pages`
- **Tech Stack**: Jekyll, HTML5, CSS3, Vanilla JavaScript, Markdown, GitHub Pages
- **Live URL**: https://othinusg.github.io/warpai/

## Architecture & Layout

- **`_config.yml`**: Site configuration (baseurl: `/warpai`, plugins: `jekyll-sitemap`).
- **`_layouts/`**:
  - `default.html`: Root landing layout.
  - `docs.html`: Documentation sidebar layout.
- **`_includes/`**: Shared components (`header.html`, `footer.html`).
- **`_data/strings.yml`**: Localization / copy strings dictionary.
- **`docs/`**: English documentation markdown files.
- **`zh/docs/`**: Chinese documentation markdown files.
- **`index.html` & `zh/index.html`**: English and Chinese landing pages.
- **`assets/`**: Stylesheets (`css/`), scripts (`js/`), fonts (`fonts/`), and media assets (`images/`).

## Conventions & Rules

1. **Path Resolution**: Always respect `{{ site.baseurl }}` when formulating internal links, asset paths, and navigation anchors to ensure proper rendering under GitHub Pages path `/warpai`.
2. **Bilingual Sync**: When modifying features, documentation, or landing page copy in English (`docs/`, `index.html`), evaluate and synchronize corresponding changes to Chinese (`zh/docs/`, `zh/index.html`) and `_data/strings.yml`.
3. **Asset Organization**: Place all static assets under `assets/` categorized into `css/`, `js/`, `fonts/`, or `images/`.
4. **Minimal Dependencies**: Use vanilla JS and native CSS; avoid introducing heavy external frameworks or build tooling unless required.
5. **Code & Commit Style**: Use English for code, commits, and comments. Follow ponytail principle: keep changes minimal, direct, and zero-bloat.
