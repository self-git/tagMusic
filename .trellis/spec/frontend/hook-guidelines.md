# Composables / Hook Guidelines

> Custom composables follow the `use*` naming convention. Mix of stateful and pure-function patterns.

---

## Composable Inventory

| File | Export | Type |
|---|---|---|
| `useAudioImport.ts` | `useAudioImport()` | Stateful (refs + functions) |
| `useConfigIo.ts` | `useConfigIo()` | Stateful (refs + functions) |
| `useCover.ts` | `useCover()` | Stateful (refs + functions) |
| `useIcloud.ts` | `useIcloud()` | Stateful (refs + functions) |
| `useLlmParse.ts` | `useLlmParse()` | Stateful (refs + functions) |
| `useRename.ts` | `useRename()` + `renderName()` | Stateful + pure function export |
| `useRules.ts` | `matchRule()`, `applyRules()`, `toRuleHints()` | Pure functions (no Vue dependency) |
| `useWriteback.ts` | `useWriteback()` | Stateful (refs + functions) |

---

## Stateful Composable Pattern

Returns a plain object with refs (state) + async functions (actions):

```typescript
// useLlmParse.ts (typical pattern)
export function useLlmParse() {
  const parsing = ref(false);
  const error = ref<string | null>(null);

  async function parse(files, config, parseConfig?, rules?): Promise<ParseResult[]> {
    parsing.value = true;
    error.value = null;
    try {
      return await invoke<ParseResult[]>("parse_filenames", { ... });
    } catch (e) {
      error.value = String(e);
      throw e;
    } finally {
      parsing.value = false;
    }
  }

  return { parsing, error, parse };
}
```

---

## Pure Function Composable

`useRules.ts` exports pure functions — importable without Vue component context:

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

---

## Composable with Store Integration

Most composables import Pinia stores internally:

```typescript
// useAudioImport.ts:24-26
const store = useAudioStore();
const settings = useSettingsStore();
const icloud = useIcloud();
```

Components call composables, composables call stores — not the other way around.

---

## Tauri API Usage

Composables are the primary consumers of `@tauri-apps/api` — **not** components directly:

```typescript
// useAudioImport.ts:2-5
import { invoke } from "@tauri-apps/api/core";
import { getCurrentWebview } from "@tauri-apps/api/webview";
import type { UnlistenFn } from "@tauri-apps/api/event";
import { open } from "@tauri-apps/plugin-dialog";
```

---

## Lifecycle in Composables

`onMounted` / `onUnmounted` for drag-drop listener setup/teardown:

```typescript
// useAudioImport.ts:83-99
let unlisten: UnlistenFn | null = null;

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

## Naming Conventions

- **All composables**: `use*` prefix (`useAudioImport`, `useIcloud`, etc.)
- **Pure function exports**: No prefix required (`renderName`, `matchRule`, `applyRules`)
- **Files**: `use*.ts` under `src/composables/`

---

## Common Mistakes

- **Calling Tauri API directly in components**: Always go through a composable
- **Not cleaning up listeners**: Always store `UnlistenFn` and call in `onUnmounted`
- **Exporting functions without error handling**: Wrap async functions with try/catch/finally pattern
