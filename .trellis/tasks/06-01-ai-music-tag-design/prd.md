# TagCast — AI 播客元数据编辑器 v1 PRD (CONFIRMED)

> **状态**：✅ PRD 已确认，进入实施阶段（2026-06-01）
> **定稿日期**：2026-06-01
> **项目命名**：`TagCast`（已确定）
> **目标交付**：3 个月内 v1 上线

## TL;DR（一分钟看完）

| 维度 | 决策 |
|---|---|
| 用户 | 个人播客收藏爱好者 |
| 平台 | macOS only |
| 技术栈 | Tauri (Rust + Web 前端) |
| 音频格式 | MP3 / M4A / FLAC / OGG / OPUS / WAV / AIFF |
| 核心 AI 能力 | LLM 智能解析"脏文件名" → 提取 title/album/artist/track |
| LLM 提供商 | 默认 DeepSeek + 用户可自配 OpenAI Compatible / Anthropic 兼容 |
| UI 模式 | 表格批量 + 单文件向导（顶部切换） |
| 节目档案 | 持久节目档案库 + 批次内整列填充（双模式） |
| 文件重命名 | 开关控制：可只改元数据 / 可按模板重命名 |
| iCloud 支持 | 默认自动下载未下载文件 + 设置可切"仅提示" |
| 写入安全 | 只改元数据+文件名 + 记录原名快照 + 一键重置 |
| 分发 | 免费 + DMG 直装（不上 App Store） |
| v2 路线 | iTunes 播客标签 / 音乐元数据 / 字幕转写 / 章节检测 / 同目录同名封面自动导入 等 |

## 1. Goal

参考 macOS 上 "Music Tag Editor / Meta / Mp3tag" 这类元数据编辑软件的能力，加入 AI 功能（BPM/Key/情绪/流派/乐器自动识别、歌词转写、相似度搜索、智能补全缺失标签等），设计一款**同时面向个人音乐收藏 + 播客收藏**的 AI 元数据管理软件。

## 2. Target Users

- **个人音乐收藏爱好者** — 大容量本地音乐库，需要 AI 帮自动补全缺失元数据、智能打标、自然语言搜索
- **播客收藏爱好者** — 本地 M4A/MP3 播客库，需要 AI 章节检测、说话人识别、内容摘要、智能分类

## 3. MVP Scope

- **v1 只做播客**（差异化竞争，市场空白大）
- 音乐收藏能力等 v2 再加
- 目标：MVP 在 3 个月内上线可用的桌面端播客元数据 AI 编辑器

## 4. v1 核心 AI 能力

**核心场景**：用户从各种网站下载的播客，文件名带有一堆"脏"后缀，例如：

- 原始文件名：`QA009：香港金像奖·国产片含男量·家庭恐怖片等丨反派影评丨爱发电.mp3`
- AI 清洗后写入 title 字段：`QA009：香港金像奖·国产片含男量·家庭恐怖片等`
- 用户手动指定/确认 album 字段：`反派影评`
- 写回 M4A/MP3 文件的 ID3/iTunes 标签

**v1 不做**（用户明确）：

- ❌ BPM/Key/情绪检测
- ❌ 歌词/字幕转写
- ❌ 章节检测
- ❌ 说话人识别
- ❌ 相似度搜索
- ❌ RSS 订阅
- ❌ 任何需要"听音频内容"才能做的 AI 功能
- ❌ iTunes 播客专属标签（推迟到 v2）

**v1 只做**：基于文件名的 LLM 智能解析 + 元数据批量写入。

## 5. Requirements

### 5.1 平台与技术栈

- **平台**：macOS only
- **技术栈**：Tauri（Rust 后端 + Web 前端）
- **音频格式**：MP3 / M4A / FLAC / OGG / OPUS / WAV / AIFF 全格式支持
- **Rust tag 库**：`lofty` (formerly taglib-rust)
- **跨平台迁移友好**：代码层面不写 Mac 专属 API，路径处理用 `std::path::Path`，系统目录用 `dirs` crate。后续若需支持 Windows/Linux，只需加 build target，代码几乎不动。

