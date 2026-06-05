# Directory Structure

> Frontend code is organized under `src/` with 7 top-level directories. Vue 3 + TypeScript + Vite.

---

## Directory Layout

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
│   ├── useRename.ts                 # Rename template rendering + standalone renderName()
│   ├── useRules.ts                  # Pure-function filename rule matching engine
│   └── useWriteback.ts              # Metadata writeback & reset via Tauri IPC
├── router/
│   └── index.ts                     # Vue Router config: hash history, /table, /wizard
├── store/                           # Pinia stores
│   ├── audio.ts                     # Audio file state: files list, nav, parse results, cover state
│   ├── profiles.ts                  # Show profile CRUD, auto-matching, library modal state
│   └── settings.ts                  # User settings: LLM config, rename, parse config, rules
├── types/                           # Shared TypeScript type definitions
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

---

## Directory Purposes

| Directory | Purpose |
|---|---|
| `types/` | Shared TypeScript interfaces mirroring Rust backend structs (camelCase alignment) |
| `store/` | Pinia stores using Composition API (`defineStore` with setup function) |
| `composables/` | Reusable stateful/stateless logic (Vue 3 Composition API) |
| `components/` | Shared presentational + stateful components (modals, editors) |
| `views/` | Route-level page components (flat, no sub-directories) |
| `router/` | Vue Router configuration (hash history) |

---

## Import Conventions

All internal imports use the `@/` path alias (maps to `src/`):

```typescript
import { useAudioStore } from "@/store/audio";
import SettingsModal from "@/components/SettingsModal.vue";
import type { AudioFileMeta } from "@/types/audio";
```

---

## Adding a New Feature

1. **Types first**: Add interfaces to `src/types/` (one file per domain)
2. **Store if needed**: Add a Pinia store in `src/store/`
3. **Composable for logic**: Add reusable logic in `src/composables/`
4. **Component for UI**: Add component in `src/components/` or `src/views/`
5. **Register route** if it's a page: add to `src/router/index.ts`
