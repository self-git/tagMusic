# Research: Vue 3 + TypeScript Frontend Codebase Analysis

- **Query**: Analyze Vue 3 + TypeScript frontend codebase under src/ for patterns, conventions, and real code examples
- **Scope**: internal
- **Date**: 2026-06-05

---

## 1. Directory Structure

### Full Source Tree Under `src/`

```
src/
├── App.vue                          # Root layout: header nav, settings/profile modals, <router-view>
├── main.ts                          # App bootstrap: createApp, Pinia, router, style.css
├── style.css                        # Tailwind directives + semantic CSS custom properties (light/dark)
├── vite-env.d.ts                    # Vite client type reference
├── shims-vue.d.ts                   # .vue module declaration for TypeScript
├── components/                      # Shared/reusable Vue components
│   ├── SettingsModal.vue            # Full-screen modal: AI service, parse, rename, rules, data tabs
│   ├── RuleEditor.vue               # Filename matching rule CRUD editor with AI generation
│   └── ProfileLibraryModal.vue      # Show profile (podcast archive) CRUD modal
├── composables/                     # Custom Vue composables (use* pattern)
│   ├── useAudioImport.ts            # File import: drag-drop, file dialog, iCloud orchestration
│   ├── useConfigIo.ts               # Settings export/import with AES-GCM encryption
│   ├── useCover.ts                  # Cover image scanning, AI matching, manual pick
│   ├── useIcloud.ts                 # iCloud file status check & download polling
│   ├── useLlmParse.ts               # LLM filename parsing via Tauri IPC
│   ├── useRename.ts                 # Rename template rendering + preview composable
│   ├── useRename.test.ts            # Tests for renderName function
│   ├── useRules.ts                  # Local filename rule matching engine (regex/separator)
│   └── useWriteback.ts              # Metadata writeback & reset via Tauri IPC
├── router/
│   └── index.ts                     # Vue Router config: hash history, /table, /wizard
├── store/                           # Pinia stores
│   ├── audio.ts                     # Audio file state: files list, nav, parse results, cover state
│   ├── profiles.ts                  # Show profile CRUD, auto-matching, library modal state
│   ├── profiles.test.ts             # Tests for profile matching logic
│   └── settings.ts                  # User settings: LLM config, rename, parse config, rules
├── types/                           # TypeScript type definitions (shared)
│   ├── audio.ts                     # AudioFileMeta interface
│   ├── cover.ts                     # CoverCandidates, CoverMatchResult, CoverSelection
│   ├── icloud.ts                    # ICloudStatus interface
│   ├── llm.ts                       # ProviderConfig, ParseConfig, ParseResult
│   ├── profile.ts                   # ShowProfile, ShowProfileInput
│   ├── rule.ts                      # FilenameRule, RuleMatch, RuleHint, etc.
│   └── write.ts                     # WriteInput, WriteOutcome, ResetOutcome
└── views/                           # Route-level page components
    ├── TableBatch.vue               # Table view: batch import, AI parse, fill, writeback
    └── SingleFileWizard.vue         # Wizard view: single-file step-through editing
```

**Key directory principles:**
- `types/` — shared TypeScript interfaces that mirror Rust backend structs (camelCase alignment)
- `store/` — Pinia stores using Composition API (`defineStore` with setup function)
- `composables/` — reusable stateful/stateless logic (Vue 3 Composition API)
- `components/` — shared presentational + stateful components
- `views/` — route-level page components (no nested sub-directories, flat)

---

## 2. Component Guidelines

### `<script setup lang="ts">` Usage

**100% of Vue SFCs use `<script setup lang="ts">`** — no Options API components exist in the codebase.

- `App.vue:1`: `<script setup lang="ts">`
- `SettingsModal.vue:1`: `<script setup lang="ts">`
- `RuleEditor.vue:1`: `<script setup lang="ts">`
- `ProfileLibraryModal.vue:1`: `<script setup lang="ts">`
- `TableBatch.vue:1`: `<script setup lang="ts">`
- `SingleFileWizard.vue:1`: `<script setup lang="ts">`

### Props Definition Pattern

**Type-based generic props** using `defineProps<{ ... }>()` — no `withDefaults`, no runtime validation:

