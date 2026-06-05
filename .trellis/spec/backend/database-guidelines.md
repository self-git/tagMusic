# Database Guidelines

> SQLite via rusqlite 0.40.0 (bundled). Connection wrapped in `Mutex<Connection>`, exposed as Tauri managed state.

---

## Library & Connection

```rust
// db.rs:6 — managed state type
pub struct Db(pub Mutex<Connection>);

// lib.rs:25-26 — injected once on startup
let database = db::init().map_err(std::io::Error::other)?;
app.manage(database);
```

Commands access it via `State<'_, Db>`:

```rust
// profiles.rs:44
pub fn list_show_profiles(db: State<'_, Db>) -> Result<Vec<ShowProfile>, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    // ...
}
```

Database file: `{data_dir}/TagCast/tagcast.db` (macOS: `~/Library/Application Support/TagCast/`).

---

## Schema (db.rs:20-58)

Two tables, created with `CREATE TABLE IF NOT EXISTS`:

```sql
CREATE TABLE IF NOT EXISTS show_profiles (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    album TEXT NOT NULL,
    artist TEXT,
    keywords TEXT NOT NULL DEFAULT '',
    created_at INTEGER NOT NULL DEFAULT (strftime('%s','now'))
);
CREATE UNIQUE INDEX IF NOT EXISTS idx_show_profiles_album ON show_profiles(album);
```

```sql
CREATE TABLE IF NOT EXISTS file_snapshots (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    current_path TEXT NOT NULL,
    original_path TEXT NOT NULL,
    original_file_name TEXT NOT NULL,
    orig_title TEXT,
    orig_album TEXT,
    orig_artist TEXT,
    orig_track INTEGER,
    had_cover INTEGER NOT NULL DEFAULT 0,
    orig_cover BLOB,
    orig_cover_mime TEXT,
    created_at INTEGER NOT NULL DEFAULT (strftime('%s','now'))
);
CREATE UNIQUE INDEX IF NOT EXISTS idx_file_snapshots_current ON file_snapshots(current_path);
```

---

## Migrations

Ad-hoc column migration using `add_column_if_missing` helper (db.rs:49-58,62-84). Uses `PRAGMA table_info()` to check column existence, then `ALTER TABLE ADD COLUMN`:

```rust
// db.rs:49-58 — v1→v2 migration: add cover columns to file_snapshots
add_column_if_missing(conn, "file_snapshots", "had_cover", "INTEGER NOT NULL DEFAULT 0")?;
add_column_if_missing(conn, "file_snapshots", "orig_cover", "BLOB")?;
add_column_if_missing(conn, "file_snapshots", "orig_cover_mime", "TEXT")?;
```

No migration version table exists — schema upgrades are additive only.

---

## Query Patterns

### DDL: `execute_batch`

```rust
// db.rs:21
conn.execute_batch(SCHEMA).map_err(|e| e.to_string())?;
```

### INSERT/UPDATE/DELETE: `execute` with `params![]`

```rust
// profiles.rs:74-78
conn.execute(
    "UPDATE show_profiles SET album = ?1, artist = ?2, keywords = ?3 WHERE id = ?4",
    params![album, profile.artist, keywords, id],
)
.map_err(|e| e.to_string())?;
```

### Single Row: `query_row`

```rust
// profiles.rs:88-92
conn.query_row(
    "SELECT id FROM show_profiles WHERE album = ?1",
    params![album],
    |row| row.get(0),
)
.map_err(|e| e.to_string())?
```

### Optional Single Row: `query_row` + `OptionalExtension`

```rust
// write.rs:170-176
let exists: Option<i64> = conn
    .query_row(
        "SELECT id FROM file_snapshots WHERE current_path = ?1",
        params![path],
        |row| row.get(0),
    )
    .optional()
    .map_err(|e| e.to_string())?;
```

### Multi-Row: `prepare` + `query_map`

```rust
// profiles.rs:46-58
let mut stmt = conn.prepare(
    "SELECT id, album, artist, keywords FROM show_profiles ORDER BY album"
)
.map_err(|e| e.to_string())?;
let rows = stmt.query_map([], |row| {
    Ok(ShowProfile { ... })
})
.map_err(|e| e.to_string())?;
rows.collect::<Result<Vec<_>, _>>()
    .map_err(|e| e.to_string())
```

### UPSERT: `ON CONFLICT DO UPDATE`

```rust
// profiles.rs:82-85
conn.execute(
    "INSERT INTO show_profiles (album, artist, keywords) VALUES (?1, ?2, ?3)
     ON CONFLICT(album) DO UPDATE SET artist = excluded.artist, keywords = excluded.keywords",
    params![album, profile.artist, keywords],
)
```

---

## Test Database

Tests use in-memory SQLite with the same production schema:

```rust
// write.rs:374-378
fn mem_conn() -> Connection {
    let conn = Connection::open_in_memory().unwrap();
    crate::db::apply_schema(&conn).unwrap();
    conn
}
```

---

## Common Mistakes

- **Using `unwrap()` on DB access**: Always use `.map_err(|e| e.to_string())?`.
- **Not locking the mutex**: Every DB access must first run `db.0.lock()...`.
- **Schema changes without migration**: New columns in existing tables require `add_column_if_missing` in `db::init()`.