### 5.2 核心功能

- 拖入音频文件 → 自动读取文件名
- 批量 LLM 解析：识别 title / album / artist / track 四个字段
- 两种审核 UI（顶部切换）：
  - **表格批量审核**：30+ 文件一次性解析，表格内整列填充/逐行编辑/批量应用
  - **单文件向导**：1-3 集零散处理
- **节目档案库**（持久）：用户为每个节目设置一次"反派影评"等元数据，AI 解析时自动匹配；新节目引导用户新建档案
- **批次内整列填充**：临时性快速操作
- **重命名开关**（顶部切换）：
  - 关：只改元数据，文件名保持原样
  - 开：按模板重命名（如 `{track} - {title}.{ext}`），AI 解析后预览新名
- **重置功能**：记录原文件名 + 原 tag，"重置"按钮一键抹除元数据 + 恢复原文件名

### 5.3 AI / LLM

- **默认 provider**：DeepSeek（用户填 API key）
- **自定义 provider**：用户在设置里添加"OpenAI Compatible"或"Anthropic 兼容"的 API 端点（填 base URL + key + model name）
- **输入**：仅文件名（不上传文件内容）
- **解析任务**：从脏文件名里提取 title / album / artist / track 四个字段
- **不读音频内容**（v1）

### 5.4 分发与商业

- **免费 + DMG 直装**：官网/网盘下载，拖入 Applications 即可
- 不上 Mac App Store（避免 notarization 折腾 + 30% 抽成）

### 5.5 写入安全

- **只改元数据 + 文件名**，不修改文件本身
- 记录"原文件名"快照到本地数据库
- **重置功能**：一键抹除所有 v1 写入的元数据 + 恢复原文件名

### 5.6 iCloud Drive 支持（关键！）

- 用户的播客库很可能存放在 `~/Library/Mobile Documents/com~apple~CloudDocs/...`（"桌面与文稿"同步 或 "iCloud Drive" 任意子目录）
- **核心难点**：iCloud 文件有"已下载/未下载"两种状态，未下载时本地只有 `.icloud` 占位文件
- **v1 行为**：
  - 拖入文件时检测是否在 iCloud 且未下载
  - **默认行为**：自动调用 `NSFileManager.startDownloadingUbiquitousItem` 触发下载，进度条提示
  - **设置开关**：可切到"仅提示不下载"，遇到时警告用户去 Finder 手动下载
- **路径处理**：
  - 解析 iCloud 别名（`~/iCloud Drive` → canonical 路径）
  - 用 `NSFileCoordinator` 协调写入，避免与 iCloud 同步冲突
- **实现方式**：写一个 Tauri 插件，Rust 用 `objc2` crate 调 Cocoa 文件 API
- **需要的 Apple API**：
  - `NSURL.resourceValues(forKeys: [.isUbiquitousItemKey, .ubiquitousItemDownloadingStatusKey])`
  - `NSFileManager.startDownloadingUbiquitousItem(at:)`
  - `NSFileCoordinator.coordinate(writingItemAt:options:.forMerging)` 协调写入
  - KVO 监听 `ubiquitousItemDownloadingStatus` 变化以更新进度

## 6. Acceptance Criteria

- [ ] 拖入 30 个 podcast MP3/M4A/FLAC/OGG/OPUS/WAV/AIFF 文件，AI 在 60 秒内完成解析（DeepSeek API 假设）
- [ ] 表格 UI 至少支持：整列下拉填充、单元格编辑、批量应用、单行应用
- [ ] 单文件向导至少支持：解析、编辑、确认、跳过
- [ ] 节目档案库支持新增/编辑/删除/自动匹配
- [ ] 重命名模板支持 `{track}` / `{title}` / `{album}` / `{artist}` / `{ext}` 至少 5 个变量
- [ ] 重置功能可一键恢复任意已处理文件到原始状态
- [ ] 至少支持 DeepSeek / OpenAI Compatible / Anthropic 兼容三种 provider 配置
- [ ] DMG 打包成功，启动后能跑通完整流程（拖入 → 解析 → 审核 → 写回 → 重置）
- [ ] **iCloud**：能识别 iCloud 路径下的未下载文件，默认自动触发下载，提供进度提示
- [ ] **iCloud**：设置开关可切换为"仅提示不下载"
- [ ] **iCloud**：写入元数据用 `NSFileCoordinator` 协调，避免与 iCloud 同步冲突
- [ ] **iCloud**：别名路径（`~/iCloud Drive`）正确解析为 canonical 路径