```typescript
// SettingsModal.vue:10
defineProps<{ open: boolean }>();
```

```typescript
// ProfileLibraryModal.vue:7
defineProps<{ open: boolean }>();
```

### Emits Pattern

**Type-based generic emits** using `defineEmits<{ ... }>()`:

```typescript
// SettingsModal.vue:11
const emit = defineEmits<{ close: [] }>();

// ProfileLibraryModal.vue:8
const emit = defineEmits<{ close: [] }>();
```

The emit type uses tuple syntax: `close: []` means no payload arguments.

### Component Imports

All use `@/` path alias for imports:

```typescript
// App.vue:4-9
import { useAudioStore } from "@/store/audio";
import { useSettingsStore } from "@/store/settings";
import { useProfilesStore } from "@/store/profiles";
import SettingsModal from "@/components/SettingsModal.vue";
import ProfileLibraryModal from "@/components/ProfileLibraryModal.vue";
```

### Modal Pattern

Modals use a props/emits open/close contract. The parent controls visibility with a reactive ref:

```html
<!-- App.vue:91-92 -->
<SettingsModal :open="settingsOpen" @close="settingsOpen = false" />
<ProfileLibraryModal :open="libraryOpen" @close="profiles.closeLibrary()" />
```

Modal components conditionally render with `v-if="open"` and close on `@click.self` or `@keydown.esc`:

```html
<!-- SettingsModal.vue:46-51 -->
<div
  v-if="open"
  class="fixed inset-0 z-50 flex items-center justify-center bg-black/50 p-6"
  @click.self="emit('close')"
  @keydown.esc.window="emit('close')"
>
```

### Template Patterns

- **Reactive store access**: Use `v-model` directly on store refs via `storeToRefs`
- **Conditional rendering**: Heavy use of `v-if`, `v-else-if`, `v-else`, `v-show`
- **Router links**: `<router-link>` with `active-class` for navigation tabs
- **Dynamic components**: `<component :is="t.icon">` for icon components (lucide-vue-next)
- **Direct mutation**: Store properties mutated directly via `v-model` (no copy-to-local pattern): `v-model="row.original.title"`, `v-model="llmProvider.apiKey"`

---

## 3. State Management (Pinia)

### Store Definition Pattern

**All stores use the Composition API setup function style** (`defineStore("name", () => { ... })`):

```typescript
// src/store/settings.ts:100
export const useSettingsStore = defineStore("settings", () => {
  // ...
  return { icloudAutoDownload, llmProvider, /* ... */ };
});
```

```typescript
// src/store/audio.ts:19
export const useAudioStore = defineStore("audio", () => {
  // ...
  return { files, currentIndex, /* ... */ };
});
```

```typescript
// src/store/profiles.ts:15
export const useProfilesStore = defineStore("profiles", () => {
  // ...
  return { profiles, libraryOpen, /* ... */ };
});
```

### State (ref)

State is declared with `ref()` from Vue:

```typescript
// settings.ts:102-111
const icloudAutoDownload = ref(readBool(ICLOUD_KEY, true));
const llmProvider = ref<ProviderConfig>(readProvider());
const renameEnabled = ref(readBool(RENAME_ENABLED_KEY, false));
const renameTemplate = ref(readString(RENAME_TEMPLATE_KEY, DEFAULT_RENAME_TEMPLATE));
const parseConfig = ref<ParseConfig>(readParseConfig());
const rules = ref<FilenameRule[]>(readRules());
```

```typescript
// audio.ts:20-28
const files = ref<AudioFileMeta[]>([]);
const currentIndex = ref(0);
const confidenceByPath = ref<Record<string, number | null>>({});
const writtenPaths = ref<Set<string>>(new Set());
const coverByPath = ref<Record<string, CoverSelection>>({});
```

### Getters (computed)

Getters are declared with `computed()`:

```typescript
// audio.ts:30-35
const currentFile = computed<AudioFileMeta | null>(
  () => files.value[currentIndex.value] ?? null,
);
const total = computed(() => files.value.length);
```

### Actions (functions)

Actions are plain functions defined inside the setup function:

