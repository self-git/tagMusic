# Directory Structure

> Backend code is organized as flat files under `src-tauri/src/` — no subdirectory modules.

---

## Directory Layout

```
src-tauri/src/
├── main.rs          # Binary entry point, calls app_lib::run()
├── lib.rs           # Crate root (app_lib): module declarations, Tauri builder, command registration
├── audio.rs         # Read audio file metadata via lofty (tags, duration, embedded cover)
├── config.rs        # Text file read/write for import/export config JSON
├── cover.rs         # Cover image scanning, base64 encoding, thumbnail generation
├── db.rs            # SQLite init, schema creation, migration helper (rusqlite)
├── icloud.rs        # macOS iCloud file status check + download trigger (objc2 FFI)
├── llm.rs           # LLM integration: filename parsing, rule generation, cover matching (reqwest)
├── profiles.rs      # CRUD for show profiles (album/artist/keywords) stored in SQLite
└── write.rs         # Write metadata tags back to audio files, rename, snapshot/reset
```

---

## Module Organization

Every module is a single `.rs` file declared in `lib.rs`:

```rust
// lib.rs:1-8
mod audio;
mod config;
mod cover;
mod db;
mod icloud;
mod llm;
mod profiles;
mod write;
```

Cross-module access uses `crate::module::item` paths (e.g., `crate::db::Db`, `crate::db::apply_schema`). There is no public module re-export.

---

## Adding a New Module

1. Create a new file under `src-tauri/src/`
2. Declare it in `lib.rs` (`mod new_module;`)
3. If it exposes Tauri commands, register them in `lib.rs` `generate_handler![]` (line 29-46)
4. If it needs `Db` access, accept `State<'_, Db>` as a parameter

---

## Cross-Module Dependencies

```
audio.rs     → cover.rs (make_thumbnail_data_url)
write.rs     → db.rs (Db, apply_schema)
profiles.rs  → db.rs (Db)
db.rs        → (standalone)
cover.rs     → (standalone)
config.rs    → (standalone)
icloud.rs    → (standalone)
llm.rs       → (standalone)
lib.rs       → all modules (registration only)
```

---

## Naming Conventions

- **Filenames**: `snake_case.rs` matching the module name
- **Crate name**: `app_lib` (lib), `tagcast` (binary)
- **`#[serde(rename_all = "camelCase")]`**: Applied on all structs serialized to frontend
- **Comments**: Module-level and inline comments are in Chinese
