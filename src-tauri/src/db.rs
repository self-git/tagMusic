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

/// 打开数据库并建表，返回托管状态。节目档案库表 show_profiles 以 album 唯一。
pub fn init() -> Result<Db, String> {
    let conn = Connection::open(db_path()?).map_err(|e| e.to_string())?;
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS show_profiles (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            album TEXT NOT NULL,
            artist TEXT,
            keywords TEXT NOT NULL DEFAULT '',
            created_at INTEGER NOT NULL DEFAULT (strftime('%s','now'))
        );
        CREATE UNIQUE INDEX IF NOT EXISTS idx_show_profiles_album ON show_profiles(album);",
    )
    .map_err(|e| e.to_string())?;
    Ok(Db(Mutex::new(conn)))
}
