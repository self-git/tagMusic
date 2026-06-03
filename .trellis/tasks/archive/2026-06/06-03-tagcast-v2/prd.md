# TagCast v2 需求 PRD

> 状态：✅ 已实现，质量门禁全绿（2026-06-03）
> 来源：v1 PRD §13 v2 Backlog（`.trellis/tasks/archive/2026-06/06-01-ai-music-tag-design/prd.md`）
> 范围：本轮 v2 = ①同目录封面 AI 智能导入 + ②AI 解析提示词用户可自定义

## Goal

在 v1（基于文件名的 LLM 解析 + 四字段元数据写回 + 一键重置）已上线、真机验证通过的基础上，新增两项能力：

1. **封面自动导入（AI 匹配）**：导入时扫描音频同目录候选图片，用 LLM 按匹配度挑最合适的一张作为封面，审核确认后随元数据写回。
2. **AI 解析提示词自定义**：把 system prompt、few-shot 示例、temperature 开放到设置供用户编辑，结构契约仍由代码守住。

两项均**不读音频/图片像素内容**，复用 v1 现有写回（`write_metadata` + 快照/重置）与文本 LLM 通道。

## 现状代码事实（勘察结论）

- 写回：`src-tauri/src/write.rs` `apply_fields` 仅写 title/album/artist/track（Some 设置/None 清除），统一 `WriteOptions::use_id3v23(true)`；**未触碰任何图片标签**。
- 快照/重置：`file_snapshots` 表记录原始四字段 + 原文件名；`ensure_snapshot` 首次写回前快照一次，`reset_files` 还原并删快照。
- LLM 解析：`src-tauri/src/llm.rs` `parse_filenames(files, config)`；`SYSTEM_PROMPT` / `build_user_prompt`（few-shot + "序号:文件名"列表 + `{results:[{index,...}]}` 结构指令）**硬编码**；`temperature:0` 写死在 `call_openai`；`call_anthropic` 无 temperature。`extract_json` + serde 解析失败即返回错误。
- Provider 配置：`ProviderConfig`（前后端 camelCase 对齐）仅 `providerType/baseUrl/apiKey/model`。
- 前端配置：`src/store/settings.ts` 用 `localStorage` 持久化 provider / iCloud / 重命名；`useLlmParse` 透传 config；`SettingsModal.vue` 编辑设置。
- 导入：`useAudioImport.ts` → `read_audio_metadata` → `AudioFileMeta{path,fileName,4字段,durationSecs}`。

## Requirements

### A. 封面自动导入（AI 匹配）

- **A1 候选扫描**：导入/解析阶段，按音频文件**所在目录**扫描候选图片，支持扩展名 `jpg/jpeg/png`（lofty 0.22 不支持 webp 嵌入，故 webp 不纳入）。扫描范围仅同目录（不递归）。
- **A2 AI 文本匹配**：复用现有文本 LLM 通道。输入为候选图片**文件名** + 该音频已解析的 `title/album`，让 LLM 选出最匹配的一张并给出置信度；不传图片像素、不读音频。
- **A3 写入时机**：检测/匹配结果在审核 UI 预览（缩略图），随用户点"写回元数据"(`write_metadata`) 一起落盘，统一走快照/重置链路。**不在导入时直接写盘**。
- **A4 手动覆盖 / 无匹配**：审核 UI 可清除选中封面或手动换一张本地图片；目录无候选或 AI 低置信时留空、不写封面。
- **A5 多格式写入**：lofty 写图片标签 — MP3/ID3v2.3 APIC、MP4 covr、FLAC/Vorbis PICTURE，按图片真实 MIME 写入。
- **A6 重置安全**：写封面前先**快照原有封面**（若有则存原图字节 + MIME，无则标记"原本无封面"）；`reset_files` 时还原原封面或清除本次写入的封面，保持"一键回到导入前状态"。

### B. AI 解析提示词自定义

- **B1 可编辑参数**：system prompt、few-shot 示例、temperature —— 三者在设置中**独立编辑**。
- **B2 默认与恢复**：提供默认模板（即现有硬编码内容），每项支持"恢复默认"。
- **B3 结构契约不开放**：`build_user_prompt` 里的"序号:文件名"列表 + `{results:[{index,...}]}` 结构指令由代码继续掌控，用户不可改，保护 index 回填链路。
- **B4 透传**：前端把解析配置随请求传后端，后端用用户值替换 `SYSTEM_PROMPT` / few-shot / temperature；缺省或留空时回落默认。
- **B5 兜底**：用户自定义导致 LLM 返回不合契约（解析失败）时，**报清晰错误并提示"可在设置里恢复默认提示词"**，不静默重试。