## 7. Definition of Done

- 单元测试覆盖：tag 读写（lofty）、LLM 解析（mock）、节目档案匹配、重置逻辑
- 集成测试：跑通端到端（mock API）
- Lint (rustfmt + clippy) / TypeScript 类型检查 / Cargo build / Tauri build 全绿
- README 写明：功能、安装、使用、API key 申请、备份/重置说明
- 已知限制清单：v1 不支持的功能、iTunes 标签推迟到 v2 等

## 8. Decision (ADR-lite)

**Context**: 用户参考 Music Tag 类软件，希望做一款带 AI 能力的元数据编辑器，但实际 AI 需求聚焦在"智能解析脏文件名"这一个场景，不需要音频内容理解。

**Decision**: v1 做"基于 LLM 的文件名智能解析 + 元数据批量写入"工具，限定为播客场景、macOS only、Tauri 栈、全格式音频、4 字段写入、表格 + 单文件两种 UI、节目档案库、DeepSeek 默认 + 自定义 provider、免费 DMG 分发、iCloud Drive 完整支持。

**Consequences**:

- ✅ 3 个月内可上线，差异化强（市场空白）
- ✅ 用户核心痛点（脏文件名清理）直接解决
- ✅ 架构轻，Rust tag 库 `lofty` 成熟
- ✅ 代码保持跨平台习惯，后续迁移 Win/Linux 只需 2-3 周
- ⚠️ v1 不做 iTunes 播客专属标签，播客 App 识别可能略弱（v2 补）
- ⚠️ v1 不做音频内容理解，BPM/章节/摘要等都无
- ⚠️ 依赖云 API，断网/限速时不可用
- ⚠️ iCloud 写入需谨慎处理（已用 NSFileCoordinator 协调）

## 9. Out of Scope (explicit)

- ❌ 音乐元数据编辑（推迟到 v2）
- ❌ BPM/Key/Mood/Genre/Energy 自动检测
- ❌ 歌词/字幕转写、说话人识别、章节检测
- ❌ 相似度搜索、自然语言搜索
- ❌ RSS 订阅、自动下载
- ❌ iTunes 播客专属标签（`itunes:author` / `itunes:season` / `itunes:summary` / `itunes:explicit` 等）
- ❌ 封面图下载/编辑（含"同目录同名图片自动作为封面导入"，推迟到 v2，见 §13）
- ❌ Mac App Store 分发
- ❌ 任何收费/订阅模型
- ❌ iOS / Windows / Linux 平台（v1 不分发；代码保持跨平台习惯以便 v2 拓展）
- ❌ 本地 LLM 推理（v2 再说）

## 10. Technical Notes

### 10.1 Rust 生态候选库

