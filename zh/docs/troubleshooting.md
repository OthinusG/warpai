---
layout: docs
lang: zh
title: 故障排查与常见问题
description: "Warpai 桌面端运行排查、Gatekeeper/SmartScreen 拦截解决、Agent 状态感知诊断、SSH 远程连接与配额查询问题速查。"
section: 配置与帮助
doc_id: troubleshooting
prev_url: /docs/config.html
prev_title: 配置项与外观主题
---

本指南汇集桌面端安装、Agent 通信、SSH 远程伴侣以及额度大盘统计中的高频问题与诊断解决方案。

---

## 安装与首次启动常见问题

### macOS 提示：“无法打开 Warpai，因为无法验证开发者”
- **产生原因**：Warpai 遵循开源发行模式，使用临时签名（Ad-hoc Signed），暂未加入苹果付费公证。
- **解决方案 1（系统设置授权）**：打开 **系统设置 > 隐私与安全性**，向下滑动至“安全性”小节，找到被拦截的 Warpai，点击 **“仍要打开”（Open Anyway）**。
- **解决方案 2（终端命令清除隔离位）**：在系统自带终端中执行命令以清除隔离属性：
  ```bash
  xattr -d com.apple.quarantine /Applications/Warpai.app
  ```

### Windows 提示：“Windows 已保护你的电脑”（SmartScreen 拦截）
- **产生原因**：新发行的开源安装包在微软 Defender 云端信誉库中建立信誉需要周期。
- **解决方案**：点击拦截提示中的 **“更多信息”**，随后点击出现的 **“仍要运行”** 按钮即可继续完成安装。

---

## Agent 感知与多智能体协同排查

### 我在终端中正常启动了 CLI Agent，但左侧协同面板中没有出现它。
1. **确认功能总开关已开启**：打开 **设置 > 功能（Features） > Agent communication**，确保主开关处于开启状态。
2. **确认特定 Agent 已被勾选**：确保列表中对应你所用 Agent（如 `Claude Code`、`Codex`、`Gemini` 等）的复选框已勾选。
3. **新建终端标签页测试**：若你是在已有标签页中打开 Agent 后才在设置中开启该功能，请新建一个终端标签页（`Cmd + T` / `Ctrl + T`）后重新运行 Agent。底层环境参数（`WARP_AGENT_ENDPOINT`）在窗格初始化时注入。
4. **自定义别名或重命名执行文件**：若你自定义了可执行文件名称或使用了别名（Alias），请在设置中通过 **Custom Agent** 进行自定义注册。

### Agent 任务已完成，但队列中的下一个待办任务迟迟没有自动执行。
Warpai 采用 **守护式 PTY 交付（Guarded PTY Delivery）** 保护终端输入安全：
1. **终端提示符有未发送草稿**：若你在终端命令行中键入了字符但尚未按下回车，Warpai 会主动挂起自动化任务投递，绝不冲撞覆盖你的输入。清空输入行后即可立即恢复。
2. **等待人工确认授权（`WaitingApproval`）**：若 Agent 触发了敏感系统调用（如请求运行 `rm` 删除或执行写操作），处于等待你输入 `[y/n]` 确认的状态，Warpai 绝不会代你按回车，请直接在终端中手动确认或拒绝。
3. **静默观察窗口**：Warpai 在终端输出停顿大约 300 毫秒后，方会最终裁定 Agent 回到真正的 `Idle` 状态。

---

## 远程 SSH 与 WSL 2 疑难排查

### SSH 连接进入服务器后，右侧项目文件树没有显示远程文件。
1. **执行目录切换（`cd`）**：远程伴侣与项目文件树由动态路径感知触发。连入服务器后，请在终端中进入具体工程目录（例如 `cd /var/www/my-project`）。
2. **检查远端用户家目录权限**：Warpai 将 Companion 部署于远端用户家目录 `~/.config/warpai/` 下，请确保远端家目录具备读写权限（权限为 `0700` 或 `0755`）。
3. **检查 SFTP 子系统支持**：确保远程服务器的 OpenSSH 服务启用了标准的 SFTP 传输子系统。

### Windows 系统下 WSL 2 伴侣无法正常连通。
1. **检查 WSL 架构版本**：Warpai 要求使用 **WSL 2**。在 PowerShell 中运行 `wsl -l -v`，确认你的 Linux 发行版 Version 标号为 2，而非 1。
2. **WSL 功能开关**：确认在 **设置 > 功能（Features） > WSL Integration** 中已开启 WSL 支持。

---

## 本地 Git 与工作树问题

### 提示报错：“System Git executable not found on PATH”
- **产生原因**：虽然日常普通终端使用和远端 SSH 伴侣拥有完全内嵌的私有 Git 运行时，但宿主机的**本地工作树模式（Worktree Mode）**与**本地 Git 差异审阅**仍需要宿主机操作系统的 `git` 命令以解析仓库元数据。
- **解决方案**：在电脑中安装 Git（macOS 可运行 `xcode-select --install`，Windows 可从 Git 官网安装 Git for Windows），随后重启 Warpai。

### 某个工作树目录提示处于锁定或冲突状态。
- Warpai 底层直接调用 Git 原生工作树机制。若你在文件系统中手动删除了某个 worktree 文件夹，可在终端中执行清理：
  ```bash
  git worktree prune
  ```

---

## 模型用量大盘与配额统计

### 为什么某个 Agent 账号显示“Unavailable”或无百分比？
- **真实数据呈现原则**：Warpai 坚决遵循诚实数据指标。若第三方服务商的 API 没有返回有效额度，Warpai 会诚实显示 `"Unavailable"`，绝不捏造虚假的“0%”或“100%”。
- **确认本地 CLI 已完成鉴权**：确保该 Agent CLI 已在本机成功登录过一次并能正常发起会话。
- **Google Antigravity**：需要 `agy` 客户端版本 `>= 1.1.11` 才能完整解析本地 `/usage` 配额指令。
- **代理配置**：用量请求支持 macOS 自动代理配置（PAC），请确认当前代理能连接服务商的用量接口。
- **Qoder / QoderCN**：保持原厂 CLI 已安装并完成登录。用量通过本地守护进程协议读取，同时兼容旧版 CLI。