```typescript
// audio.ts:39-44
function addFiles(list: AudioFileMeta[]): void {
  const seen = new Set(files.value.map((f) => f.path));
  const fresh = list.filter((f) => !seen.has(f.path));
  if (fresh.length === 0) return;
  files.value = [...files.value, ...fresh];
}
```

**Key state mutation pattern**: Always replace array refs with new arrays (never `.push()`), because TanStack Table memoizes row models by data reference:

```typescript
// audio.ts:43 — replaces reference to trigger table re-render
files.value = [...files.value, ...fresh];
```

### Async Actions (Tauri IPC)

Actions that call Tauri backend are `async` and use `await invoke<T>()`:

```typescript
// profiles.ts:21-23
async function load(): Promise<void> {
  profiles.value = await invoke<ShowProfile[]>("list_show_profiles");
}
```

```typescript
// profiles.ts:25-28
async function save(input: ShowProfileInput): Promise<void> {
  await invoke<number>("save_show_profile", { profile: input });
  await load();
}
```

### Store Consumption in Components

Components use `storeToRefs()` for reactive destructuring:

```typescript
// App.vue:12
const { total } = storeToRefs(store);
// App.vue:16
const { icloudAutoDownload, renameEnabled } = storeToRefs(settings);
// App.vue:20
const { libraryOpen } = storeToRefs(profiles);
```

For non-ref access (actions, functions), call the store directly:

```typescript
// App.vue:24
void profiles.load();
```

### localStorage Persistence

The settings store uses `watch()` with `{ deep: true }` to persist to localStorage:

```typescript
// settings.ts:113-147
watch(icloudAutoDownload, (v) => {
  localStorage.setItem(ICLOUD_KEY, v ? "1" : "0");
});
watch(llmProvider, (v) => {
  localStorage.setItem(LLM_KEY, JSON.stringify(v));
}, { deep: true });
```

---

## 4. Type Safety

### Type Organization

All shared types live under `src/types/` — one file per domain concept:

| File | Key Exports | Purpose |
|---|---|---|
| `types/audio.ts` | `AudioFileMeta` (interface) | Audio file metadata, mirrors Rust `read_audio_metadata` |
| `types/llm.ts` | `ProviderConfig`, `ParseConfig`, `ParseResult` (interfaces), `ProviderType` (type) | LLM provider + parse config |
| `types/rule.ts` | `FilenameRule` (type), `SeparatorRule`, `RegexRule` (interfaces), `RuleField`, `RuleMatch`, `RuleHint` | Filename matching rules |
| `types/cover.ts` | `CoverCandidates`, `CoverMatchResult`, `CoverSelection` (interfaces) | Cover image state |
| `types/write.ts` | `WriteInput`, `WriteOutcome`, `ResetOutcome` (interfaces) | Writeback IPC types |
| `types/profile.ts` | `ShowProfile`, `ShowProfileInput` (interfaces) | Podcast archive profiles |
| `types/icloud.ts` | `ICloudStatus` (interface) | iCloud file status |

### Interfaces vs Types

- **`interface`** used for most data structures (e.g., `AudioFileMeta`, `ShowProfile`, `CoverSelection`)
- **`type`** used for union/discriminated types and aliases:
  ```typescript
  // llm.ts:2
  export type ProviderType = "openai" | "anthropic";
  // rule.ts:7
  export type RuleField = "title" | "album" | "artist" | "track";
  // rule.ts:41
  export type FilenameRule = SeparatorRule | RegexRule;
  ```

### Nullable Fields

The codebase uses `string | null` (not `undefined`) for optional fields, mirroring Rust's `Option<String>`:

```typescript
// audio.ts:10-15
export interface AudioFileMeta {
  path: string;
  fileName: string;
  title: string | null;
  album: string | null;
  artist: string | null;
  track: number | null;
  durationSecs: number | null;
  embeddedCover: string | null;
}
```

### Type Imports

Types are always imported with `import type` for compile-time only:

```typescript
import type { AudioFileMeta } from "@/types/audio";
import type { ParseResult } from "@/types/llm";
```

### Type Utilities

