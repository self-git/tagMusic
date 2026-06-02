use crate::db::Db;
use rusqlite::params;
use serde::{Deserialize, Serialize};
use tauri::State;

/// 节目档案，返回给前端（camelCase 对齐 TS 的 ShowProfile）。
/// keywords 在库内以逗号分隔的 TEXT 存储，出入参均转成数组。
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ShowProfile {
    id: i64,
    album: String,
    artist: Option<String>,
    keywords: Vec<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ShowProfileInput {
    id: Option<i64>,
    album: String,
    artist: Option<String>,
    keywords: Vec<String>,
}

fn join_keywords(keywords: &[String]) -> String {
    keywords
        .iter()
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join(",")
}

fn split_keywords(raw: &str) -> Vec<String> {
    raw.split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect()
}

/// 列出全部节目档案，按节目名排序。
#[tauri::command]
pub fn list_show_profiles(db: State<'_, Db>) -> Result<Vec<ShowProfile>, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    let mut stmt = conn
        .prepare("SELECT id, album, artist, keywords FROM show_profiles ORDER BY album")
        .map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map([], |row| {
            Ok(ShowProfile {
                id: row.get(0)?,
                album: row.get(1)?,
                artist: row.get(2)?,
                keywords: split_keywords(&row.get::<_, String>(3)?),
            })
        })
        .map_err(|e| e.to_string())?;
    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())
}

/// 新增或更新节目档案：有 id 走 UPDATE；无 id 按 album 唯一约束 upsert。返回最终行 id。
#[tauri::command]
pub fn save_show_profile(db: State<'_, Db>, profile: ShowProfileInput) -> Result<i64, String> {
    let album = profile.album.trim();
    if album.is_empty() {
        return Err("节目名不能为空".to_string());
    }
    let keywords = join_keywords(&profile.keywords);
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    match profile.id {
        Some(id) => {
            conn.execute(
                "UPDATE show_profiles SET album = ?1, artist = ?2, keywords = ?3 WHERE id = ?4",
                params![album, profile.artist, keywords, id],
            )
            .map_err(|e| e.to_string())?;
            Ok(id)
        }
        None => {
            conn.execute(
                "INSERT INTO show_profiles (album, artist, keywords) VALUES (?1, ?2, ?3)
                 ON CONFLICT(album) DO UPDATE SET artist = excluded.artist, keywords = excluded.keywords",
                params![album, profile.artist, keywords],
            )
            .map_err(|e| e.to_string())?;
            conn.query_row(
                "SELECT id FROM show_profiles WHERE album = ?1",
                params![album],
                |row| row.get(0),
            )
            .map_err(|e| e.to_string())
        }
    }
}

/// 删除节目档案。
#[tauri::command]
pub fn delete_show_profile(db: State<'_, Db>, id: i64) -> Result<(), String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    conn.execute("DELETE FROM show_profiles WHERE id = ?1", params![id])
        .map_err(|e| e.to_string())?;
    Ok(())
}
