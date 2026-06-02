use crate::db::Db;
use lofty::config::WriteOptions;
use lofty::prelude::*;
use lofty::probe::Probe;
use lofty::tag::Tag;
use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use std::path::Path;
use tauri::State;

/// 写回入参：审核后的四字段 + 可选的重命名目标文件名（前端按模板渲染好传入）。
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WriteInput {
    path: String,
    title: Option<String>,
    album: Option<String>,
    artist: Option<String>,
    track: Option<u32>,
    /// 重命名后的文件名（含扩展名）；为空表示不重命名
    new_name: Option<String>,
}

/// 写回结果：旧路径 → 新路径（未重命名时两者相同），供前端更新行。
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WriteOutcome {
    old_path: String,
    new_path: String,
}

/// 重置结果：当前路径 → 恢复后的路径 + 原始四字段，供前端还原行。
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ResetOutcome {
    current_path: String,
    restored_path: String,
    title: Option<String>,
    album: Option<String>,
    artist: Option<String>,
    track: Option<u32>,
}

// 原始快照（写回前从磁盘读取的真实状态）
struct Snapshot {
    original_path: String,
    title: Option<String>,
    album: Option<String>,
    artist: Option<String>,
    track: Option<u32>,
}

// macOS Finder / Music.app 仅支持到 ID3v2.3，lofty 默认写 v2.4 会导致改动不可见，
// 故强制写 ID3v2.3（仅影响 MP3/WAV/AIFF 等 ID3v2 容器，对 MP4/FLAC/OGG 无副作用）。
fn write_options() -> WriteOptions {
    WriteOptions::default().use_id3v23(true)
}

// 将四字段写入文件的主标签：Some 设置，None 清除
fn apply_fields(input: &WriteInput) -> Result<(), String> {
    let p = Path::new(&input.path);
    let mut tagged = Probe::open(p)
        .map_err(|e| e.to_string())?
        .read()
        .map_err(|e| e.to_string())?;

    // 无主标签时按文件类型新建一个
    if tagged.primary_tag_mut().is_none() {
        let tag_type = tagged.primary_tag_type();
        tagged.insert_tag(Tag::new(tag_type));
    }
    let tag = tagged
        .primary_tag_mut()
        .ok_or_else(|| "无法为该文件创建标签".to_string())?;

    match &input.title {
        Some(v) => tag.set_title(v.clone()),
        None => {
            tag.remove_title();
        }
    }
    match &input.album {
        Some(v) => tag.set_album(v.clone()),
        None => {
            tag.remove_album();
        }
    }
    match &input.artist {
        Some(v) => tag.set_artist(v.clone()),
        None => {
            tag.remove_artist();
        }
    }
    match input.track {
        Some(t) => tag.set_track(t),
        None => {
            tag.remove_track();
        }
    }

    tagged
        .save_to_path(p, write_options())
        .map_err(|e| e.to_string())
}

