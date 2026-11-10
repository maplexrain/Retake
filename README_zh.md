<p align="center">
  <img src="src/assets/logo.svg" alt="Retake 图标" width="128" height="128" />
</p>

<h1 align="center">Retake</h1>

<p align="center">
  <strong>专为 Windows 打造的下一代现代化轻量桌面录屏工具</strong><br>
  基于 <em>Rust, Tauri 2, React 18, TypeScript & FFmpeg</em> 构建
</p>

<p align="center">
  <a href="https://github.com/maplexrain/Retake/releases/latest"><img src="https://img.shields.io/github/v/release/maplexrain/Retake?color=blue&label=release&logo=github" alt="最新版本" /></a>
  <a href="LICENSE"><img src="https://img.shields.io/badge/license-MIT-green.svg" alt="开源协议" /></a>
  <img src="https://img.shields.io/badge/platform-Windows%2010%20%7C%2011-0078D6?logo=windows" alt="支持平台" />
  <img src="https://img.shields.io/badge/Tauri-v2-24C8D8?logo=tauri&logoColor=white" alt="Tauri 2" />
  <img src="https://img.shields.io/badge/Rust-MSVC-DEA584?logo=rust&logoColor=white" alt="Rust" />
  <img src="https://img.shields.io/badge/React-18-61DAFB?logo=react&logoColor=black" alt="React" />
  <img src="https://img.shields.io/badge/TypeScript-5-3178C6?logo=typescript&logoColor=white" alt="TypeScript" />
  <img src="https://img.shields.io/badge/FFmpeg-引擎驱动-007808?logo=ffmpeg&logoColor=white" alt="FFmpeg" />
</p>

<p align="center">
  <a href="#-核心特性">核心特性</a> •
  <a href="#-回溯录制理念">回溯录制理念</a> •
  <a href="#-全局快捷键">快捷键</a> •
  <a href="#-下载与安装">下载安装</a> •
  <a href="#-快速上手">快速上手</a> •
  <a href="#-系统架构">系统架构</a> •
  <a href="README.md">English</a>
</p>

---

## 💡 为什么选择 Retake？

传统录屏软件在讲错一句话、演示失误或卡壳时，往往逼你只有两条路可选：要么忍痛推倒重来，要么录完之后花大量时间拖进复杂的剪辑软件里找失误片段、剪切、缝合。

**Retake 重新定义了录制工作流：实时回溯点系统**
在录制顺利时随时按下 <kbd>Alt</kbd>+<kbd>F</kbd> 标记一个回溯旗标。一旦卡壳或讲错，只需按下 <kbd>Alt</kbd>+<kbd>G</kbd>，Retake 就会瞬间将时间轴回溯到上一个回溯点，干净剔除失误片段并无缝继续录制。

```
                    ┌───────────────────────── Retake 录制流 ─────────────────────────┐
                    │                                                                 │
录制时间轴:         ───[ 片段 1: 完美开场 ]───┬───[ 片段 2: 讲错失误部分 ]───┐         │
                                              │                              │         │
操作触发:                                 Alt+F 标记                     Alt+G 一键回溯
                                          回溯点 #1                     (自动丢弃失误)
                                              │                              │         │
继续录制:           ──────────────────────────┴───[ 片段 2: 完美重录部分 ]────┴──> 💾 最终合并成品 MP4
```

---

## ✨ 核心特性

- **⚡ 回溯点引擎（Checkpoint & Rollback）**
  - **实时标记 (`Alt+F`)**：录制过程中随时打点标记，不中断音视频采集流。
  - **瞬间回溯 (`Alt+G`)**：一键回退到最近的回溯点（未标记则回溯到开头），自动丢弃失误片段继续录制。
  - **无损自动合并**：录制完成时，底层利用 FFmpeg concat demuxer 零损耗极速缝合成单份完整 MP4。
- **🛸 屏幕顶部悬浮控制栏（HUD 进度条）**
  - 录制时屏幕顶部正中间常驻精致胶囊工具栏，支持自由拖拽。
  - 实时显示录制时间轴、回溯旗标刻度（`⚑`）、当前状态、已录制时长及快捷动作（`标记`、`回溯`、`暂停`、`完成`、`取消`）。
  - 通过 Windows `WDA_EXCLUDEFROMCAPTURE` 特性避开录入——**成片中绝无悬浮栏痕迹**。
- **🎙️ 双音频源采集与实时电平动效**
  - 支持麦克风与系统声音（WASAPI 回环）双路高保真采样与自动混音。
  - 内置真实 WASAPI peak 电平跳动频谱动效，实时直观确认收声状态与环境静音。
- **🛡️ 窗口不入镜保护（Capture Exclusion）**
  - Retake 主控制面板与顶部悬浮栏均支持防录入保护，录屏时完全隐形，可在高级设置中随心开关。
- **🖥️ 灵活的捕获范围**
  - 多显示器选择下拉框，自动识别主屏幕。
  - 自定义区域框选：唤起全屏交互式透明浮层，任意拖拽截取屏幕矩形。
- **⏳ 全屏倒计时预检遮罩**
  - 录制前全屏光环倒计时，并一目了然显示所选范围、麦克风状态、系统声音及鼠标捕获设置。随时按 <kbd>ESC</kbd> 撤销。
- **📋 录制清单管理（`retake_manifest.json`）**
  - 预先规划录制片段清单（例如 `01_intro.mp4`、`02_demo.mp4` 等），支持自动读写保存目录下的清单或一键新建。
  - 点击任意片段设为录制目标；录完后自动记录时长、状态与时间戳。若不切换继续录制则自动覆盖该文件，完美契合分节视频与课件制作。
  - 支持一键打开 JSON 外部编辑、行内修改标题/文件名、一键播放已录制片段。
