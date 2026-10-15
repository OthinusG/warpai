---
layout: docs
title: 20+ CLI Agents Guide
description: "Complete cross-platform installation, authentication, and usage manual for 20+ command-line AI coding agents supported by Warpai."
section: Agent Ecosystem
doc_id: cli-agents
prev_url: /docs/using-warpai-agents.html
prev_title: Multi-Agent Workflows
next_url: /docs/mcp-api.html
next_title: Native MCP Tools API
---

This guide provides end-to-end instructions for installing, authenticating, and running the 20+ command-line AI coding agents supported natively by Warpai across **macOS**, **Windows**, and **Linux**, utilizing standard package managers (Homebrew, npm, pip, winget, scoop, etc.).

---

## Quick Reference Summary

<div class="table-wrapper">
<table>
  <thead>
    <tr>
      <th>Agent CLI</th>
      <th>Command</th>
      <th>Install (macOS)</th>
      <th>Install (Windows)</th>
      <th>Auth Command</th>
    </tr>
  </thead>
  <tbody>
    <tr>
      <td><strong>Claude Code</strong></td>
      <td><code>claude</code></td>
      <td><code>npm i -g @anthropic-ai/claude-code</code></td>
      <td><code>npm i -g @anthropic-ai/claude-code</code></td>
      <td><code>claude</code> (OAuth web)</td>
    </tr>
    <tr>
      <td><strong>OpenAI Codex</strong></td>
      <td><code>codex</code></td>
      <td><code>brew install codex</code></td>
      <td><code>npm i -g codex</code> / <code>winget</code></td>
      <td><code>codex login</code></td>
    </tr>
    <tr>
      <td><strong>Google Gemini CLI</strong></td>
      <td><code>gemini</code></td>
      <td><code>npm i -g @google/gemini-cli</code></td>
      <td><code>npm i -g @google/gemini-cli</code></td>
      <td><code>gemini auth login</code></td>
    </tr>
    <tr>
      <td><strong>Cursor Agent</strong></td>
      <td><code>cursor-agent</code></td>
      <td>Cursor IDE Command Palette</td>
      <td>Add Cursor bin to <code>%PATH%</code></td>
      <td>Shares Cursor login</td>
    </tr>
    <tr>
      <td><strong>Antigravity</strong></td>
      <td><code>agy</code></td>
      <td><code>brew tap google/antigravity &amp;&amp; brew install agy</code></td>
      <td><code>scoop install agy</code></td>
      <td><code>agy login</code></td>
    </tr>
    <tr>
      <td><strong>Qoder</strong></td>
      <td><code>qoder</code></td>
      <td>Official installer script</td>
      <td>Official Windows setup</td>
      <td><code>qoder login</code></td>
    </tr>
    <tr>
      <td><strong>QoderCN</strong></td>
      <td><code>qodercn</code></td>
      <td>Official QoderCN installer</td>
      <td>Official QoderCN setup</td>
      <td><code>qodercn login</code></td>
    </tr>
    <tr>
      <td><strong>Trae CLI</strong></td>
      <td><code>trae</code></td>
      <td>Trae IDE CLI bundle</td>
      <td>Trae IDE CLI bundle</td>
      <td>Shares Trae login</td>
    </tr>
    <tr>
      <td><strong>Mistral Vibe</strong></td>
      <td><code>vibe</code></td>
      <td><code>pip install mistral-vibe</code></td>
      <td><code>pip install mistral-vibe</code></td>
      <td><code>export MISTRAL_API_KEY=...</code></td>
    </tr>
    <tr>
      <td><strong>DeepSeek Harness</strong></td>
      <td><code>dsh-tui</code></td>
      <td><code>pip install deepseek-harness</code></td>
      <td><code>pip install deepseek-harness</code></td>
      <td><code>export DEEPSEEK_API_KEY=...</code></td>
    </tr>
    <tr>
      <td><strong>Goose</strong></td>
      <td><code>goose</code></td>
      <td><code>brew install block/goose/goose</code></td>
      <td><code>winget install Block.Goose</code></td>
      <td>Guided in CLI</td>
    </tr>
    <tr>
      <td><strong>Hermes Agent</strong></td>
      <td><code>hermes</code></td>
      <td><code>pip install hermes-agent</code></td>
      <td><code>pip install hermes-agent</code></td>
      <td><code>export OPENAI_API_KEY=...</code></td>
    </tr>
    <tr>
      <td><strong>xAI Grok Build</strong></td>
      <td><code>grok</code></td>
      <td><code>npm i -g @xai/grok-cli</code></td>
      <td><code>npm i -g @xai/grok-cli</code></td>
      <td><code>grok login</code></td>
    </tr>
    <tr>
      <td><strong>GitHub Copilot</strong></td>
      <td><code>copilot</code></td>
      <td><code>gh extension install github/gh-copilot</code></td>
      <td><code>gh extension install github/gh-copilot</code></td>
      <td><code>gh auth login</code></td>
    </tr>
    <tr>
      <td><strong>Pi CLI</strong></td>
      <td><code>pi</code></td>
      <td><code>npm i -g @pi-dev/cli</code></td>
      <td><code>npm i -g @pi-dev/cli</code></td>
      <td><code>pi login</code></td>
    </tr>
    <tr>
      <td><strong>oh-my-pi (OMP)</strong></td>
      <td><code>omp</code></td>
      <td><code>npm i -g oh-my-pi</code></td>
      <td><code>npm i -g oh-my-pi</code></td>
      <td><code>omp auth</code></td>
    </tr>
    <tr>
      <td><strong>OpenCode</strong></td>
      <td><code>opencode</code></td>
      <td><code>pip install opencode-cli</code></td>
      <td><code>pip install opencode-cli</code></td>
      <td><code>opencode login</code></td>
    </tr>
    <tr>
      <td><strong>Amp (Amplified)</strong></td>
      <td><code>amp</code></td>
      <td><code>npm i -g @ampt/cli</code></td>
      <td><code>npm i -g @ampt/cli</code></td>
      <td><code>amp login</code></td>
    </tr>
    <tr>
      <td><strong>Auggie Agent</strong></td>
      <td><code>auggie</code></td>
      <td><code>pip install auggie-agent</code></td>
      <td><code>pip install auggie-agent</code></td>
      <td><code>auggie login</code></td>
    </tr>
    <tr>
      <td><strong>Droid Agent</strong></td>
      <td><code>droid</code></td>
      <td><code>npm i -g @droid-ai/cli</code></td>
      <td><code>npm i -g @droid-ai/cli</code></td>
      <td><code>droid auth</code></td>
    </tr>
  </tbody>