- `Partial<>` used for optional override patterns: `Partial<ConfigSnapshot>`, `Partial<Record<RuleField, string>>`
- `Record<>` used for key-value mappings: `Record<string, number | null>`, `Record<RuleField, string>`
- `as const` used for readonly tuple arrays: `export const RULE_FIELDS = ["title", "album", "artist", "track"] as const;` (rule.ts:10)

### Strict TypeScript

`tsconfig.json` has full strictness:
```json
{
  "strict": true,
  "noUnusedLocals": true,
  "noUnusedParameters": true,
  "noFallthroughCasesInSwitch": true
}
```

### Path Aliases

`@/` maps to `src/` via both `tsconfig.json` (paths) and `vite.config.ts` (resolve alias):

```json
// tsconfig.json:19
"paths": { "@/*": ["src/*"] }
```
```typescript
// vite.config.ts:10
alias: { "@": path.resolve(__dirname, "./src") },
```

### Discriminated Unions

`FilenameRule` uses a discriminated union with a `type` literal field:

```typescript
// rule.ts:28-41
export interface SeparatorRule extends BaseRule {
  type: "separator";  // discriminant
  separator: string;
  mapping: Partial<Record<RuleField, number>>;
}
export interface RegexRule extends BaseRule {
  type: "regex";  // discriminant
  pattern: string;
}
export type FilenameRule = SeparatorRule | RegexRule;
```

Components narrow by checking `rule.type === "regex"` / `rule.type === "separator"`.

### Inline Type Variants

Some types are defined inline in components:
```typescript
// SettingsModal.vue:19
type Preset = "deepseek" | "openai" | "anthropic";
// SettingsModal.vue:33
type Tab = "provider" | "parse" | "rename" | "rules" | "data";
// audio.ts:9
type EditableTextField = "title" | "album" | "artist";
```

---

## 5. Styling (Tailwind CSS)

### Configuration

- **Tailwind v3.4.14** with PostCSS + Autoprefixer
- Config: `tailwind.config.js` — content scans `./index.html` and `./src/**/*.{vue,ts}`
- Custom semantic color tokens defined as Tailwind `colors` extensions

### Semantic Color System

A custom semantic color token system using CSS custom properties (`--c-*`) as RGB channels, allowing Tailwind's alpha compositing via `rgb(var(--x) / <alpha-value>)`:

**`tailwind.config.js`** defines semantic color classes:
```javascript
colors: {
  base: "rgb(var(--c-base) / <alpha-value>)",
  surface: "rgb(var(--c-surface) / <alpha-value>)",
  field: "rgb(var(--c-field) / <alpha-value>)",
  elevated: "rgb(var(--c-elevated) / <alpha-value>)",
  line: "rgb(var(--c-line) / <alpha-value>)",
  edge: "rgb(var(--c-edge) / <alpha-value>)",
  strong: "rgb(var(--c-strong) / <alpha-value>)",
  muted: "rgb(var(--c-muted) / <alpha-value>)",
  faint: "rgb(var(--c-faint) / <alpha-value>)",
  dim: "rgb(var(--c-dim) / <alpha-value>)",
  accent: { DEFAULT: "...", hover: "...", fg: "..." },
  info: { bg: "...", edge: "...", fg: "..." },
  success: { DEFAULT: "...", hover: "...", bg: "...", edge: "...", fg: "..." },
  warning: { bg: "...", edge: "...", fg: "..." },
  danger: { bg: "...", edge: "...", fg: "..." },
}
```

**`style.css`** defines the CSS custom properties with light/dark variants via `prefers-color-scheme: dark`:

```css
:root {
  color-scheme: light dark;
  --c-base: 245 245 247;      /* window bg */
  --c-surface: 255 255 255;   /* card / panel */
  --c-field: 255 255 255;     /* input bg */
  --c-elevated: 232 232 237;  /* badge / hover bg */
  --c-line: 227 227 232;      /* fine divider */
  --c-edge: 209 209 214;      /* control border */
  --c-strong: 29 29 31;       /* primary text */
  --c-muted: 95 95 99;        /* secondary text */
  --c-faint: 138 138 142;     /* tertiary text */
  --c-dim: 176 176 181;       /* quaternary / placeholder */
  /* ... accent, info, success, warning, danger tokens ... */
}
```

### Usage Patterns in Templates

Components use semantic color classes exclusively — never raw color values:

