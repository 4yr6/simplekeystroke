<div align="center">

# SimpleKeystroke

**Windows 用の極小ポータブル・キーストローク HUD — 本物の透過、常に最前面、304 KB、依存関係ゼロ。**

[English](README.md) · [简体中文](README.zh-CN.md) · [繁體中文](README.zh-TW.md) · [日本語](README.ja.md)

[![License](https://img.shields.io/badge/license-MIT-3da639.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/built%20with-Rust-ce422b.svg?logo=rust&logoColor=white)](https://www.rust-lang.org)
[![Platform](https://img.shields.io/badge/platform-Windows%2010%20%7C%2011-0078d4.svg?logo=windows&logoColor=white)](#動作環境)
[![Size](https://img.shields.io/badge/binary-304%20KB-4c8bf5.svg)](#)

<img src="docs/preview.png" width="560" alt="SimpleKeystroke">

<sub>左：待機 — 右：<kbd>W</kbd> <kbd>A</kbd> <kbd>S</kbd> <kbd>Space</kbd> を押している状態</sub>

</div>

---

## これは何？

SimpleKeystroke は、画面の隅に小さなキーボード（**W A S D** + **Space**）を描き、押したキーをその瞬間に光らせるツールです。タスクトレイに常駐し、すべてのウィンドウより前面に浮かび、完全にマウス透過——ゲーム、エディタ、配信レイアウトの邪魔をしません。

**Rust 製。** Electron も .NET も C++ ランタイムもインストーラもありません。UI は 1 ピクセルずつソフトウェア・ラスタライズして `windows-sys` 経由で Win32 のレイヤードウィンドウに転送し、DWM が合成します。結果は DLL も管理者権限も不要な、単一の自己完結 exe です。

用途：配信オーバーレイ（OBS）、画面録画、チュートリアル動画、あるいはプレイ中の移動キー確認に。

## 特徴

| 特徴 | 内容 |
| :--- | :--- |
| **本物の透過** | `UpdateLayeredWindow` によるピクセル単位のアルファ。角丸も発光も背景に自然に溶け込みます（クロマキーではありません）。 |
| **マウス透過** | ロック中は `WS_EX_TRANSPARENT` が付き、クリックは下のウィンドウへそのまま通ります。 |
| **トレイ常駐** | タスクバーボタンも枠もタイトルバーもなし。トレイアイコンの右クリックが設定画面です。 |
| **ドラッグで配置** | 一度ロックを外せばどこへでも移動でき、位置は記憶されます。 |
| **即時反映** | サイズ・不透明度・アクセントカラー・位置は再起動なしで即反映。 |
| **10 言語対応** | Windows の表示言語に自動追従します。 |
| **ポータブル** | 設定は exe 隣のテキストファイル 1 枚。削除すれば初期状態に戻ります。 |
| **軽量** | 待機時 CPU 0%、メモリ約 7 MB、変化があったときだけ再描画。 |

## クイックスタート

1. `simplekeystroke.exe` をダウンロード。
2. ダブルクリックで起動。HUD が左下に現れ、トレイにアイコンが出ます。
3. トレイアイコンを右クリックしてカスタマイズ。終了は <kbd>End</kbd> キー。

インストール不要、管理者権限不要、exe のフォルダ以外に何も書き込みません。

## トレイメニュー

| 項目 | 内容 |
| :--- | :--- |
| HUD を表示 / 隠す | 表示の切り替え（トレイのダブルクリックでも同じ）。 |
| 位置を固定 / ロック解除して移動 | 固定 = マウス透過、解除 = ドラッグで移動可能。 |
| サイズ | 小 · 中 · 大 |
| 不透明度 | 100% · 90% · 75% · 60% |
| アクセントカラー | アイスブルー · ティール · ライラック · ホワイト · アンバー |
| 位置 | 左下 · 右下 · 左上 · 右上 · 画面中央 · カスタム |
| Windows と同時に起動 | `HKCU\...\CurrentVersion\Run` に登録（もう一度選ぶと削除）。 |
| 既定値に戻す | 設定を初期化。 |
| 終了 | 完全終了——トレイアイコン、オーバーレイ、プロセスすべて消えます。 |

<div align="center">
<img src="docs/menu-ja.png" width="330" alt="トレイメニュー">
</div>

## 設定ファイル

設定は exe 隣の `simplekeystroke.ini` に `キー=値` で保存されます：

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

| キー | 値 |
| :--- | :--- |
| `scale` | `0` 小 · `1` 中 · `2` 大 |
| `opacity` | `0` 100% · `1` 90% · `2` 75% · `3` 60% |
| `theme` | `0` アイスブルー · `1` ティール · `2` ライラック · `3` ホワイト · `4` アンバー |
| `anchor` | `0` 左下 · `1` 右下 · `2` 左上 · `3` 右上 · `4` 画面中央 · `5` カスタム |
| `x`, `y` | カスタム座標（`anchor=5` のときのみ使用） |
| `locked` | `1` マウス透過 · `0` ドラッグ可 |
| `visible` | `1` 起動時に表示 · `0` 起動時に非表示 |

exe のフォルダが書き込み不可の場合は `%APPDATA%\SimpleKeystroke\settings.ini` に退避します。

## 対応言語

トレイメニューとツールチップは起動時に **Windows の表示言語** に従います：

| UI 言語 | Windows の言語 |
| :--- | :--- |
| English | 既定 / 未対応の言語すべて |
| 简体中文 | 中国語（簡体字）zh-CN、zh-SG |
| 繁體中文 | 中国語（繁体字）zh-TW、zh-HK、zh-MO |
| 日本語 | 日本語 |
| 한국어 | 韓国語 |
| Русский | ロシア語 |
| Español | スペイン語 |
| Français | フランス語 |
| Deutsch | ドイツ語 |
| Português | ポルトガル語 |

<div align="center">
<img src="docs/menu-en.png" width="300" alt="英語メニュー"> <img src="docs/menu-ja.png" width="300" alt="日本語メニュー">
</div>

## 動作環境

- Windows 10 / Windows 11（x64）
- 以上。バイナリはシステム CRT に静的リンクされています。

## ソースからビルド

```bash
git clone https://github.com/4yr6/simplekeystroke.git
cd simplekeystroke
cargo build --release
# 出力: target/release/simplekeystroke.exe
```

安定版 Rust ツールチェーンが必要です。release プロファイルは `lto = "fat"`、`codegen-units = 1`、`panic = "abort"`、`strip = true` で約 300 KB に収めています。

## 仕組み

- **ウィンドウ** — `WS_POPUP` + `WS_EX_LAYERED | WS_EX_TRANSPARENT | WS_EX_TOPMOST | WS_EX_TOOLWINDOW | WS_EX_NOACTIVATE`。
- **描画** — トップダウン 32 bit DIB に純ソフトウェアでラスタライズ：キーキャップは符号付き距離場の角丸矩形、文字はポリライン距離場のストローク（フォント読み込みなし、GDI テキストなし）。最後にプリマルチプライド alpha の 1 フレームとして `UpdateLayeredWindow` で転送。
- **入力** — 8 ms タイマーで `GetAsyncKeyState` をポーリングし、W/A/S/D/Space の明るさを目標値へイージング。変化がなければ再描画しません。
- **トレイ** — `Shell_NotifyIconW`。`TaskbarCreated` を処理するのでエクスプローラ再起動後もアイコンが残ります。
- **単一起動** — 名前付きミューテックス。再度 exe を起動しても既存のプロセスを呼び起こすだけです。

## ライセンス

[MIT](LICENSE) © 2026 4yr6
