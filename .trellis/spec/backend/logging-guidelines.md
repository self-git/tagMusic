# Logging Guidelines

> Uses `log` crate 0.4 (facade) + `tauri-plugin-log` 2. Only active in debug builds at `Info` level.

---

## Configuration

```rust
// lib.rs:17-22
if cfg!(debug_assertions) {
    app.handle().plugin(
        tauri_plugin_log::Builder::default()
            .level(log::LevelFilter::Info)
            .build(),
    )?;
}
```

---

## What to Log

Logging is minimal — only for skip/fallback cases during batch operations:

- **Skips**: When an individual file in a batch fails and the batch continues
- **Failures**: When a writeback fails for a specific file
- **Success**: When a file is successfully written

```rust
// write.rs:77 — skipping unreadable files
log::warn!("跳过无法读取的音频文件 {p}: {e}");

// write.rs:258 — metadata write failure (doesn't abort batch)
log::warn!("写回元数据失败 {}: {e}", input.path);

// write.rs:265 — successful write confirmation
log::info!("已写回元数据: {new_path}");
```

---

## Log Levels

| Level | When |
|-------|------|
| `warn!` | Individual file skip/failure in batch operations |
| `info!` | Successful file write confirmation |
| `error!` | Not currently used |
| `debug!` | Not currently used |

---

## What NOT to Log

- **Normal flows**: Don't log routine operations — logging is for exceptional cases
- **PII / API keys**: Never log LLM API keys, user file paths with personal data
- **File contents**: Log file paths conservatively; never log file contents