```html
<!-- App.vue:33 -->
<div class="flex h-screen flex-col bg-base text-strong">
  <!-- header:34 -->
  <header class="flex items-center justify-between border-b border-line px-5 py-3">
  <!-- nav buttons:44 -->
  <nav class="ml-2 flex items-center gap-1 rounded-lg bg-surface p-0.5 text-sm">
```

```html
<!-- TableBatch.vue:196-197 -->
<button class="rounded-lg border border-edge px-3 py-1.5 text-sm text-muted hover:bg-elevated">
  添加文件
</button>
<!-- primary action:203-204 -->
<button class="rounded-lg bg-accent px-3 py-1.5 text-sm font-medium text-white hover:bg-accent-hover">
  AI 解析
</button>
```

### Common Tailwind Patterns Observed

| Pattern | Example Classes |
|---|---|
| Layout | `flex`, `flex-col`, `h-screen`, `min-h-0 flex-1 overflow-auto` |
| Spacing | `px-4 py-2`, `gap-3`, `mt-1.5` |
| Borders | `border border-line`, `border-b border-edge`, `rounded-xl` |
| Typography | `text-sm text-muted`, `text-xs text-faint`, `text-lg font-semibold` |
| Interactive | `hover:bg-elevated`, `focus:border-accent`, `disabled:opacity-50` |
| Positioning | `fixed inset-0 z-50`, `sticky top-0` |

### No Scoped Styles

**Zero components use `<style scoped>` blocks.** All styling is done via Tailwind utility classes in the template. No custom component-level CSS exists.

---

## 6. Hooks/Composables

### Naming Convention

**All composables follow the `use*` naming convention.** Each composable is exported as a named function:

| File | Export Function | Purpose |
|---|---|---|
| `useAudioImport.ts` | `useAudioImport()` | File drag-drop import + iCloud orchestration |
| `useConfigIo.ts` | `useConfigIo()` | Settings export/import with AES-GCM encryption |
| `useCover.ts` | `useCover()` | Cover image scanning, AI matching, manual pick |
| `useIcloud.ts` | `useIcloud()` | iCloud file status check & download polling |
| `useLlmParse.ts` | `useLlmParse()` | LLM filename parsing via Tauri IPC |
| `useRename.ts` | `useRename()` + `renderName()` | Rename template rendering + standalone render function |
| `useRules.ts` | `matchRule()`, `applyRules()`, `toRuleHints()` | Pure-function rule matching engine (no Vue dependency) |
| `useWriteback.ts` | `useWriteback()` | Metadata writeback & reset via Tauri IPC |

### Composable Structure Pattern

Each composable returns a plain object with refs (state) + functions (actions):

```typescript
// useLlmParse.ts (typical pattern)
export function useLlmParse() {
  const parsing = ref(false);
  const error = ref<string | null>(null);

  async function parse(files, config, parseConfig?, rules?): Promise<ParseResult[]> {
    // ...
  }

  return { parsing, error, parse };
}
```

### Pure Function Composables

`useRules.ts` is notable — it exports **pure functions** (no Vue reactivity dependency), importable without component context:

```typescript
// useRules.ts:113
export function applyRules(fileName: string, rules: FilenameRule[]): RuleMatch {
  const out: RuleMatch = {};
  for (const rule of rules) {
    if (!rule.enabled) continue;
    const m = matchRule(rule, fileName);
    // field-level overlay: higher priority fills first, lower doesn't overwrite
    if (out.title === undefined && m.title !== undefined) out.title = m.title;
    // ...
  }
  return out;
}
```

### Composable with Store Integration

Most composables import Pinia stores internally:

```typescript
// useAudioImport.ts:24-26
const store = useAudioStore();
const settings = useSettingsStore();
const icloud = useIcloud();
```

### Tauri API Usage in Composables

Composables are the primary consumers of `@tauri-apps/api` — not components:

```typescript
// useAudioImport.ts:2-5
import { invoke } from "@tauri-apps/api/core";
import { getCurrentWebview } from "@tauri-apps/api/webview";
import type { UnlistenFn } from "@tauri-apps/api/event";
import { open } from "@tauri-apps/plugin-dialog";
```