- **tag 读写**：[`lofty`](https://github.com/Serial-ATA/lofty-rs)（formerly taglib-rust）— 支持 MP3/ID3, M4A/MP4, FLAC/Vorbis, OGG, OPUS, WAV/RIFF, AIFF, APE
- **HTTP client**：`reqwest`
- **异步 runtime**：`tokio`
- **SQLite**：`rusqlite` 或 `sqlx`（节目档案库 + 重置快照）
- **Tauri 命令**：通过 IPC 让前端调用 Rust 解析/写入
- **iCloud 桥接**：`objc2` + `objc2-foundation` + `objc2-app-kit`（调 `NSFileManager` / `NSFileCoordinator` / `NSURL`）
  - 或 `cocoa` crate（旧 API，新项目推荐 `objc2`）
  - 或写个 Swift 小工具通过 `Command` 调用（备选）
- **系统目录**：`dirs` crate（拿 `~/Library/Application Support/` 等跨平台目录）

### 10.2 LLM 解析 Prompt 设计思路

- 输入：脏文件名（字符串）
- 输出：JSON `{ "title": "...", "album": "...", "artist": "...", "track": <int|null>, "confidence": 0.0~1.0 }`
- Few-shot 示例：3-5 个典型脏文件名 → 解析结果对
- 提示词强调：识别"丨"、"-"、"·"、"[" 后的网站标识（爱发电、知乎、喜马拉雅、小宇宙等），剥离它们

### 10.3 前端技术栈（Tauri WebView）— 已确定

- **框架**：Vue 3 + Vite + TypeScript（Composition API，`<script setup>`）
- **样式**：Tailwind CSS + shadcn-vue（轻量、可定制）
- **状态管理**：Pinia
- **表格库**：TanStack Table（Vue adapter `@tanstack/vue-table`）
- **目录结构**：`src/{components,composables,views,router,store,assets}`（遵循项目 Vue3 约定）

### 10.4 实施计划（3 个月）

| PR | 周 | 内容 |
|---|---|---|
| PR1 | 1-2 | Tauri 脚手架 + lofty 集成 + 拖入文件读取 + 单文件向导 UI 骨架 |
| PR2 | 3-4 | iCloud 插件（objc2 桥接 Cocoa）+ 文件检测 + 自动下载 + NSFileCoordinator 工具 |
| PR3 | 5-6 | DeepSeek API 接入 + LLM 解析 prompt + 自定义 provider 配置 + 表格批量 UI |
| PR4 | 7-8 | SQLite 集成 + 节目档案库 CRUD + 自动匹配 + 引导新建 + 重命名模板 |
| PR5 | 9-10 | 重置功能 + 原文件名快照 + 原 tag 快照 + 写回元数据流程 |
| PR6 | 11-12 | 单元/集成测试 + DMG 打包脚本 + README + 发布准备 |

### 10.5 调研时间

2026-06

### 10.6 参考资料

- Music Tag Editor (App Store) — meta 编辑能力参考
- Meta (nightbirdsevolve.com/meta) — 批量编辑 UI 参考
- Mp3tag (mp3tag.app) — 工作流 action group 参考
- lofty-rs (github.com/Serial-ATA/lofty-rs) — Rust tag 库
- DeepSeek API docs — 默认 LLM 接入
- Apple Developer Documentation — `NSFileManager` / `NSFileCoordinator` / `NSURL` iCloud APIs
- arXiv 2602.03023 "Music Metadata LLMs" — 行业 AI 元数据趋势

## 11. 项目命名（已确定）

**`TagCast`** — 简短，tag + podcast，已同步到 PRD 标题、脚手架与品牌标识。

---

## 12. 实施状态

- ✅ 用户已确认 PRD 内容（2026-06-01）
- ✅ 项目命名确定为 `TagCast`
- ✅ 前端栈确定为 Vue 3 + Vite + TypeScript（见 10.3）
- 🚧 进入 Phase 2 实施，从 PR1 脚手架开始

## 13. v2 后续计划（Backlog）

> v1 不实现，记录为后续迭代候选。

- **同目录同名封面自动导入**：导入音频时，扫描其所在目录下与文件/节目同名（或 `cover` / `folder` 等约定名）的图片（jpg/png/webp），自动作为封面（APIC / `covr` 标签）写入。
  - 匹配优先级（建议）：与音频同名图片 > 节目名同名图片 > 目录内 `cover.*` / `folder.*`
  - 依赖 PR5 的标签写回通道，并需扩展 lofty 的图片标签写入。
  - 当前 v1 不读取/写入任何封面（见 §9）。
- iTunes 播客专属标签、音乐元数据编辑、字幕转写、章节检测等（见 TL;DR v2 路线）。
