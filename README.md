# TagCast — AI 播客元数据编辑器

> 用 LLM 智能清洗"脏文件名"，批量写回播客音频的标题 / 节目 / 作者 / 集数元数据。macOS 桌面应用，基于 Tauri（Rust + Vue 3）。

TagCast 解决一个具体痛点：从各类网站下载的播客文件名带着一堆平台后缀，例如
`QA009：香港金像奖·国产片含男量丨反派影评丨爱发电.mp3`。TagCast 调用大模型把它清洗成结构化元数据：

| 字段 | 提取结果 |
|---|---|
| title（标题） | `QA009：香港金像奖·国产片含男量` |
| album（节目） | `反派影评` |
| artist（作者） | （按节目档案回填） |
| track（集数） | `9` |

然后一键写回 MP3 / M4A / FLAC 等文件的标签，可选按模板重命名文件。

---

## 功能

- **批量 AI 解析**：拖入音频 → 一次性把所有文件名交给 LLM → 提取 title / album / artist / track 四字段（仅上传文件名，不上传音频内容）。
- **两种审核模式**（顶部切换）：
  - **表格批量**：30+ 文件一次解析，支持整列填充、单元格编辑、批量/单行应用。
  - **单文件向导**：1–3 集零散处理，逐个解析、编辑、确认、跳过。
- **节目档案库**（持久化）：为每个节目设一次元数据（节目名 / 作者 / 匹配关键词），AI 解析后自动匹配回填；遇到新节目引导新建。
- **重命名开关**：关闭只改元数据；开启按模板重命名，支持 `{track}` `{title}` `{album}` `{artist}` `{ext}` 五个变量（默认 `{track} - {title}.{ext}`）。
- **写入安全 + 一键重置**：写回前自动快照原文件名 + 原标签到本地数据库，"重置"可把任意已处理文件恢复到原始状态。
- **iCloud 支持**：识别 iCloud 未下载文件，默认自动触发下载（可在设置切为"仅提示"）。
- **全格式**：MP3 / M4A / FLAC / OGG / OPUS / WAV / AIFF。
- **多 LLM Provider**：默认 DeepSeek，可自配任意 OpenAI 兼容端点或 Anthropic 兼容端点。

---

## 安装

### 方式一：DMG 直装（推荐）

1. 下载 `TagCast_x.y.z_aarch64.dmg`（或 x64）。
2. 双击打开，把 `TagCast.app` 拖入"应用程序"。
3. 首次打开若提示"无法验证开发者"，到「系统设置 → 隐私与安全性」点「仍要打开」即可（v1 未做 Apple 公证）。

> 系统要求：macOS（Apple Silicon 或 Intel）。

### 方式二：从源码构建

环境依赖：

