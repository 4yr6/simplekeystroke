<div align="center">

# SimpleKeystroke

**极小、便携的 Windows 按键显示 HUD —— 真透明、始终置顶、304 KB、零依赖。**

[English](README.md) · [简体中文](README.zh-CN.md) · [繁體中文](README.zh-TW.md) · [日本語](README.ja.md)

[![License](https://img.shields.io/badge/license-MIT-3da639.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/built%20with-Rust-ce422b.svg?logo=rust&logoColor=white)](https://www.rust-lang.org)
[![Platform](https://img.shields.io/badge/platform-Windows%2010%20%7C%2011-0078d4.svg?logo=windows&logoColor=white)](#环境要求)
[![Size](https://img.shields.io/badge/binary-304%20KB-4c8bf5.svg)](#)
[![Download](https://img.shields.io/github/v/release/4yr6/simplekeystroke?label=download&color=4c8bf5)](https://github.com/4yr6/simplekeystroke/releases/latest)

<img src="docs/preview.png" width="560" alt="SimpleKeystroke">

<sub>左：待机 — 右：按住 <kbd>W</kbd> <kbd>A</kbd> <kbd>S</kbd> <kbd>空格</kbd></sub>

</div>

---

## 这是什么

SimpleKeystroke 在屏幕角落里画一个小键盘（**W A S D** + **空格**），你按下哪个键，哪个键就立刻亮起来。它常驻系统托盘、始终浮在所有窗口之上，并且完全鼠标穿透——不会挡住你的游戏、编辑器，也不会干扰直播画面。

**用 Rust 写的。** 没有 Electron、没有 .NET、没有 C++ 运行时、没有安装包。整个界面是逐像素软件光栅化后，通过 `windows-sys` 推送给 Win32 分层窗口，由系统合成器完成混色。最终只有一个自包含的 exe，不需要任何 DLL，也不需要管理员权限。

适合：直播叠加（OBS）、录屏、教程演示，或者只是打游戏时看看自己的走位键。

## 特性

| 特性 | 说明 |
| :--- | :--- |
| **真透明** | `UpdateLayeredWindow` 逐像素 alpha——圆角和发光会和背后的画面自然融合，不是抠色的方块。 |
| **鼠标穿透** | 锁定时窗口带 `WS_EX_TRANSPARENT`，点击直接穿透到下层。 |
| **托盘托管** | 没有任务栏按钮、没有边框标题栏，右键托盘图标就是全部设置。 |
| **拖动定位** | 解锁一次，拖到任意位置，位置会被记住。 |
| **实时生效** | 尺寸、透明度、主题色、位置改完立即生效，不用重启。 |
| **10 种界面语言** | 启动时自动跟随 Windows 显示语言。 |
| **完全便携** | 配置就是 exe 旁边一个纯文本文件，删掉即恢复默认。 |
| **很省** | 待机 0% CPU、约 7 MB 内存，只有内容变化才重绘。 |

## 快速开始

1. 下载 [`simplekeystroke.exe`](https://github.com/4yr6/simplekeystroke/releases/latest)。
2. 双击运行。HUD 出现在左下角，托盘出现图标。
3. 右键托盘图标自定义。按 <kbd>End</kbd> 退出。

不用安装、不用管理员权限，除了 exe 所在目录不往任何地方写文件。

## 托盘菜单

| 菜单项 | 作用 |
| :--- | :--- |
| 显示 / 隐藏 HUD | 切换显隐（双击托盘图标等效）。 |
| 锁定位置 / 解锁并拖动 | 锁定 = 鼠标穿透；解锁 = 可以拖到任意位置。 |
| 尺寸 | 小 · 中 · 大 |
| 不透明度 | 100% · 90% · 75% · 60% |
| 主题色 | 冰蓝 · 青绿 · 淡紫 · 纯白 · 琥珀 |
| 位置 | 左下角 · 右下角 · 左上角 · 右上角 · 屏幕居中 · 自定义 |
| 开机自动启动 | 写入 `HKCU\...\CurrentVersion\Run`（再点一次即删除）。 |
| 恢复默认设置 | 回到干净的初始配置。 |
| 退出 | 彻底退出——托盘图标、叠加层、进程一起消失。 |

<div align="center">
<img src="docs/menu-zh.png" width="330" alt="托盘菜单">
</div>

## 配置文件

设置保存在 exe 旁边的 `simplekeystroke.ini`，就是简单的 `键=值`：

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

| 键 | 取值 |
| :--- | :--- |
| `scale` | `0` 小 · `1` 中 · `2` 大 |
| `opacity` | `0` 100% · `1` 90% · `2` 75% · `3` 60% |
| `theme` | `0` 冰蓝 · `1` 青绿 · `2` 淡紫 · `3` 纯白 · `4` 琥珀 |
| `anchor` | `0` 左下角 · `1` 右下角 · `2` 左上角 · `3` 右上角 · `4` 屏幕居中 · `5` 自定义 |
| `x`、`y` | 自定义坐标，`anchor=5` 时才用 |
| `locked` | `1` 鼠标穿透 · `0` 可拖动 |
| `visible` | `1` 启动时显示 · `0` 启动时隐藏 |

如果 exe 所在目录不可写，会自动回落到 `%APPDATA%\SimpleKeystroke\settings.ini`。

## 多语言

托盘菜单和悬浮提示会在启动时跟随 **Windows 显示语言**：

| 界面语言 | Windows 语言 |
| :--- | :--- |
| English | 默认 / 任何未支持的语言 |
| 简体中文 | 中文（简体）zh-CN、zh-SG |
| 繁體中文 | 中文（繁體）zh-TW、zh-HK、zh-MO |
| 日本語 | 日语 |
| 한국어 | 韩语 |
| Русский | 俄语 |
| Español | 西班牙语 |
| Français | 法语 |
| Deutsch | 德语 |
| Português | 葡萄牙语 |

<div align="center">
<img src="docs/menu-en.png" width="300" alt="英文菜单"> <img src="docs/menu-ja.png" width="300" alt="日文菜单">
</div>

## 环境要求

- Windows 10 或 Windows 11（x64）
- 别无他求。二进制静态链接系统 CRT。

## 从源码构建

```bash
git clone https://github.com/4yr6/simplekeystroke.git
cd simplekeystroke
cargo build --release
# 产物：target/release/simplekeystroke.exe
```

需要稳定版 Rust 工具链。release 配置启用了 `lto = "fat"`、`codegen-units = 1`、`panic = "abort"`、`strip = true`，所以只有约 300 KB。

## 实现原理

- **窗口** — `WS_POPUP` + `WS_EX_LAYERED | WS_EX_TRANSPARENT | WS_EX_TOPMOST | WS_EX_TOOLWINDOW | WS_EX_NOACTIVATE`。
- **渲染** — 自顶向下 32 位 DIB，纯软件光栅化：键帽用有符号距离场的圆角矩形，字母用折线距离场描边（不加载字体、不用 GDI 文本），最后一帧以预乘 alpha 经 `UpdateLayeredWindow` 提交。
- **输入** — 8 ms 定时器轮询 `GetAsyncKeyState` 读取 W/A/S/D/空格，每帧把亮度向目标值缓动；状态没变化就不重绘。
- **托盘** — `Shell_NotifyIconW`，并处理 `TaskbarCreated` 广播，资源管理器重启后图标不会消失。
- **单实例** — 命名互斥体，再次双击 exe 只会唤醒已在运行的那个。

## 许可

[MIT](LICENSE) © 2026 4yr6
