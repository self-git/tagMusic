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
/// - file_snapshots：写回前的原文件名 + 原 tag 快照，以 current_path 唯一，支撑一键重置。
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
            created_at INTEGER NOT NULL DEFAULT (strftime('%s','now'))
        );
        CREATE UNIQUE INDEX IF NOT EXISTS idx_file_snapshots_current ON file_snapshots(current_path);",
    )
    .map_err(|e| e.to_string())
}

/// 打开数据库并建表，返回托管状态。
pub fn init() -> Result<Db, String> {
    let conn = Connection::open(db_path()?).map_err(|e| e.to_string())?;
    apply_schema(&conn)?;
    Ok(Db(Mutex::new(conn)))
}
