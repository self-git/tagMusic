# Frontend Development Guidelines

> Best practices for Vue 3 + TypeScript frontend development in this project (Vite + Pinia + Tailwind CSS).

---

## Guidelines Index

| Guide | Description | Status |
|-------|-------------|--------|
| [Directory Structure](./directory-structure.md) | Module organization and file layout | Done |
| [Component Guidelines](./component-guidelines.md) | `<script setup>`, props, emits, modal patterns | Done |
| [Composables / Hooks](./hook-guidelines.md) | `use*` composables, Tauri IPC, lifecycle | Done |
| [State Management](./state-management.md) | Pinia stores, reactivity, localStorage | Done |
| [Type Safety](./type-safety.md) | Interfaces, discriminated unions, IPC alignment | Done |
| [Quality Guidelines](./quality-guidelines.md) | Lint, tests, styling, forbidden patterns | Done |

---

## Quick Reference

- **SFC style**: 100% `<script setup lang="ts">`
- **Props**: `defineProps<{ open: boolean }>()`
- **Emits**: `defineEmits<{ close: [] }>()`
- **Stores**: `defineStore("name", () => { ... })` (Composition API)
- **Store access**: `storeToRefs()` for refs; direct for actions
- **Composables**: `use*` naming, Tauri IPC lives here
- **Types**: `interface` for data, `type` for unions; `string | null` for optionals
- **Imports**: `@/` path alias; `import type` for type-only imports
- **Styling**: Tailwind utility classes only; semantic color tokens (`bg-base`, `text-muted`)
- **Tauri**: `invoke<T>()` with `snake_case` command names
- **Build**: `vue-tsc --noEmit` before `vite build`
- **Test**: Vitest, `node` env, co-located `*.test.ts`