## Acceptance Criteria

- [ ] 同目录存在候选图片时，解析后审核 UI 能展示 AI 选中的封面缩略图与置信度
- [ ] 写回元数据后，MP3/M4A/FLAC 文件用其他播放器/查看器能看到嵌入封面
- [ ] 审核 UI 可清除封面、可手动换一张本地图片再写回
- [ ] 目录无候选图或 AI 低置信时，不写入任何封面，且不报错
- [ ] 重置：写过封面的文件一键重置后，封面还原为导入前状态（原本无封面则清除）
- [ ] 设置面板可编辑 system prompt / few-shot / temperature，并各自"恢复默认"
- [ ] 自定义提示词随解析请求生效（可通过改提示词观察解析行为变化验证）
- [ ] 自定义提示词改坏导致返回不合契约时，报错信息含"恢复默认提示词"提示
- [ ] `cargo test` / `cargo clippy -D warnings` / `cargo fmt --check` / `npm test` / `npm run type-check` 全绿

## Definition of Done

- 单元测试：lofty 封面写入/读取往返、封面快照→写回→重置往返、提示词参数化（自定义值生效 + 缺省回落默认）、候选匹配结果回填（mock LLM）
- Lint / typecheck / build 全绿
- README 补充：封面自动导入说明、AI 提示词自定义说明 + 恢复默认
- 已知限制：仅支持 jpg/jpeg/png 封面（lofty 0.22 不支持 webp 嵌入）、AI 文本匹配依赖文件名信息量

## Technical Approach

### 后端（Rust）

- **封面写入**（`write.rs`）：
  - `WriteInput` 增 `cover_path: Option<String>`（要嵌入的图片路径）+ `clear_cover: bool`（显式清除）。
  - `apply_fields` 用 `lofty::picture::Picture::from_reader` 读图片 → `tag.set_picture(0, pic)` / 清除；MIME 按扩展名/探测。
  - 快照扩展：`file_snapshots` 增列 `orig_cover BLOB NULL` + `orig_cover_mime TEXT NULL` + `had_cover INTEGER`（区分"原本无封面"）。`ensure_snapshot` 读首张图片入库。`restore_fields` / `reset_files` 还原或清除。`db::apply_schema` 同步建列（保持可测纯函数）。
- **提示词参数化**（`llm.rs`）：
  - 新增 `ParseConfig { system_prompt: Option<String>, few_shot: Option<String>, temperature: Option<f64> }`，作为 `parse_filenames` 新增可选入参。
  - 拆分 `build_user_prompt`：可替换的 few-shot 段 + 代码固定的"序号列表 + results 结构"段。
  - `call_openai` / `call_anthropic` 用 `ParseConfig` 覆盖 system / temperature，缺省回落现有常量默认。
- **封面 AI 匹配命令**（`llm.rs` 新增）：`match_covers(items: [{path,title,album,candidates:[fileName]}], config) -> [{path, chosen: Option<String>, confidence}]`。纯文本提示词（代码控制，不进 B 项自定义范围），低置信返回 None。
- **候选扫描命令**：`scan_cover_candidates(audio_paths) -> Map<path, Vec<image_path>>`（同目录 jpg/jpeg/png/webp）。

### 前端（Vue 3 + TS）

- `types/llm.ts`：增 `ParseConfig` 类型、封面匹配结果类型；`types/write.ts`/`audio.ts` 增封面字段。
- `store/settings.ts`：持久化 `parseConfig`（systemPrompt/fewShot/temperature），默认值来自共享常量 + 恢复默认；新增封面相关无需常驻设置。
- `SettingsModal.vue`：新增"解析提示词"区（三字段 + 各自恢复默认）。
- `useLlmParse.ts`：透传 `parseConfig`；新增封面匹配/扫描 composable。
- `store/audio.ts`：每行保存封面选择（路径/缩略图/置信度/清除标记）。
- `TableBatch.vue` / `SingleFileWizard.vue`：封面缩略图预览 + 清除 + 手动选图。
- `useWriteback.ts`：写回 payload 带 `coverPath`/`clearCover`。
- 缩略图预览：用 Tauri asset 协议（`convertFileSrc`）或后端返回 data URL。

