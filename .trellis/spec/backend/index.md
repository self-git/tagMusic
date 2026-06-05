# Backend Development Guidelines

> Best practices for Rust backend development in this project (Tauri 2 + rusqlite + lofty).

---

## Guidelines Index

| Guide | Description | Status |
|-------|-------------|--------|
| [Directory Structure](./directory-structure.md) | Flat module organization and file layout | Done |
| [Database Guidelines](./database-guidelines.md) | SQLite with rusqlite: schema, queries, migrations | Done |
| [Error Handling](./error-handling.md) | `Result<T, String>` pattern, error propagation | Done |
| [Quality Guidelines](./quality-guidelines.md) | Lint, test patterns, forbidden patterns | Done |
| [Logging Guidelines](./logging-guidelines.md) | `log` crate usage, log levels, what to log | Done |

---

## Quick Reference

- **Error type**: `Result<T, String>` — no `thiserror`/`anyhow`
- **Database**: rusqlite 0.40 bundled, `Db( Mutex<Connection> )` as Tauri state
- **Modules**: Flat files in `src-tauri/src/`, declared in `lib.rs`
- **Commands**: 14 Tauri commands, registered in `lib.rs` `generate_handler![]`
- **Tests**: Inline `#[cfg(test)] mod tests` in source files
- **Lint**: `cargo clippy --all-targets && cargo fmt --check`
