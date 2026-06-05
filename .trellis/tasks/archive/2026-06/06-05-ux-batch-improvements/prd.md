# 优化：选图目录定位 / AI解析按选中 / 封面自动匹配 / 配置导入导出

## Goal

围绕表格批量视图（`TableBatch.vue`）与设置面板（`SettingsModal.vue`），修复 4 个交互/可用性问题，提升批量处理播客音频时的效率与确定性。

## What I already know（代码勘察结论）

- **问题1 选图目录**：`useCover.pickCover(path)` 调用 `open({ multiple:false, filters })`，未传 `defaultPath`，故文件选择对话框不会定位到音频所在目录。Tauri dialog 支持 `defaultPath`，可取 `path` 的父目录。
- **问题2 AI解析范围**：`TableBatch.parseAll()` 全程基于 `files.value`（全部文件）。选中集合 `selected` 是组件内 `ref<Set<string>>`，导入时不会自动选中。导入入口在 `useAudioImport.importPaths` → `store.addFiles`。
- **问题3 封面自动匹配**：后端 `scan_cover_candidates`（`cover.rs`）返回同目录图片完整路径列表（已排序），无文件大小、无 "cover" 命名优先级；前端 `useCover.scanAndMatch` 拿候选后直接交 AI 匹配。
- **问题4 配置导入导出**：设置项均在 localStorage（6 个键：`icloudAutoDownload` / `llmProvider` / `renameEnabled` / `renameTemplate` / `parseConfig` / `filenameRules`）；节目档案库在 SQLite（经 Rust IPC，`useProfilesStore`）。当前设置面板有 4 个分类页，无导入导出入口。

## Requirements（evolving）

### 问题1：选图定位到文件所在目录
- 点击「选图」时，文件选择对话框的 `defaultPath` 设为该音频文件的父目录。

### 问题2：AI 解析仅处理选中文件 + 新文件默认选中
- 拖入 / 新增的文件，默认进入选中集合。
- 「AI 解析」按钮仅解析**选中**文件；若选中为空，用状态栏提示「未选中文件」而非解析全部。

### 问题3：cover.* 单张小图直接选中
- 同目录存在以 `cover` 命名的图片（cover.png/cover.jpg/cover.jpeg…），且这类图**只有一张**、大小 < 1MB 时，直接选中为封面（不走 AI 匹配）。

### 问题4：设置导出 / 导入恢复
- 设置面板新增「导出配置」与「导入配置」入口。
- 导出：将配置写入 JSON 文件（保存对话框）。
- 导入：读取 JSON 文件并恢复配置。

## Decisions（已确认）

- **问题2 范围**：AI 解析、封面扫描匹配、节目档案自动回填，全部只针对**选中**文件，行为一致。
- **问题3 判定**：同目录中以 `cover` 为文件名（cover.png/cover.jpg/cover.jpeg…，大小写不敏感）的图片**只有一张**且 **<1MB** 时，直接选中为封面，并跳过该文件的 AI 封面匹配；不满足则该文件仍走 AI 匹配。
- **问题4 范围**：导出/导入 = 6 项设置 + API Key；**不含**节目档案库（SQLite）。API Key 用应用**内置密钥对称加密**（Web Crypto AES-GCM），导出文件不明文存 Key，导入时自动解密。导入采用**整体覆盖**：文件中存在的项直接替换当前值。

## Acceptance Criteria

- [ ] 点击「选图」时对话框 `defaultPath` 定位到音频所在目录
- [ ] 拖入/新增文件后默认选中
- [ ] 「AI 解析」只处理选中文件；选中为空时状态栏提示「未选中文件」，不解析
- [ ] 解析流程中的封面扫描匹配、节目档案回填同样只处理选中文件
- [ ] 同目录唯一 `cover.*` 且 <1MB 时自动选中为封面，且不进入 AI 匹配
- [ ] 设置面板可「导出配置」为 JSON 文件（API Key 加密），并通过「导入配置」整体覆盖恢复

## Out of Scope

- 「写回元数据」按钮的处理范围（仍处理全部，本次不改）
- 重命名 / 写回逻辑本身
- 节目档案库（SQLite）的导入导出

## Technical Notes

- 前端：`src/composables/useCover.ts`、`src/composables/useAudioImport.ts`、`src/views/TableBatch.vue`、`src/store/audio.ts`、`src/store/settings.ts`、`src/components/SettingsModal.vue`
- 后端：`src-tauri/src/cover.rs`（封面扫描，需新增大小/命名信息）
- Tauri 插件：`@tauri-apps/plugin-dialog`（open 的 defaultPath / save）、`@tauri-apps/plugin-fs`（如需读写 JSON）
