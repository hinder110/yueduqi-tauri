# 阅读器 (YueDuQi) — Tauri 桌面版

基于 [Legado（阅读）](https://github.com/gedoor/legado) 原理的多书源小说阅读器桌面应用。Rust 后端 + React 前端，打包后仅 14MB。

## 功能

- 多书源搜索：番茄小说、笔趣阁、七星阁
- 章节目录浏览
- 阅读器：字号三档切换、日间/夜间模式、上下章翻页
- 热门推荐（番茄热搜榜）

## 下载

去 [Releases](https://github.com/hinder110/yueduqi-tauri/releases) 下载最新 `yueduqi-tauri.exe`，双击运行。Windows 10/11 适用，无需安装。

## 从源码构建

**前置要求**：Node.js 18+、Rust 工具链（MSVC）、Visual Studio C++ Build Tools

```bash
git clone https://github.com/hinder110/yueduqi-tauri.git
cd yueduqi-tauri
npm install
npm run tauri dev      # 开发模式，带热更新
npm run tauri build    # 打包 exe，输出在 src-tauri/target/release/
```

## 书源

| 书源 | 类型 | 封面 | 说明 |
|------|------|------|------|
| 番茄小说 (光遇API) | JSON API | 有 | 有热门推荐，正文每日限流 3 次 |
| 笔趣阁900 | HTML 解析 | 交叉匹配 | GBK 解码，无限制 |
| 七星阁小说网 | HTML 解析 | 自带 | UTF-8，无限制 |

## 技术栈

**桌面框架**：[Tauri v2](https://v2.tauri.app/) — Rust 后端 + 系统 WebView 前端，体积仅 14MB

**前端**：React 19 + TypeScript + Vite + react-router-dom

**后端 (Rust)**：
- `reqwest` — HTTP 请求
- `scraper` — HTML 解析（CSS Selector）
- `encoding_rs` — GBK 解码
- `serde` / `serde_json` — 序列化

## 项目结构

```
src-tauri/src/
├── main.rs              # 入口
├── lib.rs               # Tauri commands，对接前端 IPC
├── types.rs             # Book / Chapter / ChapterContent
└── parsers/
    ├── mod.rs           # 书源路由器 + 笔趣阁封面合并
    ├── guangyu.rs       # 光遇 API 解析器（七 host fallback）
    ├── biquge.rs        # 笔趣阁 HTML 解析器（GBK）
    └── qixinge.rs       # 七星阁 HTML 解析器（UTF-8）
```

解析器架构：每个书源独立实现三个方法（`search_books` / `get_chapters` / `get_chapter_content`），由 `mod.rs` 路由器根据用户选择的书源分发。

## 对比 Web 版

| | [yueduqi](https://github.com/hinder110/yueduqi) (Node.js) | yueduqi-tauri (Rust) |
|---|---|---|
| 后端 | Express + Axios + Cheerio | Tauri commands + reqwest + scraper |
| 前端调用 | `fetch('/api/...')` | `invoke('search_books', ...)` |
| 打包 | Docker 镜像 ~200MB | 单 exe ~14MB |
| 运行 | 浏览器 + 终端 | 双击桌面窗口 |
| 部署 | npm/Docker | 下载 exe |

## License

MIT