</table>
</div>

---

## 1. Anthropic Claude Code (`claude`)

- **Install**:
  - macOS / Linux: `npm install -g @anthropic-ai/claude-code` or `brew install claude-code`
  - Windows: `npm install -g @anthropic-ai/claude-code`
- **Auth**: Run `claude` and complete web browser OAuth login.
- **Run in Warpai**: Type `claude` in any terminal tab.

---

## 2. OpenAI Codex (`codex`)

- **Install**:
  - macOS: `brew install codex` or `npm i -g @openai/codex`
  - Windows: `npm i -g @openai/codex` or `winget install OpenAI.Codex`
- **Auth**: `codex login` or set `export OPENAI_API_KEY="sk-..."`
- **Run in Warpai**: Type `codex` in any terminal tab.

---

## 3. Google Gemini CLI (`gemini`)

- **Install**: `npm install -g @google/gemini-cli`
- **Auth**: `gemini auth login` or set `export GEMINI_API_KEY="AIzaSy..."`
- **Run in Warpai**: Type `gemini`.

---

## 4. Cursor Agent CLI (`cursor-agent`)

- **Install**:
  - macOS: Open Cursor IDE > `Cmd+Shift+P` > `Install 'cursor' command in PATH`.
  - Windows: Add `%LOCALAPPDATA%\Programs\cursor\resources\app\bin` to `%PATH%`.