// 首次写回前，读取磁盘当前真实状态作为原始快照（已存在则不覆盖，保留最初值）
fn ensure_snapshot(conn: &Connection, path: &str) -> Result<(), String> {
    let exists: Option<i64> = conn
        .query_row(
            "SELECT id FROM file_snapshots WHERE current_path = ?1",
            params![path],
            |row| row.get(0),
        )
        .optional()
        .map_err(|e| e.to_string())?;
    if exists.is_some() {
        return Ok(());
    }

    let p = Path::new(path);
    let file_name = p
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or_default()
        .to_string();
    let tagged = Probe::open(p)
        .map_err(|e| e.to_string())?
        .read()
        .map_err(|e| e.to_string())?;
    let tag = tagged.primary_tag().or_else(|| tagged.first_tag());
    let (title, album, artist, track) = match tag {
        Some(t) => (
            t.title().map(|s| s.to_string()),
            t.album().map(|s| s.to_string()),
            t.artist().map(|s| s.to_string()),
            t.track(),
        ),
        None => (None, None, None, None),
    };

    conn.execute(
        "INSERT INTO file_snapshots
            (current_path, original_path, original_file_name, orig_title, orig_album, orig_artist, orig_track)
         VALUES (?1, ?1, ?2, ?3, ?4, ?5, ?6)",
        params![path, file_name, title, album, artist, track],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

// 重命名同目录文件，并同步快照的 current_path；目标已存在则报错避免覆盖
fn rename_file(conn: &Connection, path: &str, new_name: &str) -> Result<String, String> {
    let p = Path::new(path);
    let parent = p.parent().unwrap_or_else(|| Path::new("."));
    let target = parent.join(new_name);
    let target_str = target.to_string_lossy().to_string();
    if target_str == path {
        return Ok(target_str);
    }
    if target.exists() {
        return Err(format!("目标文件名已存在：{new_name}"));
    }
    std::fs::rename(p, &target).map_err(|e| e.to_string())?;
    conn.execute(
        "UPDATE file_snapshots SET current_path = ?1 WHERE current_path = ?2",
        params![target_str, path],
    )
    .map_err(|e| e.to_string())?;
    Ok(target_str)
}

/// 写回元数据：逐个文件先快照原始状态，再写入标签，最后按需重命名。
#[tauri::command]
pub fn write_metadata(
    db: State<'_, Db>,
    files: Vec<WriteInput>,
) -> Result<Vec<WriteOutcome>, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    let mut outcomes = Vec::with_capacity(files.len());
    for input in &files {
        ensure_snapshot(&conn, &input.path)?;
        apply_fields(input).map_err(|e| {
            log::warn!("写回元数据失败 {}: {e}", input.path);
            e
        })?;
        let new_path = match input.new_name.as_deref() {
            Some(name) if !name.is_empty() => rename_file(&conn, &input.path, name)?,
            _ => input.path.clone(),
        };
        log::info!("已写回元数据: {new_path}");
        outcomes.push(WriteOutcome {
            old_path: input.path.clone(),
            new_path,
        });
    }
    Ok(outcomes)
}

fn load_snapshot(conn: &Connection, path: &str) -> Result<Option<Snapshot>, String> {
    conn.query_row(
        "SELECT original_path, orig_title, orig_album, orig_artist, orig_track
         FROM file_snapshots WHERE current_path = ?1",
        params![path],
        |row| {
            Ok(Snapshot {
                original_path: row.get(0)?,
                title: row.get(1)?,
                album: row.get(2)?,
                artist: row.get(3)?,
                track: row.get(4)?,
            })
        },
    )
    .optional()
    .map_err(|e| e.to_string())
}

// 用快照中的原始四字段覆盖写回当前文件
fn restore_fields(path: &str, snap: &Snapshot) -> Result<(), String> {
    apply_fields(&WriteInput {
        path: path.to_string(),
        title: snap.title.clone(),
        album: snap.album.clone(),
        artist: snap.artist.clone(),
        track: snap.track,
        new_name: None,
    })
}

/// 一键重置：恢复原 tag + 原文件名，并删除快照（文件回到未处理状态）。无快照的路径跳过。
#[tauri::command]
pub fn reset_files(db: State<'_, Db>, paths: Vec<String>) -> Result<Vec<ResetOutcome>, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    let mut outcomes = Vec::new();
    for path in &paths {
        let Some(snap) = load_snapshot(&conn, path)? else {
            continue;
        };
        restore_fields(path, &snap)?;
        // 若曾重命名，把文件移回原始路径
        let restored_path = if snap.original_path != *path {
            let target = Path::new(&snap.original_path);
            if target.exists() {
                return Err(format!("原始文件名已被占用：{}", snap.original_path));
            }
            std::fs::rename(Path::new(path), target).map_err(|e| e.to_string())?;
            snap.original_path.clone()
        } else {
            path.clone()
        };
        conn.execute(
            "DELETE FROM file_snapshots WHERE current_path = ?1",
            params![path],
        )
        .map_err(|e| e.to_string())?;
        outcomes.push(ResetOutcome {
            current_path: path.clone(),
            restored_path,
            title: snap.title,
            album: snap.album,
            artist: snap.artist,
            track: snap.track,
        });
    }
    Ok(outcomes)
}

/// 列出已写回（存在快照）的文件当前路径，供前端标记可重置状态。
#[tauri::command]
pub fn list_snapshot_paths(db: State<'_, Db>) -> Result<Vec<String>, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    let mut stmt = conn
        .prepare("SELECT current_path FROM file_snapshots")
        .map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map([], |row| row.get::<_, String>(0))
        .map_err(|e| e.to_string())?;
    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use lofty::probe::Probe;

    // 构造一个最小可解析的 PCM WAV（lofty 可向其写入 ID3v2 / id3 chunk）
    fn minimal_wav() -> Vec<u8> {
        let data: [u8; 4] = [0, 0, 0, 0];
        let mut buf = Vec::new();
        buf.extend_from_slice(b"RIFF");
        let riff_size: u32 = 4 + (8 + 16) + (8 + data.len() as u32);
        buf.extend_from_slice(&riff_size.to_le_bytes());
        buf.extend_from_slice(b"WAVE");
        buf.extend_from_slice(b"fmt ");
        buf.extend_from_slice(&16u32.to_le_bytes());
        buf.extend_from_slice(&1u16.to_le_bytes()); // PCM
        buf.extend_from_slice(&1u16.to_le_bytes()); // mono
        buf.extend_from_slice(&8000u32.to_le_bytes()); // sample rate
        buf.extend_from_slice(&16000u32.to_le_bytes()); // byte rate
        buf.extend_from_slice(&2u16.to_le_bytes()); // block align
        buf.extend_from_slice(&16u16.to_le_bytes()); // bits per sample
        buf.extend_from_slice(b"data");
        buf.extend_from_slice(&(data.len() as u32).to_le_bytes());
        buf.extend_from_slice(&data);
        buf
    }

    // 证明 apply_fields 真的把标签落盘：写入后重新读取应能取回标题
    #[test]
    fn writes_title_to_disk() {
        let path =
            std::env::temp_dir().join(format!("tagcast_write_test_{}.wav", std::process::id()));
        std::fs::write(&path, minimal_wav()).unwrap();
        let path_str = path.to_string_lossy().to_string();

        apply_fields(&WriteInput {
            path: path_str.clone(),
            title: Some("单元测试标题".to_string()),
            album: Some("测试节目".to_string()),
            artist: None,
            track: Some(7),
            new_name: None,
        })
        .expect("写入应成功");

        let tagged = Probe::open(&path).unwrap().read().unwrap();
        let tag = tagged.primary_tag().or_else(|| tagged.first_tag()).unwrap();
        assert_eq!(tag.title().as_deref(), Some("单元测试标题"));
        assert_eq!(tag.album().as_deref(), Some("测试节目"));
        assert_eq!(tag.track(), Some(7));

        std::fs::remove_file(&path).ok();
    }
}
