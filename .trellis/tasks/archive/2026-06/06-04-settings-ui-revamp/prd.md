# 优化设置面板 UI：Apple 风格 + 全局浅深色主题

## Goal

参考 macOS「系统设置」，把设置面板从「480px 单列纵向滚动弹窗」改造为「左侧分类导航 + 右侧内容区」的 Apple 风格主从布局；并让整个应用支持跟随系统的浅色/深色主题。重点解决「文件名匹配规则」(`RuleEditor.vue`) 在窄弹窗里字段密集、排列拥挤的体验问题。

## Requirements

### R1 设置面板 = 近全屏 master-detail
- 形态：近全屏大面板（占窗口 90%+），覆盖在主界面之上（保留遮罩 + 关闭方式）。
- 布局：左侧 ~200px 分类导航栏 + 右侧内容区，点选左侧分类，右侧只渲染该分类内容。
- 左侧分类（4 类，配 lucide 图标 + Apple 风格圆角色块底）：
  1. **AI 服务** — LLM Provider（预设 + 协议/BaseURL/APIKey/Model）
  2. **AI 解析** — 解析提示词（systemPrompt / fewShot / temperature + 各自恢复默认）
  3. **重命名** — 重命名模板 + 变量说明
  4. **匹配规则** — `RuleEditor`（文件名匹配规则）
- header 上的「重命名」「iCloud 自动下载」两个全局开关**保留在顶部不动**。

### R2 RuleEditor 内部重排（核心痛点）
- 利用更宽的右侧内容区，把规则卡片内密集字段（分隔段映射、固定值、AI 生成、预览）重新排布，不再换行挤压。
- 按 Apple 设计语言统一控件外观（输入框、按钮、分组、间距、圆角）。
- 功能、绑定、持久化、拖拽排序逻辑保持不变。

### R3 全局浅/深色主题（跟随系统）
- 整个应用（header、表格批量、单文件向导、节目档案库、设置面板）都支持浅色/深色。
- 跟随 macOS 系统外观自动切换（无需手动开关）。
- 实现方式：语义化颜色 token（CSS 变量 + `@media (prefers-color-scheme: dark)` 翻转），映射进 Tailwind `theme.extend.colors`；用 token 替换现有散落的 `neutral-*`/`sky-*` 等硬编码深色类（约 248 处，6 个文件）。

### R4 图标
- 安装 `lucide-vue-next`，左侧分类项与必要处使用统一图标。

## Acceptance Criteria

- [x] 点「设置」打开近全屏面板，左侧 4 类导航，点击切换右侧内容。
- [x] 匹配规则在右侧宽区域内排版清晰，字段不再拥挤换行。
- [x] 浅/深色随系统外观自动切换；浅色与深色下文字/背景对比度可读，全应用一致。
- [x] 所有原有设置项（Provider/解析/重命名/规则）功能、绑定、localStorage 持久化行为不变。
- [x] header 两个开关位置与行为不变。
- [x] 面板有明确关闭方式（X / ESC / 点遮罩）。

## Definition of Done

- `npm run type-check` 通过；既有 vitest 测试不回归。
- 不改动 store 数据结构与后端（Rust）。
- 不引入与本任务无关的重构。

## Technical Approach

- **主题层**：在 `src/style.css` 定义语义 token CSS 变量（如 `--color-base / --color-surface / --color-elevated / --color-border / --color-text / --color-text-muted / --color-accent`），用 `@media (prefers-color-scheme: dark)` 提供深色取值；`tailwind.config.js` 把这些 token 注册为颜色（`rgb(var(--xxx) / <alpha-value>)`）。各组件用语义类（如 `bg-base text-text border-border`）替换硬编码 `neutral-*`，写一次即随系统翻转。
- **设置面板**：`SettingsModal.vue` 重构为 master-detail 容器（左 nav + 右 `<component :is>` / 条件渲染），按分类拆为子组件或区块；保留 `useSettingsStore` 全部绑定。
- **RuleEditor**：仅调整 template 排版与控件样式（响应更宽容器，字段分组横排），脚本逻辑不动。
- **图标**：`npm i lucide-vue-next`。

## Decision (ADR-lite)

- **Context**：设置项全堆在窄弹窗、规则编辑器拥挤；用户要 Apple 系统设置式体验且整体翻新。
- **Decision**：近全屏 master-detail 弹窗 + 4 类导航（header 开关不并入）+ lucide 图标 + 全局语义 token 主题跟随系统浅/深色。
- **Consequences**：全应用色彩类需迁移到语义 token（一次性较大但机械、低风险，且换来浅/深色一键支持与统一外观）；后续若要手动主题开关，token 方案可平滑扩展。

## Out of Scope

- 后端 / Rust 改动。
- 设置项功能逻辑变更（仅排版、导航结构、主题色）。
- 手动「浅/深色切换开关」（本期只跟随系统）。

## Technical Notes

- 涉及文件：
  - 主题：`src/style.css`、`tailwind.config.js`，以及全部含深色类的视图/组件（`src/App.vue`、`src/views/TableBatch.vue`、`src/views/SingleFileWizard.vue`、`src/components/RuleEditor.vue`、`src/components/ProfileLibraryModal.vue`、`src/components/SettingsModal.vue`）。
  - 设置面板：`src/components/SettingsModal.vue`（重构）、可能新增分类子组件、`src/components/RuleEditor.vue`（排版）。
- `src/store/settings.ts` 不动；`src/App.vue` 仅可能微调 header 配色 token。
- 依赖新增：`lucide-vue-next`。

## Implementation Plan (small PRs)

- **PR1 主题基建**：定义语义 token（`style.css` + `tailwind.config.js`），跑通 `prefers-color-scheme` 翻转；先迁移 `App.vue` header 验证链路。
- **PR2 全应用色彩迁移**：把 TableBatch / SingleFileWizard / ProfileLibraryModal 的 `neutral-*`/`sky-*` 替换为语义 token，校验浅/深色可读性。
- **PR3 设置面板重构**：`SettingsModal.vue` 改 master-detail（近全屏 + 左 4 类导航 + lucide 图标 + 关闭方式），各分类内容迁入。
- **PR4 RuleEditor 重排**：宽容器下重排规则卡片字段 + Apple 风格控件统一。
- 收尾：`type-check` + vitest 全绿。
