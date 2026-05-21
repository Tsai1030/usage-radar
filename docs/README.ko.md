<div align="center">

# usage-radar

**Claude Code와 OpenAI Codex CLI 사용량을 데스크톱에서 한눈에 — 대시보드로 alt-tab 할 필요 없이.**

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

## 왜 만들었나

Anthropic과 OpenAI 모두 구독 사용량을 브라우저 탭 안에 숨겨두고 있습니다. **Claude Code** 또는 **OpenAI Codex CLI** 헤비 유저는 데스크톱에서 한눈에 볼 수 있는 지시기가 필요합니다 — 탭 전환 없이, 로그인 없이, 브라우저 확장 없이.

**usage-radar**는 화면 우상단에 고정되는 작은 둥근 카드로, 로컬 CLI 로그를 읽어 세션과 주간 진행률을 표시합니다. 그게 전부입니다.

## 기능

- 🎯 **두 개 소스, 한눈에** — Codex는 OpenAI 그린, Claude는 Anthropic 따뜻한 오렌지. 탭으로 전환.
- 🪶 **초경량** — Tauri 2 + React. 릴리스 바이너리 약 10 MB, RAM 약 30 MB.
- 🔒 **완전 로컬** — 백엔드 없음, 텔레메트리 없음, 로그인 없음. 파서는 할당량 필드만 읽고 **프롬프트나 도구 출력은 절대 읽지 않습니다**.
- 🎛️ **탭 인식 설정** — 톱니바퀴가 Claude / Codex 탭별로 다른 내용을 표시, 카드는 자동으로 크기 조정.
- 🎚️ **캘리브레이션** — Anthropic 대시보드의 %를 입력해 Claude 바를 정렬.
- 🏷️ **데이터 신선도** — CLI 데이터가 오래되면 `stale · 17h ago` 태그 표시.
- 🟢 **시스템 트레이** — 트레이로 최소화, 클릭으로 복원. 우클릭으로 Show / Hide / Settings / Quit.
- 📌 **항상 위 + 무프레임** — 윈도우가 아니라 플로팅 위젯처럼 동작.

## 빠른 시작

### 일반 사용자

[최신 릴리스](https://github.com/Tsai1030/usage-radar/releases/latest)에서 사전 빌드된 설치 파일을 다운로드하세요.

1. `.msi` 또는 NSIS `.exe`(Windows) 받기. macOS / Linux 설치 파일은 추후 릴리스에서 제공 예정. 그때까지는 아래 개발자 가이드 참고.
2. 더블 클릭해 설치. Windows SmartScreen이 *알 수 없는 게시자* 경고를 표시(코드 서명은 v0.3에서 예정) → **추가 정보 → 실행** 클릭.
3. 시작 메뉴에서 **Usage Radar** 실행.
4. 카드가 우상단에 나타나고, 트레이 아이콘이 시계 옆에 표시됩니다.

### 개발자

필요한 도구 두 가지 (한 번만 설치, 모든 프로젝트에서 공유):

- **Rust** 툴체인
- **bun** 패키지 매니저

<details>
<summary><b>Windows</b></summary>

```powershell
# 1. 도구 설치 (1회, 전역)
winget install --id Rustlang.Rustup
winget install --id Oven-sh.Bun

# 2. Clone
git clone https://github.com/Tsai1030/usage-radar.git
cd usage-radar
```

그 다음 파일 탐색기에서 **`start.bat` 더블 클릭**. 끝.

`start.bat`은 PowerShell ExecutionPolicy Bypass로 `scripts/start.ps1`을 실행하므로 정책 설정이 필요 없습니다. Rust와 bun을 확인하고, 필요시 `bun install`을 실행한 후 앱을 시작합니다.

첫 Rust 컴파일은 약 1 GB 다운로드 + 5~10분 소요. 이후 실행은 몇 초.

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

### 설치 파일 빌드

```powershell
.\scripts\build.ps1       # Windows
# 또는
bun run tauri build       # 모든 플랫폼
```

결과물은 `src-tauri/target/release/bundle/`에 생성됩니다.

## 사용법

1. **실행**: 카드가 우상단에 나타나고, 트레이 아이콘이 시계 옆에 표시됩니다.
2. **소스 전환**: 두 개의 탭(Codex / Claude) 클릭.
3. **숨기기**: `−` 버튼으로 트레이로 최소화(트레이 아이콘 클릭으로 복원).
4. **설정**: `⚙` 버튼. 각 탭마다 전용 패널:
   - **Claude**: 구독 등급(Pro / Max 5× / Max 20×) 선택. Anthropic web의 %를 입력해 캘리브레이션.
   - **Codex**: CLI 데이터가 오래된 경우, OpenAI 대시보드의 %를 입력해 덮어쓰기. 비우면 = 라이브 값 사용.
5. **종료**: 트레이 우클릭 메뉴에서 Quit.

## 캘리브레이션

⚙ 버튼은 탭마다 다른 내용을 표시하고, 카드 크기도 자동 조정됩니다.

<p align="center">
  <img src="../public/Claude_settings.png" alt="Claude settings" width="260">
  &nbsp;&nbsp;
  <img src="../public/Codex_settings.png"  alt="Codex settings"  width="260">
</p>

**Claude는 공개된 사용량 API가 없으므로** 바는 "추정치"입니다. 공식 대시보드에 맞추려면:

1. Anthropic web 대시보드를 열고 **session %**와 **weekly %**를 확인.
2. Claude 설정의 **Session %** / **Weekly %** 두 칸에 그 숫자를 입력.
3. 앱이 현재 로컬 count를 그 %로 나누어 실제 cap을 계산하고 저장.
4. Claude를 계속 사용하면 바도 비례해서 상승. 어긋나면 다시 입력하면 됩니다.

**Codex**는 CLI가 활성 상태일 때 실제 `rate_limits`를 사용해 정확합니다. 최근 웹/IDE를 사용해 CLI 데이터가 stale인 경우, Codex 설정에서 OpenAI 대시보드 %를 입력해 덮어쓰고, 비우면 라이브로 복귀합니다.

## 프라이버시

설계상 **완전 로컬 전용** 도구입니다:

- **백엔드 없음, 텔레메트리 없음, 자동 업데이트 phone-home 없음.**
- Claude 파서가 읽는 것: `type`, `userType`, `isSidechain`, `timestamp`, `message.usage`(토큰 카운트만).
- Codex 파서가 읽는 것: `rate_limits` 객체(퍼센트, 윈도우 분, 리셋 타임스탬프).
- **절대 읽지 않는 것**: 프롬프트 내용, 도구 출력, 파일 내용, 시크릿류.
- 설정은 `~/.usage-radar/settings.json`에 저장(로컬 파일).

오픈소스이므로 직접 검증 가능합니다.

## 라이선스

[MIT](../LICENSE) — 마음대로 사용하세요.
