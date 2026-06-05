# Error Handling

> All errors use `Result<T, String>`. No custom error types, no `thiserror`, no `anyhow`.

---

## Core Pattern: `Result<T, String>`

Every fallible function returns `Result<T, String>`. Upstream errors are converted with `.map_err(|e| e.to_string())`:

```rust
// db.rs:21 — schema creation
.execute_batch(SCHEMA).map_err(|e| e.to_string())?;

// llm.rs:172 — HTTP client construction
Client::builder().build().map_err(|e| e.to_string())?;

// audio.rs:31-33 — file probe/read
Probe::open(path).map_err(|e| e.to_string())?;
```

---

## Inline Error Strings

When the error is a domain constraint (not an upstream error), create a string directly:

```rust
// profiles.rs:68 — validation
return Err("节目名不能为空".to_string());

// write.rs:120 — tag creation failure
.ok_or_else(|| "无法为该文件创建标签".to_string())?;

// llm.rs:207 — missing expected JSON field
.ok_or_else(|| "LLM 返回缺少 choices[0].message.content".to_string())?;

// write.rs:236 — file conflict
return Err(format!("目标文件名已存在：{new_name}"));
```

---

## Error Propagation in Tauri Commands

Commands that can fail return `Result<T, String>`. Tauri serializes the `String` error to the frontend automatically:

```rust
// llm.rs:259 — async command
pub async fn parse_filenames(...) -> Result<Vec<ParseResult>, String>

// write.rs:249 — sync command
pub fn write_metadata(db: State<'_, Db>, files: Vec<WriteInput>) -> Result<Vec<WriteOutcome>, String>
```

Commands that never fail return plain types (no `Result`):

```rust
// audio.rs:71 — best-effort, skips failures internally
pub fn read_audio_metadata(paths: Vec<String>) -> Vec<AudioFileMeta>

// icloud.rs:16 — always succeeds (non-macOS returns defaults)
pub fn check_icloud_status(paths: Vec<String>) -> Vec<ICloudStatus>
```

---

## Error Context for Custom Prompts

When the user has customized parsing prompts, add extra context to help them debug:

```rust
// llm.rs:281-287
.map_err(|e| {
    let mut msg = format!("解析 LLM 返回 JSON 失败: {e}; 原文: {content}");
    if customized {
        msg.push_str("\n提示：当前使用了自定义解析提示词，可在设置中恢复默认提示词后重试。");
    }
    msg
})
```

---

## Tauri Setup Error

The only place `std::io::Error` is used — Tauri's setup closure expects it:

```rust
// lib.rs:25
let database = db::init().map_err(std::io::Error::other)?;
```

---

## Common Mistakes

- **Using `anyhow`/`thiserror`**: The codebase intentionally avoids these crates. Stick to `Result<T, String>`.
- **Returning `io::Error` in commands**: Tauri commands should use `Result<T, String>` — only the setup closure uses `io::Error`.
- **Silently swallowing errors**: If a command truly cannot fail, return a plain type. If it can, use `Result<T, String>`.
