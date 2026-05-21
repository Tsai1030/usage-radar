<div align="center">

# usage-radar

**Claude Code と OpenAI Codex CLI の使用量をデスクトップから一目で確認 — ダッシュボードに切り替える必要なし。**

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

## なぜこれを作ったのか

Anthropic と OpenAI はどちらもサブスクリプションの使用量をブラウザの中に隠しています。**Claude Code** や **OpenAI Codex CLI** のヘビーユーザーは、デスクトップ上で一目で分かるインジケータが欲しいはずです — タブ切り替え不要、ログイン不要、ブラウザ拡張不要。

**usage-radar** は画面右上に固定される小さな角丸カードで、ローカル CLI ログを読み取り、セッションと週間の進捗を表示します。それだけです。

## 機能

- 🎯 **2 つのソースをワンクリック切り替え** — Codex は OpenAI グリーン、Claude は Anthropic ウォームオレンジ。
- 🪶 **超軽量** — Tauri 2 + React。リリースバイナリ約 10 MB、RAM 約 30 MB。
- 🔒 **完全ローカル** — バックエンド・テレメトリ・ログインなし。パーサーは使用量フィールドのみ読み込み、**プロンプトやツール出力は一切読み込みません**。
- 🎛️ **タブ対応の設定** — ⚙ ボタンが Claude / Codex タブごとに異なる内容を表示。カードは自動でサイズ調整。
- 🎚️ **キャリブレーション** — Anthropic ダッシュボードの % を入力するだけで Claude バーが整合します。
- 🏷️ **データの新鮮度表示** — CLI が古い場合 `stale · 17h ago` タグが出ます。
- 🟢 **システムトレイ対応** — トレイに最小化、クリックで復元。右クリックで Show / Hide / Settings / Quit。
- 📌 **常に最前面 + 枠なし** — ウィンドウではなくフローティングウィジェットとして振る舞います。

## クイックスタート

### 一般ユーザー向け

事前ビルド済みのインストーラーは [Releases ページ](https://github.com/Tsai1030/usage-radar/releases) に順次公開されます。v0.1 リリース前は以下の開発者向け手順を使ってください。

リリース後:
1. `.msi`(Windows)/ `.dmg`(macOS)/ `.AppImage`(Linux)をダウンロード。
2. ダブルクリックでインストール。
3. スタートメニュー / アプリケーションから起動。
4. カードが右上に表示され、トレイアイコンが時計の横に出ます。

### 開発者向け

必要なツール 2 つ(1 回だけインストールすれば、全プロジェクトで共有):

- **Rust** ツールチェーン
- **bun** パッケージマネージャー

<details>
<summary><b>Windows</b></summary>

```powershell
# 1. ツールのインストール(初回のみ、グローバル)
winget install --id Rustlang.Rustup
winget install --id Oven-sh.Bun

# 2. Clone
git clone https://github.com/Tsai1030/usage-radar.git
cd usage-radar
```

その後、エクスプローラーで **`start.bat` をダブルクリック**するだけ。

`start.bat` は PowerShell の ExecutionPolicy Bypass で `scripts/start.ps1` を実行するので、ポリシー設定は不要です。Rust と bun を確認し、必要なら `bun install` を実行してからアプリを起動します。

初回 Rust コンパイルは約 1 GB ダウンロード + 5〜10 分。その後は秒単位で起動します。

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

### インストーラーをビルド

```powershell
.\scripts\build.ps1       # Windows
# または
bun run tauri build       # 任意のプラットフォーム
```

出力は `src-tauri/target/release/bundle/` に出ます。

## 使い方

1. **起動**: カードが右上に出現、トレイアイコンが時計の横に出ます。
2. **ソース切り替え**: 2 つのタブ(Codex / Claude)をクリック。
3. **非表示**: `−` ボタンでトレイに最小化(トレイアイコンクリックで復元)。
4. **設定**: `⚙` ボタン。タブごとに専用パネル:
   - **Claude**: サブスクリプション層(Pro / Max 5× / Max 20×)を選択。Anthropic web の % を入力するとキャリブレーション。
   - **Codex**: CLI データが古い場合、OpenAI ダッシュボードの % を入力して上書き。空欄 = ライブ値を使用。
5. **終了**: トレイ右クリックメニューから Quit。

## キャリブレーション

⚙ ボタンはタブごとに異なる内容を表示し、カードのサイズも自動調整されます。

<p align="center">
  <img src="../public/Claude_settings.png" alt="Claude settings" width="260">
  &nbsp;&nbsp;
  <img src="../public/Codex_settings.png"  alt="Codex settings"  width="260">
</p>

**Claude には公開された使用量 API がない**ため、バーは「推定値」です。公式ダッシュボードに合わせるには:

1. Anthropic web ダッシュボードを開き、**session %** と **weekly %** を確認。
2. Claude 設定の **Session %** / **Weekly %** にその数字を入力。
3. アプリはローカル count をその % で割り、真の cap を計算して保存。
4. Claude を使い続けるとバーは比例で上昇。ずれを感じたら再入力するだけ。

**Codex** は CLI が稼働中なら本物の `rate_limits` を使うので数字は正確です。最近 web/IDE 経由で使っていた(CLI データが古い)場合、Codex 設定で OpenAI ダッシュボードの % を入力して上書き、空欄に戻すとライブに戻ります。

## プライバシー

設計レベルで**ローカル限定**のツールです:

- **バックエンドなし、テレメトリなし、自動更新の phone home なし。**
- Claude パーサーが読むのは: `type`, `userType`, `isSidechain`, `timestamp`, `message.usage`(トークン数のみ)。
- Codex パーサーが読むのは: `rate_limits` オブジェクト(パーセント、ウィンドウ分数、リセット時刻)。
- **絶対に読まない**: プロンプト内容、ツール出力、ファイル内容、シークレット類。
- 設定は `~/.usage-radar/settings.json` に保存(ローカルファイル)。

オープンソースなので自分で確認できます。

## ライセンス

[MIT](../LICENSE) — お好きにどうぞ。
