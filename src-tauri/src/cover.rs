use serde::Serialize;
use std::collections::HashMap;
use std::io::Cursor;
use std::path::Path;

// 可作为封面嵌入的图片扩展名（lofty 0.22 支持，故不含 webp，PRD v2 A1）
const IMAGE_EXTENSIONS: [&str; 3] = ["jpg", "jpeg", "png"];

// 自动直选封面的大小上限（1MB）：同目录唯一 cover.* 且小于此值时直接选中，跳过 AI 匹配
const PREFERRED_COVER_MAX_BYTES: u64 = 1024 * 1024;

// 内嵌封面缩略图最长边像素（控制内存与渲染开销）
const THUMBNAIL_MAX_PX: u32 = 160;

// 标准 base64 编码（无依赖）：用于把封面图片转成 data URL 供前端 <img> 预览
pub(crate) fn base64_encode(data: &[u8]) -> String {
    const T: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::with_capacity(data.len().div_ceil(3) * 4);
    for chunk in data.chunks(3) {
        let b0 = chunk[0] as u32;
        let b1 = *chunk.get(1).unwrap_or(&0) as u32;
        let b2 = *chunk.get(2).unwrap_or(&0) as u32;
        let n = (b0 << 16) | (b1 << 8) | b2;
        out.push(T[((n >> 18) & 63) as usize] as char);
        out.push(T[((n >> 12) & 63) as usize] as char);
        out.push(if chunk.len() > 1 {
            T[((n >> 6) & 63) as usize] as char
        } else {
            '='
        });
        out.push(if chunk.len() > 2 {
            T[(n & 63) as usize] as char
        } else {
            '='
        });
    }
    out
}

/// 把内嵌封面字节解码、缩放为小缩略图并编码为 JPEG data URL（供导入时展示）。
/// best-effort：解码/编码失败返回 None，不阻断导入流程。
pub(crate) fn make_thumbnail_data_url(bytes: &[u8]) -> Option<String> {
    let img = image::load_from_memory(bytes).ok()?;
    let thumb = img.thumbnail(THUMBNAIL_MAX_PX, THUMBNAIL_MAX_PX);
    let mut buf: Vec<u8> = Vec::new();
    thumb
        .write_to(&mut Cursor::new(&mut buf), image::ImageFormat::Jpeg)
        .ok()?;
    Some(format!("data:image/jpeg;base64,{}", base64_encode(&buf)))
}

fn mime_for(path: &Path) -> &'static str {
    match path
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_lowercase())
        .as_deref()
    {
        Some("png") => "image/png",
        _ => "image/jpeg",
    }
}

/// 读取图片文件并返回 data URL（base64），供前端缩略图预览。仅用于已扫描出的图片路径。
#[tauri::command]
pub fn read_image_data_url(path: String) -> Result<String, String> {
    let p = Path::new(&path);
    let bytes = std::fs::read(p).map_err(|e| format!("读取图片失败 {path}: {e}"))?;
    Ok(format!(
        "data:{};base64,{}",
        mime_for(p),
        base64_encode(&bytes)
    ))
}

/// 单个音频的候选封面：同目录下的图片完整路径列表
/// preferred：同目录唯一 cover.* 且 <1MB 时的首选封面路径（前端直选、跳过 AI），否则 None
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CoverCandidates {
    path: String,
    images: Vec<String>,
    preferred: Option<String>,
}

fn is_image(p: &Path) -> bool {
    p.extension()
        .and_then(|e| e.to_str())
        .map(|e| IMAGE_EXTENSIONS.contains(&e.to_lowercase().as_str()))
        .unwrap_or(false)
}

// 扫描目录下的图片完整路径（不递归）；目录不可读时返回空
fn scan_dir_images(dir: &Path) -> Vec<String> {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return vec![];
    };
    let mut images: Vec<String> = entries
        .filter_map(Result::ok)
        .map(|e| e.path())
        .filter(|p| p.is_file() && is_image(p))
        .map(|p| p.to_string_lossy().to_string())
        .collect();
    images.sort();
    images
}

// 首选封面：images 中以 cover 命名（大小写不敏感）的图片只有一张、且文件 <1MB 时返回其路径
fn preferred_cover(images: &[String]) -> Option<String> {
    let mut covers = images.iter().filter(|p| {
        Path::new(p)
            .file_stem()
            .and_then(|s| s.to_str())
            .map(|s| s.eq_ignore_ascii_case("cover"))
            .unwrap_or(false)
    });
    let only = covers.next()?;
    if covers.next().is_some() {
        return None;
    }
    let size = std::fs::metadata(only).ok()?.len();
    (size < PREFERRED_COVER_MAX_BYTES).then(|| only.clone())
}

