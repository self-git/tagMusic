use rusqlite::Connection;
use std::path::PathBuf;
use std::sync::Mutex;

/// SQLite 连接的 Tauri 托管状态。命令通过 `State<'_, Db>` 取用，Mutex 串行化访问。
pub struct Db(pub Mutex<Connection>);

// 数据库落地路径：~/Library/Application Support/TagCast/tagcast.db（dirs 跨平台取数据目录）
fn db_path() -> Result<PathBuf, String> {
    let dir = dirs::data_dir()
        .ok_or_else(|| "无法定位应用数据目录".to_string())?
        .join("TagCast");
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    Ok(dir.join("tagcast.db"))
}

/// 建表/建索引（幂等）。抽成独立函数，便于测试用内存库复用同一套 schema。
/// - show_profiles：节目档案库，以 album 唯一。
/// - file_snapshots：写回前的原文件名 + 原 tag + 原封面快照，以 current_path 唯一，支撑一键重置。
pub fn apply_schema(conn: &Connection) -> Result<(), String> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS show_profiles (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            album TEXT NOT NULL,
            artist TEXT,
            keywords TEXT NOT NULL DEFAULT '',
            created_at INTEGER NOT NULL DEFAULT (strftime('%s','now'))
        );
        CREATE UNIQUE INDEX IF NOT EXISTS idx_show_profiles_album ON show_profiles(album);

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
        CREATE UNIQUE INDEX IF NOT EXISTS idx_file_snapshots_current ON file_snapshots(current_path);",
    )
    .map_err(|e| e.to_string())?;

    // v1→v2 迁移：已发布旧库的 file_snapshots 缺封面列，幂等补列（CREATE TABLE IF NOT EXISTS 不改已存在表）
    add_column_if_missing(
        conn,
        "file_snapshots",
        "had_cover",
        "INTEGER NOT NULL DEFAULT 0",
    )?;
    add_column_if_missing(conn, "file_snapshots", "orig_cover", "BLOB")?;
    add_column_if_missing(conn, "file_snapshots", "orig_cover_mime", "TEXT")?;
    Ok(())
}

// 若表缺少指定列则 ALTER 补上（用 PRAGMA table_info 判断，避免重复添加报错）
fn add_column_if_missing(
    conn: &Connection,
    table: &str,
    column: &str,
    decl: &str,
) -> Result<(), String> {
    let mut stmt = conn
        .prepare(&format!("PRAGMA table_info({table})"))
        .map_err(|e| e.to_string())?;
    let exists = stmt
        .query_map([], |row| row.get::<_, String>(1))
        .map_err(|e| e.to_string())?
        .filter_map(Result::ok)
        .any(|name| name == column);
    if !exists {
        conn.execute(
            &format!("ALTER TABLE {table} ADD COLUMN {column} {decl}"),
            [],
        )
        .map_err(|e| e.to_string())?;
    }
    Ok(())
}

/// 打开数据库并建表，返回托管状态。
pub fn init() -> Result<Db, String> {
    let conn = Connection::open(db_path()?).map_err(|e| e.to_string())?;
    apply_schema(&conn)?;
    Ok(Db(Mutex::new(conn)))
}
