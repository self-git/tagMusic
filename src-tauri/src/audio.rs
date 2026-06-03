use lofty::picture::PictureType;
use lofty::prelude::*;
use lofty::probe::Probe;
use serde::Serialize;
use std::path::Path;

/// 单个音频文件的元数据，返回给前端（camelCase 对齐 TS 的 AudioFileMeta）。
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AudioFileMeta {
    path: String,
    file_name: String,
    title: Option<String>,
    album: Option<String>,
    artist: Option<String>,
    track: Option<u32>,
    duration_secs: Option<u64>,
    // 文件内嵌封面的缩略图 data URL（无内嵌封面或解码失败为 None）：作为表格封面列的基准展示
    embedded_cover: Option<String>,
}

fn read_one(path: &str) -> Result<AudioFileMeta, String> {
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

    // 优先主标签，缺失时退回第一个可用标签
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

    // 内嵌封面：优先前置封面，否则取首张；解码缩放为缩略图 data URL（best-effort）
    let embedded_cover = tag
        .and_then(|t| {
            t.get_picture_type(PictureType::CoverFront)
                .or_else(|| t.pictures().first())
        })
        .and_then(|pic| crate::cover::make_thumbnail_data_url(pic.data()));

    let duration_secs = Some(tagged.properties().duration().as_secs());

    Ok(AudioFileMeta {
        path: path.to_string(),
        file_name,
        title,
        album,
        artist,
        track,
        duration_secs,
        embedded_cover,
    })
}

/// 批量读取音频文件元数据。无法解析的文件被跳过（记录日志），不阻断其余文件。
#[tauri::command]
pub fn read_audio_metadata(paths: Vec<String>) -> Vec<AudioFileMeta> {
    paths
        .into_iter()
        .filter_map(|p| match read_one(&p) {
            Ok(meta) => Some(meta),
            Err(e) => {
                log::warn!("跳过无法读取的音频文件 {p}: {e}");
                None
            }
        })
        .collect()
}
