<div align="center">

# SimpleKeystroke

**極小、可攜的 Windows 按鍵顯示 HUD —— 真透明、永遠置頂、304 KB、零依賴。**

[English](README.md) · [简体中文](README.zh-CN.md) · [繁體中文](README.zh-TW.md) · [日本語](README.ja.md)

[![License](https://img.shields.io/badge/license-MIT-3da639.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/built%20with-Rust-ce422b.svg?logo=rust&logoColor=white)](https://www.rust-lang.org)
[![Platform](https://img.shields.io/badge/platform-Windows%2010%20%7C%2011-0078d4.svg?logo=windows&logoColor=white)](#環境需求)
[![Size](https://img.shields.io/badge/binary-304%20KB-4c8bf5.svg)](#)
[![Download](https://img.shields.io/github/v/release/4yr6/simplekeystroke?label=download&color=4c8bf5)](https://github.com/4yr6/simplekeystroke/releases/latest)

<img src="docs/preview.png" width="560" alt="SimpleKeystroke">

<sub>左：待機 — 右：按住 <kbd>W</kbd> <kbd>A</kbd> <kbd>S</kbd> <kbd>空白鍵</kbd></sub>

</div>

---

## 這是什麼

SimpleKeystroke 在螢幕角落畫一個小鍵盤（**W A S D** + **空白鍵**），你按下哪個鍵，那個鍵就立刻亮起來。它常駐系統匣、永遠浮在所有視窗之上，而且完全滑鼠穿透——不會擋住你的遊戲、編輯器，也不會干擾直播畫面。

**以 Rust 撰寫。** 沒有 Electron、沒有 .NET、沒有 C++ 執行期、沒有安裝程式。整個介面是逐像素軟體光柵化後，透過 `windows-sys` 推送給 Win32 分層視窗，由系統合成器完成混色。最終只有一個自包含的 exe，不需要任何 DLL，也不需要系統管理員權限。

適合：直播疊層（OBS）、螢幕錄影、教學示範，或者只是玩遊戲時看看自己的走位鍵。

## 特色

| 特色 | 說明 |
| :--- | :--- |
| **真透明** | `UpdateLayeredWindow` 逐像素 alpha——圓角與發光會和背後畫面自然融合，不是去背色的方塊。 |
| **滑鼠穿透** | 鎖定時視窗帶 `WS_EX_TRANSPARENT`，點擊直接穿透到下層。 |
| **系統匣常駐** | 沒有工作列按鈕、沒有邊框標題列，右鍵系統匣圖示就是全部設定。 |
| **拖曳定位** | 解鎖一次，拖到任意位置，位置會被記住。 |
| **即時生效** | 尺寸、不透明度、主題色、位置改完立即生效，不必重啟。 |
| **10 種介面語言** | 啟動時自動跟隨 Windows 顯示語言。 |
| **完全可攜** | 設定就是 exe 旁邊一個純文字檔，刪掉即回復預設。 |
| **很省** | 待機 0% CPU、約 7 MB 記憶體，只有內容變化才重繪。 |

## 快速開始

1. 下載 [`simplekeystroke.exe`](https://github.com/4yr6/simplekeystroke/releases/latest)。
2. 雙擊執行。HUD 出現在左下角，系統匣出現圖示。
3. 右鍵系統匣圖示自訂。按 <kbd>End</kbd> 結束。

不必安裝、不必系統管理員權限，除了 exe 所在資料夾不往任何地方寫檔案。

## 系統匣選單

| 選單項目 | 作用 |
| :--- | :--- |
| 顯示 / 隱藏 HUD | 切換顯示（雙擊系統匣圖示等效）。 |
| 鎖定位置 / 解鎖並拖曳 | 鎖定 = 滑鼠穿透；解鎖 = 可拖到任意位置。 |
| 尺寸 | 小 · 中 · 大 |
| 不透明度 | 100% · 90% · 75% · 60% |
| 主題色 | 冰藍 · 青綠 · 淡紫 · 純白 · 琥珀 |
| 位置 | 左下角 · 右下角 · 左上角 · 右上角 · 畫面置中 · 自訂 |
| 開機自動啟動 | 寫入 `HKCU\...\CurrentVersion\Run`（再點一次即刪除）。 |
| 還原預設值 | 回到乾淨的初始設定。 |
| 結束 | 完全結束——系統匣圖示、疊層、行程一起消失。 |

<div align="center">
<img src="docs/menu-zh.png" width="330" alt="系統匣選單">
</div>

## 設定檔

設定保存在 exe 旁邊的 `simplekeystroke.ini`，就是簡單的 `鍵=值`：

```ini
scale=1
opacity=0
theme=0
anchor=0
x=24
y=1380
locked=1
visible=1
```

| 鍵 | 取值 |
| :--- | :--- |
| `scale` | `0` 小 · `1` 中 · `2` 大 |
| `opacity` | `0` 100% · `1` 90% · `2` 75% · `3` 60% |
| `theme` | `0` 冰藍 · `1` 青綠 · `2` 淡紫 · `3` 純白 · `4` 琥珀 |
| `anchor` | `0` 左下角 · `1` 右下角 · `2` 左上角 · `3` 右上角 · `4` 畫面置中 · `5` 自訂 |
| `x`、`y` | 自訂座標，`anchor=5` 時才使用 |
| `locked` | `1` 滑鼠穿透 · `0` 可拖曳 |
| `visible` | `1` 啟動時顯示 · `0` 啟動時隱藏 |

若 exe 所在資料夾不可寫，會自動回退到 `%APPDATA%\SimpleKeystroke\settings.ini`。

## 多語言

系統匣選單與滑鼠提示會在啟動時跟隨 **Windows 顯示語言**：

| 介面語言 | Windows 語言 |
| :--- | :--- |
| English | 預設 / 任何未支援的語言 |
| 简体中文 | 中文（簡體）zh-CN、zh-SG |
| 繁體中文 | 中文（繁體）zh-TW、zh-HK、zh-MO |
| 日本語 | 日語 |
| 한국어 | 韓語 |
| Русский | 俄語 |
| Español | 西班牙語 |
| Français | 法語 |
| Deutsch | 德語 |
| Português | 葡萄牙語 |

<div align="center">
<img src="docs/menu-en.png" width="300" alt="英文選單"> <img src="docs/menu-ja.png" width="300" alt="日文選單">
</div>

## 環境需求

- Windows 10 或 Windows 11（x64）
- 別無所求。二進位靜態連結系統 CRT。

## 從原始碼建置

```bash
git clone https://github.com/4yr6/simplekeystroke.git
cd simplekeystroke
cargo build --release
# 產物：target/release/simplekeystroke.exe
```

需要穩定版 Rust 工具鏈。release 設定啟用了 `lto = "fat"`、`codegen-units = 1`、`panic = "abort"`、`strip = true`，所以只有約 300 KB。

## 實作原理

- **視窗** — `WS_POPUP` + `WS_EX_LAYERED | WS_EX_TRANSPARENT | WS_EX_TOPMOST | WS_EX_TOOLWINDOW | WS_EX_NOACTIVATE`。
- **渲染** — 自頂向下 32 位 DIB，純軟體光柵化：鍵帽用有號距離場的圓角矩形，字母用折線距離場描邊（不載入字型、不用 GDI 文字），最後一帧以預乘 alpha 經 `UpdateLayeredWindow` 提交。
- **輸入** — 8 ms 計時器輪詢 `GetAsyncKeyState` 讀取 W/A/S/D/空白鍵，每帧把亮度向目標值緩動；狀態沒變化就不重繪。
- **系統匣** — `Shell_NotifyIconW`，並處理 `TaskbarCreated` 廣播，檔案總管重啟後圖示不會消失。
- **單一實例** — 具名互斥鎖，再次雙擊 exe 只會喚醒已在執行中的那個。

## 授權

[MIT](LICENSE) © 2026 4yr6