/// 为每个音频扫描其所在目录的候选封面图片（同目录、不递归）。
/// 同目录复用一次扫描结果，避免重复 IO。
#[tauri::command]
pub fn scan_cover_candidates(audio_paths: Vec<String>) -> Vec<CoverCandidates> {
    let mut cache: HashMap<String, Vec<String>> = HashMap::new();
    audio_paths
        .into_iter()
        .map(|path| {
            let dir = Path::new(&path)
                .parent()
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_default();
            let images = cache
                .entry(dir.clone())
                .or_insert_with(|| scan_dir_images(Path::new(&dir)))
                .clone();
            let preferred = preferred_cover(&images);
            CoverCandidates {
                path,
                images,
                preferred,
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    // 扫描应只返回支持的图片扩展名，忽略音频/其他文件，且大小写不敏感
    #[test]
    fn scan_filters_to_supported_images() {
        let dir = std::env::temp_dir().join(format!(
            "tagcast_cover_scan_{}_{:?}",
            std::process::id(),
            std::thread::current().id()
        ));
        std::fs::create_dir_all(&dir).unwrap();
        for name in [
            "a.jpg",
            "b.PNG",
            "c.jpeg",
            "skip.webp",
            "audio.mp3",
            "note.txt",
        ] {
            std::fs::write(dir.join(name), b"x").unwrap();
        }
        let audio = dir.join("audio.mp3").to_string_lossy().to_string();

        let result = scan_cover_candidates(vec![audio.clone()]);
        assert_eq!(result.len(), 1);
        let imgs = &result[0].images;
        assert_eq!(imgs.len(), 3);
        assert!(imgs.iter().any(|p| p.ends_with("a.jpg")));
        assert!(imgs.iter().any(|p| p.ends_with("b.PNG")));
        assert!(imgs.iter().any(|p| p.ends_with("c.jpeg")));
        assert!(imgs.iter().all(|p| !p.ends_with("skip.webp")));

        std::fs::remove_dir_all(&dir).ok();
    }

    // 唯一 cover.* 且 <1MB → 首选；存在两张 cover.* 或超 1MB → 不首选
    #[test]
    fn preferred_cover_rules() {
        let dir = std::env::temp_dir().join(format!(
            "tagcast_preferred_{}_{:?}",
            std::process::id(),
            std::thread::current().id()
        ));
        std::fs::create_dir_all(&dir).unwrap();

        // 唯一小图 cover.jpg → 首选
        let cover = dir.join("cover.jpg");
        std::fs::write(&cover, b"small").unwrap();
        let only = cover.to_string_lossy().to_string();
        assert_eq!(preferred_cover(&[only.clone()]), Some(only.clone()));

        // 非 cover 命名不参与判定，仍然首选唯一 cover.jpg
        let other = dir.join("art.png").to_string_lossy().to_string();
        assert_eq!(preferred_cover(&[only.clone(), other]), Some(only.clone()));

        // 两张 cover.* → 不首选
        let cover_png = dir.join("cover.png");
        std::fs::write(&cover_png, b"small").unwrap();
        assert_eq!(
            preferred_cover(&[only.clone(), cover_png.to_string_lossy().to_string()]),
            None
        );

        // 超过 1MB → 不首选
        let big = dir.join("cover.jpeg");
        std::fs::write(&big, vec![0u8; (PREFERRED_COVER_MAX_BYTES + 1) as usize]).unwrap();
        assert_eq!(preferred_cover(&[big.to_string_lossy().to_string()]), None);

        std::fs::remove_dir_all(&dir).ok();
    }

    // base64 编码应符合标准（含填充），覆盖 0/1/2 个余字节三种情况
    #[test]
    fn base64_matches_known_vectors() {
        assert_eq!(base64_encode(b""), "");
        assert_eq!(base64_encode(b"f"), "Zg==");
        assert_eq!(base64_encode(b"fo"), "Zm8=");
        assert_eq!(base64_encode(b"foo"), "Zm9v");
        assert_eq!(base64_encode(b"foobar"), "Zm9vYmFy");
    }
}
