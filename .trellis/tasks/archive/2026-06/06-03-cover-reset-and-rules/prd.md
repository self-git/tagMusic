# 封面重置同步与文件名规则引擎

## Goal

优化两处体验：
1. 重置文件时，封面列应同步回到默认/原始状态；并且能正确读出文件已内嵌的封面（包括本软件自己写入的）。
2. 引入用户可自定义的「文件名匹配规则」，支持多条规则、优先级可拖拽/设置调整，调用 AI 时把这些规则与优先级以结构化方式传给模型。

## What I already know（代码事实）

### 问题 1 — 封面
- `src-tauri/src/audio.rs::read_audio_metadata` 读取标签时**没有读取内嵌封面**，`AudioFileMeta` 也无 cover 字段 → 导入后封面列永远只反映「同目录候选图 + AI/手动匹配」，看不到文件本身已有的封面。
- `src/store/audio.ts::applyResetOutcomes` 重置后**没有清理 `coverByPath`** → 重置后封面列仍显示之前 AI/手动匹配的图，没有回到默认/空。
- 后端 `reset_files`（write.rs）已能把磁盘封面字节还原为原始（had_cover 快照机制完整），所以「磁盘」是对的，问题在「前端展示」与「导入读取」两端。

### 问题 2 — 文件名规则
- 当前解析是纯 AI：前端 `useLlmParse` → 后端 `parse_filenames`（llm.rs）只把文件名列表发给 LLM。
- 解析配置 `ParseConfig`（systemPrompt / fewShot / temperature）持久化在 localStorage（settings.ts），后端用 `resolved()` 回落默认。
- 字段固定四项：title / album / artist / track（见 `AudioFileMeta`、`ParseResult`）。
- 暂无任何本地正则/分隔规则引擎。

## Decisions

### 问题 1 — 封面（已定 · 方案 A）
- 导入时读取文件**内嵌封面**作为「基准封面」并展示。
- AI/手动匹配为「叠加层」，覆盖基准展示。
- 重置 = 清除叠加层 → 封面列回落显示文件当前真实封面（重置后磁盘已还原原始封面，故显示原封面；原本无封面则显示「无」）。

### 问题 2 — 规则引擎（已定）
- 执行模型：**混合（方案 C）**。本地可执行规则按优先级先跑；未命中/未填满走 AI，且把规则结构化注入 AI prompt 作引导。
- 规则类型：**两种** —— ①分隔符模板（指定分隔符切分 → 第 N 段映射字段）②正则 + 命名捕获组（(?<title>)/(?<album>)/(?<artist>)/(?<track>)）。
- **固定值/常量赋值（追加）**：两种规则均可配「固定值」——规则命中（正则整体匹配 / 分隔符存在）后为指定字段写入常量，覆盖捕获/分段。解决「捕获组只能提取已有文本、无法赋常量」的根本限制（如 含 `QA\d{3}` → 节目=会员问答）。固定值一并注入 AI 提示。
- 编辑器对非法正则（含重复命名组）显式提示「正则语法错误」，不再静默当作未命中。
- **字段级叠加（方案 A）**：按优先级依次跑所有规则，高优先级已填字段不被覆盖，低优先级只补空缺；跑完仍空缺的字段交给 AI 补。
- **AI 辅助生成规则**：规则编辑器内一个自然语言输入框 +「生成」按钮 → AI 返回正则/分隔配置填入表单；保存前可用当前已导入文件名实时预览匹配结果。

## Requirements
### 问题 1（封面）
- [ ] 导入时读出文件内嵌封面（基准），表格封面列展示
- [ ] AI/手动匹配作为叠加层覆盖基准展示
- [ ] 重置后清除叠加层，封面列回落显示文件真实封面（原有封面则显示原封面，原无则「无」）

### 问题 2（规则引擎）
- [ ] 规则数据模型：支持「分隔符」与「正则」两种类型，可填 title/album/artist/track
- [ ] 规则列表 UI：增删改 + 拖拽/上下移动调整优先级
- [ ] 本地匹配引擎：按优先级字段级叠加（前端 TS 执行，正则用 JS 命名捕获组）
- [ ] AI 兜底：仍有空缺字段的文件走 `parse_filenames`，并把规则按优先级结构化注入 prompt
- [ ] AI 生成规则：自然语言 → 正则/分隔配置（新增后端命令，复用 provider 配置）
- [ ] 规则编辑器内用已导入文件名实时预览匹配
- [ ] 规则持久化（localStorage，复用 settings store 模式）

## Technical Approach（草案）

### 问题 1
- 后端 `audio.rs`：`AudioFileMeta` 增 `embeddedCover: Option<String>`（缩略图 data URL）。`read_one` 读取前置/首张内嵌封面 → **解码缩放为小缩略图(约 128px)的 JPEG** → base64 data URL。
- 新增依赖 `image`（仅 jpeg/png 特性，纯 Rust）用于解码+缩放，避免批量大图占内存/卡顿（用户确认要缩略图）。缩放失败时回落不显示（best-effort，不阻断导入）。
- 前端 `store/audio.ts`：封面展示改为「基准(embeddedCover) ← 叠加层(coverByPath)」；`coverThumb()` 优先叠加层、否则基准。`applyResetOutcomes` 清除该 path 的 `coverByPath` 叠加。
- 后端 `reset_files` 已正确还原磁盘封面，无需改动。

### 问题 2
- 新增 `src/types/rule.ts`：`FilenameRule`（id/name/enabled/type:'separator'|'regex'/配置/优先级隐含于数组顺序）。
- 新增 `src/composables/useRules.ts` 或 store：本地匹配引擎（字段级叠加）+ 预览。
- `settings.ts`：新增 rules 持久化。
- 解析链路 `useLlmParse`：先本地跑规则 → 收集空缺字段文件 → 调 `parse_filenames`（透传结构化规则） → 字段级合并回结果。
- 后端 `llm.rs`：`parse_filenames` 增可选 `rules` 结构注入 prompt；新增 `generate_filename_rule(description, config)` 命令产出正则/分隔配置。

## Out of Scope
- 不改后端 `reset_files`/快照机制（已正确）。
- 规则不做导入/导出文件、不做云同步（仅本地 localStorage）。
- 规则匹配只针对文件名（不含目录、不读音频内容）。

## Open Questions
- （已定）内嵌封面后端生成缩略图，新增 `image` 依赖。

## Technical Notes
- 关键文件：`src-tauri/src/audio.rs`、`src-tauri/src/write.rs`、`src-tauri/src/cover.rs`、`src-tauri/src/llm.rs`、`src/store/audio.ts`、`src/store/settings.ts`、`src/composables/useCover.ts`、`src/composables/useLlmParse.ts`、`src/composables/useWriteback.ts`、`src/views/TableBatch.vue`、`src/types/*.ts`
