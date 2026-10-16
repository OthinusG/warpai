---
layout: docs
lang: zh
title: 20+ 款 CLI Agent 安装与配置指南
description: "Warpai 原生支持的 20+ 款主流命令行 AI 编码 Agent（Claude Code、Codex、Gemini、Cursor、Antigravity 等）跨平台安装、鉴权与极速配置完全手册。"
section: Agent 生态
doc_id: cli-agents
prev_url: /docs/using-warpai-agents.html
prev_title: 多 Agent 协同工作流
next_url: /docs/mcp-api.html
next_title: 原生 MCP 协同工具体系
---

本指南提供 Warpai 原生深度集成的 20+ 款命令行 AI 编码 Agent 在 **macOS**、**Windows** 与 **Linux** 环境下的安装命令、账号鉴权步骤与运行说明。

<div class="docs-shot-placeholder" data-shot="SHOT-25">
  <div class="shot-header">
    <span class="shot-badge">SHOT-25 (P2)</span>
    <span class="shot-title">20+ 款 CLI 智能体生态运行实景</span>
  </div>
  <p class="shot-desc">20+ 款 CLI 智能体生态：Warpai 窗口中多标签/分屏同时运行 Claude Code、OpenAI Codex、Antigravity、Gemini 等不同智能体 CLI，展示零配置状态感知与环境隔离。</p>
  <div class="shot-meta">
    <span>建议尺寸: 1440x810 (16:9)</span>
    <span>目标路径: docs/images/cli-agents-ecosystem.png</span>
    <span class="shot-tag">截图待补充 · 对应表格 docs/SCREENSHOT-INVENTORY.xlsx</span>
  </div>
</div>

---

## 快速参考索引表

<div class="table-wrapper">
<table>
  <thead>
    <tr>
      <th>Agent CLI</th>
      <th>终端命令</th>
      <th>macOS / Linux 安装命令</th>
      <th>Windows 安装命令</th>
      <th>登录与鉴权方式</th>
    </tr>
  </thead>
  <tbody>
    <tr>
      <td><strong>Claude Code</strong></td>
      <td><code>claude</code></td>
      <td><code>npm i -g @anthropic-ai/claude-code</code></td>
      <td><code>npm i -g @anthropic-ai/claude-code</code></td>
      <td><code>claude</code> (浏览器 OAuth 授权)</td>
    </tr>
    <tr>
      <td><strong>OpenAI Codex</strong></td>
      <td><code>codex</code></td>
      <td><code>npm i -g @openai/codex</code></td>
      <td><code>npm i -g @openai/codex</code></td>
      <td><code>codex login</code> 或环境变量</td>
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
      <td>Cursor IDE 命令面板安装</td>
      <td>添加 Cursor bin 至 <code>%PATH%</code></td>
      <td>自动复用 Cursor 桌面端会话凭据</td>
    </tr>
    <tr>
      <td><strong>Antigravity</strong></td>
      <td><code>agy</code></td>
      <td>官方安装脚本 / 包管理</td>
      <td><code>scoop install agy</code></td>
      <td><code>agy login</code></td>
    </tr>
    <tr>
      <td><strong>Qoder</strong></td>
      <td><code>qoder</code></td>
      <td>官方安装脚本</td>
      <td>官方 Windows 安装包</td>
      <td><code>qoder login</code></td>
    </tr>
    <tr>
      <td><strong>QoderCN</strong></td>
      <td><code>qodercn</code></td>
      <td>官方 QoderCN 脚本</td>
      <td>官方 QoderCN 安装包</td>
      <td><code>qodercn login</code></td>
    </tr>
    <tr>
      <td><strong>Trae CLI</strong></td>
      <td><code>trae</code></td>
      <td>Trae IDE 内置二进制</td>
      <td>Trae IDE 内置二进制</td>
      <td>自动复用 Trae 会话</td>
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
      <td>官方 GitHub Release 二进制</td>
      <td><code>winget install Block.Goose</code></td>
      <td>首次启动命令行内交互配置</td>
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

- **安装**：
  - macOS / Linux：`npm install -g @anthropic-ai/claude-code`
  - Windows：`npm install -g @anthropic-ai/claude-code`
- **鉴权**：在终端运行 `claude` 并完成浏览器 OAuth 网页授权。
- **在 Warpai 中运行**：在任意终端标签页直接输入 `claude` 即可，左侧面板自动点亮 Claude 图标并跟踪其执行/空闲状态。

---

## 2. OpenAI Codex (`codex`)

- **安装**：
  - macOS / Linux：`npm i -g @openai/codex`
  - Windows：`npm i -g @openai/codex`
