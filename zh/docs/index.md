---
layout: docs
lang: zh
title: 文档概览
description: "Warpai 官方文档概览：专为自主编码 Agent 打造的本地优先、GPU 加速多 Agent 原生终端。"
section: 快速入门
doc_id: overview
next_url: /docs/install.html
next_title: 安装指南
---

欢迎查阅 **Warpai** 官方技术文档 —— 专为多 Agent 深度协同打造的本地优先、高隐私原生终端工作空间。

Warpai 将独立的 CLI Agent 会话提升为第一公民工作单元。针对代码工程，Warpai 提供严格的 **Git Worktree 并发编码模式**，支持在物理隔离的分支中并行开发不同功能并执行自动化对等评审；针对泛行业与非 Coding 场景，重磅提供 **Project 全向协同模式**，打破单一指挥链，让所有 Agent 自由互通消息与任务，在科研论文、视觉设计、量化金融、专利法务等场景中高效攻坚。

<div class="docs-shot-placeholder" data-shot="SHOT-01">
  <div class="shot-header">
    <span class="shot-badge">SHOT-01 (P0)</span>
    <span class="shot-title">Warpai All-in-One Hero Workspace</span>
  </div>
  <p class="shot-desc">三栏全景视觉：左侧 Agent 协同面板（多 Agent 在线状态与任务池）、中间 GPU 终端执行代码块（Claude Warm Light 主题 + 垂直标签页）、右侧展示 Project Explorer 与代码文件。</p>
  <div class="shot-meta">
    <span>建议尺寸: 1920x1080 (16:9)</span>
    <span>目标路径: docs/images/warpai-hero-showcase.png</span>
    <span class="shot-tag">截图待补充 · 对应表格 docs/SCREENSHOT-INVENTORY.xlsx</span>
  </div>
</div>

---

## 快速导航

<div class="table-wrapper">
<table>
  <thead>
    <tr>
      <th>分类</th>
      <th>文档章节</th>
      <th>内容概述</th>
    </tr>
  </thead>
  <tbody>
    <tr>
      <td><strong>快速入门</strong></td>
      <td><a href="{{ '/zh/docs/install.html' | relative_url }}">安装指南</a></td>
      <td>原生安装包下载（macOS Apple 芯片与 Windows x64）、系统安全性信任说明、SSH 远程伴侣配置。</td>
    </tr>
    <tr>
      <td></td>
      <td><a href="{{ '/zh/docs/quick-start.html' | relative_url }}">快速上手 (5分钟)</a></td>
      <td>5 分钟极速指引：启动你的第一个 Agent、快速熟悉工作空间布局与额度监控。</td>
    </tr>
    <tr>
      <td><strong>核心架构</strong></td>
      <td><a href="{{ '/zh/docs/concepts.html' | relative_url }}">核心概念与安全保障</a></td>
      <td>深入解析守护式 PTY 交付、VT100 内存屏幕观察器、Git Worktree 物理隔离与 Project 全向协同模式。</td>
    </tr>
    <tr>
      <td></td>
      <td><a href="{{ '/zh/docs/features.html' | relative_url }}">全景特性与原版对比</a></td>
      <td>Warpai 与上游 Warp 原版的深度功能对比：彻底剥离遥测、零云端依赖与原生多 Agent 底座。</td>
    </tr>
    <tr>
      <td><strong>使用指南</strong></td>
      <td><a href="{{ '/zh/docs/using.html' | relative_url }}">使用指南总览</a></td>
      <td>面向日常开发、代码审阅、多任务并行的顶层工作流指引。</td>
    </tr>
    <tr>
      <td></td>
      <td><a href="{{ '/zh/docs/project-mode.html' | relative_url }}">Project 模式实战手册</a></td>
      <td>重点扶持非 Coding 场景：打破层级、全员互通消息与任务，涵盖科研论文、视觉设计、金融量化等 6+ 场景实战教程。</td>
    </tr>
    <tr>
      <td></td>
      <td><a href="{{ '/zh/docs/worktree-mode.html' | relative_url }}">Worktree 模式完全手册</a></td>
      <td>面向 Git 代码仓库的大规模并行开发：物理工作树隔离、多分支并行编码、Coordinator 统筹与自动化对等评审。</td>
    </tr>
    <tr>
      <td></td>
      <td><a href="{{ '/zh/docs/using-warpai-agents.html' | relative_url }}">多智能体协同架构</a></td>
      <td>Worktree 并发编码模式与 Project 泛行业全向网状协同双架构原理与机制。</td>
    </tr>
    <tr>
      <td></td>
      <td><a href="{{ '/zh/docs/using-terminal.html' | relative_url }}">终端与命令块深入</a></td>
      <td>命令块交互、智能命令补全、垂直标签页、内置轻量 Vim 编辑器与实时 Markdown 预览。</td>
    </tr>
    <tr>
      <td><strong>Agent 生态</strong></td>
      <td><a href="{{ '/zh/docs/cli-agents.html' | relative_url }}">20+ 款 CLI Agent 配置</a></td>
      <td>Claude Code、Codex、Gemini、Antigravity、Cursor 等 20+ 款 Agent CLI 的安装与认证完整手册。</td>
    </tr>
    <tr>
      <td></td>
      <td><a href="{{ '/zh/docs/mcp-api.html' | relative_url }}">原生 MCP 协同工具体系</a></td>
      <td>38+ 项内置 MCP 本地协议工具（<code>warp-lite-communication</code>）接口参考。</td>
    </tr>
    <tr>
      <td><strong>配置与帮助</strong></td>
      <td><a href="{{ '/zh/docs/config.html' | relative_url }}">配置项与外观主题</a></td>
      <td><code>settings.toml</code> 参数详解、外观主题切换、快捷键绑定与长任务防止休眠（Keep-Awake）。</td>
    </tr>
    <tr>
      <td></td>
      <td><a href="{{ '/zh/docs/troubleshooting.html' | relative_url }}">故障排查与常见问题</a></td>
      <td>环境自检清单、常见终端报错排查、SSH 连通性与权限问题诊断。</td>
    </tr>
  </tbody>
</table>
</div>

---

## 核心设计哲学

1. **本地优先与纯粹隐私**：零遥测埋点、零云端账号壁垒、不锁定专有闭源模型。你的代码、Shell 命令与 Agent 交互内容均保留在本地受控环境中，绝不在未经许可的情况下外传。
2. **开放无界的 Agent 生态**：不强推任何内置单一模型订阅。直接复用你熟悉的 CLI Agent 工具（Claude Code、Codex、Cursor、Gemini 等），沿用现有 API 密钥与授权。
3. **双架构驱动规模化协同**：严肃的代码工程借助物理隔离的 Git Worktree 独立开发不同功能；跨领域的泛行业协作通过 Project 模式打破层级、全员互通消息与任务，构筑多智能体并行交付的坚实基石。