- [Node.js](https://nodejs.org/) 18+
- [Rust](https://rustup.rs/) 1.77+
- Xcode Command Line Tools（`xcode-select --install`）

```bash
git clone <repo-url> && cd music-tag-AI
npm install

# 本地开发（热更新）
npm run tauri:dev

# 打包 DMG（含质量门禁）
npm run build:dmg
# 产物：src-tauri/target/release/bundle/dmg/*.dmg
```

> 注：`build:dmg` 在打包时设置 `CI=true`，让 Tauri 跳过 `bundle_dmg.sh` 中需要 Finder 自动化权限、
> 在终端环境下常失败（`Failed running AppleScript`）的窗口美化步骤。若直接执行 `npm run tauri:build`
> 遇到该报错，同样用 `CI=true npm run tauri:build` 即可；DMG 本体完整可用，仅少自定义窗口外观。

---

## 使用

1. 打开应用，先到右上角「设置」填入 LLM API key（见下方）。
2. 拖入音频文件（或点击导入）。iCloud 未下载文件会按设置自动下载。
3. 点「AI 解析」，等待大模型返回结果。
4. 在表格 / 向导中审核、编辑四字段；命中节目档案的会自动回填节目名 / 作者。
5. （可选）打开顶部「重命名」开关，按模板预览新文件名。
6. 点「写回」把元数据（及重命名）落盘。
7. 需要还原时，选中文件点「重置」即可回到原始状态。

### 节目档案库

点右上角「节目档案库」，为常听节目建档：

- **节目名**：写回到 album 字段，也是自动匹配的依据。
- **作者**：命中后回填到 artist。
- **匹配关键词**：任一关键词出现在原始文件名中即视为该节目（例如关键词 `爱发电` 可命中 `…丨反派影评丨爱发电.mp3`）。

---

## API Key 申请

TagCast 自己不含任何模型，需你提供 LLM API key。在「设置」中配置 Provider：

| Provider | 协议类型 | Base URL | 模型示例 | 申请地址 |
|---|---|---|---|---|
| DeepSeek（默认） | OpenAI 兼容 | `https://api.deepseek.com` | `deepseek-chat` | <https://platform.deepseek.com/> |
| OpenAI / 兼容端点 | OpenAI 兼容 | `https://api.openai.com/v1` | `gpt-4o-mini` 等 | <https://platform.openai.com/> |
| Anthropic / 兼容端点 | Anthropic 兼容 | `https://api.anthropic.com` | `claude-3-5-haiku` 等 | <https://console.anthropic.com/> |

> 选 "OpenAI 兼容" 时 Base URL 不要带 `/chat/completions`，应用会自动补全 `/chat/completions`；Anthropic 兼容会自动补全 `/v1/messages`。
> API key 仅保存在本机 `localStorage`，不会上传到除所选 Provider 之外的任何地方。

---

## 备份 / 重置说明

- 写回**只修改元数据标签和文件名**，不改音频内容本身。
- 每个文件首次写回**前**，TagCast 会把原文件名 + 原标签（title/album/artist/track）快照到本地数据库：
  - 路径：`~/Library/Application Support/TagCast/tagcast.db`
- 「重置」按钮会用快照恢复原标签、把文件改回原文件名，并删除该快照（文件回到"未处理"状态）。
- 想手动备份/迁移你的节目档案与快照，直接复制上面的 `tagcast.db` 即可。
- 重置依赖快照：若你在 TagCast 之外又手动改了文件名/标签，重置只能恢复到 TagCast 记录的那一次原始状态。

---

## 测试与质量

```bash
# Rust 后端：tag 读写 / LLM 解析(mock) / 节目关键词 / 快照-重置往返
cd src-tauri && cargo test

# 前端纯逻辑：重命名模板渲染 / 节目档案匹配
npm test

# 类型检查 & lint
npm run type-check
cd src-tauri && cargo clippy --all-targets && cargo fmt --check
```

---

## 已知限制（v1）

v1 聚焦"基于文件名的元数据清洗"，以下功能**不在 v1 范围**：

- ❌ 音乐元数据编辑（v1 只做播客，音乐留待 v2）
- ❌ BPM / Key / 情绪 / 流派 / 乐器自动识别
- ❌ 歌词 / 字幕转写、说话人识别、章节检测
- ❌ 相似度搜索、自然语言搜索
- ❌ RSS 订阅
- ❌ 任何需要"听音频内容"才能做的 AI 功能
- ❌ iTunes 播客专属标签（`itunes:author` / `itunes:season` / `itunes:summary` 等，推迟到 v2）
- ❌ 封面图下载 / 编辑（含"同目录同名图片自动导入封面"，推迟到 v2）
- ❌ Mac App Store 分发、收费 / 订阅
- ❌ iOS / Windows / Linux（v1 不分发；代码保持跨平台习惯以便 v2 拓展）
- ❌ 本地 LLM 推理（依赖云端 API，断网 / 限速时不可用）

其他注意事项：

- 为兼容 macOS Finder / Music.app（仅识别到 ID3v2.3），MP3 等 ID3 容器统一写 **ID3v2.3** 标签。
- iCloud 写入通过原生文件 API 协调，但极端并发同步场景下仍建议先确认文件已完整下载。

---

## 技术栈

- **后端**：Rust + Tauri 2，标签读写用 [`lofty`](https://github.com/Serial-ATA/lofty-rs)，HTTP 用 `reqwest`，持久化用 `rusqlite`（SQLite），iCloud 桥接用 `objc2`。
- **前端**：Vue 3 + Vite + TypeScript（Composition API），状态管理 Pinia，表格 TanStack Table，样式 Tailwind CSS。

---

## License

未指定（私有项目）。