- **Auth**: Inherits Cursor desktop session credentials automatically.
- **Run in Warpai**: Type `cursor-agent`.

---

## 5. Google Antigravity (`agy`)

- **Install**:
  - macOS: `brew tap google/antigravity && brew install agy`
  - Windows: `scoop bucket add antigravity https://github.com/google/antigravity-scoop && scoop install agy`
  - Linux: `curl -fsSL https://antigravity.google/install.sh | bash`
- **Auth**: `agy login`
- **Run in Warpai**: Type `agy`. Live quota metadata is reflected in the quota monitor footer.

---

## 6. DeepSeek Harness (`dsh-tui`)

<div class="docs-callout note">
  <div class="docs-callout-title">TUI Installation Pathway</div>
  <p>Per project instructions, only the interactive <code>dsh-tui</code> (Terminal User Interface) route is installed and documented.</p>
</div>

- **Install**:
  - macOS / Linux: `pip install --upgrade deepseek-harness` or `curl -fsSL https://raw.githubusercontent.com/deepseek-ai/deepseek-harness/master/install-tui.sh | bash`
  - Windows: `pip install --upgrade deepseek-harness` or `scoop install dsh-tui`
- **Auth**: `export DEEPSEEK_API_KEY="sk-..."` (or `$env:DEEPSEEK_API_KEY="..."` on Windows)
- **Run in Warpai**: Type `dsh-tui` in any terminal pane.

---

## 7. Qoder & QoderCN (`qoder`, `qodercn`)

- **Global**: `curl -fsSL https://install.qoder.com/install.sh | bash`
- **China Edition (QoderCN)**: `curl -fsSL https://install.qoder.cn/install.sh | bash`
- **Auth**: `qoder login` / `qodercn login`
- **Run in Warpai**: Discovered as independent clients with dedicated settings rows.

---

## 8. Mistral Vibe (`vibe`)

- **Install**: `pip install --upgrade mistral-vibe`
- **Auth**: `export MISTRAL_API_KEY="your-key"`
- **Run in Warpai**: Type `vibe`.

---

## 9. Block Goose (`goose`)

- **Install**:
  - macOS: `brew install block/goose/goose`
  - Windows: `winget install Block.Goose`
  - Linux: `curl -fsSL https://github.com/block/goose/releases/latest/download/goose-linux-x86_64.tar.gz | tar -xz -C /usr/local/bin`
- **Run in Warpai**: `goose session`

---

## 10. Additional Supported Agents (10-20)

- **Hermes Agent (`hermes`)**: `pip install hermes-agent` &rarr; `export OPENAI_API_KEY="..."` &rarr; `hermes`
- **xAI Grok Build (`grok`)**: `npm i -g @xai/grok-cli` &rarr; `grok login` &rarr; `grok`
- **GitHub Copilot CLI (`copilot`)**: `gh extension install github/gh-copilot` &rarr; `gh auth login` &rarr; `copilot`
- **Pi CLI (`pi`)**: `npm i -g @pi-dev/cli` &rarr; `pi login` &rarr; `pi`
- **oh-my-pi (`omp`)**: `npm i -g oh-my-pi` &rarr; `omp auth` &rarr; `omp`
- **OpenCode CLI (`opencode`)**: `pip install opencode-cli` &rarr; `opencode auth` &rarr; `opencode`
- **Amp / Amplified (`amp`)**: `npm i -g @ampt/cli` &rarr; `amp login` &rarr; `amp`
- **Auggie AI Agent (`auggie`)**: `pip install auggie-agent` &rarr; `auggie login` &rarr; `auggie`
- **Droid Coding Agent (`droid`)**: `npm i -g @droid-ai/cli` &rarr; `droid auth` &rarr; `droid`
- **Trae CLI (`trae`)**: Provided via Trae IDE executable bundle.