- **鉴权**：运行 `codex login` 或配置环境变量 `export OPENAI_API_KEY="sk-..."`。
- **在 Warpai 中运行**：直接输入 `codex`。

---

## 3. Google Gemini CLI (`gemini`)

- **安装**：`npm install -g @google/gemini-cli`
- **鉴权**：运行 `gemini auth login` 或配置 `export GEMINI_API_KEY="AIzaSy..."`。
- **在 Warpai 中运行**：输入 `gemini`。

---

## 4. Cursor Agent CLI (`cursor-agent`)

- **安装**：
  - macOS：打开 Cursor 桌面端 > 按 `Cmd+Shift+P` > 选择 `Install 'cursor' command in PATH`。
  - Windows：将 `%LOCALAPPDATA%\Programs\cursor\resources\app\bin` 添加到系统 `%PATH%` 中。
- **鉴权**：自动继承本地 Cursor 桌面会话凭据。
- **在 Warpai 中运行**：输入 `cursor-agent`。

---

## 5. Google Antigravity (`agy`)

- **安装**：
  - macOS / Linux：`curl -fsSL https://antigravity.google/install.sh | bash`
  - Windows：`scoop install agy`
- **鉴权**：运行 `agy login` 完成鉴权。
- **在 Warpai 中运行**：输入 `agy`，左侧协作面板底部实时捕获并展示其模型额度与窗口重置倒计时。

---

## 6. DeepSeek Harness (`dsh-tui`)

<div class="docs-callout note">
  <div class="docs-callout-title">仅保留交互式 TUI 路径</div>
  <p>依据项目规范，仅支持并维护交互式终端界面 <code>dsh-tui</code>（Terminal User Interface）安装与运行路径。</p>
</div>

- **安装**：
  - macOS / Linux：`pip install --upgrade deepseek-harness`
  - Windows：`pip install --upgrade deepseek-harness`
- **鉴权**：设置环境变量 `export DEEPSEEK_API_KEY="sk-..."`（Windows PowerShell 下为 `$env:DEEPSEEK_API_KEY="..."`）。
- **在 Warpai 中运行**：在任意终端窗格输入 `dsh-tui`。

---

## 7. Qoder 与 QoderCN (`qoder`, `qodercn`)

- **国际版**：`curl -fsSL https://install.qoder.com/install.sh | bash`
- **国内专版 (QoderCN)**：`curl -fsSL https://install.qoder.cn/install.sh | bash`
- **鉴权**：`qoder login` 或 `qodercn login`。
- **在 Warpai 中运行**：Warpai 将其识别为独立客户端，在设置中提供独立开关。

---

## 8. Mistral Vibe (`vibe`)

- **安装**：`pip install --upgrade mistral-vibe`
- **鉴权**：`export MISTRAL_API_KEY="your-key"`
- **在 Warpai 中运行**：输入 `vibe`。

---

## 9. Block Goose (`goose`)

- **安装**：从官方 GitHub Releases 获取对应系统的预编译包。
- **在 Warpai 中运行**：输入 `goose session` 开启交互。

---

## 10. 更多已适配的命令行 Agent (10-20)

- **Hermes Agent (`hermes`)**：`pip install hermes-agent` &rarr; `export OPENAI_API_KEY="..."` &rarr; 运行 `hermes`。
- **xAI Grok Build (`grok`)**：`npm i -g @xai/grok-cli` &rarr; 运行 `grok login` &rarr; 运行 `grok`。
- **GitHub Copilot CLI (`copilot`)**：`gh extension install github/gh-copilot` &rarr; `gh auth login` &rarr; 运行 `copilot`。
- **Pi CLI (`pi`)**：`npm i -g @pi-dev/cli` &rarr; `pi login` &rarr; 运行 `pi`。
- **oh-my-pi (`omp`)**：`npm i -g oh-my-pi` &rarr; `omp auth` &rarr; 运行 `omp`。
- **OpenCode CLI (`opencode`)**：`pip install opencode-cli` &rarr; `opencode auth` &rarr; 运行 `opencode`。
- **Amp / Amplified (`amp`)**：`npm i -g @ampt/cli` &rarr; `amp login` &rarr; 运行 `amp`。
- **Auggie AI Agent (`auggie`)**：`pip install auggie-agent` &rarr; `auggie login` &rarr; 运行 `auggie`。
- **Droid Coding Agent (`droid`)**：`npm i -g @droid-ai/cli` &rarr; `droid auth` &rarr; 运行 `droid`。
- **Trae CLI (`trae`)**：通过 Trae IDE 安装包内置提取并直接调用。
