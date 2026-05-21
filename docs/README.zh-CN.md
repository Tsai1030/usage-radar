<div align="center">

# usage-radar

**在桌面查看 Claude Code 与 OpenAI Codex CLI 的用量 — 无需切窗口、无需登录。**

[English](../README.md) · [繁體中文](README.zh-TW.md) · [简体中文](README.zh-CN.md) · [日本語](README.ja.md) · [한국어](README.ko.md)

[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](../LICENSE)
[![Built with Tauri](https://img.shields.io/badge/built%20with-Tauri%202-FFC131.svg)](https://tauri.app)
![React 18](https://img.shields.io/badge/react-18-61DAFB.svg)
![Rust stable](https://img.shields.io/badge/rust-stable-orange.svg)

<br>

<img src="../public/Codex.png"  alt="Codex tab"  width="260">
&nbsp;&nbsp;
<img src="../public/Claude.png" alt="Claude tab" width="260">

</div>

---

## 为什么做这个

Anthropic 和 OpenAI 都把订阅用量信息埋在浏览器某个页面里。**Claude Code** 或 **OpenAI Codex CLI** 的重度用户，希望桌面上有一个一眼能看的指示器 — 不用切窗口、不用登录、不需要浏览器扩展。

**usage-radar** 是钉在屏幕右上角的一张圆角小卡片,读取本机 CLI 日志,显示 session 与 weekly 进度。仅此而已。

## 功能

- 🎯 **两个来源、一眼看完** — Codex 用 OpenAI 绿、Claude 用 Anthropic 暖橙,tab 一键切换。
- 🪶 **极轻量** — Tauri 2 + React,release binary 约 10 MB,RAM 约 30 MB。
- 🔒 **完全本机** — 无 backend、无遥测、无登录。parser 只读额度字段,**从不读 prompt 或工具输出**。
- 🎛️ **Tab 感知设置** — 齿轮在 Claude / Codex 各 tab 上显示对应内容,卡片自动伸缩。
- 🎚️ **校准功能** — 用 Anthropic dashboard 的 % 校准 Claude 的进度条。
- 🏷️ **数据新鲜度** — 当 CLI 数据陈旧时会显示 `stale · 17h ago`。
- 🟢 **系统托盘** — 缩小到 tray、点击还原;右键菜单有 Show / Hide / Settings / Quit。
- 📌 **Always-on-top + 无边框** — 像浮动 widget,而不是窗口。

## 快速开始

### 普通用户

预编译安装包以后会发布在 [Releases 页面](https://github.com/Tsai1030/usage-radar/releases)。v0.1 release 之前请走下面开发者流程。

将来有 release 之后:
1. 下载 `.msi`(Windows)/ `.dmg`(macOS)/ `.AppImage`(Linux)。
2. 双击安装。
3. 从开始菜单 / 应用程序启动。
4. 卡片自动出现在屏幕右上角,tray 图标出现在时钟旁。

### 开发者

需要两个工具(一次安装、全局共用):

- **Rust** toolchain
- **bun** 包管理器

<details>
<summary><b>Windows (PowerShell)</b></summary>

```powershell
# 1. 安装工具(一次性)
winget install --id Rustlang.Rustup
winget install --id Oven-sh.Bun

# 2. Clone 并启动
git clone https://github.com/Tsai1030/usage-radar.git
cd usage-radar
.\scripts\start.ps1
```

`start.ps1` 会检查必要工具、安装依赖、然后启动 app。
第一次 Rust 编译会下载约 1 GB、花 5–10 分钟,之后启动是秒级。

</details>

<details>
<summary><b>macOS / Linux</b></summary>

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
curl -fsSL https://bun.sh/install | bash

git clone https://github.com/Tsai1030/usage-radar.git
cd usage-radar
bun install
bun run tauri dev
```

</details>

### 打包安装包

```powershell
.\scripts\build.ps1       # Windows
# 或
bun run tauri build       # 任何平台
```

输出在 `src-tauri/target/release/bundle/`。

## 使用说明

1. **启动**:卡片出现在右上角,tray 图标出现在时钟旁。
2. **切换来源**:点两个 tab(Codex / Claude)。
3. **隐藏**:按 `−` 按钮(会缩到 tray,点 tray 图标还原)。
4. **设置**:按 `⚙` 按钮。每个 tab 有自己的面板:
   - **Claude**:选订阅级别(Pro / Max 5× / Max 20×)做基准。可输入 Anthropic web 显示的 % 来校准。
   - **Codex**:CLI 数据陈旧时,输入 OpenAI dashboard 显示的 % 覆盖。清空 = 使用 live 数值。
5. **退出**:tray 右键菜单选 Quit。

## 校准

齿轮在不同 tab 显示不同内容,卡片自动调整高度。

<p align="center">
  <img src="../public/Claude_settings.png" alt="Claude settings" width="260">
  &nbsp;&nbsp;
  <img src="../public/Codex_settings.png"  alt="Codex settings"  width="260">
</p>

**Claude 没有公开的用量 API**,所以进度条是「估算」。想要对齐官方 dashboard:

1. 打开 Anthropic web dashboard,看 **session %** 和 **weekly %**。
2. 在 Claude 设置的 **Session %** / **Weekly %** 两格填那个数字。
3. App 拿当下的本机 count 除以你填的 %,算出真正的 cap 并保存。
4. 继续用 Claude,进度条同比例上升。发现偏离时再重填即可。

**Codex** 在 CLI 活跃时用真实的 `rate_limits`,数字是准的。如果你最近用的是网页/IDE(CLI 数据 stale),打开 Codex 设置填 OpenAI dashboard 的 % 覆盖,清空就回到 live。

## 隐私

这是彻底**仅本机**的工具:

- **无 backend、无遥测、无自动更新回连。**
- Claude parser 只读:`type`, `userType`, `isSidechain`, `timestamp`, `message.usage`(token 计数)。
- Codex parser 只读:`rate_limits` 对象(百分比、窗口长度、reset 时间)。
- **从不读**:prompt 内容、工具输出、文件内容、任何 secret。
- 设置存在 `~/.usage-radar/settings.json`(本机文件)。

代码开源,你可以自己验证。

## 许可证

[MIT](../LICENSE) — 随便用。
