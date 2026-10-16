---
layout: docs
lang: zh
title: 配置项与外观主题
description: "Warpai settings.toml 配置文件详解、经典 Claude Warm Light 主题外观、快捷键速查与 Agent 通信配置项。"
section: 配置与帮助
doc_id: config
prev_url: /docs/mcp-api.html
prev_title: 原生 MCP 协同工具体系
next_url: /docs/troubleshooting.html
next_title: 故障排查与常见问题
---

Warpai 既提供直观的可视化设置面板（macOS 上按 `Cmd + ,`，Windows 上按 `Ctrl + ,`），也支持声明式的 `settings.toml` 纯文本配置文件。所有设置与数据均严格保存在本地用户目录中。

---

## 配置文件路径

- **macOS / Linux**：`~/.config/warpai/settings.toml`
- **Windows**：`%USERPROFILE%\.config\warpai\settings.toml`

Warpai 实时监听该文件的变动。在 `settings.toml` 中手动修改或在 GUI 设置界面中操作，均能实现双向实时热同步，无需重启应用（界面语言切换除外）。

---

## 设置：Agent 通信开关（Agent Communication）

若要允许本地 Agent 通过内置 MCP 总线相互感知与协作：
1. 打开 **设置 > 功能（Features） > Agent communication**。
2. 将总开关切换为 **开启（ON）**。
3. 勾选你本机已安装的 CLI Agent。

<div class="docs-shot-placeholder" data-shot="SHOT-20">
  <div class="shot-header">
    <span class="shot-badge">SHOT-20 (P2)</span>
    <span class="shot-title">Settings: Agent Communication Toggle</span>
  </div>
  <p class="shot-desc">设置面板中的 Agent 通信配置页：展示 Master 开关开启状态，以及列出的 20+ 款主流 Agent 勾选清单（Claude, Codex, Gemini 等）。</p>
  <div class="shot-meta">
    <span>建议尺寸: 1100x750</span>
    <span>目标路径: docs/images/settings-agent-communication.png</span>
    <span class="shot-tag">截图待补充 · 对应表格 docs/SCREENSHOT-INVENTORY.xlsx</span>
  </div>
</div>

---

## 外观主题定制：Claude Warm Light 经典暖光 与 Ink 纯黑暗调

Warpai 原生搭载两款核心主题：默认的 **Claude Warm Light**（温暖纸张质感、极佳可读性的浅色暖调）与 **Ink Pure Dark**（纯黑极客底色、高对比度霓虹/柔光暗调）。

<div class="docs-shot-placeholder" data-shot="SHOT-21">
  <div class="shot-header">
    <span class="shot-badge">SHOT-21 (P2)</span>
    <span class="shot-title">Claude Warm Light 经典暖光浅色主题全景</span>
  </div>
  <p class="shot-desc">默认 Claude Warm Light 经典暖光浅色主题全景展示：呈现舒适优雅的暖调质感与高对比度语法配色。</p>
  <div class="shot-meta">
    <span>建议尺寸: 1440x810 (16:9)</span>
    <span>目标路径: docs/images/appearance-claude-warm-light.png</span>
    <span class="shot-tag">截图待补充 · 对应表格 docs/SCREENSHOT-INVENTORY.xlsx</span>
  </div>
</div>

<div class="docs-shot-placeholder" data-shot="SHOT-28">
  <div class="shot-header">
    <span class="shot-badge">SHOT-28 (P2)</span>
    <span class="shot-title">Ink Pure Dark 纯黑极客暗色主题全景</span>
  </div>
  <p class="shot-desc">Ink 纯黑经典暗色主题全景：展示 Warpai 切换至 Ink Pure Dark 主题下的深黑底色、高对比度霓虹/柔光语法高亮及暗调协同面板。</p>
  <div class="shot-meta">
    <span>建议尺寸: 1440x810 (16:9)</span>
    <span>目标路径: docs/images/appearance-ink-pure-dark.png</span>
    <span class="shot-tag">截图待补充 · 对应表格 docs/SCREENSHOT-INVENTORY.xlsx</span>
  </div>
</div>

---

## 核心常用快捷键速查表

<div class="table-wrapper">
<table>
  <thead>
    <tr>
      <th>操作指令</th>
      <th>macOS 快捷键</th>
      <th>Windows 快捷键</th>
    </tr>
  </thead>
  <tbody>
    <tr>
      <td><strong>新建标签页</strong></td>
      <td><code>Cmd + T</code></td>
      <td><code>Ctrl + T</code></td>
    </tr>
    <tr>
      <td><strong>关闭当前标签页</strong></td>
      <td><code>Cmd + W</code></td>
      <td><code>Ctrl + W</code></td>
    </tr>
    <tr>
      <td><strong>左右垂直分屏</strong></td>
      <td><code>Cmd + D</code></td>
      <td><code>Ctrl + Shift + D</code></td>
    </tr>
    <tr>
      <td><strong>上下水平分屏</strong></td>
      <td><code>Cmd + Shift + D</code></td>
      <td><code>Ctrl + Shift + E</code></td>
    </tr>
    <tr>
      <td><strong>切换垂直/水平标签栏</strong></td>
      <td><code>Cmd + Shift + V</code></td>
      <td><code>Ctrl + Shift + V</code></td>
    </tr>
    <tr>
      <td><strong>全局命令面板</strong></td>
      <td><code>Cmd + P</code></td>
      <td><code>Ctrl + P</code></td>
    </tr>
    <tr>
      <td><strong>命令执行历史搜索</strong></td>
      <td><code>Cmd + R</code></td>
      <td><code>Ctrl + R</code></td>
    </tr>
    <tr>
      <td><strong>显隐项目文件树</strong></td>
      <td><code>Cmd + B</code></td>
      <td><code>Ctrl + B</code></td>
    </tr>
    <tr>
      <td><strong>打开偏好设置</strong></td>
      <td><code>Cmd + ,</code></td>
      <td><code>Ctrl + ,</code></td>
    </tr>
  </tbody>
</table>
</div>
