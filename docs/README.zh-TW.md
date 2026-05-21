<div align="center">

# usage-radar

**從桌面看 Claude Code 與 OpenAI Codex CLI 的用量 — 不用切視窗、不用登入。**

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

## 為什麼做這個

Anthropic 和 OpenAI 都把訂閱用量藏在瀏覽器的某個分頁裡。**Claude Code** 或 **OpenAI Codex CLI** 的重度使用者，會希望桌面上有一個一眼就能看見的指示器 — 不切視窗、不登入、不用裝瀏覽器擴充。

**usage-radar** 就是釘在螢幕右上角的一張圓角小卡，讀本機 CLI log，秀 session 與 weekly 進度。就這樣。

## 功能

- 🎯 **兩個來源、一眼看完** — Codex 用 OpenAI 綠、Claude 用 Anthropic 暖橘，tab 一鍵切換。
- 🪶 **極輕量** — Tauri 2 + React，release binary 約 10 MB，RAM 約 30 MB。
- 🔒 **完全本機** — 無 backend、無 telemetry、無登入。parser 只讀額度欄位，**從不讀 prompt 或工具輸出**。
- 🎛️ **Tab 感知設定** — 齒輪在 Claude / Codex 各 tab 上顯示對應內容，卡片自動伸縮。
- 🎚️ **校準功能** — 用 Anthropic dashboard 的 % 校準 Claude 的 bar。
- 🏷️ **資料新鮮度** — 當 CLI 資料舊了會顯示 `stale · 17h ago`。
- 🟢 **系統匣** — 縮小到 tray、點擊還原；右鍵選單有 Show / Hide / Settings / Quit。
- 📌 **Always-on-top + 無框** — 像浮動 widget，不像視窗。

## 快速開始

### 一般使用者

之後會在 [Releases 頁面](https://github.com/Tsai1030/usage-radar/releases) 提供預先打包的安裝檔。v0.1 release 之前請走下面開發者流程。

未來有 release 之後：
1. 下載 `.msi`（Windows）/ `.dmg`（macOS）/ `.AppImage`（Linux）。
2. 雙擊安裝。
3. 從開始選單 / 應用程式啟動。
4. 卡片自動跳出在螢幕右上、tray icon 出現在時鐘旁。

### 開發者

需要兩個工具（一次安裝、全域共用）：

- **Rust** toolchain
- **bun** 套件管理器

<details>
<summary><b>Windows (PowerShell)</b></summary>

```powershell
# 1. 安裝工具（一次性）
winget install --id Rustlang.Rustup
winget install --id Oven-sh.Bun

# 2. Clone 並啟動
git clone https://github.com/Tsai1030/usage-radar.git
cd usage-radar
.\scripts\start.ps1
```

`start.ps1` 會檢查必要工具、安裝依賴、然後啟動 app。
第一次 Rust 編譯會下載約 1 GB、花 5–10 分鐘，之後啟動是秒級。

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

### 打包安裝檔

```powershell
.\scripts\build.ps1       # Windows
# 或
bun run tauri build       # 任何平台
```

輸出在 `src-tauri/target/release/bundle/`。

## 使用說明

1. **啟動**：卡片出現右上角，tray icon 出現在時鐘旁。
2. **切來源**：點兩個 tab（Codex / Claude）。
3. **隱藏**：按 `−` 按鈕（會縮到 tray，點 tray icon 還原）。
4. **設定**：按 `⚙` 按鈕。每個 tab 有自己的面板：
   - **Claude**：選你的訂閱層級（Pro / Max 5× / Max 20×）做基準。可以輸入 Anthropic web 顯示的 % 來校準。
   - **Codex**：CLI 資料舊的時候，輸入 OpenAI dashboard 顯示的 % 覆寫。清空 = 使用 live 數值。
5. **離開**：tray 右鍵選 Quit。

## 校準

齒輪在不同 tab 顯示不同內容，卡片自動調整高度。

<p align="center">
  <img src="../public/Claude_settings.png" alt="Claude settings" width="260">
  &nbsp;&nbsp;
  <img src="../public/Codex_settings.png"  alt="Codex settings"  width="260">
</p>

**Claude 沒有公開的用量 API**，所以 bar 是「估算」。要對齊官方 dashboard：

1. 打開 Anthropic web dashboard，看 **session %** 和 **weekly %**。
2. 在 Claude 設定的 **Session %** / **Weekly %** 兩格填那個數字。
3. App 拿當下的本機 count 除以你的 %，算出真正的 cap 並儲存。
4. 你繼續用 Claude，bar 同比例往上。發現偏掉時再重填一次即可。

**Codex** 在 CLI 活躍時用真實的 `rate_limits`，數字是準的。如果你最近是用網頁/IDE（CLI 資料 stale），打開 Codex 設定填 OpenAI dashboard 的 % 覆寫，清空就回到 live。

## 隱私

這是徹底**本機只用**的工具：

- **無 backend、無 telemetry、無自動更新回報。**
- Claude parser 只讀：`type`, `userType`, `isSidechain`, `timestamp`, `message.usage`（token 計數）。
- Codex parser 只讀：`rate_limits` 物件（百分比、視窗長度、reset 時間）。
- **從不讀**：prompt 內容、工具輸出、檔案內容、任何 secret。
- 設定存在 `~/.usage-radar/settings.json`（本機檔）。

程式碼開源，你可以自己驗證。

## 授權

[MIT](../LICENSE) — 隨你用。