```typescript
// useLlmParse.ts:2
import { invoke } from "@tauri-apps/api/core";
```

```typescript
// useIcloud.ts:2
import { invoke } from "@tauri-apps/api/core";
```

### Lifecycle Usage in Composables

`onMounted` / `onUnmounted` are used in composables for setup/teardown (e.g., drag-drop listener):

```typescript
// useAudioImport.ts:83-99
onMounted(async () => {
  unlisten = await getCurrentWebview().onDragDropEvent((event) => {
    // handle drag/drop
  });
});
onUnmounted(() => {
  unlisten?.();
});
```

---

## 7. Quality (Lint, Type-Check, Test)

### package.json Scripts

```jsonc
// package.json:6-16
"scripts": {
  "dev": "vite",
  "build": "vue-tsc --noEmit && vite build",
  "preview": "vite preview",
  "type-check": "vue-tsc --noEmit",
  "test": "vitest run",
  "test:watch": "vitest",
  "tauri": "tauri",
  "tauri:dev": "tauri dev",
  "tauri:build": "tauri build",
  "build:dmg": "bash scripts/build-dmg.sh"
}
```

### Type Checking

- **vue-tsc** is used for type checking (`vue-tsc --noEmit`)
- Integrated into `build` script: type-check runs before vite build
- Separate `type-check` script for CI/local

### Testing

**Framework**: Vitest v2.1.9

**Config** (`vitest.config.ts`):
```typescript
export default defineConfig({
  resolve: { alias: { "@": path.resolve(__dirname, "./src") } },
  test: {
    environment: "node",    // No DOM needed — pure logic tests
    include: ["src/**/*.test.ts"],
  },
});
```

**Current test files**:
1. `src/composables/useRename.test.ts` — 5 tests for `renderName()` template rendering
2. `src/store/profiles.test.ts` — 4 tests for profile matching logic

**Test patterns**:
- Use `describe`/`it`/`expect` from vitest
- Factory functions for test fixtures:
  ```typescript
  // profiles.test.ts:7-23
  function profile(overrides: Partial<ShowProfile> = {}): ShowProfile {
    return { id: 1, album: "反派影评", artist: "波米", keywords: [], ...overrides };
  }
  function file(overrides: Partial<AudioFileMeta> = {}): AudioFileMeta {
    return {
      path: "/dir/x.mp3", fileName: "x.mp3", title: null, album: null,
      artist: null, track: null, durationSecs: null, embeddedCover: null, ...overrides,
    };
  }
  ```
- Pinia stores tested with `setActivePinia(createPinia())` in `beforeEach`
- Tests co-located with source: `useRename.test.ts` next to `useRename.ts`

### Linting

- **No ESLint config file found** (no `.eslintrc.*`, `eslint.config.*`)
- Type strictness enforced via `tsconfig.json`: `strict: true`, `noUnusedLocals: true`, `noUnusedParameters: true`

### Build Tooling

| Tool | Version | Config File |
|---|---|---|
| Vite | ^5.4.10 | `vite.config.ts` |
| TypeScript | ^5.6.3 | `tsconfig.json` + `tsconfig.node.json` |
| vue-tsc | ^2.1.10 | (uses tsconfig.json) |
| Vitest | ^2.1.9 | `vitest.config.ts` |
| Tailwind CSS | ^3.4.14 | `tailwind.config.js` |
| PostCSS | ^8.4.47 | `postcss.config.js` |

---

## 8. Tauri Frontend Patterns

### IPC Invocation

All backend calls use `invoke<T>()` from `@tauri-apps/api/core`:

