# Component Guidelines

> 100% of Vue SFCs use `<script setup lang="ts">`. No Options API components exist in the codebase.

---

## Component Structure

Every component follows this pattern:

```vue
<script setup lang="ts">
import { ref } from "vue";

// ... state, composables, functions ...
</script>

<template>
  <!-- Tailwind utility classes only -->
</template>
```

**No `<style scoped>` blocks** — all styling is done via Tailwind utility classes in templates.

---

## Props Convention

Use type-based generic props with `defineProps<{ ... }>()`:

```typescript
// SettingsModal.vue:10
defineProps<{ open: boolean }>();

// ProfileLibraryModal.vue:7
defineProps<{ open: boolean }>();
```

- No `withDefaults` needed for required props
- No runtime validation — TypeScript types are trusted

---

## Emits Convention

Use type-based generic emits with tuple syntax:

```typescript
// SettingsModal.vue:11
const emit = defineEmits<{ close: [] }>();

// ProfileLibraryModal.vue:8
const emit = defineEmits<{ close: [] }>();
```

`close: []` means the event has no payload arguments.

---

## Component Imports

All components imported with `@/` alias:

```typescript
import SettingsModal from "@/components/SettingsModal.vue";
import ProfileLibraryModal from "@/components/ProfileLibraryModal.vue";
```

---

## Modal Pattern

Modals use an open/close props-emits contract. Parent controls visibility:

```html
<!-- App.vue:91-92 -->
<SettingsModal :open="settingsOpen" @close="settingsOpen = false" />
<ProfileLibraryModal :open="libraryOpen" @close="profiles.closeLibrary()" />
```

Modal component renders with `v-if="open"` and closes on overlay click or Escape:

```html
<!-- SettingsModal.vue:46-51 -->
<div
  v-if="open"
  class="fixed inset-0 z-50 flex items-center justify-center bg-black/50 p-6"
  @click.self="emit('close')"
  @keydown.esc.window="emit('close')"
>
```

---

## Template Patterns

- **Store access**: Use `v-model` directly on `storeToRefs()` refs
- **Conditionals**: `v-if`, `v-else-if`, `v-else`, `v-show`
- **Navigation**: `<router-link>` with `active-class` for tabs
- **Icons**: `<component :is="t.icon">` for lucide-vue-next icons
- **Direct mutation**: Store refs mutated via `v-model` (no copy-to-local pattern):
  ```html
  v-model="row.original.title"
  v-model="llmProvider.apiKey"
  ```

---

## Common Mistakes

- **Adding `<style scoped>`**: Use Tailwind utility classes instead
- **Using Options API**: All components must use `<script setup lang="ts">`
- **Copying props to local refs**: Bind directly to `storeToRefs()` or define reactive refs at the composable level
- **Emitting with payload without typing**: Always type emits with `defineEmits<{ event: [PayloadType] }>()`