## Decision (ADR-lite)

- **Context**：v1 已稳定，v2 优先做两项不依赖音频内容、复用现有链路的能力。
- **Decision**：封面走"AI 文本匹配 + 审核后写回 + 快照可重置"；提示词自定义放开 system/few-shot/temperature，结构契约由代码守住，兜底报错引导恢复默认。
- **Consequences**：
  - ✅ 复用 v1 写回/重置/LLM 文本通道，工程量可控
  - ✅ 保持"一键回到导入前"安全承诺（封面纳入快照）
  - ⚠️ AI 文本匹配依赖图片文件名信息量，文件名无意义时匹配弱（留空兜底）
  - ⚠️ webp 等格式在部分播放器渲染兼容性需在 README 标注
  - ⚠️ 快照存原图字节会增大本地 DB 体积（可接受，单文件封面通常 < 数百 KB）

## Out of Scope（本轮 v2 不做）

- iTunes 播客专属标签、音乐元数据编辑、字幕转写、章节检测（推迟 v3+）
- 封面 AI 视觉/多模态匹配（仅做纯文本文件名匹配）
- 跨目录/递归扫描封面、网络下载封面
- 封面裁剪/压缩/格式转换
- 封面匹配提示词的用户自定义（仅文件名解析提示词开放）

## Implementation Plan（小 PR）

- **PR1 — AI 提示词自定义**：`ParseConfig` 后端参数化 + 默认/回落 + 兜底报错提示；前端设置面板三字段 + 恢复默认 + 透传。（自包含，先落地）
- **PR2 — 封面写入通道**：lofty 图片写入 + `WriteInput` 扩展 + 快照扩列 + 重置还原（后端为主 + 往返测试）。
- **PR3 — 封面 AI 匹配 + 前端审核**：候选扫描命令 + `match_covers` 文本匹配 + 前端缩略图预览/清除/手动选图 + 接入写回。

## Technical Notes

- lofty 封面：`lofty::picture::{Picture, MimeType, PictureType}`，`Tag::push_picture` / `remove_picture_type` / `pictures()`。ID3v2.3 下写 APIC，MP4 写 covr，FLAC/Vorbis 写 PICTURE block。
- **lofty 0.22 坑（已修复）**：`Tag::push_picture` 转 APIC 帧时硬编码 `TextEncoding::UTF8`，在 ID3v2.3 下转 UTF-16；当 `Picture.description` 为 `None` 时只写单字节 `0x00`，回读按 UTF-16 解析触发 "invalid byte order mark" 报错，MP3 封面不可读。修复：写前把描述补为 `Some(String::new())`（写出带 BOM 的合法空描述）。见 `write.rs::set_front_cover`。
- **lofty 0.22 不支持 webp**：`MimeType`/`from_reader` 仅识别 png/jpeg/gif/bmp/tiff，故候选与嵌入仅限 jpg/jpeg/png。
- 缩略图预览：未启用 asset 协议，改用后端 `read_image_data_url` 返回 base64 data URL（无依赖手写 base64）。
- 提示词：`extract_json` + serde 解析失败已是错误源头，B5 仅在使用了自定义提示词时追加恢复默认引导。
- 跨平台：扫描用 `std::fs` + `std::path`，不写 Mac 专属 API（延续 v1 习惯）。

## 实现状态（2026-06-03）

- ✅ PR1 提示词自定义：`llm.rs` `ParseConfig` 参数化（system/few-shot/temperature + 留空回落默认 + 自定义失败追加恢复默认提示）；前端 `settings.parseConfig` 持久化 + `SettingsModal` 三字段编辑/恢复默认 + `useLlmParse` 透传。
- ✅ PR2 封面写入通道：`write.rs` `CoverOp` + `WriteInput.coverPath/clearCover`；`db.rs` 快照扩列 `had_cover/orig_cover/orig_cover_mime`（含旧库幂等 ALTER 迁移）；重置还原/清除原封面。
- ✅ PR3 封面 AI 匹配 + 审核：`cover.rs` `scan_cover_candidates` / `read_image_data_url`，`llm.rs` `match_covers`（纯文本、置信阈值 0.5）；前端 `useCover` + `store.coverByPath` + `TableBatch` 封面列（缩略图/选图/清除）+ 接入写回。
- ✅ 验证：`cargo test`(25) / `cargo clippy -D warnings` / `cargo fmt --check` / `npm run type-check` / `npm test`(10) 全绿。