```typescript
// Audio metadata reading
const metas = await invoke<AudioFileMeta[]>("read_audio_metadata", { paths: readable });

// LLM parsing
return await invoke<ParseResult[]>("parse_filenames", { files: inputs, config, parseConfig, rules });

// Writeback
const outcomes = await invoke<WriteOutcome[]>("write_metadata", { files: inputs });

// Reset
const outcomes = await invoke<ResetOutcome[]>("reset_files", { paths });

// Profile CRUD
profiles.value = await invoke<ShowProfile[]>("list_show_profiles");
await invoke<number>("save_show_profile", { profile: input });
await invoke("delete_show_profile", { id });

// iCloud
return invoke<ICloudStatus[]>("check_icloud_status", { paths });
await Promise.all(paths.map((p) => invoke("start_icloud_download", { path: p })));

// Cover scanning
const candidates = await invoke<CoverCandidates[]>("scan_cover_candidates", { audioPaths });
const results = await invoke<CoverMatchResult[]>("match_covers", { items, config });

// File I/O
await invoke("write_text_file", { path, contents: JSON.stringify(payload) });
const text = await invoke<string>("read_text_file", { path });

// Image data URL
return await invoke<string>("read_image_data_url", { path: image });

// Snapshots
const paths = await invoke<string[]>("list_snapshot_paths");

// AI rule generation
const result = await invoke<GenerateRuleResult>("generate_filename_rule", { ... });
```

### IPC Naming Convention

Backend commands use `snake_case`:
- `read_audio_metadata`
- `parse_filenames`
- `write_metadata`
- `reset_files`
- `list_show_profiles`
- `save_show_profile`
- `delete_show_profile`
- `check_icloud_status`
- `start_icloud_download`
- `scan_cover_candidates`
- `match_covers`
- `read_image_data_url`
- `write_text_file`
- `read_text_file`
- `list_snapshot_paths`
- `generate_filename_rule`

### Type Alignment

Frontend TypeScript interfaces mirror Rust structs with `camelCase` serialization:
```typescript
// Example comment from types/audio.ts:2-3
// 单个音频文件的元数据，结构与 Rust 端 `read_audio_metadata` 命令返回值一一对应。
// 字段命名与 Rust 的 `#[serde(rename_all = "camelCase")]` 保持一致。
```

### Tauri Plugin Usage

```typescript
// File dialogs (plugin-dialog)
import { open, save } from "@tauri-apps/plugin-dialog";

const selected = await open({
  multiple: true,
  filters: [{ name: "Audio", extensions: ["mp3", "m4a", ...] }],
});

const path = await save({
  defaultPath: "tagcast-config.json",
  filters: [{ name: "JSON", extensions: ["json"] }],
});
```

### Event Listening (Drag-Drop)

Webview-level drag-drop events via `@tauri-apps/api/webview`:

```typescript
// useAudioImport.ts:84-95
unlisten = await getCurrentWebview().onDragDropEvent((event) => {
  const payload = event.payload;
  if (payload.type === "enter" || payload.type === "over") {
    isDragging.value = true;
  } else if (payload.type === "drop") {
    isDragging.value = false;
    void importPaths(payload.paths);
  } else {
    isDragging.value = false;
  }
});
```

Cleanup pattern: `UnlistenFn` stored and called in `onUnmounted`:
```typescript
let unlisten: UnlistenFn | null = null;
onUnmounted(() => { unlisten?.(); });
```

### Error Handling

IPC calls use try/catch/finally with error stored in a reactive ref:
```typescript
const error = ref<string | null>(null);
try {
  // ... invoke ...
} catch (e) {
  error.value = String(e);
  throw e; // re-throw for caller
} finally {
  loading.value = false;
}
```

UI displays error conditionally:
```html
<p v-if="error" class="border-b border-danger-edge bg-danger-bg px-4 py-1.5 text-sm text-danger-fg">
  解析失败：{{ error }}
</p>
```

---

## Summary of Key Conventions

| Convention | Detail |
|---|---|
| SFC style | 100% `<script setup lang="ts">` |
| Props | `defineProps<{ open: boolean }>()` |
| Emits | `defineEmits<{ close: [] }>()` |
| Stores | Composition API `defineStore("name", () => { ... })` |
| Store access | `storeToRefs()` for reactive; direct access for actions |
| Composables | `use*` naming, return `{ refs, functions }` |
| Types | `interface` for data, `type` for unions; under `types/` |
| Imports | `@/` path alias for all internal imports |
| Styling | Tailwind utility classes only; semantic color tokens via CSS custom properties |
| Tauri IPC | `invoke<T>()` with snake_case command names |
| Error handling | reactive `error` ref + try/catch/finally |
| Testing | Vitest, `node` environment, co-located `*.test.ts` |
| Build | `vue-tsc --noEmit` type-check before `vite build` |
