# Type Safety

> Strict TypeScript with `interface` for data shapes, `type` for unions. Types mirror Rust backend structs.

---

## Type Organization

One file per domain under `src/types/`:

| File | Key Exports | Purpose |
|---|---|---|
| `types/audio.ts` | `AudioFileMeta` (interface) | Mirrors Rust `read_audio_metadata` |
| `types/llm.ts` | `ProviderConfig`, `ParseConfig`, `ParseResult` (interfaces), `ProviderType` (type) | LLM provider + parse config |
| `types/rule.ts` | `FilenameRule` (type), `SeparatorRule`, `RegexRule` (interfaces), `RuleField`, `RuleMatch` | Filename matching rules |
| `types/cover.ts` | `CoverCandidates`, `CoverMatchResult`, `CoverSelection` (interfaces) | Cover image state |
| `types/write.ts` | `WriteInput`, `WriteOutcome`, `ResetOutcome` (interfaces) | Writeback IPC types |
| `types/profile.ts` | `ShowProfile`, `ShowProfileInput` (interfaces) | Podcast archive profiles |
| `types/icloud.ts` | `ICloudStatus` (interface) | iCloud file status |

---

## Interfaces vs Types

- **`interface`** for data structures: `AudioFileMeta`, `ShowProfile`, `CoverSelection`
- **`type`** for unions, discriminated unions, and aliases:
  ```typescript
  // llm.ts:2
  export type ProviderType = "openai" | "anthropic";
  // rule.ts:7
  export type RuleField = "title" | "album" | "artist" | "track";
  ```

---

## Nullable Fields

Use `string | null` (not `undefined`), mirroring Rust's `Option<String>`:

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

---

## Type Imports

Always use `import type` for compile-time-only imports:

```typescript
import type { AudioFileMeta } from "@/types/audio";
import type { ParseResult } from "@/types/llm";
```

---

## Discriminated Unions

`FilenameRule` uses a discriminated union with a `type` literal discriminant:

```typescript
// rule.ts:28-41
export interface SeparatorRule extends BaseRule {
  type: "separator";
  separator: string;
  mapping: Partial<Record<RuleField, number>>;
}
export interface RegexRule extends BaseRule {
  type: "regex";
  pattern: string;
}
export type FilenameRule = SeparatorRule | RegexRule;
```

Narrow by checking `rule.type === "regex"` / `rule.type === "separator"`.

---

## Inline Type Aliases

Some small enums are defined inline:

```typescript
// SettingsModal.vue:19
type Preset = "deepseek" | "openai" | "anthropic";
// SettingsModal.vue:33
type Tab = "provider" | "parse" | "rename" | "rules" | "data";
// audio.ts:9
type EditableTextField = "title" | "album" | "artist";
```

---

## TypeScript Config

Full strictness in `tsconfig.json`:

```json
{
  "strict": true,
  "noUnusedLocals": true,
  "noUnusedParameters": true,
  "noFallthroughCasesInSwitch": true
}
```

Path alias `@/` → `src/` configured in both `tsconfig.json` (paths) and `vite.config.ts` (resolve alias).

---

## Type Utilities

| Utility | Usage |
|---|---|
| `Partial<>` | Optional overrides: `Partial<ConfigSnapshot>`, `Partial<Record<RuleField, string>>` |
| `Record<>` | Key-value maps: `Record<string, number | null>`, `Record<RuleField, string>` |
| `as const` | Readonly tuples: `export const RULE_FIELDS = ["title", ...] as const;` |

---

## Tauri IPC Type Alignment

Frontend TypeScript interfaces mirror Rust structs. Rust uses `#[serde(rename_all = "camelCase")]`:

```typescript
// types/audio.ts:2-3 — comment explaining alignment
// 单个音频文件的元数据，结构与 Rust 端 `read_audio_metadata` 命令返回值一一对应。
// 字段命名与 Rust 的 `#[serde(rename_all = "camelCase")]` 保持一致。
```

---

## Forbidden Patterns

- **`any`**: Never use. Always provide explicit types.
- **`as` type assertions**: Avoid. Prefer type narrowing or `interface` definitions.
- **`undefined` for optional fields**: Use `string | null`, never `string | undefined`.
- **Runtime type imports**: Use `import type` for all type-only imports.