- **🎨 纯平工作室架构（Card-Free）**
  - 坚决摒弃“卡片套卡片”的视觉噪声设计。发丝分割线、一体化分段控制器与紧凑自适应窗口高度。
- **🌐 响应式多语言支持（i18n）**
  - 英文优先，顶栏支持 `EN / 中` 一键无缝切换，多窗口（主面板、悬浮栏、倒计时、选区）实时广播响应。

---

## ⌨️ 全局快捷键

Retake 全局快捷键在任何窗口或全屏程序下均可全局触发，无需将 Retake 置于焦点：

| 快捷键 | 功能动作 | 适用范围 / 阶段 |
| :---: | :--- | :--- |
| <kbd>Alt</kbd> + <kbd>1</kbd> | **开始 / 完成** | 空闲时开始录制 · 录制或暂停时完成并保存 · 倒计时中取消 |
| <kbd>Alt</kbd> + <kbd>2</kbd> | **暂停 / 恢复** | 在录制与暂停之间切换 |
| <kbd>Alt</kbd> + <kbd>F</kbd> | **标记回溯点** | 在当前时间戳标记一个回溯旗标（`⚑`） |
| <kbd>Alt</kbd> + <kbd>G</kbd> | **回溯** | 丢弃最新回溯点之后的失误片段，保持录制继续 |
| <kbd>Alt</kbd> + <kbd>3</kbd> | **作废取消** | 放弃本次录制并彻底删除所有临时片段 |

---

## 📦 下载与安装

请前往 [**GitHub Releases 最新发布页**](https://github.com/maplexrain/Retake/releases/latest) 下载：

| 格式 | 文件名 | 说明 |
| :--- | :--- | :--- |
| **安装版 (NSIS)** | `Retake_0.1.0_x64-setup.exe` | 标准 Windows 安装向导，自动生成桌面与开始菜单快捷方式 |
| **便携绿色版** | `retake.exe` | 单文件免安装，解压即用 |

---

## 🛠️ 系统环境要求

| 依赖组件 | 最低要求 | 说明 |
| :--- | :--- | :--- |
| **操作系统** | Windows 10 2004+ (内部版本 19041+) / Windows 11 | `WDA_EXCLUDEFROMCAPTURE` 窗口防录入特性所必需 |
| **FFmpeg** | 包含 `libx264` 与 `gdigrab` 的版本 | 底层核心音视频录制与混流引擎 |
| **WebView2** | 现代 Edge WebView2 Runtime | Win10 / Win11 通常已内置 |

> [!TIP]
> **Windows 下安装 FFmpeg**:
> ```powershell
> winget install Gyan.FFmpeg
> ```
> 或前往 [ffmpeg.org](https://ffmpeg.org/download.html) 下载，并将 `ffmpeg.exe` 所在目录加入系统环境变量 `PATH`，亦可直接在 Retake 应用内的「高级录制参数与路径 → ffmpeg 路径」填入绝对路径。

---

## 🚀 快速上手（源码开发）

### 1. 克隆与依赖安装
```bash
git clone git@github.com:maplexrain/Retake.git
cd Retake
npm install
```

### 2. 启动本地开发
```bash
npm run dev
# 或
npm run tauri dev
```

### 3. 打包发布安装程序
```bash
npm run tauri build
```
编译产物与安装包将自动输出至 `src-tauri/target/release/` 目录。

---

## 🏛️ 项目工程架构

```
Retake/
├── src/                          # 前端工程 (React 18 + TypeScript + Vite)
│   ├── ui/
│   │   ├── Main.tsx              # 主控制台 (范围选择、音频电平控制、录制面板、折叠设置)
│   │   ├── FloatingHud.tsx       # 屏幕顶部防录入悬浮栏 (时间轴、回溯旗标、即时控制)
│   │   ├── Countdown.tsx         # 全屏光环倒计时与配置预检遮罩
│   │   ├── Selector.tsx          # 全屏半透明交互式选区层
│   │   └── icons.tsx             # 统一轻量 SVG 图标集
│   ├── i18n.ts                   # 响应式国际化引擎 (中英双语与跨窗口广播)
│   ├── api.ts                    # 强类型 Tauri IPC 交互封装
│   └── styles.css                # 扁平纯平工作室级样式表
│
└── src-tauri/                    # 后端工程 (Rust + Tauri 2 + Windows 原生 API)
    ├── src/
    │   ├── main.rs               # 窗口调度、全局快捷键注册与音频电平轮询
    │   ├── recorder.rs           # 多片段会话状态机、回溯点管理与流无损合并
    │   ├── sysaudio.rs           # WASAPI 回环低延迟采集与实时电平检测
    │   ├── win.rs                # Windows 原生 API 封装 (防录入亲和性设置、显示器枚举)
    │   ├── ffmpeg_util.rs        # FFmpeg 进程调度与 demuxer 描述符生成
    │   └── project.rs            # 配置持久化与 Schema 定义
    └── Cargo.toml                # Rust 依赖声明与编译特性配置
```

---

## 🤝 参与贡献

欢迎提交 Issue 与 Pull Request！在开始之前，请阅读我们的 [贡献指南](CONTRIBUTING.md) 与 [行为准则](CODE_OF_CONDUCT.md)。

---

## 📄 开源协议

本项目采用 **MIT 开源协议**，详情请参阅 [`LICENSE`](LICENSE)。

<p align="center">
  Made with ❤️ by <a href="https://github.com/maplexrain">maplexrain</a>
</p>
