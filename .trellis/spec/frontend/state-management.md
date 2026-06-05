# State Management

> Pinia with Composition API setup function style. 3 stores: audio, profiles, settings.

---

## Store Definition Pattern

All stores use `defineStore("name", () => { ... })` (Composition API):

```typescript
// store/settings.ts:100
export const useSettingsStore = defineStore("settings", () => { ... });

// store/audio.ts:19
export const useAudioStore = defineStore("audio", () => { ... });

// store/profiles.ts:15
export const useProfilesStore = defineStore("profiles", () => { ... });
```

---

## State (ref)

State declared with `ref()`:

```typescript
// settings.ts:102-111
const icloudAutoDownload = ref(readBool(ICLOUD_KEY, true));
const llmProvider = ref<ProviderConfig>(readProvider());
const renameEnabled = ref(readBool(RENAME_ENABLED_KEY, false));
const rules = ref<FilenameRule[]>(readRules());

// audio.ts:20-28
const files = ref<AudioFileMeta[]>([]);
const currentIndex = ref(0);
const writtenPaths = ref<Set<string>>(new Set());
const coverByPath = ref<Record<string, CoverSelection>>({});
```

---

## Array Mutation: Always Replace by Reference

TanStack Table memoizes row models by data reference — never mutate arrays with `.push()`:

```typescript
// audio.ts:43 — CORRECT: replaces reference to trigger table re-render
files.value = [...files.value, ...fresh];

// WRONG:
files.value.push(...fresh); // TanStack won't see the change
```

---

## Getters (computed)

```typescript
// audio.ts:30-35
const currentFile = computed<AudioFileMeta | null>(
  () => files.value[currentIndex.value] ?? null,
);
const total = computed(() => files.value.length);
```

---

## Actions (async functions)

Actions are plain `async` functions that call Tauri IPC:

```typescript
// profiles.ts:21-23
async function load(): Promise<void> {
  profiles.value = await invoke<ShowProfile[]>("list_show_profiles");
}

// profiles.ts:25-28
async function save(input: ShowProfileInput): Promise<void> {
  await invoke<number>("save_show_profile", { profile: input });
  await load();
}
```

---

## Store Consumption in Components

Use `storeToRefs()` for reactive destructuring. Direct store access for actions:

```typescript
// App.vue:12-24
const { total } = storeToRefs(store);                          // reactive refs
const { icloudAutoDownload, renameEnabled } = storeToRefs(settings);
const { libraryOpen } = storeToRefs(profiles);

void profiles.load();                                           // direct action call
```

---

## localStorage Persistence (settings store)

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

## When to Use Global State

Use Pinia store when:
- State is shared across multiple components/views
- State needs to persist across navigation
- State is mutated by composables that don't own the component tree

Use local `ref()`/`reactive()` when:
- State is used only within a single composable
- State is temporary/transient (e.g., loading flags, form draft)
- State doesn't need to survive route changes

---

## Common Mistakes

- **Mutating arrays with `.push()`**: Always use spread to replace the reference: `arr.value = [...arr.value, newItem]`
- **Using store properties without `storeToRefs()`**: Destructuring loses reactivity — use `storeToRefs()` for refs
- **Not using `{ deep: true }` on watch for objects**: Object/array watches need `{ deep: true }`
