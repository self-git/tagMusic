# Quality Guidelines

> TypeScript strictness via `vue-tsc`, Vitest for testing, no ESLint. All styling via Tailwind utility classes.

---

## Lint & Type Check

- **No ESLint**: Type strictness enforced solely via `tsconfig.json` (`strict: true`, `noUnusedLocals`, `noUnusedParameters`)
- **Type check**: `vue-tsc --noEmit` (integrated into `build` script)

Run before committing:

```bash
npm run type-check
```

---

## Build Pipeline

`build` script runs type-check before Vite build:

```json
// package.json:8
"build": "vue-tsc --noEmit && vite build"
```

---

## Testing

### Framework

Vitest v2.1.9 with `node` environment:

```typescript
// vitest.config.ts
test: {
  environment: "node",
  include: ["src/**/*.test.ts"],
}
```

### Test Files

| File | Tests | Focus |
|---|---|---|
| `composables/useRename.test.ts` | 5 | `renderName()` template rendering |
| `store/profiles.test.ts` | 4 | Profile matching logic |

Tests are **co-located** with source (e.g., `useRename.test.ts` next to `useRename.ts`).

### Test Patterns

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
- Pinia store tests use `setActivePinia(createPinia())` in `beforeEach`

### Running Tests

```bash
npm test
```

---

## Styling Conventions

100% Tailwind CSS — no `<style scoped>` blocks. Use **semantic color tokens only**:

```html
<!-- A primary section -->
<div class="flex h-screen flex-col bg-base text-strong">
  <header class="border-b border-line px-5 py-3">
  <button class="rounded-lg bg-accent px-3 py-1.5 text-sm font-medium text-white hover:bg-accent-hover">
    AI 解析
  </button>
```

Common class patterns:

| Pattern | Classes |
|---|---|
| Layout | `flex`, `flex-col`, `h-screen`, `min-h-0 flex-1 overflow-auto` |
| Spacing | `px-4 py-2`, `gap-3`, `mt-1.5` |
| Borders | `border border-line`, `border-b border-edge`, `rounded-xl` |
| Typography | `text-sm text-muted`, `text-xs text-faint`, `text-lg font-semibold` |
| Interactive | `hover:bg-elevated`, `focus:border-accent`, `disabled:opacity-50` |
| Positioning | `fixed inset-0 z-50`, `sticky top-0` |

---

## Forbidden Patterns

- **`<style scoped>`**: Use Tailwind utility classes exclusively
- **Options API**: All components must use `<script setup lang="ts">`
- **Raw color values in templates**: Use semantic tokens (`bg-base`, `text-muted`, `border-line`), not `bg-gray-100`
- **`any` type**: Always provide explicit types
- **`as` assertions**: Prefer type narrowing
- **`string | undefined`**: Use `string | null`

---

## Code Review Checklist

- [ ] `vue-tsc --noEmit` passes
- [ ] No `<style scoped>` blocks added
- [ ] All Tailwind classes use semantic tokens
- [ ] No `any` types
- [ ] `string | null` for optional fields (not `undefined`)
- [ ] New logic has tests
