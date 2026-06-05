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



## Session 1: PR6 测试 + DMG 打包 + README + DMG 排障

**Date**: 2026-06-03
**Task**: PR6 测试 + DMG 打包 + README + DMG 排障
**Branch**: `main`

### Summary

完成 PR6：Rust 15 + 前端 10 个测试、scripts/build-dmg.sh、README；定位并修复 bundle_dmg.sh 的 AppleScript 失败（tauri:build 设 CI=true 触发 --skip-jenkins），真机验证 DMG 打包通过；PRD 记录 PR6 完成并新增 v2「AI 解析提示词用户可自定义」backlog。任务已归档。

### Main Changes

(Add details)

### Git Commits

| Hash | Message |
|------|---------|
| `1c7bf11` | (see git log) |
| `60eff3f` | (see git log) |

### Testing

- [OK] (Add test results)

### Status

[OK] **Completed**

### Next Steps

- None - task complete


## Session 2: TagCast v2: 封面 AI 自动导入 + 解析提示词自定义

**Date**: 2026-06-03
**Task**: TagCast v2: 封面 AI 自动导入 + 解析提示词自定义
**Branch**: `main`

### Summary

实现 v2 两项能力。封面：cover.rs 同目录候选扫描 + read_image_data_url 缩略图，llm.rs match_covers 纯文本匹配(阈值0.5)，write.rs CoverOp + WriteInput.coverPath/clearCover，db.rs 快照扩列(had_cover/orig_cover/orig_cover_mime,含旧库幂等ALTER迁移)可重置还原，前端 useCover + store.coverByPath + TableBatch 封面列(缩略图/选图/清除/单击大图预览)。提示词：llm.rs ParseConfig 参数化(system/few-shot/temperature,留空回落默认,自定义失败追加恢复默认提示)，前端 settings.parseConfig 持久化 + SettingsModal 三字段编辑/恢复默认。关键修复:lofty 0.22 ID3v2.3 对 None 描述只写单字节0x00导致UTF-16回读BOM报错,补 Some("") 修复;webp 不支持故收窄jpg/jpeg/png。验证 cargo test(25)/clippy/fmt + npm type-check/test(10) 全绿。

### Main Changes

(Add details)

### Git Commits

| Hash | Message |
|------|---------|
| `6463df6` | (see git log) |

### Testing

- [OK] (Add test results)

### Status

[OK] **Completed**

### Next Steps

- None - task complete


## Session 3: 封面内嵌读取与重置同步 + 文件名规则引擎

**Date**: 2026-06-03
**Task**: 封面内嵌读取与重置同步 + 文件名规则引擎
**Branch**: `main`

### Summary

修复重置后封面不同步并新增导入读取内嵌封面缩略图（image crate）；新增文件名规则引擎：分隔/正则两类规则、优先级可拖拽调整、本地字段级叠加优先+AI兜底并结构化注入规则提示；规则支持固定值（常量赋值）解决归类需求，AI 可由自然语言生成规则与固定值；修复 WebKit 下拖拽排序失效（dragstart 写入 dataTransfer）。

### Main Changes

(Add details)

### Git Commits

| Hash | Message |
|------|---------|
| `f32ef4a` | (see git log) |

### Testing

- [OK] (Add test results)

### Status

[OK] **Completed**

### Next Steps

- None - task complete


## Session 4: 设置面板 Apple 风格重构 + 全局浅/深色主题

**Date**: 2026-06-04
**Task**: 设置面板 Apple 风格重构 + 全局浅/深色主题
**Branch**: `main`

### Summary

把 480px 单列设置弹窗重构为 macOS 系统设置式近全屏 master-detail（左侧 4 类导航：AI 服务/AI 解析/重命名/匹配规则，lucide 图标+Apple 圆角色块，右侧内容区，X/Esc/遮罩关闭）。建立语义颜色 token 体系：style.css 用 CSS 变量定义浅色默认值 + @media(prefers-color-scheme:dark) 整组翻转，tailwind.config.js 注册为颜色(rgb(var(--x)/<alpha-value>))；全应用(App/TableBatch/SingleFileWizard/ProfileLibraryModal/SettingsModal/RuleEditor)约 248 处硬编码 neutral-*/sky-*/状态色迁移到语义 token，随系统外观自动切换。RuleEditor 在宽容器下把原本拥挤换行的分段映射/固定值重排为带标签的 4 列网格，控件统一 Apple 风格，脚本逻辑零改动。新增 lucide-vue-next。验证：npm run build(vue-tsc+vite) 通过、vitest 10/10、产物 CSS 含全部语义类与深色翻转块、无 lint 错误。

### Main Changes

(Add details)

### Git Commits

| Hash | Message |
|------|---------|
| `e0aa4cd` | (see git log) |

### Testing

- [OK] (Add test results)

### Status

[OK] **Completed**

### Next Steps

- None - task complete


## Session 5: 批量UX优化：选图定位/AI解析按选中/封面直选/配置数据管理

**Date**: 2026-06-05
**Task**: 批量UX优化：选图定位/AI解析按选中/封面直选/配置数据管理
**Branch**: `main`

### Summary

选图对话框定位到文件目录；AI解析仅处理选中文件且新增文件默认选中；同目录唯一cover.*<1MB直选封面跳过AI；设置新增数据管理页导出/导入配置(API Key AES-GCM加密)。type-check/cargo test/vitest 全绿。

### Main Changes

(Add details)

### Git Commits

| Hash | Message |
|------|---------|
| `3396866` | (see git log) |

### Testing

- [OK] (Add test results)

### Status

[OK] **Completed**

### Next Steps

- None - task complete
