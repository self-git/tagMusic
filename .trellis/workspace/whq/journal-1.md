# Journal - whq (Part 1)

> AI development session journal
> Started: 2026-06-01

---

## 2026-06-01 — Session 1: AI 播客元数据编辑器 v1 PRD 定稿

### 触发

用户参考 macOS "Music Tag" 类软件，希望做一款带 AI 能力的元数据编辑器。

### 过程

通过 15 轮头脑风暴收敛 v1 范围。关键决策（按时间顺序）：

1. **目标用户**：个人音乐收藏 + 播客收藏爱好者（双）
2. **MVP 范围**：v1 只做播客（差异化竞争）
3. **数据源**：仅本地文件（不上 RSS 订阅）
4. **AI 推理**：云 API 优先（不读音频内容）
5. **元数据字段**：title + album + artist + track（4 个）
6. **平台/技术栈**：macOS only + Tauri（Rust + Web 前端）
7. **音频格式**：全格式（MP3/M4A/FLAC/OGG/OPUS/WAV/AIFF）
8. **审核 UI**：表格批量 + 单文件向导（双模式）
9. **节目档案**：持久档案库 + 批次内整列填充（双模式）
10. **LLM 提供商**：默认 DeepSeek + 自定义 OpenAI Compatible / Anthropic 兼容
11. **写入安全**：只改元数据+文件名 + 记录原名快照 + 一键重置
12. **重命名**：开关切换（只改元数据 / 按模板重命名）
13. **iTunes 播客标签**：推迟到 v2
14. **分发**：免费 + DMG 直装
15. **iCloud 支持**：默认自动下载未下载文件 + 设置可切"仅提示不下载"

### 用户特别关注的点

- **iCloud 路径**：用户明确要求支持 `~/Library/Mobile Documents/com~apple~CloudDocs/...` 下的文件，包括未下载的 `.icloud` 占位符场景
- **跨平台迁移**：用户问到"后续想做多平台好不好迁"——已回答 Tauri 设计上跨平台友好，代码层不写 Mac 专属 API 即可

### 交付物

- ✅ PRD 已定稿写入 `.trellis/tasks/06-01-ai-music-tag-design/prd.md`（含 TL;DR、12 个章节、完整 Acceptance Criteria、6 个 PR 实施计划）
- ⏸️ **等待用户最终确认 + 项目命名**（候选：PodMeta / PodcastTagger / TagCast / 播客标签）

### 待用户决定

1. 是否同意 PRD 内容（如果需要修改请直接说）
2. 项目命名（4 个候选 + 用户可自提）
3. 确认后才会 `task.py start` 进入实施阶段

### 不会立即做的事

- 不跑 `task.py start`（用户明确"等我确定了再开发"）
- 不创建子任务（PR1-PR6 子任务等用户确认后开）
- 不 git init（待用户确认后开）

