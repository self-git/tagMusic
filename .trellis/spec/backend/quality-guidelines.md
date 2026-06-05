# Quality Guidelines

> Code quality standards for Rust backend development.

---

## Lint & Format

- **No `.clippy.toml`**: Standard clippy defaults apply
- **`cargo fmt`**: Standard Rust formatting (edition 2021)
- **Rust version**: 1.77.2 (Cargo.toml)

Run before committing:

```bash
cd src-tauri && cargo clippy --all-targets && cargo fmt --check
```

---

## Forbidden Patterns

- **`unwrap()` / `expect()` in production code**: Use `.map_err(|e| e.to_string())?` instead.
- **`anyhow` / `thiserror` crates**: Intentionally not used. All errors are `Result<T, String>`. See [Error Handling](./error-handling.md).
- **Raw `unsafe` blocks without `// SAFETY:` comments**: FFI code in `icloud.rs` uses `// SAFETY:` comments (see icloud.rs:64,78). Always document safety invariants.
- **Direct `println!` / `eprintln!`**: Use `log::warn!` / `log::info!` for diagnostics. See [Logging Guidelines](./logging-guidelines.md).

---

## Required Patterns

- **Tauri commands**: Register all `#[tauri::command]` functions in `lib.rs` `generate_handler![]` (lib.rs:29-46).
- **`State<'_, Db>`**: Access database through Tauri managed state — never open a raw connection.
- **`#[serde(rename_all = "camelCase")]`**: Apply to all structs serialized to frontend.
- **`// SAFETY:` comments**: Document safety invariants for all unsafe blocks.

---

## Testing

All tests are inline `#[cfg(test)] mod tests {}` blocks at the bottom of source files — no separate `tests/` directory.

### Files with Tests

| File | Approx. Tests | Focus |
|---|---|---|
| `llm.rs:490+` | ~12 | JSON extraction, result assembly, prompt construction, cover matching |
| `write.rs:369+` | ~9 | Tag write/clear, snapshot roundtrip, rename, cover write/restore |
| `cover.rs:151+` | 3 | Image scanning, preferred cover logic, base64 encoding |
| `profiles.rs:107+` | 5 | Keywords join/split/roundtrip |

### Test Fixtures

Tests create minimal valid audio/image files in temp directories:

```rust
// write.rs:403-422 — minimal valid PCM WAV bytes
fn minimal_wav() -> Vec<u8> { ... }

// write.rs:622-634 — minimal valid PNG
fn png_file(tag: &str, marker: u8) -> (std::path::PathBuf, String) { ... }
```

Files are named with PID + Thread ID to avoid cross-test collisions (write.rs:383-384):

```rust
format!("tagcast_test_{}_{}_{:?}.wav", tag, std::process::id(), std::thread::current().id())
```

Cleanup via `std::fs::remove_file` at end of each test.

### Test Database

Use in-memory SQLite with the same production schema:

```rust
// write.rs:374-378
fn mem_conn() -> Connection {
    let conn = Connection::open_in_memory().unwrap();
    crate::db::apply_schema(&conn).unwrap();
    conn
}
```

### Running Tests

```bash
cd src-tauri && cargo test
```

---

## Code Review Checklist

- [ ] No `unwrap()` / `expect()` outside tests
- [ ] All Tauri commands registered in `generate_handler![]`
- [ ] `#[serde(rename_all = "camelCase")]` on frontend-facing structs
- [ ] New DB columns added via `add_column_if_missing` in `db::init()`
- [ ] `// SAFETY:` comments on unsafe blocks
- [ ] Tests cover new logic
